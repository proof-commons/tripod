//! Fixture-language tests.

use std::collections::BTreeSet;

use tapscript::{StackItem, TapscriptInstruction, TapscriptProgram};
use target_elements::OpcodeId;

use super::support::{TEST_GENESIS_ID, TEST_NETWORK_ID, development_binding, reviewed_target};
use crate::error::NativeConformanceError;
use crate::fixture::{
    ExpectedPrimitiveOutcome, NativeCaseGroup, NativeCaseId, PrimitiveFixture, PrimitiveFixtureSet,
    canonical_fixture_set,
};
use crate::protocol::{ObservedFailureClass, WireExecutionDomain};

/// One fixture exercising `add64`, stated by hand.
fn add64_fixture(ordinal: u32) -> PrimitiveFixture {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let program = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(OpcodeId::Add64)])
        .expect("the program is within the work limit");
    let stack = [
        StackItem::signed_le64(&target, 2),
        StackItem::signed_le64(&target, 3),
    ];
    PrimitiveFixture::new(
        &target,
        &binding,
        NativeCaseId::new(NativeCaseGroup::Arithmetic, Some(OpcodeId::Add64), ordinal),
        &program,
        &stack,
        None,
        ExpectedPrimitiveOutcome::Accept {
            final_stack: vec![StackItem::signed_le64(&target, 5).bytes().to_vec(), vec![1]],
            final_altstack: Vec::new(),
        },
    )
    .expect("the reviewed domain has a wire spelling")
}

#[test]
fn a_fixture_binds_the_contract_and_the_binding_it_was_stated_against() {
    let target = reviewed_target();
    let fixture = add64_fixture(0);

    assert_eq!(
        fixture.target_contract_version(),
        target.definition().version().get(),
    );
    assert_eq!(
        fixture.leaf_version(),
        target.definition().leaf_version().get()
    );
    assert_eq!(fixture.execution_domain(), WireExecutionDomain::Tapscript);
    assert_eq!(fixture.network_id(), TEST_NETWORK_ID);
    assert_eq!(fixture.genesis_id(), TEST_GENESIS_ID);
}

#[test]
fn fixture_script_bytes_are_the_typed_programs_own() {
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(OpcodeId::Add64)])
        .expect("the program is within the work limit");
    assert_eq!(add64_fixture(0).script(), program.encode(&target));
}

#[test]
fn the_census_is_sorted_by_case_and_independent_of_declaration_order() {
    let ascending =
        PrimitiveFixtureSet::new([add64_fixture(0), add64_fixture(1)]).expect("distinct cases");
    let descending =
        PrimitiveFixtureSet::new([add64_fixture(1), add64_fixture(0)]).expect("distinct cases");

    assert_eq!(
        ascending, descending,
        "declaration order must not be a fact"
    );
    let ordinals: Vec<u32> = ascending.iter().map(|f| f.case().ordinal()).collect();
    assert_eq!(ordinals, vec![0, 1]);
}

#[test]
fn a_duplicate_case_identity_is_refused() {
    let error = PrimitiveFixtureSet::new([add64_fixture(7), add64_fixture(7)])
        .expect_err("one identity, two fixtures");
    assert!(matches!(
        error,
        NativeConformanceError::DuplicateFixtureCase(case) if case.ordinal() == 7,
    ));
}

#[test]
fn the_canonical_census_is_empty_and_says_so() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let census =
        canonical_fixture_set(&target, &binding).expect("an empty census has no duplicate");
    assert!(
        census.is_empty(),
        "no primitive fixture has been authored yet",
    );
}

#[test]
fn a_case_identity_round_trips_through_its_wire_form() {
    let case = NativeCaseId::new(
        NativeCaseGroup::Signature,
        Some(OpcodeId::CheckSigVerify),
        3,
    );
    let json = serde_json::to_string(&case).expect("a case serializes");
    assert!(
        json.contains("check_sig_verify"),
        "a case travels under the primitive's spelling: {json}",
    );
    let parsed: NativeCaseId = serde_json::from_str(&json).expect("a case parses");
    assert_eq!(parsed, case);

    let without = NativeCaseId::new(NativeCaseGroup::PushEncoding, None, 0);
    let json = serde_json::to_string(&without).expect("a case serializes");
    let parsed: NativeCaseId = serde_json::from_str(&json).expect("a case parses");
    assert_eq!(parsed, without);
}

#[test]
fn an_unknown_primitive_spelling_fails_closed() {
    let json = r#"{"group":"arithmetic","opcode":"op_invented_upstream","ordinal":0}"#;
    serde_json::from_str::<NativeCaseId>(json).expect_err("an unspelled primitive is refused");
}

#[test]
fn an_unknown_case_field_fails_closed() {
    let json = r#"{"group":"arithmetic","opcode":null,"ordinal":0,"extra":1}"#;
    serde_json::from_str::<NativeCaseId>(json).expect_err("an unknown field is refused");
}

#[test]
fn an_unknown_fixture_field_fails_closed() {
    let json = serde_json::to_value(add64_fixture(0)).expect("a fixture serializes");
    let mut object = json.as_object().expect("a fixture is an object").clone();
    object.insert("surprise".to_owned(), serde_json::Value::from(1));
    serde_json::from_value::<PrimitiveFixture>(serde_json::Value::Object(object))
        .expect_err("an unknown field is refused");
}

#[test]
fn a_fixture_round_trips_through_json() {
    let fixture = add64_fixture(2);
    let json = serde_json::to_string(&fixture).expect("a fixture serializes");
    let parsed: PrimitiveFixture = serde_json::from_str(&json).expect("a fixture parses");
    assert_eq!(parsed, fixture);
}

#[test]
fn a_rejecting_fixture_states_its_failure_class() {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let program = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(OpcodeId::Div64)])
        .expect("the program is within the work limit");
    let fixture = PrimitiveFixture::new(
        &target,
        &binding,
        NativeCaseId::new(NativeCaseGroup::Arithmetic, Some(OpcodeId::Div64), 0),
        &program,
        &[],
        None,
        ExpectedPrimitiveOutcome::Reject {
            class: ObservedFailureClass::StackUnderflow,
        },
    )
    .expect("the reviewed domain has a wire spelling");

    assert_eq!(
        fixture.expected(),
        &ExpectedPrimitiveOutcome::Reject {
            class: ObservedFailureClass::StackUnderflow,
        },
    );
}

#[test]
fn every_group_has_one_spelling() {
    let mut spellings = BTreeSet::new();
    for group in NativeCaseGroup::ALL {
        assert!(
            spellings.insert(group.wire_name()),
            "duplicate group spelling {}",
            group.wire_name(),
        );
        let json = serde_json::to_string(group).expect("a group serializes");
        assert_eq!(json, format!("\"{}\"", group.wire_name()));
    }
    assert_eq!(spellings.len(), NativeCaseGroup::ALL.len());
}
