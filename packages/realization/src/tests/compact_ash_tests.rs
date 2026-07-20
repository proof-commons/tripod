use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    ARCHITECTURE, AssetId, BoundId, DeltaKind, ObjectId, OperationId, ProjectionId, RootId, RootUse,
};

use crate::{
    Count, ObservedAsset, ObservedCanonicalDelta, ObservedObject, ObservedObjectKind,
    ObservedObjectRef, ObservedOpenFlow, ObservedRootEffect, ObservedSide, OperationObservation,
    ProtocolAmount, RealizationScope, RelationId, RelationKind, RelationStatus, RelationSubject,
    RepresentationMode, TransactionSide, derive, evaluate_operation,
};

type ObservationMutation = Box<dyn Fn(&mut OperationObservation)>;
type RelationCase = (&'static str, ObservationMutation, RelationId);
type SponsorCase = (&'static str, ObservationMutation);

fn compact_scope() -> RealizationScope {
    RealizationScope::from_operations([OperationId::CompactAsh]).unwrap()
}

fn pilot_realization() -> crate::ScopedRealizationSpec {
    derive(&ARCHITECTURE, compact_scope()).unwrap()
}

fn ash(side: ObservedSide, ordinal: u32, value: u64) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::Ash),
        asset: ObservedAsset::Declared(AssetId::U),
        value: ProtocolAmount::new(value).unwrap(),
        owner: None,
        representation: RepresentationMode::Explicit,
    }
}

fn lbtc(side: ObservedSide, ordinal: u32, value: u64) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::PlainLbtc),
        asset: ObservedAsset::Declared(AssetId::Lbtc),
        value: ProtocolAmount::new(value).unwrap(),
        owner: None,
        representation: RepresentationMode::Explicit,
    }
}

fn valid_observation() -> OperationObservation {
    let input0 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 0,
    };
    let input1 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 1,
    };
    let output0 = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 0,
    };

    OperationObservation {
        operation: OperationId::CompactAsh,
        objects: vec![
            ash(ObservedSide::Input, 0, 40),
            ash(ObservedSide::Input, 1, 60),
            ash(ObservedSide::Output, 0, 100),
        ],
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::new(),
        canonical_deltas: vec![ObservedCanonicalDelta {
            asset: AssetId::U,
            kind: DeltaKind::OwnerlessLateral,
            amount: ProtocolAmount::new(100).unwrap(),
            sources: vec![input0, input1],
            destinations: vec![output0],
            destruction_tag: None,
        }],
        open_flows: Vec::new(),
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::AshBatchMax, Count::new(64)),
            (BoundId::FeeSponsorInputMax, Count::new(16)),
        ]),
    }
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
    RelationId::new(OperationId::CompactAsh, kind, subject)
}

fn input_cardinality() -> RelationId {
    relation_id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
    )
}

fn output_cardinality() -> RelationId {
    relation_id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::Ash,
        },
    )
}

fn input_recognition() -> RelationId {
    relation_id(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
    )
}

fn output_recognition() -> RelationId {
    relation_id(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::Ash,
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

fn open_flow_policy() -> RelationId {
    relation_id(
        RelationKind::SponsorIsolation,
        RelationSubject::Projection {
            projection: ProjectionId::TransitionCertificate,
        },
    )
}

fn authorization() -> RelationId {
    relation_id(RelationKind::Authorization, RelationSubject::Operation)
}

fn input_closure() -> RelationId {
    relation_id(
        RelationKind::OutputClosure,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
    )
}

fn output_closure() -> RelationId {
    relation_id(
        RelationKind::OutputClosure,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::Ash,
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
            object: ObjectId::Ash,
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
fn valid_compact_ash_satisfies_every_runtime_relation() {
    let report = evaluate(&valid_observation());

    assert!(
        report.is_conformant(),
        "failed relations: {:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
}

#[test]
fn compact_ash_canonical_delta_amount_mutation_fails() {
    let mut observation = valid_observation();

    observation.canonical_deltas[0].amount = ProtocolAmount::new(99).unwrap();

    let report = evaluate(&observation);

    assert!(
        failed(&report, &canonical_delta_policy()),
        "a compact-ASH canonical delta whose amount disagrees with its referenced objects must fail"
    );
}

#[test]
fn compact_ash_negative_observations_name_the_load_bearing_relation() {
    for (name, mutate, relation) in relation_cases() {
        let mut observation = valid_observation();
        mutate(&mut observation);
        let report = evaluate(&observation);

        assert!(
            failed(&report, &relation),
            "case {name} did not fail {relation:?}"
        );
    }
}

#[allow(clippy::too_many_lines)]
fn relation_cases() -> Vec<RelationCase> {
    vec![
        (
            "one ash input",
            Box::new(|observation| {
                observation.objects[1].kind = ObservedObjectKind::Declared(ObjectId::PlainLbtc);
            }),
            input_cardinality(),
        ),
        (
            "bound below input count",
            Box::new(|observation| {
                observation.bounds.insert(BoundId::AshBatchMax, Count::ONE);
            }),
            input_cardinality(),
        ),
        (
            "no ash output",
            Box::new(|observation| {
                observation.objects[2].kind = ObservedObjectKind::Declared(ObjectId::PlainLbtc);
            }),
            output_cardinality(),
        ),
        (
            "two ash outputs",
            Box::new(|observation| observation.objects.push(ash(ObservedSide::Output, 1, 0))),
            output_cardinality(),
        ),
        (
            "wrong output amount",
            Box::new(|observation| observation.objects[2].value = ProtocolAmount::new(99).unwrap()),
            conservation(),
        ),
        (
            "wrong input asset",
            Box::new(|observation| {
                observation.objects[0].asset = ObservedAsset::Declared(AssetId::Lbtc);
            }),
            input_recognition(),
        ),
        (
            "wrong output asset",
            Box::new(|observation| {
                observation.objects[2].asset = ObservedAsset::Declared(AssetId::Lbtc);
            }),
            output_recognition(),
        ),
        (
            "wrong canonical delta kind",
            Box::new(|observation| {
                observation.canonical_deltas[0].kind = DeltaKind::Lateral;
            }),
            canonical_delta_policy(),
        ),
        (
            "undeclared open flow role",
            Box::new(|observation| {
                observation.open_flows.push(ObservedOpenFlow {
                    kind: architecture::OpenFlowKind::RequestCreation,
                    sources: Vec::new(),
                    destinations: Vec::new(),
                    fee: ProtocolAmount::ZERO,
                });
            }),
            open_flow_policy(),
        ),
        (
            "foreign input object",
            Box::new(|observation| observation.objects[0].kind = ObservedObjectKind::Unrecognized),
            input_closure(),
        ),
        (
            "live receipt output",
            Box::new(|observation| {
                observation.objects[2].kind = ObservedObjectKind::Declared(ObjectId::ReceiptLive);
            }),
            output_closure(),
        ),
        (
            "unexpected protocol signer",
            Box::new(|observation| {
                observation
                    .protocol_signers
                    .insert(crate::OwnerId([9_u8; 32]));
            }),
            authorization(),
        ),
        (
            "burn projection",
            Box::new(|observation| {
                observation.projections.insert(ProjectionId::BurnEvent);
            }),
            projections(),
        ),
        (
            "clear projection",
            Box::new(|observation| {
                observation.projections.insert(ProjectionId::ClearEvent);
            }),
            projections(),
        ),
        (
            "residue projection",
            Box::new(|observation| {
                observation
                    .projections
                    .insert(ProjectionId::DistributionResidue);
            }),
            projections(),
        ),
        (
            "missing transition projection",
            Box::new(|observation| observation.projections.clear()),
            projections(),
        ),
        (
            "private ash representation",
            Box::new(|observation| {
                observation.objects[2].representation = RepresentationMode::PrivateCommitted;
            }),
            representation(),
        ),
        (
            "unexpected root effect",
            Box::new(|observation| {
                observation.root_effects.push(ObservedRootEffect {
                    root: RootId::State,
                    use_kind: RootUse::Succession,
                });
            }),
            roots(),
        ),
    ]
}

#[test]
fn compact_conservation_is_blocked_when_recognition_fails() {
    let mut observation = valid_observation();
    observation.objects[2].asset = ObservedAsset::Declared(AssetId::Lbtc);
    let report = evaluate(&observation);
    let verdict = report.verdict(&conservation()).unwrap();

    assert!(matches!(verdict.status, RelationStatus::Blocked { .. }));
}

#[test]
fn compact_ash_sponsor_isolation_is_load_bearing() {
    for (name, mutate) in sponsor_cases() {
        let mut observation = valid_observation();
        mutate(&mut observation);
        let report = evaluate(&observation);

        assert!(
            failed(&report, &sponsor()),
            "case {name} did not fail sponsor isolation"
        );
    }
}

fn sponsor_cases() -> Vec<SponsorCase> {
    let source = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 2,
    };
    let destination = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 1,
    };
    vec![
        (
            "u mixed into open flow",
            Box::new(move |observation| {
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
        (
            "unclaimed lbtc",
            Box::new(|observation| observation.objects.push(lbtc(ObservedSide::Input, 2, 10))),
        ),
        (
            "duplicate sponsor source",
            Box::new(move |observation| {
                observation.objects.push(lbtc(ObservedSide::Input, 2, 10));
                observation.objects.push(lbtc(ObservedSide::Input, 3, 1));
                observation.open_flows.push(ObservedOpenFlow {
                    kind: architecture::OpenFlowKind::FeeSponsor,
                    sources: vec![
                        source,
                        ObservedObjectRef {
                            side: ObservedSide::Input,
                            ordinal: 3,
                        },
                    ],
                    destinations: Vec::new(),
                    fee: ProtocolAmount::new(10).unwrap(),
                });
            }),
        ),
        (
            "sponsor imbalance",
            Box::new(move |observation| {
                observation.objects.push(lbtc(ObservedSide::Input, 2, 10));
                observation.objects.push(lbtc(ObservedSide::Output, 1, 6));
                observation.open_flows.push(ObservedOpenFlow {
                    kind: architecture::OpenFlowKind::FeeSponsor,
                    sources: vec![source],
                    destinations: vec![destination],
                    fee: ProtocolAmount::new(3).unwrap(),
                });
            }),
        ),
    ]
}
