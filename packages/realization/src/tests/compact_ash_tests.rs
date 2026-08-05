use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    ARCHITECTURE, AssetId, BoundId, DeltaKind, ObjectId, OperationId, ProjectionId, RootId, RootUse,
};

use crate::{
    Count, ObservedAsset, ObservedCanonicalFlow, ObservedCanonicalPartition, ObservedObject,
    ObservedObjectKind, ObservedObjectRef, ObservedOpenFlow, ObservedRootEffect, ObservedSide,
    OperationObservation, OwnerId, ProtocolAmount, RealizationScope, RelationId, RelationKind,
    RelationStatus, RelationSubject, RepresentationMode, TransactionSide, derive,
    evaluate_operation,
};

type ObservationMutation = Box<dyn Fn(&mut OperationObservation)>;
type RelationCase = (&'static str, ObservationMutation, RelationId);
type SponsorCase = (&'static str, ObservationMutation);

const CAROL: OwnerId = OwnerId([3_u8; 32]);

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
        value: crate::ObservedValue::Protocol(ProtocolAmount::new(value).unwrap()),
        owner: None,
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

/// A sponsor member deliberately left out of the sponsor flow.
fn unclaimed_lbtc(side: ObservedSide, ordinal: u32) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::PlainLbtc),
        asset: ObservedAsset::Declared(AssetId::Lbtc),
        value: crate::ObservedValue::SponsorOpaque,
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
        canonical_partition: ObservedCanonicalPartition {
            issuances: Vec::new(),
            flows: vec![ObservedCanonicalFlow {
                asset: AssetId::U,
                sources: vec![input0, input1],
                destinations: vec![output0],
                movement_kind: Some(DeltaKind::OwnerlessLateral),
                destructions: Vec::new(),
            }],
        },
        open_flows: Vec::new(),
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::AshBatchMax, Count::new(64)),
            (BoundId::FeeSponsorInputMax, Count::new(16)),
        ]),
    }
}

fn valid_sponsored_observation() -> OperationObservation {
    let mut observation = valid_observation();
    let sponsor_input = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 2,
    };
    let sponsor_change = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 1,
    };

    observation
        .objects
        .push(lbtc_owned(ObservedSide::Input, 2, CAROL));
    observation
        .objects
        .push(lbtc_owned(ObservedSide::Output, 1, CAROL));
    observation.open_flows.push(ObservedOpenFlow {
        kind: architecture::OpenFlowKind::FeeSponsor,
        sources: vec![sponsor_input],
        destinations: vec![sponsor_change],
        fee: ProtocolAmount::new(3).unwrap(),
    });
    observation.sponsor_signers.insert(CAROL);

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

fn open_flow_policy() -> RelationId {
    relation_id(
        RelationKind::OpenFlowPolicy,
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
        RelationKind::AllowedObjectFamilies,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
    )
}

fn output_closure() -> RelationId {
    relation_id(
        RelationKind::AllowedObjectFamilies,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::Ash,
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

fn substrate_conservation() -> RelationId {
    relation_id(
        RelationKind::SubstrateConservation,
        RelationSubject::Asset {
            asset: AssetId::Lbtc,
        },
    )
}

// --- T2: the substrate-conservation premise is typed, never passed. ---
//
// The former "sponsor imbalance" fixture was really a role-structure
// failure (an unowned source); value imbalance itself is invisible to
// the sponsor-erased evaluator by design. These tests pin the honest
// shape of that claim: a caller-authored observation can pass sponsor
// role isolation, but the whole-transaction value equation stays an
// explicit external-evidence requirement.

#[test]
fn substrate_conservation_is_evidence_required_never_passed() {
    for observation in [valid_observation(), valid_sponsored_observation()] {
        let report = evaluate(&observation);
        let verdict = report.verdict(&substrate_conservation()).unwrap();

        assert!(matches!(
            verdict.status,
            RelationStatus::EvidenceRequired {
                requirement: crate::ExternalEvidenceRequirement::SubstrateConservation {
                    operation: OperationId::CompactAsh,
                    asset: AssetId::Lbtc,
                },
            }
        ));
        assert!(!report.has_semantic_failure());
        assert!(!report.is_evidence_complete());
        assert!(report.is_conformant());
    }
}

#[test]
fn an_empty_sponsor_flow_with_nonzero_fee_cannot_appear_fully_proven() {
    // The malformed shape from the static review: one fee-sponsor flow,
    // zero sources, zero destinations, nonzero fee. Role isolation has
    // nothing to reject structurally — the report must therefore still
    // carry the undischarged conservation requirement rather than
    // claiming the transaction proven.
    let mut observation = valid_observation();
    observation.open_flows.push(ObservedOpenFlow {
        kind: architecture::OpenFlowKind::FeeSponsor,
        sources: Vec::new(),
        destinations: Vec::new(),
        fee: ProtocolAmount::new(7).unwrap(),
    });

    let report = evaluate(&observation);

    assert!(!failed(&report, &sponsor()));
    assert!(!report.is_evidence_complete());
    assert!(matches!(
        report.verdict(&substrate_conservation()).unwrap().status,
        RelationStatus::EvidenceRequired { .. }
    ));
}

#[test]
fn failed_sponsor_isolation_blocks_the_conservation_requirement() {
    let mut observation = valid_sponsored_observation();
    observation.sponsor_signers.clear();

    let report = evaluate(&observation);

    assert!(failed(&report, &sponsor()));
    assert!(matches!(
        report.verdict(&substrate_conservation()).unwrap().status,
        RelationStatus::Blocked { .. }
    ));
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
fn compact_ash_weld_accepts_the_published_architecture() {
    // The compact-ASH weld now checks every architecture field exactly
    // (sponsor input/output, bound/flow/witness sets, no data outputs);
    // it must still accept the real published architecture.
    crate::validate::validate_compact_ash_architecture(&ARCHITECTURE)
        .expect("the compact-ASH weld must accept the published architecture");
}

#[test]
fn compact_ash_canonical_delta_amount_mutation_fails() {
    let mut observation = valid_observation();

    // Break the flow arithmetic: the destination value no longer sums to
    // the sources.
    observation.objects[2].value = crate::ObservedValue::Protocol(ProtocolAmount::new(99).unwrap());

    let report = evaluate(&observation);

    assert!(
        failed(&report, &canonical_delta_policy()),
        "a compact-ASH canonical delta whose amount disagrees with its referenced objects must fail"
    );
}

#[test]
fn zero_value_ash_input_fails_recognition() {
    let mut observation = valid_observation();

    observation.objects[0].value = crate::ObservedValue::Protocol(ProtocolAmount::ZERO);

    let report = evaluate(&observation);

    assert!(failed(&report, &input_recognition()));
}

#[test]
fn zero_value_ash_output_fails_recognition() {
    let mut observation = valid_observation();

    observation.objects[2].value = crate::ObservedValue::Protocol(ProtocolAmount::ZERO);

    let report = evaluate(&observation);

    assert!(failed(&report, &output_recognition()));
}

#[test]
fn ash_with_owner_fails_recognition() {
    let mut observation = valid_observation();

    observation.objects[2].owner = Some(crate::OwnerId([9_u8; 32]));

    let report = evaluate(&observation);

    assert!(failed(&report, &output_recognition()));
}

#[test]
fn empty_flow_fails_policy() {
    let mut observation = valid_observation();

    observation
        .canonical_partition
        .flows
        .push(ObservedCanonicalFlow {
            asset: AssetId::U,
            sources: Vec::new(),
            destinations: Vec::new(),
            movement_kind: None,
            destructions: Vec::new(),
        });

    let report = evaluate(&observation);
    assert!(failed(&report, &canonical_delta_policy()));
}

#[test]
fn a_mixed_movement_and_destruction_flow_is_representable() {
    // A partial-clear-shaped flow: two ASH sources, a residual ASH
    // destination, and a reconciliation destruction leg — a movement and
    // a destruction sharing one source set (40 + 60 = 50 + 50). The old
    // flattened representation rejected this as a partition overlap; the
    // exact partition accepts it structurally.
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

    let observation = OperationObservation {
        operation: OperationId::CompactAsh,
        objects: vec![
            ash(ObservedSide::Input, 0, 40),
            ash(ObservedSide::Input, 1, 60),
            ash(ObservedSide::Output, 0, 50),
        ],
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::new(),
        canonical_partition: ObservedCanonicalPartition {
            issuances: Vec::new(),
            flows: vec![ObservedCanonicalFlow {
                asset: AssetId::U,
                sources: vec![input0, input1],
                destinations: vec![output0],
                movement_kind: Some(DeltaKind::OwnerlessLateral),
                destructions: vec![crate::ObservedDestructionLeg {
                    tag: architecture::TagId::Recon,
                    amount: ProtocolAmount::new(50).unwrap(),
                }],
            }],
        },
        open_flows: Vec::new(),
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::AshBatchMax, Count::new(64)),
            (BoundId::FeeSponsorInputMax, Count::new(16)),
        ]),
    };

    let normalized =
        crate::validate_observation(observation).expect("mixed flow is structurally valid");
    let flow = &normalized.canonical_partition.flows[0];
    assert!(flow.movement_kind.is_some());
    assert_eq!(flow.destructions.len(), 1);
}

#[test]
fn reusing_a_source_across_flows_is_a_partition_overlap() {
    // Between flows, a source is used by exactly one flow. Two flows
    // sharing input 0 is a hard overlap — the rule the flattened
    // representation could not enforce independently of within-flow
    // sharing.
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

    let flow = |destination: ObservedObjectRef| ObservedCanonicalFlow {
        asset: AssetId::U,
        sources: vec![input0],
        destinations: vec![destination],
        movement_kind: Some(DeltaKind::OwnerlessLateral),
        destructions: Vec::new(),
    };

    let observation = OperationObservation {
        operation: OperationId::CompactAsh,
        objects: vec![
            ash(ObservedSide::Input, 0, 100),
            ash(ObservedSide::Output, 0, 60),
            ash(ObservedSide::Output, 1, 40),
        ],
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::new(),
        canonical_partition: ObservedCanonicalPartition {
            issuances: Vec::new(),
            flows: vec![flow(output0), flow(output1)],
        },
        open_flows: Vec::new(),
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::AshBatchMax, Count::new(64)),
            (BoundId::FeeSponsorInputMax, Count::new(16)),
        ]),
    };

    assert!(matches!(
        crate::validate_observation(observation),
        Err(crate::RealizationError::ObservedCanonicalPartitionOverlap)
    ));
}

#[test]
fn an_unwitnessed_distribution_vault_output_fails_partition_membership() {
    // The vault is a canonical-value family this pilot never moves, so
    // the retired family-pinned membership check ignored it entirely.
    // Partition exactness covers every canonical-value object: a U
    // output outside every flow and issuance is unwitnessed and fails
    // the policy regardless of which object family carries it.
    let mut observation = valid_observation();

    observation.objects.push(ObservedObject {
        reference: ObservedObjectRef {
            side: ObservedSide::Output,
            ordinal: 1,
        },
        kind: ObservedObjectKind::Declared(ObjectId::DistributionVault),
        asset: ObservedAsset::Declared(AssetId::U),
        value: crate::ObservedValue::Protocol(ProtocolAmount::new(5).unwrap()),
        owner: None,
        representation: RepresentationMode::Explicit,
    });

    let report = evaluate(&observation);
    assert!(failed(&report, &canonical_delta_policy()));
}

#[test]
fn zero_amount_ownerless_lateral_delta_fails_policy() {
    let mut observation = valid_observation();

    // A movement flow whose destination total is zero fails the
    // movement-kind rule.
    observation.objects[2].value = crate::ObservedValue::Protocol(ProtocolAmount::ZERO);

    let report = evaluate(&observation);
    assert!(failed(&report, &canonical_delta_policy()));
}

#[test]
fn valid_sponsored_compact_ash_conforms() {
    let report = evaluate(&valid_sponsored_observation());

    assert!(
        report.is_conformant(),
        "failed relations: {:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
}

// Sponsor-value opacity (F2-006): a sponsor amount is never a
// protocol-readable fact, so a zero-valued PLAIN_LBTC member is an
// ordinary sponsor object. What remains load-bearing is the role
// structure — family recognition, owner authorization, exact
// membership, and conservation — never the amount itself.

/// A fully zero sponsor sidecar: one zero-valued signed input, one
/// zero-valued change output, fee zero, exactly claimed.
fn zero_sidecar_observation() -> OperationObservation {
    let mut observation = valid_observation();
    let sponsor_input = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 2,
    };
    let sponsor_change = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 1,
    };

    observation
        .objects
        .push(lbtc_owned(ObservedSide::Input, 2, CAROL));
    observation
        .objects
        .push(lbtc_owned(ObservedSide::Output, 1, CAROL));
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

    // A second, zero-valued signed sponsor input joins the existing
    // 10 → 7 + 3 envelope; the balance is unchanged and every member
    // is claimed.
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

    // A zero-valued input still consumes an outpoint: its owner must
    // authorize the transaction like any other sponsor.
    observation.sponsor_signers.clear();

    let report = evaluate(&observation);
    assert!(failed(&report, &sponsor()));
}

#[test]
fn unclaimed_zero_sponsor_members_fail_isolation() {
    // Opacity is not omission: an owned zero-valued sponsor member
    // outside every sponsor flow fails exact membership.
    for side in [ObservedSide::Input, ObservedSide::Output] {
        let mut observation = valid_observation();
        let ordinal = match side {
            ObservedSide::Input => 2,
            ObservedSide::Output => 1,
        };
        observation.objects.push(lbtc_owned(side, ordinal, CAROL));
        observation.sponsor_signers.insert(CAROL);

        let report = evaluate(&observation);
        assert!(
            failed(&report, &sponsor()),
            "unclaimed zero sponsor member on {side:?} must fail isolation"
        );
    }
}

#[test]
fn zero_value_ownerless_plain_lbtc_still_fails_recognition() {
    // Value zero does not turn an ordinary output into an anchor: an
    // ownerless PLAIN_LBTC is malformed whatever its amount, and the
    // anchor is a different declared family entirely.
    let mut observation = valid_observation();
    observation
        .objects
        .push(unclaimed_lbtc(ObservedSide::Input, 2));

    let report = evaluate(&observation);
    assert!(failed(&report, &sponsor_input_recognition()));
}

#[test]
fn declared_anchor_fails_the_pilot_family_closure() {
    // The converse anchor-identity direction: a declared CPFP_ANCHOR
    // is recognized structurally and rejected here because the pilot's
    // family closure has no anchor slot — not because of its value.
    let mut observation = valid_observation();
    observation.objects.push(crate::ObservedObject {
        reference: ObservedObjectRef {
            side: ObservedSide::Output,
            ordinal: 1,
        },
        kind: ObservedObjectKind::Declared(ObjectId::CpfpAnchor),
        asset: ObservedAsset::Declared(AssetId::Lbtc),
        value: crate::ObservedValue::Protocol(ProtocolAmount::ZERO),
        owner: None,
        representation: RepresentationMode::Explicit,
    });

    let report = evaluate(&observation);
    assert!(!report.is_conformant());
}

#[test]
fn balanced_theft_is_rejected_by_the_protocol_relation() {
    // Protocol value short by one, sponsor change high by one: the
    // whole transaction still conserves and every sponsor amount is
    // positive, so positivity would never catch it. The pinned
    // protocol relation does.
    let mut observation = valid_sponsored_observation();

    // The theft: one unit is taken from the protocol output. The
    // matching enlargement of sponsor change, which made the whole
    // transaction still balance, can no longer be written at all —
    // sponsor objects carry no amount (S3). That is the point: the
    // vector is rejected because the protocol relation pins the
    // canonical delta, never because sponsor arithmetic was inspected.
    let stolen = observation
        .objects
        .iter_mut()
        .find(|object| {
            object.reference.side == ObservedSide::Output
                && object.kind == ObservedObjectKind::Declared(ObjectId::Ash)
        })
        .expect("fixture has the ASH output");
    let taken = stolen
        .value
        .protocol()
        .expect("a protocol output carries a readable amount");
    stolen.value = crate::ObservedValue::Protocol(ProtocolAmount::new(taken.get() - 1).unwrap());

    let report = evaluate(&observation);
    assert!(failed(&report, &canonical_delta_policy()));
}

#[test]
fn sponsor_shapes_do_not_interfere_with_protocol_verdicts() {
    // Noninterference: an otherwise identical transition with no
    // sponsor, a positive sponsor envelope, a mixed envelope, or a
    // zero sidecar must produce identical verdicts on every relation
    // that is not sponsor-local.
    let sponsor_free = |relation: &crate::RelationId| {
        !matches!(
            relation.subject(),
            RelationSubject::ObjectFamily {
                object: ObjectId::PlainLbtc,
                ..
            } | RelationSubject::Sponsor
        )
    };

    let mut mixed = valid_sponsored_observation();
    let zero_input = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 3,
    };
    mixed
        .objects
        .push(lbtc_owned(ObservedSide::Input, 3, CAROL));
    mixed.open_flows[0].sources.push(zero_input);

    let shapes = [
        ("no sponsor", valid_observation()),
        ("positive sponsor", valid_sponsored_observation()),
        ("mixed sponsor", mixed),
        ("zero sidecar", zero_sidecar_observation()),
    ];

    let baseline: Vec<_> = evaluate(&shapes[0].1)
        .verdicts
        .iter()
        .filter(|verdict| sponsor_free(&verdict.relation))
        .cloned()
        .collect();

    for (name, observation) in &shapes {
        let report = evaluate(observation);
        assert!(
            report.is_conformant(),
            "shape {name} failed: {:?}",
            report.failed_relations().collect::<Vec<_>>()
        );

        let projected: Vec<_> = report
            .verdicts
            .iter()
            .filter(|verdict| sponsor_free(&verdict.relation))
            .cloned()
            .collect();
        assert_eq!(projected, baseline, "shape {name} moved a protocol verdict");
    }
}

#[test]
fn ownerless_plain_lbtc_fails_recognition() {
    let mut observation = valid_observation();
    observation
        .objects
        .push(unclaimed_lbtc(ObservedSide::Input, 2));

    let report = evaluate(&observation);
    assert!(failed(&report, &sponsor_input_recognition()));
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
            Box::new(|observation| {
                observation.objects[2].value =
                    crate::ObservedValue::Protocol(ProtocolAmount::new(99).unwrap());
            }),
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
                observation.canonical_partition.flows[0].movement_kind = Some(DeltaKind::Lateral);
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
            Box::new(|observation| {
                observation
                    .objects
                    .push(unclaimed_lbtc(ObservedSide::Input, 2));
            }),
        ),
        (
            "duplicate sponsor source",
            Box::new(move |observation| {
                observation
                    .objects
                    .push(unclaimed_lbtc(ObservedSide::Input, 2));
                observation
                    .objects
                    .push(lbtc_owned(ObservedSide::Input, 3, CAROL));
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
            "unowned sponsor source",
            Box::new(move |observation| {
                observation
                    .objects
                    .push(unclaimed_lbtc(ObservedSide::Input, 2));
                observation
                    .objects
                    .push(lbtc_owned(ObservedSide::Output, 1, CAROL));
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

// Weld mutation coverage (review finding 2): every OperationSpec field
// the weld checks must observably reject a mutation, including the
// fields whose published pilot value is empty or a default. The kind
// field decides whether the operation is an enforced covenant branch
// at all, so it comes first.

/// A copy of the published architecture with the compact-ASH operation
/// row rewritten by `mutate`.
fn compact_ash_mutated(
    mutate: impl FnOnce(&mut architecture::OperationSpec),
) -> architecture::Architecture {
    let mut mutated = ARCHITECTURE;

    let mut operations = mutated.operations.to_vec();
    let operation = operations
        .iter_mut()
        .find(|operation| operation.id == OperationId::CompactAsh)
        .expect("the architecture declares compact-ASH");
    mutate(operation);
    mutated.operations = operations.leak();

    mutated
}

fn weld_rejects(
    architecture: &architecture::Architecture,
    field: crate::ArchitectureMismatchField,
) {
    assert_eq!(
        crate::validate::validate_compact_ash_architecture(architecture),
        Err(crate::RealizationError::ArchitectureOperationMismatch {
            operation: OperationId::CompactAsh,
            field,
        }),
    );
}

#[test]
fn compact_ash_weld_rejects_a_client_protocol_reclassification() {
    let mutated = compact_ash_mutated(|operation| {
        operation.kind = architecture::OperationKind::ClientProtocol;
    });

    weld_rejects(&mutated, crate::ArchitectureMismatchField::OperationKind);
}

#[test]
fn compact_ash_weld_rejects_a_new_issuance() {
    let mutated = compact_ash_mutated(|operation| {
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
fn compact_ash_weld_rejects_a_new_quantity_read() {
    let mutated = compact_ash_mutated(|operation| {
        operation.reads = vec![architecture::QuantityId::Floor].leak();
    });

    weld_rejects(&mutated, crate::ArchitectureMismatchField::Reads);
}

#[test]
fn compact_ash_weld_rejects_a_new_quantity_write() {
    let mutated = compact_ash_mutated(|operation| {
        operation.writes = vec![architecture::QuantityId::Floor].leak();
    });

    weld_rejects(&mutated, crate::ArchitectureMismatchField::Writes);
}

#[test]
fn a_draft_valid_quantity_read_is_still_rejected_by_the_weld() {
    // The mutation that motivates checking reads in the weld at all: a
    // new quantity read with its reciprocal reader declaration passes
    // architecture draft validation, so no other rule catches it.
    let mut mutated = compact_ash_mutated(|operation| {
        operation.reads = vec![architecture::QuantityId::Floor].leak();
    });

    let mut quantities = mutated.quantities.to_vec();
    let quantity = quantities
        .iter_mut()
        .find(|quantity| quantity.id == architecture::QuantityId::Floor)
        .expect("the architecture declares the floor quantity");
    let mut readers = quantity.readers.to_vec();
    readers.push(architecture::ReaderId::Operation(OperationId::CompactAsh));
    quantity.readers = readers.leak();
    mutated.quantities = quantities.leak();

    architecture::validate_draft(&mutated).expect("a reciprocal quantity read is draft-valid");

    weld_rejects(&mutated, crate::ArchitectureMismatchField::Reads);
}

type WeldMutationCase = (
    &'static str,
    Box<dyn Fn(&mut architecture::OperationSpec)>,
    crate::ArchitectureMismatchField,
);

/// One focused mutation per welded field family (F2-002): each case
/// rewrites exactly one aspect of the published compact-ASH row and
/// names the mismatch class the weld must report.
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
            "ash input authorization",
            Box::new(|operation| {
                edit_input(operation, ObjectId::Ash, |input| {
                    input.authorization = architecture::InputAuthorization::InputOwner;
                });
            }),
            Field::AshInput,
        ),
        (
            "ash input minimum",
            Box::new(|operation| {
                edit_input(operation, ObjectId::Ash, |input| input.minimum = 1);
            }),
            Field::AshInput,
        ),
        (
            "ash output maximum",
            Box::new(|operation| {
                edit_output(operation, ObjectId::Ash, |output| {
                    output.maximum = architecture::MaxCount::Exact(2);
                });
            }),
            Field::AshOutput,
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
                inputs.retain(|input| input.object == ObjectId::Ash);
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
                operation.bounds = vec![BoundId::AshBatchMax].leak();
            }),
            Field::Bounds,
        ),
        (
            "delta kind",
            Box::new(|operation| {
                let mut deltas = operation.canonical_deltas.to_vec();
                deltas[0].kind = DeltaKind::Lateral;
                operation.canonical_deltas = deltas.leak();
            }),
            Field::CanonicalDelta,
        ),
        (
            "delta condition",
            Box::new(|operation| {
                let mut deltas = operation.canonical_deltas.to_vec();
                deltas[0].condition = architecture::DeltaCondition::PositiveAdmittedPrincipal;
                operation.canonical_deltas = deltas.leak();
            }),
            Field::CanonicalDelta,
        ),
        (
            "delta destruction tag",
            Box::new(|operation| {
                let mut deltas = operation.canonical_deltas.to_vec();
                deltas[0].destruction_tag = Some(architecture::TagId::Burn);
                operation.canonical_deltas = deltas.leak();
            }),
            Field::CanonicalDelta,
        ),
        (
            "extra canonical delta",
            Box::new(|operation| {
                let mut deltas = operation.canonical_deltas.to_vec();
                deltas.push(deltas[0]);
                operation.canonical_deltas = deltas.leak();
            }),
            Field::CanonicalDelta,
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
            "added value-flow class",
            Box::new(|operation| {
                let mut flows = operation.value_flows.to_vec();
                flows.push(architecture::ValueFlowClass::OwnerConsented);
                operation.value_flows = flows.leak();
            }),
            Field::ValueFlows,
        ),
        (
            "removed witness",
            Box::new(|operation| {
                let mut witnesses = operation.witnesses.to_vec();
                witnesses.retain(|witness| *witness != architecture::WitnessId::UtxoLifecycle);
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
fn compact_ash_weld_rejects_every_field_mutation() {
    for (name, mutate, field) in weld_mutation_cases() {
        let mutated = compact_ash_mutated(|operation| mutate(operation));

        assert_eq!(
            crate::validate::validate_compact_ash_architecture(&mutated),
            Err(crate::RealizationError::ArchitectureOperationMismatch {
                operation: OperationId::CompactAsh,
                field,
            }),
            "mutation case {name}",
        );
    }
}

#[test]
fn sponsor_objects_carry_no_protocol_amount() {
    // S3, the deciding property, now structural rather than tested by
    // comparison: a sponsor member has no readable amount at all, so
    // there is no denomination for a protocol verdict to depend on.
    // The earlier version of this test built two regions with
    // different denominations and compared verdicts; it can no longer
    // be written, which is a stronger result than its passing was.
    let observation = valid_sponsored_observation();

    let sponsors = observation
        .objects
        .iter()
        .filter(|object| object.kind == ObservedObjectKind::Declared(ObjectId::PlainLbtc))
        .collect::<Vec<_>>();

    assert!(!sponsors.is_empty(), "fixture has sponsor members");
    for sponsor in sponsors {
        assert_eq!(sponsor.value, crate::ObservedValue::SponsorOpaque);
        assert_eq!(sponsor.value.protocol(), None);
    }

    assert!(!failed(&evaluate(&observation), &sponsor()));
}

// --- Guide-6 §4: compact-ASH sponsor cardinality.
//
// The architecture declares the sponsor family's input and output
// cardinalities for compact ASH exactly as it does for live transfer.
// Realization used to expose only sponsor *recognition*, so those
// architecture facts were owned by no relation and no carrier analysis
// could see the sponsor family at all. The relations below make the
// existing architecture semantics explicit; the typed census rule that
// follows keeps the two sides welded. ---

fn sponsor_input_cardinality() -> RelationId {
    relation_id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::PlainLbtc,
        },
    )
}

fn sponsor_output_cardinality() -> RelationId {
    relation_id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::PlainLbtc,
        },
    )
}

fn compact_relation(id: &RelationId) -> crate::Relation {
    pilot_realization()
        .relation(id)
        .unwrap_or_else(|| panic!("{id:?} is declared"))
        .relation
        .clone()
}

#[test]
fn compact_ash_declares_sponsor_input_cardinality() {
    assert_eq!(
        compact_relation(&sponsor_input_cardinality()),
        crate::Relation::Cardinality {
            side: ObservedSide::Input,
            object: ObjectId::PlainLbtc,
            minimum: Count::ZERO,
            maximum: crate::CardinalityMaximum::Bound(BoundId::FeeSponsorInputMax),
        },
    );
}

#[test]
fn compact_ash_declares_sponsor_output_cardinality() {
    assert_eq!(
        compact_relation(&sponsor_output_cardinality()),
        crate::Relation::Cardinality {
            side: ObservedSide::Output,
            object: ObjectId::PlainLbtc,
            minimum: Count::ZERO,
            maximum: crate::CardinalityMaximum::Exact(Count::ONE),
        },
    );
}

#[test]
fn compact_ash_sponsor_input_maximum_is_the_architecture_bound() {
    let crate::Relation::Cardinality { maximum, .. } =
        compact_relation(&sponsor_input_cardinality())
    else {
        panic!("sponsor input cardinality is a cardinality relation");
    };

    assert_eq!(
        maximum,
        crate::CardinalityMaximum::Bound(BoundId::FeeSponsorInputMax),
    );
}

#[test]
fn compact_ash_sponsor_output_maximum_is_exactly_one() {
    let crate::Relation::Cardinality { maximum, .. } =
        compact_relation(&sponsor_output_cardinality())
    else {
        panic!("sponsor output cardinality is a cardinality relation");
    };

    assert_eq!(maximum, crate::CardinalityMaximum::Exact(Count::ONE));
}

#[test]
fn compact_ash_sponsor_cardinality_depends_on_sponsor_recognition() {
    let realization = pilot_realization();
    let declared = realization.operations[&OperationId::CompactAsh]
        .relation_dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.prerequisite.clone(),
                dependency.dependent.clone(),
                dependency.edge,
            )
        })
        .collect::<BTreeSet<_>>();

    for (recognition, cardinality) in [
        (sponsor_input_recognition(), sponsor_input_cardinality()),
        (sponsor_output_recognition(), sponsor_output_cardinality()),
    ] {
        assert!(
            declared.contains(&(
                recognition.clone(),
                cardinality.clone(),
                crate::RelationEdge::RecognitionBeforeCardinality,
            )),
            "{recognition:?} must precede {cardinality:?}",
        );
    }
}

#[test]
fn no_compact_ash_relation_names_a_sponsor_amount() {
    // The repair exposes sponsor *counts* and family recognition. A
    // sponsor amount operand would breach the erasure boundary the rest
    // of the realization maintains, so no relation may carry one.
    let realization = pilot_realization();

    for declaration in realization.relations() {
        match &declaration.relation {
            crate::Relation::AmountConservation {
                input_objects,
                output_objects,
                ..
            } => {
                assert!(!input_objects.contains(&ObjectId::PlainLbtc));
                assert!(!output_objects.contains(&ObjectId::PlainLbtc));
            }
            crate::Relation::Cardinality { .. }
            | crate::Relation::Recognition { .. }
            | crate::Relation::AllowedObjectFamilies { .. }
            | crate::Relation::OwnerAuthorization { .. }
            | crate::Relation::PermissionlessAuthorization
            | crate::Relation::SponsorIsolation
            | crate::Relation::SponsorEnvelopeMultiplicity { .. }
            | crate::Relation::RootPolicy { .. }
            | crate::Relation::ProjectionPolicy { .. }
            | crate::Relation::CanonicalDeltaPolicy { .. }
            | crate::Relation::OpenFlowPolicy { .. }
            | crate::Relation::Constructibility { .. }
            | crate::Relation::Representation { .. }
            | crate::Relation::LifecycleExit { .. }
            | crate::Relation::ExpressionPredicate { .. }
            | crate::Relation::SubstrateConservation { .. } => {}
        }
    }
}

/// Reassemble the compact-ASH scope after one declaration mutation.
fn reassembled(
    mutate: impl FnOnce(&mut crate::OperationRealization),
) -> Result<crate::ScopedRealizationSpec, crate::RealizationError> {
    let mut operations = derive(&ARCHITECTURE, compact_scope()).unwrap().operations;

    mutate(operations.get_mut(&OperationId::CompactAsh).unwrap());

    crate::derive::assemble_scoped_realization(
        &ARCHITECTURE,
        crate::ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
        compact_scope(),
        operations,
    )
}

/// Drop one relation and every dependency edge that named it.
fn without_relation(operation: &mut crate::OperationRealization, id: &RelationId) {
    operation.relations.retain(|relation| &relation.id != id);
    operation
        .relation_dependencies
        .retain(|dependency| &dependency.prerequisite != id && &dependency.dependent != id);
}

/// Replace one relation's declared payload, keeping its stable ID.
fn with_relation(
    operation: &mut crate::OperationRealization,
    id: &RelationId,
    replacement: &crate::Relation,
) {
    for relation in &mut operation.relations {
        if &relation.id == id {
            relation.relation = replacement.clone();
        }
    }
}

#[test]
fn missing_sponsor_input_cardinality_fails_derivation() {
    let error = reassembled(|operation| without_relation(operation, &sponsor_input_cardinality()))
        .unwrap_err();

    assert_eq!(
        error,
        crate::RealizationError::MissingArchitectureRelation {
            operation: OperationId::CompactAsh,
            kind: RelationKind::Cardinality,
            subject: RelationSubject::ObjectFamily {
                side: TransactionSide::Input,
                object: ObjectId::PlainLbtc,
            },
        },
    );
}

#[test]
fn missing_sponsor_output_cardinality_fails_derivation() {
    let error = reassembled(|operation| without_relation(operation, &sponsor_output_cardinality()))
        .unwrap_err();

    assert_eq!(
        error,
        crate::RealizationError::MissingArchitectureRelation {
            operation: OperationId::CompactAsh,
            kind: RelationKind::Cardinality,
            subject: RelationSubject::ObjectFamily {
                side: TransactionSide::Output,
                object: ObjectId::PlainLbtc,
            },
        },
    );
}

#[test]
fn missing_sponsor_recognition_fails_derivation() {
    let error = reassembled(|operation| without_relation(operation, &sponsor_input_recognition()))
        .unwrap_err();

    assert_eq!(
        error,
        crate::RealizationError::MissingArchitectureRelation {
            operation: OperationId::CompactAsh,
            kind: RelationKind::Recognition,
            subject: RelationSubject::ObjectFamily {
                side: TransactionSide::Input,
                object: ObjectId::PlainLbtc,
            },
        },
    );
}

#[test]
fn transposed_sponsor_cardinality_sides_fail_derivation() {
    // Each ID keeps its subject while the payloads swap: the input
    // relation now carries the output bounds and vice versa. Nothing
    // else in the census can see this, because both relations still
    // exist and both bounds still appear somewhere.
    let input = compact_relation(&sponsor_input_cardinality());
    let output = compact_relation(&sponsor_output_cardinality());
    let error = reassembled(|operation| {
        with_relation(operation, &sponsor_input_cardinality(), &output);
        with_relation(operation, &sponsor_output_cardinality(), &input);
    })
    .unwrap_err();

    assert_eq!(
        error,
        crate::RealizationError::ArchitectureRelationMismatch {
            relation: sponsor_input_cardinality(),
        },
    );
}

#[test]
fn a_wrong_sponsor_input_bound_fails_derivation() {
    let error = reassembled(|operation| {
        with_relation(
            operation,
            &sponsor_input_cardinality(),
            &crate::Relation::Cardinality {
                side: ObservedSide::Input,
                object: ObjectId::PlainLbtc,
                minimum: Count::ZERO,
                maximum: crate::CardinalityMaximum::Bound(BoundId::AshBatchMax),
            },
        );
    })
    .unwrap_err();

    assert_eq!(
        error,
        crate::RealizationError::ArchitectureRelationMismatch {
            relation: sponsor_input_cardinality(),
        },
    );
}

#[test]
fn a_wrong_sponsor_output_maximum_fails_derivation() {
    let error = reassembled(|operation| {
        with_relation(
            operation,
            &sponsor_output_cardinality(),
            &crate::Relation::Cardinality {
                side: ObservedSide::Output,
                object: ObjectId::PlainLbtc,
                minimum: Count::ZERO,
                maximum: crate::CardinalityMaximum::Exact(Count::new(2)),
            },
        );
    })
    .unwrap_err();

    assert_eq!(
        error,
        crate::RealizationError::ArchitectureRelationMismatch {
            relation: sponsor_output_cardinality(),
        },
    );
}

#[test]
fn a_wrong_recognized_sponsor_asset_fails_derivation() {
    let error = reassembled(|operation| {
        with_relation(
            operation,
            &sponsor_input_recognition(),
            &crate::Relation::Recognition {
                side: ObservedSide::Input,
                object: ObjectId::PlainLbtc,
                asset: AssetId::U,
            },
        );
    })
    .unwrap_err();

    assert_eq!(
        error,
        crate::RealizationError::ArchitectureRelationMismatch {
            relation: sponsor_input_recognition(),
        },
    );
}
