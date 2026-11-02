//! Exact compact comparison classes over retained emitted controls.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};
use compiler::operation_plan::{LifecycleExitStatus, LifecycleRequirement};
use target_elements::OpcodeId;

use crate::pattern::ash_input_recognition_fragment;
use crate::tests::{compact_ash_plan, pattern_symbols, reviewed_target};
use crate::{
    AbstractLimits, AbstractStackState, CandidateRelocatableTapscriptBundle,
    CompactEmittedLifecycleRefusal as LifecycleRefusal,
    CompactEmittedRepresentationRefusal as RepresentationRefusal,
    CompactSecretFreePathRefusal as SecretRefusal, LeafRole, SponsorChangePresence,
    TapscriptInstruction as Instruction, TapscriptProgram, demonstration_policy,
    emit_candidate_bundle, validate_compact_emitted_lifecycle,
    validate_compact_emitted_representation, validate_compact_secret_free_path, validate_program,
};

fn bundle() -> CandidateRelocatableTapscriptBundle {
    let target = reviewed_target();
    emit_candidate_bundle(
        &target,
        &compact_ash_plan(),
        demonstration_policy(),
        pattern_symbols(&target),
    )
    .expect("the retained bundle emits")
}

fn coordinator(bundle: &CandidateRelocatableTapscriptBundle) -> LeafRole {
    let shape = bundle
        .shapes()
        .shapes()
        .find(|shape| {
            shape.ash_inputs() == 2
                && shape.sponsor_inputs() == 0
                && shape.sponsor_change() == SponsorChangePresence::Absent
        })
        .expect("the retained sponsorless shape exists");
    LeafRole::Coordinator { shape }
}

fn leaves(bundle: &CandidateRelocatableTapscriptBundle) -> BTreeMap<LeafRole, TapscriptProgram> {
    bundle
        .constructor()
        .leaves()
        .iter()
        .map(|(key, leaf)| (*key, leaf.program().clone()))
        .collect()
}

fn outstanding(bundle: &CandidateRelocatableTapscriptBundle) -> BTreeSet<LifecycleRequirement> {
    bundle
        .outstanding_lifecycle()
        .requirements()
        .cloned()
        .collect()
}

#[test]
fn retained_protocol_leaves_accept_and_one_inserted_signature_primitive_is_refused() {
    let bundle = bundle();
    for leaf in bundle.constructor().leaves().values() {
        assert_eq!(leaf.witness().data_items(), 0);
        assert_eq!(validate_compact_secret_free_path(leaf.program()), Ok(()));
    }
    let control = bundle
        .constructor()
        .leaf(coordinator(&bundle))
        .expect("leaf")
        .program();
    let mut instructions = control.instructions().to_vec();
    instructions.insert(0, Instruction::Opcode(OpcodeId::CheckSig));
    let subject = TapscriptProgram::new(instructions.clone()).expect("bounded instructions");
    assert_eq!(
        validate_compact_secret_free_path(&subject),
        Err(SecretRefusal::UnexpectedProtocolSecret),
    );
    assert_eq!(
        instructions.remove(0),
        Instruction::Opcode(OpcodeId::CheckSig)
    );
    assert_eq!(
        TapscriptProgram::new(instructions).expect("restored"),
        *control
    );
}

#[test]
fn a_stack_failure_without_authorization_receives_no_secret_refusal() {
    let bundle = bundle();
    let control = bundle
        .constructor()
        .leaf(coordinator(&bundle))
        .expect("leaf")
        .program();
    let mut instructions = control.instructions().to_vec();
    instructions.insert(0, Instruction::Opcode(OpcodeId::Drop));
    let subject = TapscriptProgram::new(instructions.clone()).expect("bounded instructions");
    let target = reviewed_target();
    assert!(
        validate_program(
            &target,
            &subject,
            &AbstractStackState::from_main(Vec::new()),
            AbstractLimits::for_target(&target),
        )
        .is_err()
    );
    assert_eq!(validate_compact_secret_free_path(&subject), Ok(()));
    instructions.remove(0);
    assert_eq!(
        TapscriptProgram::new(instructions).expect("restored"),
        *control
    );
}

#[test]
fn only_the_value_prefix_equality_omission_receives_representation_credit() {
    let bundle = bundle();
    let target = reviewed_target();
    let control = ash_input_recognition_fragment(&target, bundle.unresolved_symbols(), 0)
        .expect("canonical recognition");
    let compare = |offered: &TapscriptProgram| {
        validate_compact_emitted_representation(&target, &bundle, offered)
    };
    assert_eq!(compare(&control), Ok(()));
    let inspection = control
        .instructions()
        .iter()
        .position(|instruction| *instruction == Instruction::Opcode(OpcodeId::InspectInputValue))
        .expect("value introspection");
    assert!(matches!(
        control.instructions()[inspection + 1],
        Instruction::Push(_)
    ));
    let authentication = inspection + 2;
    assert_eq!(
        control.instructions()[authentication],
        Instruction::Opcode(OpcodeId::EqualVerify)
    );
    let mut instructions = control.instructions().to_vec();
    let removed = instructions.remove(authentication);
    assert_eq!(
        compare(&TapscriptProgram::new(instructions.clone()).expect("subject")),
        Err(RepresentationRefusal::MissingExplicitValueAuthentication),
    );
    instructions.insert(authentication, removed);
    assert_eq!(
        TapscriptProgram::new(instructions.clone()).expect("restored"),
        control
    );

    let other = instructions
        .iter()
        .position(|instruction| *instruction == Instruction::Opcode(OpcodeId::EqualVerify))
        .expect("asset recognition equality");
    assert_ne!(other, authentication);
    let removed = instructions.remove(other);
    assert_eq!(
        compare(&TapscriptProgram::new(instructions.clone()).expect("neighbour")),
        Err(RepresentationRefusal::RecognitionStructureMismatch),
    );
    instructions.insert(other, removed);
    assert_eq!(
        TapscriptProgram::new(instructions.clone()).expect("restored"),
        control
    );

    instructions.insert(0, Instruction::Opcode(OpcodeId::CheckSig));
    assert_eq!(
        compare(&TapscriptProgram::new(instructions.clone()).expect("neighbour")),
        Err(RepresentationRefusal::RecognitionStructureMismatch),
    );
    instructions.remove(0);
    assert_eq!(
        TapscriptProgram::new(instructions).expect("restored"),
        control
    );
}

#[test]
fn one_omitted_coordinator_path_is_a_keyed_compact_exit_refusal() {
    let bundle = bundle();
    let control = leaves(&bundle);
    let declarations = outstanding(&bundle);
    assert_eq!(
        validate_compact_emitted_lifecycle(&bundle, &control, &declarations),
        Ok(())
    );
    let key = coordinator(&bundle);
    let mut subject = control.clone();
    let removed = subject.remove(&key).expect("retained coordinator");
    assert_eq!(
        validate_compact_emitted_lifecycle(&bundle, &subject, &declarations),
        Err(LifecycleRefusal::MissingCompactExit { leaf: key }),
    );
    subject.insert(key, removed);
    assert_eq!(subject, control);
}

#[test]
fn one_omitted_member_path_receives_no_keyed_compact_exit_credit() {
    let bundle = bundle();
    let control = leaves(&bundle);
    let declarations = outstanding(&bundle);
    assert_eq!(
        validate_compact_emitted_lifecycle(&bundle, &control, &declarations),
        Ok(())
    );
    let key = LeafRole::Member { ash_inputs: 2 };
    let mut subject = control.clone();
    let removed = subject.remove(&key).expect("retained member");
    assert_eq!(
        validate_compact_emitted_lifecycle(&bundle, &subject, &declarations),
        Err(LifecycleRefusal::ConstructorStructureMismatch),
    );
    subject.insert(key, removed);
    assert_eq!(subject, control);
}

#[test]
fn one_omitted_clear_declaration_is_not_an_emitted_path_claim() {
    let bundle = bundle();
    let programs = leaves(&bundle);
    let control = outstanding(&bundle);
    assert_eq!(control.len(), 1);
    let requirement = control.first().expect("outstanding requirement").clone();
    assert_eq!(requirement.object, ObjectId::Ash);
    assert_eq!(requirement.exit, OperationId::Clear);
    assert_eq!(
        requirement.representation,
        realization::RepresentationMode::Explicit
    );
    assert_eq!(
        requirement.status,
        LifecycleExitStatus::DeclaredOutsideCompilerScope
    );
    assert_eq!(
        validate_compact_emitted_lifecycle(&bundle, &programs, &control),
        Ok(())
    );
    let mut subject = control.clone();
    assert!(subject.remove(&requirement));
    assert_eq!(
        validate_compact_emitted_lifecycle(&bundle, &programs, &subject),
        Err(LifecycleRefusal::MissingClearDeclaration {
            requirement: requirement.clone()
        }),
    );
    subject.insert(requirement);
    assert_eq!(subject, control);
}

#[test]
fn lifecycle_neighbours_and_simultaneous_omissions_receive_no_single_omission_credit() {
    let bundle = bundle();
    let programs = leaves(&bundle);
    let declarations = outstanding(&bundle);
    let key = coordinator(&bundle);
    let mut changed_programs = programs.clone();
    let original = changed_programs
        .insert(key, TapscriptProgram::new(Vec::new()).expect("empty"))
        .expect("retained program");
    assert_eq!(
        validate_compact_emitted_lifecycle(&bundle, &changed_programs, &declarations),
        Err(LifecycleRefusal::ConstructorStructureMismatch),
    );
    changed_programs.insert(key, original);
    assert_eq!(changed_programs, programs);

    let requirement = declarations
        .first()
        .expect("outstanding requirement")
        .clone();
    let mut changed_declarations = declarations.clone();
    changed_declarations.remove(&requirement);
    let mut changed = requirement.clone();
    changed.status = LifecycleExitStatus::AvailableInCompilerScope;
    changed_declarations.insert(changed.clone());
    assert_eq!(
        validate_compact_emitted_lifecycle(&bundle, &programs, &changed_declarations),
        Err(LifecycleRefusal::OutstandingDeclarationsMismatch),
    );
    changed_declarations.remove(&changed);
    changed_declarations.insert(requirement.clone());
    assert_eq!(changed_declarations, declarations);

    let removed = changed_programs.remove(&key).expect("retained program");
    changed_declarations.remove(&requirement);
    assert_eq!(
        validate_compact_emitted_lifecycle(&bundle, &changed_programs, &changed_declarations),
        Err(LifecycleRefusal::ConstructorStructureMismatch),
    );
    changed_programs.insert(key, removed);
    changed_declarations.insert(requirement);
    assert_eq!(changed_programs, programs);
    assert_eq!(changed_declarations, declarations);
}
