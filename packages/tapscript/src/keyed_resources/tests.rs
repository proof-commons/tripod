use super::*;
fn reviewed_target() -> target_elements::ReviewedElementsTapscriptDefinition {
    target_elements::reviewed_elements_tapscript().expect("reviewed target")
}
use KeyedResourceTerm::Exact;
use ResourceDimension as D;

#[test]
fn grammar_resources_cover_every_dimension_without_inventing_deployment_bounds() {
    let target = reviewed_target();
    for (schedule, bytes, primitives, peak, width) in [
        (KeyedSchedule::Birth, 10, 5, 3, 32),
        (KeyedSchedule::Continuation, 7, 6, 4, 40),
        (KeyedSchedule::Leaf, 52, 12, 4, 40),
    ] {
        let projection = keyed_resource_projection(&target, schedule).expect("grammar");
        assert_eq!(projection.dimensions.len(), D::ALL.len());
        assert!(D::ALL.iter().all(|d| projection.dimensions.contains_key(d)));
        for (dimension, value) in [
            (D::ScriptBytes, bytes),
            (D::OperationCost, 0),
            (D::ValidationBudget, 0),
            (D::InitialStackItems, 0),
            (D::InitialWitnessItemBytes, 0),
            (D::PeakStackItems, peak),
        ] {
            assert_eq!(projection.dimensions[&dimension], Exact(value));
        }
        assert_eq!(
            projection.dimensions[&D::StackElementBytes],
            KeyedResourceTerm::ElementWidthBound(width)
        );
        assert_eq!(projection.primitive_count, primitives);
        assert_eq!(
            projection.dimensions[&D::TransactionWeight],
            KeyedResourceTerm::TransactionContextRequired
        );
        assert_eq!(
            projection.dimensions[&D::PackageLimit],
            KeyedResourceTerm::DeploymentRequired
        );
        assert_eq!(
            projection.consensus_bounds,
            *target.definition().resources().consensus().bounds()
        );
        assert_eq!(
            projection.policy_bounds,
            *target.definition().resources().policy().bounds()
        );
        if schedule == KeyedSchedule::Leaf {
            assert_eq!(projection.dimensions[&D::ControlPathDepth], Exact(0));
            assert_eq!(
                projection.dimensions[&D::WitnessBytes],
                KeyedResourceTerm::LeafWitnessFormula
            );
        } else {
            assert_eq!(
                projection.dimensions[&D::ControlPathDepth],
                KeyedResourceTerm::CompositionRequired
            );
            assert_eq!(
                projection.dimensions[&D::WitnessBytes],
                KeyedResourceTerm::CompositionRequired
            );
        }
    }
}

#[test]
fn independent_witness_serialization_and_arbitrary_member_sums_are_exact() {
    // Serialized empty-argument depth-zero script witness: vector count,
    // script length/body, control length/body. No execution arguments.
    let script = crate::keyed_pattern::compose_keyed_leaf(&reviewed_target(), &[0x42; 32])
        .expect("leaf")
        .encode(&reviewed_target());
    let mut witness = vec![2, 52];
    witness.extend(script);
    witness.push(33);
    witness.extend([0xc4; 33]); // length only; this is not a control validity claim.
    assert_eq!(witness.len(), 88);
    for members in [0, 1, 2, 1564, 1565, 1_000_000] {
        assert_eq!(keyed_witness_bytes(members), Ok(members * 88));
        assert_eq!(
            keyed_transaction_weight(&reviewed_target(), members, 100, 17),
            Ok(417 + members * 91)
        );
    }
    assert_eq!(keyed_witness_bytes(u64::MAX / 88), Ok((u64::MAX / 88) * 88));
    assert_eq!(
        keyed_witness_bytes(u64::MAX / 88 + 1),
        Err(KeyedResourceOverflow)
    );
    assert_eq!(
        keyed_transaction_weight(&reviewed_target(), 1, u64::MAX, 0),
        Err(KeyedResourceOverflow)
    );
    assert_eq!(
        keyed_transaction_weight(&reviewed_target(), 1, 0, u64::MAX),
        Err(KeyedResourceOverflow)
    );
}

#[test]
fn compact_size_transitions_are_framed_without_wrapping() {
    for (n, prefix) in [
        (0, 1),
        (252, 1),
        (253, 3),
        (65_535, 3),
        (65_536, 5),
        (4_294_967_295, 5),
        (4_294_967_296, 9),
    ] {
        assert_eq!(compact_size_width(n), prefix);
        assert_eq!(framed_bytes(n), Ok(n + prefix));
    }
    assert_eq!(framed_bytes(u64::MAX - 9), Ok(u64::MAX));
    assert_eq!(framed_bytes(u64::MAX - 8), Err(KeyedResourceOverflow));
}

#[test]
fn encoding_widths_back_the_written_element_bounds_and_budget_uses_only_script_stack() {
    use target_elements::{EncodingClass as E, PayloadWidth};
    let target = reviewed_target();
    for (class, minimum, maximum) in [
        (E::WitnessProgram, 2, 40),
        (E::OutPointTxid, 32, 32),
        (E::ScriptPubKeySha256, 32, 32),
        (E::OutPointIndex, 4, 4),
        (E::OutPointFlags, 1, 1),
        (E::ScriptNumber, 0, 4),
    ] {
        let widths = match target.definition().encodings()[&class].payload() {
            PayloadWidth::Exact(n) => (n.get(), n.get()),
            PayloadWidth::Bounded { minimum, maximum } => (minimum, maximum.get()),
            PayloadWidth::Absent => (0, 0),
        };
        assert_eq!(widths, (minimum, maximum));
    }
    // Validation budget uses the serialized script-witness stack, NOT the
    // four-field 91-byte input witness. The extra three fields fund no checks.
    assert_eq!(
        target
            .definition()
            .resources()
            .consensus()
            .validation_budget_offset()
            + keyed_witness_bytes(1).expect("script stack"),
        138
    );
    assert_eq!(
        keyed_transaction_weight(&target, u64::MAX / 91 + 1, 0, 0),
        Err(KeyedResourceOverflow)
    );
    assert_eq!(KeyedResourceOverflow.refusal_kind(), "arithmetic_overflow");
    assert_eq!(
        KeyedResourceOverflow.to_string(),
        "keyed resource arithmetic overflow"
    );
    let _: &dyn std::error::Error = &KeyedResourceOverflow;
}
