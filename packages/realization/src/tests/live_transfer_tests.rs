use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    ARCHITECTURE, AssetId, BoundId, DeltaKind, ObjectId, OperationId, ProjectionId, RootId, RootUse,
};

use crate::{
    Count, ObservedAsset, ObservedCanonicalFlow, ObservedCanonicalPartition, ObservedObject,
    ObservedObjectKind, ObservedObjectRef, ObservedOpenFlow, ObservedRootEffect,
    ObservedRootEffectKind, ObservedSide, OperationObservation, OwnerId, ProtocolAmount,
    RealizationScope, RelationId, RelationKind, RelationStatus, RelationSubject,
    RepresentationMode, TransactionSide, derive, evaluate_operation,
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
        value: crate::ObservedValue::Protocol(ProtocolAmount::new(value).unwrap()),
        owner: Some(owner),
        representation: RepresentationMode::Explicit,
    }
}

/// A sponsor member. It carries no amount: sponsor values are erased
/// by the protocol projection (S3), so a fixture cannot express one
/// even in order to test that it is ignored.
fn lbtc_owned(side: ObservedSide, ordinal: u32, owner: OwnerId) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::PlainLbtc),
        asset: ObservedAsset::Declared(AssetId::Lbtc),
        value: crate::ObservedValue::SponsorOpaque,
        owner: Some(owner),
        representation: RepresentationMode::Explicit,
    }
}

fn lbtc(side: ObservedSide, ordinal: u32) -> ObservedObject {
    lbtc_owned(side, ordinal, CAROL)
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
        canonical_partition: ObservedCanonicalPartition {
            issuances: Vec::new(),
            flows: vec![ObservedCanonicalFlow {
                asset: AssetId::U,
                sources: vec![input0],
                destinations: vec![output0, output1],
                movement_kind: Some(DeltaKind::Lateral),
                destructions: Vec::new(),
            }],
        },
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
        .push(lbtc_owned(ObservedSide::Input, 1, CAROL));
    observation
        .objects
        .push(lbtc_owned(ObservedSide::Output, 2, CAROL));
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
        lbtc_owned(ObservedSide::Input, 1, CAROL),
        lbtc_owned(ObservedSide::Input, 2, DAVE),
        lbtc_owned(ObservedSide::Output, 2, CAROL),
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
        &realization.expression_graph,
        &realization.expression_node_by_id,
        &realization.expression_evaluation_order,
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

fn sponsor_input_recognition() -> RelationId {
    relation_id(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::PlainLbtc,
        },
    )
}

fn sponsor_output_recognition() -> RelationId {
    relation_id(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::PlainLbtc,
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
        RelationKind::AllowedObjectFamilies,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    )
}

fn output_closure() -> RelationId {
    relation_id(
        RelationKind::AllowedObjectFamilies,
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
        RelationKind::CanonicalDeltaPolicy,
        RelationSubject::Projection {
            projection: ProjectionId::TransitionCertificate,
        },
    )
}

fn sponsor() -> RelationId {
    relation_id(RelationKind::SponsorIsolation, RelationSubject::Sponsor)
}

fn sponsor_multiplicity() -> RelationId {
    relation_id(
        RelationKind::SponsorEnvelopeMultiplicity,
        RelationSubject::Sponsor,
    )
}

#[test]
fn two_sponsor_envelopes_fail_multiplicity() {
    let mut observation = valid_sponsored_observation();
    // A second, disjoint, individually balanced fee-sponsor envelope: the
    // count rises to two, which the one-envelope rule forbids.
    observation.open_flows.push(ObservedOpenFlow {
        kind: architecture::OpenFlowKind::FeeSponsor,
        sources: Vec::new(),
        destinations: Vec::new(),
        fee: ProtocolAmount::ZERO,
    });

    let report = evaluate(&observation);

    assert!(
        failed(&report, &sponsor_multiplicity()),
        "a second declared fee-sponsor envelope must fail sponsor-envelope multiplicity"
    );
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

    // Break the flow arithmetic: the source value no longer sums to the
    // destinations.
    observation.objects[0].value = crate::ObservedValue::Protocol(ProtocolAmount::new(99).unwrap());

    let report = evaluate(&observation);

    assert!(
        failed(&report, &canonical_delta_policy()),
        "a canonical delta whose amount disagrees with its referenced objects must fail"
    );
}

#[test]
fn zero_value_live_input_fails_recognition() {
    let mut observation = valid_split_observation();

    observation.objects[0].value = crate::ObservedValue::Protocol(ProtocolAmount::ZERO);

    let report = evaluate(&observation);

    assert!(failed(&report, &input_recognition()));
}

#[test]
fn zero_value_live_output_fails_recognition() {
    let mut observation = valid_split_observation();

    observation.objects[1].value = crate::ObservedValue::Protocol(ProtocolAmount::ZERO);

    let report = evaluate(&observation);

    assert!(failed(&report, &output_recognition()));
}

#[test]
fn an_unwitnessed_time_locked_receipt_fails_partition_membership() {
    // A time-locked receipt is a canonical-value family this pilot
    // never moves, so the retired family-pinned membership check
    // ignored it entirely. Partition exactness covers every
    // canonical-value object: a U output outside every flow and
    // issuance is unwitnessed and fails the policy.
    let mut observation = valid_split_observation();

    observation.objects.push(ObservedObject {
        reference: ObservedObjectRef {
            side: ObservedSide::Output,
            ordinal: 2,
        },
        kind: ObservedObjectKind::Declared(ObjectId::ReceiptTimeLocked),
        asset: ObservedAsset::Declared(AssetId::U),
        value: crate::ObservedValue::Protocol(ProtocolAmount::new(5).unwrap()),
        owner: Some(BOB),
        representation: RepresentationMode::Explicit,
    });

    let report = evaluate(&observation);
    assert!(failed(&report, &canonical_delta_policy()));
}

#[test]
fn zero_amount_lateral_delta_fails_policy() {
    let mut observation = valid_split_observation();

    // A movement flow whose destination total is zero fails the
    // movement-kind rule.
    observation.objects[1].value = crate::ObservedValue::Protocol(ProtocolAmount::ZERO);
    observation.objects[2].value = crate::ObservedValue::Protocol(ProtocolAmount::ZERO);

    let report = evaluate(&observation);
    assert!(failed(&report, &canonical_delta_policy()));
}

#[test]
fn sponsor_input_without_owner_fails_recognition() {
    let mut observation = valid_sponsored_observation();

    observation
        .objects
        .iter_mut()
        .find(|object| {
            object.reference.side == ObservedSide::Input
                && object.kind == ObservedObjectKind::Declared(ObjectId::PlainLbtc)
        })
        .expect("fixture has a sponsor input")
        .owner = None;

    let report = evaluate(&observation);
    assert!(failed(&report, &sponsor_input_recognition()));
}

#[test]
fn sponsor_change_without_owner_fails_recognition() {
    let mut observation = valid_sponsored_observation();

    observation
        .objects
        .iter_mut()
        .find(|object| {
            object.reference.side == ObservedSide::Output
                && object.kind == ObservedObjectKind::Declared(ObjectId::PlainLbtc)
        })
        .expect("fixture has sponsor change")
        .owner = None;

    let report = evaluate(&observation);
    assert!(failed(&report, &sponsor_output_recognition()));
}

// Sponsor-value opacity (F2-006), mirroring the compact-ASH
// battery: the amount of an ordinary sponsor member is never a
// recognition operand; role structure and conservation carry the load.

/// A fully zero sponsor sidecar on the live transfer: one zero-valued
/// signed input, one zero-valued change output, fee zero, claimed.
fn zero_sidecar_observation() -> OperationObservation {
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
        .push(lbtc_owned(ObservedSide::Input, 1, CAROL));
    observation
        .objects
        .push(lbtc_owned(ObservedSide::Output, 2, CAROL));
    observation.open_flows.push(ObservedOpenFlow {
        kind: architecture::OpenFlowKind::FeeSponsor,
        sources: vec![sponsor_input],
        destinations: vec![sponsor_change],
        fee: ProtocolAmount::ZERO,
    });
    observation.sponsor_signers.insert(CAROL);

    observation
}

#[test]
fn zero_sponsor_sidecar_is_accepted() {
    let report = evaluate(&zero_sidecar_observation());

    assert!(
        report.is_conformant(),
        "failed relations: {:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
}

#[test]
fn mixed_zero_and_positive_sponsor_members_are_accepted() {
    let mut observation = valid_sponsored_observation();

    let zero_input = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 3,
    };
    observation
        .objects
        .push(lbtc_owned(ObservedSide::Input, 3, CAROL));
    observation.open_flows[0].sources.push(zero_input);

    let report = evaluate(&observation);

    assert!(
        report.is_conformant(),
        "failed relations: {:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
}

// S3: sponsor conservation belongs to the substrate. Elements
// validates that a transaction's inputs and outputs balance, so this
// layer does not re-derive it — and doing so cost a read of exactly
// the amounts sponsor erasure removes from the protocol read-set.
// Zeroing a sponsor amount is therefore recognized AND isolated: the
// role structure is exact, and any imbalance is the base layer's to
// reject. Previously these two cases asserted the opposite.
#[test]
fn a_sponsor_input_carrying_a_protocol_amount_fails_recognition() {
    // S3: sponsor amounts are erased at the projection boundary, so a
    // sponsor member has no amount to zero — the earlier version of
    // this test set one. An observation that smuggles a readable
    // amount into a sponsor object is malformed, and recognition says
    // so rather than quietly accepting it.
    let mut observation = valid_sponsored_observation();

    observation
        .objects
        .iter_mut()
        .find(|object| {
            object.reference.side == ObservedSide::Input
                && object.kind == ObservedObjectKind::Declared(ObjectId::PlainLbtc)
        })
        .expect("fixture has a sponsor member")
        .value = crate::ObservedValue::Protocol(ProtocolAmount::ZERO);

    assert!(failed(
        &evaluate(&observation),
        &sponsor_input_recognition()
    ));
}

#[test]
fn a_sponsor_output_carrying_a_protocol_amount_fails_recognition() {
    // S3: sponsor amounts are erased at the projection boundary, so a
    // sponsor member has no amount to zero — the earlier version of
    // this test set one. An observation that smuggles a readable
    // amount into a sponsor object is malformed, and recognition says
    // so rather than quietly accepting it.
    let mut observation = valid_sponsored_observation();

    observation
        .objects
        .iter_mut()
        .find(|object| {
            object.reference.side == ObservedSide::Output
                && object.kind == ObservedObjectKind::Declared(ObjectId::PlainLbtc)
        })
        .expect("fixture has a sponsor member")
        .value = crate::ObservedValue::Protocol(ProtocolAmount::ZERO);

    assert!(failed(
        &evaluate(&observation),
        &sponsor_output_recognition()
    ));
}

#[test]
fn zero_sponsor_input_without_signature_fails_isolation() {
    let mut observation = zero_sidecar_observation();

    observation.sponsor_signers.clear();

    let report = evaluate(&observation);
    assert!(failed(&report, &sponsor()));
}

#[test]
fn unclaimed_zero_sponsor_member_fails_isolation() {
    // Opacity is not omission: an owned zero-valued sponsor member
    // outside every sponsor flow fails exact membership.
    let mut observation = valid_split_observation();
    observation
        .objects
        .push(lbtc_owned(ObservedSide::Input, 1, CAROL));
    observation.sponsor_signers.insert(CAROL);

    let report = evaluate(&observation);
    assert!(failed(&report, &sponsor()));
}

#[test]
fn zero_value_ownerless_plain_lbtc_still_fails_recognition() {
    // Value zero does not turn an ordinary object into an anchor: an
    // ownerless PLAIN_LBTC is malformed whatever its amount.
    let mut observation = valid_split_observation();
    observation.objects.push(ObservedObject {
        reference: ObservedObjectRef {
            side: ObservedSide::Input,
            ordinal: 1,
        },
        kind: ObservedObjectKind::Declared(ObjectId::PlainLbtc),
        asset: ObservedAsset::Declared(AssetId::Lbtc),
        value: crate::ObservedValue::Protocol(ProtocolAmount::ZERO),
        owner: None,
        representation: RepresentationMode::Explicit,
    });

    let report = evaluate(&observation);
    assert!(failed(&report, &sponsor_input_recognition()));
}

#[test]
fn canonical_delta_empty_duplicate_fails() {
    let mut missing = valid_split_observation();
    missing.canonical_partition.flows.clear();

    let missing_report = evaluate(&missing);

    assert!(
        failed(&missing_report, &canonical_delta_policy()),
        "live transfer without its required lateral U delta must fail"
    );

    let realization = pilot_realization();
    let mut duplicated = valid_split_observation();
    duplicated
        .canonical_partition
        .flows
        .push(duplicated.canonical_partition.flows[0].clone());

    let error = evaluate_operation(
        &realization.relation_graph,
        &realization.relation_node_by_id,
        &realization.relation_evaluation_order,
        &realization.expression_graph,
        &realization.expression_node_by_id,
        &realization.expression_evaluation_order,
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
            Box::new(|observation| {
                observation.objects[1].value =
                    crate::ObservedValue::Protocol(ProtocolAmount::new(41).unwrap());
            }),
            conservation(),
        ),
        (
            "wrong canonical delta kind",
            Box::new(|observation| {
                observation.canonical_partition.flows[0].movement_kind =
                    Some(DeltaKind::OwnerlessLateral);
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
                    effect: ObservedRootEffectKind::Succession,
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
                observation.objects.push(lbtc(ObservedSide::Input, 1));
                observation.objects.push(lbtc(ObservedSide::Output, 2));
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

// Weld mutation coverage (review finding 2), mirroring the compact-ASH
// suite: the live-transfer weld must accept the published architecture
// and observably reject mutations of the operation fields whose pilot
// value is empty or a default.

#[test]
fn live_transfer_weld_accepts_the_published_architecture() {
    crate::validate::validate_live_transfer_architecture(&ARCHITECTURE)
        .expect("the live-transfer weld must accept the published architecture");
}

/// A copy of the published architecture with the live-transfer
/// operation row rewritten by `mutate`.
fn live_transfer_mutated(
    mutate: impl FnOnce(&mut architecture::OperationSpec),
) -> architecture::Architecture {
    let mut mutated = ARCHITECTURE;

    let mut operations = mutated.operations.to_vec();
    let operation = operations
        .iter_mut()
        .find(|operation| operation.id == OperationId::TransferLive)
        .expect("the architecture declares live transfer");
    mutate(operation);
    mutated.operations = operations.leak();

    mutated
}

fn weld_rejects(
    architecture: &architecture::Architecture,
    field: crate::ArchitectureMismatchField,
) {
    assert_eq!(
        crate::validate::validate_live_transfer_architecture(architecture),
        Err(crate::RealizationError::ArchitectureOperationMismatch {
            operation: OperationId::TransferLive,
            field,
        }),
    );
}

#[test]
fn live_transfer_weld_rejects_a_client_protocol_reclassification() {
    let mutated = live_transfer_mutated(|operation| {
        operation.kind = architecture::OperationKind::ClientProtocol;
    });

    weld_rejects(&mutated, crate::ArchitectureMismatchField::OperationKind);
}

#[test]
fn live_transfer_weld_rejects_a_new_issuance() {
    let mutated = live_transfer_mutated(|operation| {
        operation.issuances = vec![architecture::IssuanceSpec {
            asset: AssetId::U,
            authority: AssetId::U,
            condition: architecture::IssuanceCondition::PositiveAdmittedPrincipal,
        }]
        .leak();
    });

    weld_rejects(&mutated, crate::ArchitectureMismatchField::Issuances);
}

#[test]
fn live_transfer_weld_rejects_a_new_quantity_read() {
    let mutated = live_transfer_mutated(|operation| {
        operation.reads = vec![architecture::QuantityId::Floor].leak();
    });

    weld_rejects(&mutated, crate::ArchitectureMismatchField::Reads);
}

#[test]
fn live_transfer_weld_rejects_a_new_quantity_write() {
    let mutated = live_transfer_mutated(|operation| {
        operation.writes = vec![architecture::QuantityId::Floor].leak();
    });

    weld_rejects(&mutated, crate::ArchitectureMismatchField::Writes);
}

type WeldMutationCase = (
    &'static str,
    Box<dyn Fn(&mut architecture::OperationSpec)>,
    crate::ArchitectureMismatchField,
);

/// One focused mutation per welded field family (F2-002), mirroring
/// the compact-ASH table for the live-transfer row.
#[allow(clippy::too_many_lines)]
fn weld_mutation_cases() -> Vec<WeldMutationCase> {
    use crate::ArchitectureMismatchField as Field;

    fn edit_input(
        operation: &mut architecture::OperationSpec,
        object: ObjectId,
        edit: impl Fn(&mut architecture::InputSpec),
    ) {
        let mut inputs = operation.inputs.to_vec();
        edit(
            inputs
                .iter_mut()
                .find(|input| input.object == object)
                .expect("the pilot declares the input family"),
        );
        operation.inputs = inputs.leak();
    }

    fn edit_output(
        operation: &mut architecture::OperationSpec,
        object: ObjectId,
        edit: impl Fn(&mut architecture::OutputSpec),
    ) {
        let mut outputs = operation.outputs.to_vec();
        edit(
            outputs
                .iter_mut()
                .find(|output| output.object == object)
                .expect("the pilot declares the output family"),
        );
        operation.outputs = outputs.leak();
    }

    vec![
        (
            "primary authorization",
            Box::new(|operation| {
                operation.authorization = architecture::PermissionClass::Operator;
            }),
            Field::Authorization,
        ),
        (
            "receipt input authorization",
            Box::new(|operation| {
                edit_input(operation, ObjectId::ReceiptLive, |input| {
                    input.authorization = architecture::InputAuthorization::Permissionless;
                });
            }),
            Field::ReceiptInput,
        ),
        (
            "receipt input minimum",
            Box::new(|operation| {
                edit_input(operation, ObjectId::ReceiptLive, |input| input.minimum = 2);
            }),
            Field::ReceiptInput,
        ),
        (
            "receipt output maximum",
            Box::new(|operation| {
                edit_output(operation, ObjectId::ReceiptLive, |output| {
                    output.maximum = architecture::MaxCount::Exact(1);
                });
            }),
            Field::ReceiptOutput,
        ),
        (
            "sponsor input maximum",
            Box::new(|operation| {
                edit_input(operation, ObjectId::PlainLbtc, |input| {
                    input.maximum = architecture::MaxCount::Exact(4);
                });
            }),
            Field::SponsorInput,
        ),
        (
            "sponsor output maximum",
            Box::new(|operation| {
                edit_output(operation, ObjectId::PlainLbtc, |output| {
                    output.maximum = architecture::MaxCount::Exact(2);
                });
            }),
            Field::SponsorOutput,
        ),
        (
            "added input family",
            Box::new(|operation| {
                let mut inputs = operation.inputs.to_vec();
                inputs.push(architecture::InputSpec {
                    object: ObjectId::State,
                    minimum: 1,
                    maximum: architecture::MaxCount::Exact(1),
                    authorization: architecture::InputAuthorization::CovenantCompanion,
                });
                operation.inputs = inputs.leak();
            }),
            Field::InputFamilies,
        ),
        (
            "removed input family",
            Box::new(|operation| {
                let mut inputs = operation.inputs.to_vec();
                inputs.retain(|input| input.object == ObjectId::ReceiptLive);
                operation.inputs = inputs.leak();
            }),
            Field::InputFamilies,
        ),
        (
            "added output family",
            Box::new(|operation| {
                let mut outputs = operation.outputs.to_vec();
                outputs.push(architecture::OutputSpec {
                    object: ObjectId::State,
                    minimum: 1,
                    maximum: architecture::MaxCount::Exact(1),
                });
                operation.outputs = outputs.leak();
            }),
            Field::OutputFamilies,
        ),
        (
            "changed bound set",
            Box::new(|operation| {
                operation.bounds =
                    vec![BoundId::TransferInputMax, BoundId::FeeSponsorInputMax].leak();
            }),
            Field::Bounds,
        ),
        (
            "delta kind",
            Box::new(|operation| {
                let mut deltas = operation.canonical_deltas.to_vec();
                deltas[0].kind = DeltaKind::OwnerlessLateral;
                operation.canonical_deltas = deltas.leak();
            }),
            Field::CanonicalDeltas,
        ),
        (
            "delta condition",
            Box::new(|operation| {
                let mut deltas = operation.canonical_deltas.to_vec();
                deltas[0].condition = architecture::DeltaCondition::PositiveAdmittedPrincipal;
                operation.canonical_deltas = deltas.leak();
            }),
            Field::CanonicalDeltas,
        ),
        (
            "delta destruction tag",
            Box::new(|operation| {
                let mut deltas = operation.canonical_deltas.to_vec();
                deltas[0].destruction_tag = Some(architecture::TagId::Burn);
                operation.canonical_deltas = deltas.leak();
            }),
            Field::CanonicalDeltas,
        ),
        (
            "extra canonical delta",
            Box::new(|operation| {
                let mut deltas = operation.canonical_deltas.to_vec();
                deltas.push(deltas[0]);
                operation.canonical_deltas = deltas.leak();
            }),
            Field::CanonicalDeltas,
        ),
        (
            "added data output",
            Box::new(|operation| {
                operation.data_outputs = vec![architecture::DataOutputSpec {
                    kind: architecture::DataOutputKind::Destruction,
                    tag: architecture::TagId::Burn,
                    asset: Some(AssetId::U),
                    minimum: 1,
                    maximum: architecture::MaxCount::Exact(1),
                    condition: architecture::DeltaCondition::Always,
                }]
                .leak();
            }),
            Field::DataOutputs,
        ),
        (
            "removed open flow",
            Box::new(|operation| {
                operation.open_flows = vec![].leak();
            }),
            Field::OpenFlows,
        ),
        (
            "removed value-flow class",
            Box::new(|operation| {
                operation.value_flows = vec![architecture::ValueFlowClass::OwnerConsented].leak();
            }),
            Field::ValueFlows,
        ),
        (
            "removed witness",
            Box::new(|operation| {
                let mut witnesses = operation.witnesses.to_vec();
                witnesses
                    .retain(|witness| *witness != architecture::WitnessId::ReceiptClassClosure);
                operation.witnesses = witnesses.leak();
            }),
            Field::Witnesses,
        ),
        (
            "changed root use",
            Box::new(|operation| {
                operation.roots = vec![architecture::RootUseSpec {
                    root: RootId::ALL[0],
                    use_kind: RootUse::Succession,
                }]
                .leak();
            }),
            Field::RootPolicy,
        ),
        (
            "changed projection rule",
            Box::new(|operation| {
                operation.projections = vec![].leak();
            }),
            Field::ProjectionPolicy,
        ),
    ]
}

#[test]
fn live_transfer_weld_rejects_every_field_mutation() {
    for (name, mutate, field) in weld_mutation_cases() {
        let mutated = live_transfer_mutated(|operation| mutate(operation));

        assert_eq!(
            crate::validate::validate_live_transfer_architecture(&mutated),
            Err(crate::RealizationError::ArchitectureOperationMismatch {
                operation: OperationId::TransferLive,
                field,
            }),
            "mutation case {name}",
        );
    }
}

#[test]
fn a_draft_valid_quantity_write_is_still_rejected_by_the_weld() {
    // The write-side counterpart of the compact-ASH read regression: a
    // new quantity write with its reciprocal writer declaration passes
    // architecture draft validation, so only the weld rejects it.
    let mut mutated = live_transfer_mutated(|operation| {
        operation.writes = vec![architecture::QuantityId::Floor].leak();
    });

    let mut quantities = mutated.quantities.to_vec();
    let quantity = quantities
        .iter_mut()
        .find(|quantity| quantity.id == architecture::QuantityId::Floor)
        .expect("the architecture declares the floor quantity");
    let mut writers = quantity.writers.to_vec();
    writers.push(OperationId::TransferLive);
    quantity.writers = writers.leak();
    mutated.quantities = quantities.leak();

    architecture::validate_draft(&mutated).expect("a reciprocal quantity write is draft-valid");

    weld_rejects(&mutated, crate::ArchitectureMismatchField::Writes);
}

// --- T2: the substrate-conservation premise is typed, never passed. ---

#[test]
fn live_transfer_substrate_conservation_is_evidence_required_never_passed() {
    for observation in [valid_split_observation(), valid_sponsored_observation()] {
        let report = evaluate(&observation);
        let verdict = report
            .verdict(&relation_id(
                RelationKind::SubstrateConservation,
                RelationSubject::Asset {
                    asset: AssetId::Lbtc,
                },
            ))
            .unwrap();

        assert!(matches!(
            verdict.status,
            RelationStatus::EvidenceRequired {
                requirement: crate::ExternalEvidenceRequirement::SubstrateConservation {
                    operation: OperationId::TransferLive,
                    asset: AssetId::Lbtc,
                },
            }
        ));
        assert!(!report.has_semantic_failure());
        assert!(!report.is_evidence_complete());
    }
}
