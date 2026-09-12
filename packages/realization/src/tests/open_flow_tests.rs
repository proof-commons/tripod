//! Open-flow reference structure at observation normalization (SR2-07).
//!
//! Normalization owns the reference structure: which side a reference
//! sits on, that no two open flows claim the same one, and that the
//! CPFP anchor never joins an open-value partition. Whether the
//! partition is *complete* stays with the relations — see the boundary
//! note on `OperationObservation::validate_and_normalize`, and the
//! final test here, which pins that split.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    ARCHITECTURE, AssetId, BoundId, DeltaKind, ObjectId, OperationId, ProjectionId,
};

use crate::{
    Count, ObservedAsset, ObservedCanonicalFlow, ObservedCanonicalPartition, ObservedObject,
    ObservedObjectKind, ObservedObjectRef, ObservedOpenFlow, ObservedSide, ObservedValue,
    OperationObservation, OwnerId, ProtocolAmount, RealizationError, RealizationScope,
    RepresentationMode, derive, evaluate_operation, validate_observation,
};

const CAROL: OwnerId = OwnerId([3_u8; 32]);

fn input(ordinal: u32) -> ObservedObjectRef {
    ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal,
    }
}

fn output(ordinal: u32) -> ObservedObjectRef {
    ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal,
    }
}

fn ash(side: ObservedSide, ordinal: u32, value: u64) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::Ash),
        asset: ObservedAsset::Declared(AssetId::U),
        value: ObservedValue::Protocol(ProtocolAmount::new(value).unwrap()),
        owner: None,
        representation: RepresentationMode::Explicit,
    }
}

fn sponsor(side: ObservedSide, ordinal: u32) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::PlainLbtc),
        asset: ObservedAsset::Declared(AssetId::Lbtc),
        value: ObservedValue::SponsorOpaque,
        owner: Some(CAROL),
        representation: RepresentationMode::Explicit,
    }
}

fn anchor(side: ObservedSide, ordinal: u32) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::CpfpAnchor),
        asset: ObservedAsset::Declared(AssetId::Lbtc),
        value: ObservedValue::Protocol(ProtocolAmount::ZERO),
        owner: None,
        representation: RepresentationMode::Explicit,
    }
}

fn fee_sponsor_flow(
    sources: Vec<ObservedObjectRef>,
    destinations: Vec<ObservedObjectRef>,
) -> ObservedOpenFlow {
    ObservedOpenFlow {
        kind: architecture::OpenFlowKind::FeeSponsor,
        sources,
        destinations,
        fee: ProtocolAmount::ZERO,
    }
}

/// A compact-ASH shaped observation with two sponsor inputs and two
/// sponsor outputs, so every structural mutation below has room.
fn observation() -> OperationObservation {
    OperationObservation {
        operation: OperationId::CompactAsh,
        objects: vec![
            ash(ObservedSide::Input, 0, 40),
            ash(ObservedSide::Input, 1, 60),
            ash(ObservedSide::Output, 0, 100),
            sponsor(ObservedSide::Input, 2),
            sponsor(ObservedSide::Input, 3),
            sponsor(ObservedSide::Output, 1),
            sponsor(ObservedSide::Output, 2),
        ],
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::from([CAROL]),
        canonical_partition: ObservedCanonicalPartition {
            issuances: Vec::new(),
            flows: vec![ObservedCanonicalFlow {
                asset: AssetId::U,
                sources: vec![input(0), input(1)],
                destinations: vec![output(0)],
                movement_kind: Some(DeltaKind::OwnerlessLateral),
                destructions: Vec::new(),
            }],
        },
        open_flows: vec![fee_sponsor_flow(
            vec![input(2), input(3)],
            vec![output(1), output(2)],
        )],
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::AshBatchMax, Count::new(64)),
            (BoundId::FeeSponsorInputMax, Count::new(16)),
        ]),
    }
}

fn rejects(observation: OperationObservation, expected: RealizationError) {
    assert_eq!(validate_observation(observation), Err(expected));
}

#[test]
fn the_fixture_is_structurally_valid() {
    validate_observation(observation()).expect("the base observation normalizes");
}

#[test]
fn an_output_reference_may_not_be_an_open_flow_source() {
    let mut observed = observation();
    observed.open_flows[0].sources = vec![output(1)];

    rejects(
        observed,
        RealizationError::WrongObservedReferenceSide(output(1)),
    );
}

#[test]
fn an_input_reference_may_not_be_an_open_flow_destination() {
    let mut observed = observation();
    observed.open_flows[0].destinations = vec![input(2)];

    rejects(
        observed,
        RealizationError::WrongObservedReferenceSide(input(2)),
    );
}

#[test]
fn a_source_may_not_be_claimed_by_two_open_flows() {
    let mut observed = observation();
    observed.open_flows[0].sources = vec![input(2)];
    observed
        .open_flows
        .push(fee_sponsor_flow(vec![input(2)], Vec::new()));

    rejects(
        observed,
        RealizationError::ObservedOpenFlowOverlap(input(2)),
    );
}

#[test]
fn a_destination_may_not_be_claimed_by_two_open_flows() {
    let mut observed = observation();
    observed.open_flows[0].destinations = vec![output(1)];
    observed
        .open_flows
        .push(fee_sponsor_flow(Vec::new(), vec![output(1)]));

    rejects(
        observed,
        RealizationError::ObservedOpenFlowOverlap(output(1)),
    );
}

/// A protocol open flow of `kind` over the same reference shapes.
fn protocol_flow(
    kind: architecture::OpenFlowKind,
    sources: Vec<ObservedObjectRef>,
    destinations: Vec<ObservedObjectRef>,
) -> ObservedOpenFlow {
    ObservedOpenFlow {
        kind,
        sources,
        destinations,
        fee: ProtocolAmount::ZERO,
    }
}

#[test]
fn a_reference_may_not_be_claimed_by_both_regions() {
    // S2-01: the sponsor region is exactly the fee-sponsor membership,
    // so a reference cannot be sponsor and protocol at once. It would
    // have to be erased and readable in the same projection.
    let mut observed = observation();
    observed.open_flows[0].sources = vec![input(2)];
    observed.open_flows.push(protocol_flow(
        architecture::OpenFlowKind::Redemption,
        vec![input(2)],
        Vec::new(),
    ));

    rejects(
        observed,
        RealizationError::ObservedOpenFlowOverlap(input(2)),
    );
}

#[test]
fn the_flow_role_of_a_reference_is_its_claiming_flow() {
    // Object family does not determine role (F.1). The same ordinary
    // L-BTC family resolves to whichever region claims it, and to no
    // region at all when nothing does.
    let mut observed = observation();
    observed.open_flows[0].sources = vec![input(2)];
    observed.open_flows[0].destinations = vec![output(1)];
    observed.open_flows.push(protocol_flow(
        architecture::OpenFlowKind::Redemption,
        vec![input(3)],
        Vec::new(),
    ));

    let normalized = validate_observation(observed).expect("disjoint regions normalize");

    assert_eq!(
        normalized.flow_role(input(2)),
        crate::ObservedFlowRole::Sponsor
    );
    assert_eq!(
        normalized.flow_role(output(1)),
        crate::ObservedFlowRole::Sponsor
    );
    assert_eq!(
        normalized.flow_role(input(3)),
        crate::ObservedFlowRole::Protocol(architecture::OpenFlowKind::Redemption),
    );
    assert_eq!(
        normalized.flow_role(output(2)),
        crate::ObservedFlowRole::Unclaimed,
    );
}

#[test]
fn a_cpfp_anchor_may_not_be_an_open_flow_source() {
    let mut observed = observation();
    observed.objects[3] = anchor(ObservedSide::Input, 2);

    rejects(
        observed,
        RealizationError::AnchorInObservedOpenFlow(input(2)),
    );
}

#[test]
fn a_cpfp_anchor_may_not_be_an_open_flow_destination() {
    // The kernel exempts the anchor from the partition rather than
    // admitting it, and it does so for every flow kind — so the
    // exclusion is structural here too, not a per-operation policy.
    let mut observed = observation();
    observed.objects[5] = anchor(ObservedSide::Output, 1);

    rejects(
        observed,
        RealizationError::AnchorInObservedOpenFlow(output(1)),
    );
}

#[test]
fn an_unclaimed_sponsor_member_is_a_relation_verdict_not_a_parse_error() {
    // The boundary this finding turns on. An ordinary L-BTC member no
    // open flow claims is a real transaction breaking a real rule, so
    // it must survive normalization and be *reported* by the relation
    // that owns exact sponsor membership. Rejecting it structurally
    // would say the observation could not be read, and would remove
    // the obligation from the relation census the compiler consumes.
    let mut observed = observation();
    observed.open_flows[0].sources = vec![input(2)];

    let normalized = validate_observation(observed).expect("an unclaimed member still normalizes");

    let realization = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::CompactAsh]).unwrap(),
    )
    .unwrap();
    let report = evaluate_operation(
        &realization.relation_graph,
        &realization.relation_node_by_id,
        &realization.relation_evaluation_order,
        &realization.expression_graph,
        &realization.expression_node_by_id,
        &realization.expression_evaluation_order,
        &normalized,
    )
    .unwrap();

    assert!(
        !report.is_conformant(),
        "the sponsor-isolation relation must reject the unclaimed member",
    );
}

fn announcement_sponsored() -> OperationObservation {
    let mut observed = super::announce_maturity_tests::observation();
    observed.objects.extend([sponsor(ObservedSide::Input, 1), sponsor(ObservedSide::Output, 1)]);
    observed.open_flows.push(fee_sponsor_flow(vec![input(1)], vec![output(1)]));
    observed.sponsor_signers.insert(CAROL);
    observed
}

fn announcement_sponsor_status(observed: &OperationObservation) -> crate::RelationStatus {
    use super::announce_maturity_tests as announcement;
    announcement::status(observed, &announcement::id(crate::RelationKind::SponsorIsolation, crate::RelationSubject::Sponsor))
}

#[test]
fn announcement_sponsor_ownership_is_required() {
    let mut observed = announcement_sponsored();
    assert_eq!(announcement_sponsor_status(&observed), crate::RelationStatus::Passed);
    observed.sponsor_signers.clear();
    assert!(matches!(announcement_sponsor_status(&observed), crate::RelationStatus::Failed { .. }));
}

#[test]
fn announcement_sponsor_amounts_must_be_erased() {
    use super::announce_maturity_tests as announcement;
    for side in [ObservedSide::Input, ObservedSide::Output] {
        let mut observed = announcement_sponsored();
        observed.objects.iter_mut().find(|o| o.reference == ObservedObjectRef { side, ordinal: 1 }).unwrap().value =
            ObservedValue::Protocol(ProtocolAmount::ONE);
        let relation = announcement::id(crate::RelationKind::Recognition, crate::RelationSubject::ObjectFamily {
            side: match side { ObservedSide::Input => crate::TransactionSide::Input, ObservedSide::Output => crate::TransactionSide::Output },
            object: ObjectId::PlainLbtc,
        });
        assert!(matches!(announcement::status(&observed, &relation), crate::RelationStatus::Failed { .. }));
    }
}

#[test]
fn announcement_unclaimed_sponsor_members_reject() {
    let mut observed = announcement_sponsored();
    observed.open_flows.clear();
    assert!(matches!(announcement_sponsor_status(&observed), crate::RelationStatus::Failed { .. }));
}

#[test]
fn announcement_non_sponsor_open_flows_reject() {
    use super::announce_maturity_tests as announcement;
    for kind in architecture::OpenFlowKind::ALL.iter().filter(|kind| **kind != architecture::OpenFlowKind::FeeSponsor) {
        let mut observed = announcement::observation();
        observed.open_flows.push(ObservedOpenFlow { kind: *kind, sources: Vec::new(), destinations: Vec::new(), fee: ProtocolAmount::ZERO });
        let relation = announcement::id(crate::RelationKind::OpenFlowPolicy, crate::RelationSubject::Operation);
        assert_eq!(announcement::status(&observed, &relation),
            crate::RelationStatus::Failed { reason: crate::RelationFailure::OpenFlowPolicy });
    }
}
