use super::*;
use std::collections::BTreeSet;
use tapscript::{InputRole, LeafRole, OutputRole, TapscriptInstruction as I};
use target_elements::OpcodeId as O;

#[allow(
    clippy::too_many_lines,
    reason = "one audit over every retained leaf instruction and layout"
)]
fn audit_carrier() -> crate::bundle::FixtureBundle {
    let bundle = crate::bundle::fixture_bundle().expect("candidate");
    let tree = transaction::taproot::commit_tree(bundle.target(), bundle.linked()).expect("tree");
    assert_eq!(
        tree.merkle_root(),
        target_elements_conformance::reference::FIXTURE_MERKLE_ROOT
    );
    assert_eq!(
        *bundle.pin().program(),
        target_elements_conformance::reference::FIXTURE_REFERENCE_OUTPUT_KEY
    );
    assert_eq!(*bundle.pin().program(), crate::bundle::PINNED_PROGRAM);
    assert_eq!(bundle.linked().programs().len(), 12);
    assert_eq!(bundle.linked().layouts().len(), 9);
    let mut bytes = 0;
    for (role, leaf) in bundle.linked().programs() {
        let program = leaf.program();
        bytes += program.encode(bundle.target()).len();
        assert_eq!(bundle.linked().witness_roles()[role].data_items(), 0);
        for (index, instruction) in program.instructions().iter().enumerate() {
            if let I::Opcode(opcode) = instruction {
                assert!(
                    matches!(
                        opcode,
                        O::PushCurrentInputIndex
                            | O::InspectNumInputs
                            | O::InspectNumOutputs
                            | O::InspectInputAsset
                            | O::InspectInputScriptPubKey
                            | O::InspectInputValue
                            | O::InspectOutputAsset
                            | O::InspectOutputScriptPubKey
                            | O::InspectOutputValue
                            | O::EqualVerify
                            | O::Verify
                            | O::Rotate
                            | O::Substring
                            | O::Duplicate
                            | O::GreaterThan64
                            | O::LessThan64
                            | O::GreaterThanOrEqual64
                            | O::ScriptNumToLe64
                            | O::Add64
                    ),
                    "unclassified instruction {opcode:?}"
                );
                if let LeafRole::Coordinator { shape } = role {
                    if !matches!(
                        opcode,
                        O::InspectInputScriptPubKey | O::InspectOutputScriptPubKey
                    ) {
                        continue;
                    }
                    let previous = &program.instructions()[index - 1];
                    let numeric = |value: u8| {
                        I::Push(
                            tapscript::StackItem::script_number(bundle.target(), i64::from(value))
                                .expect("index"),
                        )
                    };
                    if *opcode == O::InspectInputScriptPubKey {
                        assert!(
                            *previous == I::Opcode(O::PushCurrentInputIndex)
                                || (0..shape.ash_inputs())
                                    .any(|position| *previous == numeric(position))
                        );
                    }
                    if *opcode == O::InspectOutputScriptPubKey {
                        assert!(
                            *previous == numeric(0)
                                || (shape.sponsored()
                                    && *previous
                                        == numeric(
                                            u8::try_from(shape.outputs() - 1).expect("fee index")
                                        ))
                        );
                    }
                } else {
                    assert!(!matches!(
                        opcode,
                        O::InspectInputScriptPubKey | O::InspectOutputScriptPubKey
                    ));
                }
            }
        }
    }
    assert_eq!(bytes, 3861);
    for (shape, layout) in bundle.linked().layouts() {
        let mut positions = BTreeSet::new();
        for placement in layout.inputs() {
            for position in placement.first()..placement.end() {
                assert!(positions.insert(position));
            }
        }
        assert_eq!(positions, (0..shape.inputs()).collect());
        assert_eq!(
            layout.input_run(InputRole::Sponsor).is_some(),
            shape.sponsored()
        );
        assert_eq!(layout.output_position(OutputRole::Successor), Some(0));
        assert_eq!(
            layout
                .outputs()
                .iter()
                .map(|output| output.position())
                .collect::<BTreeSet<_>>(),
            (0..shape.outputs()).collect()
        );
    }
    bundle
}

fn check(reason: Closure, relation: &str, mutations: &[&str], sponsors: &[&str]) {
    let bundle = audit_carrier();
    let rows = roster(&bundle).expect("roster");
    let expected = sponsors
        .iter()
        .flat_map(|sponsor| {
            mutations.iter().map(move |mutation| {
                format!("{sponsor}/{relation}/RuntimeCarrier/FocusedReject({mutation})")
            })
        })
        .collect::<BTreeSet<_>>();
    let found = rows
        .values()
        .filter(|row| closure(row) == Some(reason))
        .map(|row| {
            assert_eq!(row.manifest.obligation.id, row.manifest.requirement);
            assert_eq!(
                row.expected,
                crate::matrix::EvidenceBoundary::ScriptPathRejection
            );
            crate::compact_campaign::requirement_name(&row.manifest.requirement)
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(found, expected);
}

#[test]
fn compact_delta_family_has_no_independent_carrier_coordinate() {
    check(
        Closure::NoIndependentDeltaFamilyCoordinate,
        "CanonicalDeltaPolicy/Operation",
        &[
            "MissingCanonicalDeltaFamily",
            "UnexpectedCanonicalDeltaFamily",
        ],
        &["Absent", "Present"],
    );
}
#[test]
fn compact_canonical_membership_is_derived_once_per_position() {
    check(
        Closure::NoIndependentCanonicalMembershipCoordinate,
        "CanonicalDeltaPolicy/Operation",
        &["DuplicateCanonicalSourceOrDestination"],
        &["Absent", "Present"],
    );
}
#[test]
fn compact_open_flow_claims_have_no_carrier_encoding() {
    check(
        Closure::NoIndependentOpenFlowClaimCoordinate,
        "OpenFlowPolicy/Operation",
        &["UndeclaredOpenFlow"],
        &["Absent", "Present"],
    );
}
#[test]
fn compact_absent_sponsor_claim_domain_is_empty() {
    check(
        Closure::EmptySponsorClaimDomain,
        "SponsorIsolation/Sponsor",
        &["SponsorProtocolOverlap", "SponsorMemberUnclaimed"],
        &["Absent"],
    );
}
#[test]
fn compact_sponsor_membership_is_fixed_by_disjoint_positions() {
    check(
        Closure::NoIndependentSponsorMembershipCoordinate,
        "SponsorIsolation/Sponsor",
        &["SponsorProtocolOverlap", "SponsorMemberUnclaimed"],
        &["Present"],
    );
}
#[test]
fn compact_root_effects_have_no_independent_carrier_coordinate() {
    check(
        Closure::NoIndependentRootEffectCoordinate,
        "RootPolicy/Operation",
        &["WrongRootEffect"],
        &["Absent", "Present"],
    );
}
#[test]
fn compact_transition_certificate_is_derived_not_serialized() {
    check(
        Closure::DerivedTransitionCertificateHasNoRemovalCoordinate,
        "ProjectionPolicy/Operation",
        &["MissingRequiredProjection"],
        &["Absent", "Present"],
    );
}
#[test]
fn compact_specialized_projection_has_no_carrier_coordinate() {
    check(
        Closure::NoIndependentSpecializedProjectionCoordinate,
        "ProjectionPolicy/Operation",
        &["ForbiddenProjectionPresent"],
        &["Absent", "Present"],
    );
}

#[test]
fn sponsor_authorization_covers_uninspected_programs() {
    use crate::fixture::{SemanticFixtureId, SponsorCase as FixtureSponsor, semantic_case};
    use crate::materialize::{AshFunding, SponsorCoin, sponsor_signing_requests, vector_id};
    use transaction::{Outpoint, SighashProfile, TargetTransaction, Txid};
    let bundle = audit_carrier();
    let rows = roster(&bundle).expect("roster");
    let names = rows
        .values()
        .filter(|row| closure(row) == Some(Closure::SponsorAuthorizationCoversCoordinate))
        .map(|row| crate::compact_campaign::requirement_name(&row.manifest.requirement))
        .collect::<BTreeSet<_>>();
    assert_eq!(names, ["Input", "Output"].map(|side| format!("Present/Recognition/ObjectFamily {{ side: {side}, object: PlainLbtc }}/RuntimeCarrier/FocusedReject(WrongRecognizedObject)")).into_iter().collect());
    for count in 2..=4 {
        let case = semantic_case(
            SemanticFixtureId::new("authorization-coordinate", count),
            crate::matrix::POSITIVE_SEMANTIC[0],
            &vec![100; count as usize],
            FixtureSponsor::sponsored_with_change(1),
        )
        .expect("case");
        let funding = AshFunding::new(
            vector_id(&case),
            (0..count)
                .map(|index| {
                    Outpoint::new(Txid::from_internal([0x42; 32]), index).expect("outpoint")
                })
                .collect(),
        )
        .expect("funding");
        let mut program = vec![0, 20];
        program.extend([0x24; 20]);
        let coin = SponsorCoin::new(
            Outpoint::new(Txid::from_internal([0x43; 32]), 0).expect("sponsor"),
            100_000,
            program,
        );
        let tasks = sponsor_signing_requests(&bundle, &case, &funding, std::slice::from_ref(&coin))
            .expect("requests");
        assert_eq!(tasks.len(), 1);
        let request = tasks[0].request();
        assert_eq!(request.profile(), SighashProfile::AllInputsAllOutputs);
        let transaction =
            TargetTransaction::decode(request.transaction()).expect("finalized bytes");
        assert_eq!(
            transaction.inputs()[usize::from(request.input())].outpoint(),
            coin.outpoint()
        );
        assert_eq!(request.protected(), transaction.outputs());
    }
}

#[test]
fn second_roster_preserves_keys_and_derives_its_operation_census() {
    let bundle = audit_carrier();
    let first = crate::compact_campaign::roster(&bundle).expect("first");
    let second = roster(&bundle).expect("second");
    assert_eq!(
        first.keys().collect::<Vec<_>>(),
        second.keys().collect::<Vec<_>>()
    );
    for (key, row) in &second {
        assert_eq!(row.manifest, first[key].manifest);
    }
    assert_eq!(
        second.values().filter(|row| closure(row).is_some()).count(),
        20
    );
    assert_eq!(staged(&second).len(), 11);
    assert_eq!(sponsor_coins(&second), 14);
    let counts = operation_census(&second);
    assert_eq!(
        counts,
        OperationCensus {
            issuance: 2,
            constructor_probes: 1,
            sponsor_batches: 1,
            ash_batches: 30,
            signings: 22,
            submissions: 31
        }
    );
    assert_eq!(counts.total(), 87);
    assert_ne!(
        address_input(&second, "probe", "1"),
        address_input(&second, "record", "1")
    );
    assert_ne!(
        address_input(&second, "record", "1"),
        address_input(&second, "record", "2")
    );
    assert_eq!(
        crate::compact_campaign::operation_census(&first).total(),
        92
    );
}
