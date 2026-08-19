//! Guide-11 reproductions and, where a wave has landed, guarantees.
//!
//! # Two kinds of test live here
//!
//! Every test began as a reproduction: it demonstrated a finding from
//! the Guide-11 preflight register by *passing* while the defect was
//! present, asserting that the wrong thing happened. As each wave
//! repairs its finding it flips the assertions of that finding's tests,
//! which then stand as the guarantee that the repair holds.
//!
//! - `G11-R09` is **CLOSED** by Wave 4. The advertised V1 revision was
//!   validated against the V2 census and algebra; V1 is no longer
//!   advertised, the offered revision is checked, and the supported
//!   list holds only revisions a complete definition can be built for.
//! - `G11-R10` is **CLOSED** by Wave 4. The signature weld ignored the
//!   unknown-public-key rule and most of the success algebra; it now
//!   derives one complete expected behaviour from the subcontract and
//!   compares every signature opcode against it.
//! - `G11-H01` is **CLOSED** by Wave 5. The declared maximum stack
//!   growth was welded to nothing; every row is now derived from the
//!   primitive's own success and non-aborting failure effects, plus the
//!   transient the two verifying signature forms hold before their
//!   implicit verification consumes it.
//!
//! No production code path is touched: these are constructions over the
//! public and crate-visible surfaces exactly as an external caller
//! reaches them.

use std::collections::BTreeMap;

use crate::authorization::{
    AuthorizationContract, SignaturePrimitiveContract, UnknownPublicKeyTypeRule,
};
use crate::definition::{
    TargetContractVersion, TargetDefinition, TargetDefinitionParts, reviewed_elements_tapscript,
    validate_target_definition,
};
use crate::error::TargetError;
use crate::opcode::{
    FailureCause, FailureContract, OpcodeId, OpcodeResourceCost, OpcodeSpec, StackContract,
    StackValueType,
};
use crate::operand::OperandContract;
use crate::success::{SuccessCase, SuccessCondition, SuccessContract, SuccessStackEffect};

/// The reviewed contract's parts, as a mutable starting point.
fn parts() -> TargetDefinitionParts {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let source = reviewed.definition();
    TargetDefinitionParts {
        version: source.version(),
        execution_domain: source.execution_domain(),
        leaf_version: source.leaf_version(),
        opcodes: source.opcodes().clone(),
        encodings: source.encodings().clone(),
        pushes: source.pushes().clone(),
        authorization: source.authorization().clone(),
        confidential_values: source.confidential_values().clone(),
        issuance: source.issuance().clone(),
        resources: source.resources().clone(),
        capabilities: source.capabilities().clone(),
        evidence_requirements: source.evidence_requirements().clone(),
    }
}

// -- G11-R09 -----------------------------------------------------------

/// `G11-R09`: a complete V2 body stamped V1 is refused.
///
/// `validate_target_definition` is not version-dispatched: it applies
/// `OpcodeId::ALL` and the current capability, evidence, and encoding
/// censuses whatever revision the definition claims. Advertising V1
/// under that validator meant the revision field did not identify the
/// contract shape it named. V1 is no longer advertised, and the offered
/// revision is checked, so a V2 body cannot travel under a V1 number.
#[test]
fn g11_r09_a_v2_body_stamped_v1_is_refused() {
    assert!(
        !TargetContractVersion::SUPPORTED.contains(&TargetContractVersion::V1),
        "V1 is no longer advertised as supported",
    );

    let mut parts = parts();
    assert_eq!(parts.version, TargetContractVersion::V2);
    parts.version = TargetContractVersion::V1;
    let errors = validate_target_definition(TargetDefinition::new(parts))
        .expect_err("a V2 census stamped V1 is not a validated contract");
    assert!(
        errors.iter().any(|error| matches!(
            error,
            TargetError::UnsupportedTargetContractVersion { offered: 1 },
        )),
        "the refusal names the unimplemented revision, got {errors:?}",
    );
}

/// `G11-R09`: the historical revision cannot be constructed by number.
///
/// The other half of the same repair. A V1 definition used to be
/// refused for lacking V2 primitives it was never meant to carry — a
/// validator applying one global census to a revision it does not
/// implement. Now the number itself is refused, before any census is
/// consulted, and the constant remains only as a historical name.
#[test]
fn g11_r09_the_historical_revision_is_not_constructible_by_number() {
    let error =
        TargetContractVersion::supported(1).expect_err("an unimplemented revision is refused");
    assert!(
        matches!(
            error,
            TargetError::UnsupportedTargetContractVersion { offered: 1 },
        ),
        "the refusal names the offered number, got {error:?}",
    );
    assert_eq!(
        TargetContractVersion::supported(2).expect("V2 is implemented"),
        TargetContractVersion::V2,
    );
    assert_eq!(TargetContractVersion::V1.get(), 1, "the name is retained");
}

/// `G11-R09`: every advertised revision can actually be constructed.
///
/// The review's standing requirement on the list itself: membership
/// means a complete accepted definition of that revision exists, not
/// that the number has been used before. Each supported revision is
/// checked two ways — it round-trips through the constructor, and a
/// complete definition stamped with it validates.
#[test]
fn g11_r09_every_supported_revision_has_a_constructible_definition() {
    assert_eq!(
        TargetContractVersion::SUPPORTED,
        &[TargetContractVersion::V2]
    );

    for revision in TargetContractVersion::SUPPORTED {
        assert_eq!(
            TargetContractVersion::supported(revision.get())
                .expect("a supported revision is accepted by number"),
            *revision,
        );

        let mut parts = parts();
        parts.version = *revision;
        validate_target_definition(TargetDefinition::new(parts))
            .expect("a supported revision has a complete accepted definition");
    }
}

/// `G11-R09`: a V2 definition missing a V2 primitive is still refused.
///
/// The census check must not have been loosened by the revision
/// repair: removing one of the compound-proof primitives from a V2
/// body is incompleteness, and is reported as such.
#[test]
fn g11_r09_a_v2_definition_missing_a_v2_primitive_is_refused() {
    let mut parts = parts();
    assert_eq!(parts.version, TargetContractVersion::V2);
    // One of the compound-proof primitives the second revision added.
    let removed = OpcodeId::Concatenate;
    let mut opcodes: BTreeMap<OpcodeId, OpcodeSpec> = parts.opcodes.clone();
    opcodes.remove(&removed);
    parts.opcodes = opcodes;

    let errors = validate_target_definition(TargetDefinition::new(parts))
        .expect_err("a definition missing a primitive is refused");
    assert!(
        errors
            .iter()
            .any(|error| matches!(error, TargetError::MissingOpcodeContract(id) if *id == removed)),
        "an incomplete V2 census is refused for the primitive it lacks, got {errors:?}",
    );
}

// -- G11-R10 -----------------------------------------------------------

/// Every signature primitive, branching and verifying alike.
const SIGNATURE_OPCODES: &[OpcodeId] = &[
    OpcodeId::CheckSig,
    OpcodeId::CheckSigVerify,
    OpcodeId::CheckSigFromStack,
    OpcodeId::CheckSigFromStackVerify,
];

/// The reviewed parts with the signature subcontract's unknown-key rule
/// replaced, and nothing else touched.
fn with_unknown_key_rule(rule: UnknownPublicKeyTypeRule) -> TargetDefinitionParts {
    let mut parts = parts();
    let authorization = parts.authorization.clone();
    let signature = authorization.signature();

    parts.authorization = AuthorizationContract::new(
        SignaturePrimitiveContract::new(
            signature.public_key_encoding(),
            signature.signature_encoding(),
            signature.empty_signature(),
            signature.invalid_signature(),
            rule,
            signature.budget_per_check(),
            signature.evidence().iter().copied(),
        ),
        authorization.sighash().clone(),
        authorization.relative_timelock().clone(),
    );
    parts
}

/// Rebuild one opcode's spec with a new stack contract, leaving its
/// identity, byte, domains, resources, and evidence alone.
fn with_stack(parts: &mut TargetDefinitionParts, opcode: OpcodeId, stack: StackContract) {
    let spec = parts.opcodes.get(&opcode).expect("the opcode is declared");
    let replaced = OpcodeSpec::new(
        spec.id(),
        spec.code(),
        spec.domains().iter().copied(),
        stack,
        spec.resources(),
        spec.evidence().iter().copied(),
    );
    parts.opcodes.insert(opcode, replaced);
}

/// Assert that a mutated definition is refused, and refused *for the
/// signature weld* rather than for some incidental shape defect.
#[track_caller]
fn refused_by_the_signature_weld(parts: TargetDefinitionParts) {
    let errors = validate_target_definition(TargetDefinition::new(parts))
        .expect_err("a contradictory signature contract is not a validated contract");
    assert!(
        errors.contains(&TargetError::SignatureContractMismatch),
        "the refusal is the signature weld's, got {errors:?}",
    );
}

/// `G11-R10`: the reviewed contract remains valid.
///
/// The control for every mutation below. A weld that refused the
/// reviewed contract would be describing some other target, and each
/// refusal below would mean nothing.
#[test]
fn g11_r10_the_reviewed_signature_contract_remains_valid() {
    validate_target_definition(TargetDefinition::new(parts()))
        .expect("the reviewed contract validates unmutated");
}

/// `G11-R10`: a subcontract rejecting unknown keys cannot coexist with
/// opcodes that succeed on them.
///
/// The finding itself. `weld_signature` never read
/// `unknown_public_key_type`, so the subcontract could say an unknown
/// key type is refused while every signature opcode kept its unknown-key
/// success case and its unknown-nonempty operand admission — a validated
/// contract contradicting itself about one of the target's sharpest
/// authorization behaviours.
#[test]
fn g11_r10_a_contradictory_unknown_key_rule_is_refused() {
    let parts = with_unknown_key_rule(UnknownPublicKeyTypeRule::Rejected);

    // The premise: every signature opcode still advertises the
    // unknown-key success and admission that the changed rule denies.
    for opcode in SIGNATURE_OPCODES {
        let spec = parts.opcodes.get(opcode).expect("the opcode is declared");
        assert!(
            spec.stack()
                .success()
                .cases()
                .iter()
                .any(|case| case.condition() == SuccessCondition::UnknownKeyTypeUnverified),
            "{opcode:?} advertises unknown-key success",
        );
        assert!(
            spec.stack().operands().iter().any(|operand| matches!(
                operand,
                OperandContract::PublicKey {
                    unknown_nonempty_allowed: true,
                    ..
                },
            )),
            "{opcode:?} admits unknown nonempty keys",
        );
    }

    refused_by_the_signature_weld(parts);
}

/// `G11-R10`: removing the unknown-key success case is refused.
///
/// The mirror of the test above, mutating the opcode side instead of
/// the subcontract side. The rule says unknown keys succeed without
/// verification; an opcode with no such form denies it.
#[test]
fn g11_r10_removing_the_unknown_key_success_case_is_refused() {
    let mut parts = parts();
    assert_eq!(
        parts.authorization.signature().unknown_public_key_type(),
        UnknownPublicKeyTypeRule::SucceedsWithoutVerification,
    );

    for opcode in SIGNATURE_OPCODES {
        let spec = parts.opcodes.get(opcode).expect("the opcode is declared");
        let cases = spec
            .stack()
            .success()
            .cases()
            .into_iter()
            .filter(|case| case.condition() != SuccessCondition::UnknownKeyTypeUnverified)
            .collect::<Vec<_>>();
        let stack = StackContract::new(
            spec.stack().operands().to_vec(),
            SuccessContract::Alternatives { cases },
            spec.stack().failure().clone(),
        );
        with_stack(&mut parts, *opcode, stack);
    }

    refused_by_the_signature_weld(parts);
}

/// `G11-R10`: narrowing the key operand against the rule is refused.
///
/// The public-key position can be narrowed to the exact recognized
/// encoding while the success contract still advertises unknown-key
/// success. Every remaining local shape check passes; only a weld
/// reading both views sees it.
#[test]
fn g11_r10_refusing_unknown_nonempty_keys_against_the_rule_is_refused() {
    let mut parts = parts();

    for opcode in SIGNATURE_OPCODES {
        let spec = parts.opcodes.get(opcode).expect("the opcode is declared");
        let operands = spec
            .stack()
            .operands()
            .iter()
            .map(|operand| match operand {
                OperandContract::PublicKey {
                    recognized_encoding,
                    ..
                } => OperandContract::PublicKey {
                    recognized_encoding: *recognized_encoding,
                    unknown_nonempty_allowed: false,
                },
                other => other.clone(),
            })
            .collect::<Vec<_>>();
        let stack = StackContract::new(
            operands,
            spec.stack().success().clone(),
            spec.stack().failure().clone(),
        );
        with_stack(&mut parts, *opcode, stack);
    }

    refused_by_the_signature_weld(parts);
}

/// `G11-R10`: withdrawing empty-signature admission is refused.
///
/// The empty-signature *failure effect* is retained: the subcontract
/// and the opcode both still say what an empty signature does. What is
/// withdrawn is the operand admission that lets an empty item reach it,
/// so the named outcome describes behaviour nothing can produce.
#[test]
fn g11_r10_withdrawing_empty_signature_admission_is_refused() {
    let mut parts = parts();

    for opcode in SIGNATURE_OPCODES {
        let spec = parts.opcodes.get(opcode).expect("the opcode is declared");
        let operands = spec
            .stack()
            .operands()
            .iter()
            .map(|operand| match operand {
                OperandContract::Signature {
                    nonempty_encoding, ..
                } => OperandContract::Signature {
                    nonempty_encoding: *nonempty_encoding,
                    empty_allowed: false,
                },
                other => other.clone(),
            })
            .collect::<Vec<_>>();
        let stack = StackContract::new(
            operands,
            spec.stack().success().clone(),
            spec.stack().failure().clone(),
        );
        with_stack(&mut parts, *opcode, stack);

        // The failure effect really is still there.
        assert!(
            parts
                .opcodes
                .get(opcode)
                .expect("the opcode is declared")
                .stack()
                .failure()
                .effects()
                .iter()
                .any(|effect| effect.cause() == FailureCause::EmptySignature),
            "{opcode:?} keeps its empty-signature failure effect",
        );
    }

    refused_by_the_signature_weld(parts);
}

/// `G11-R10`: a branching form that pushes nothing is refused.
///
/// A branching signature check exists to leave a Boolean a program can
/// branch on. One that pushes nothing is a verifying form wearing a
/// branching form's identity, and the difference is invisible to any
/// check that does not compare the success results against the form.
#[test]
fn g11_r10_a_branching_form_pushing_no_boolean_is_refused() {
    let mut parts = parts();
    let opcode = OpcodeId::CheckSig;
    let spec = parts.opcodes.get(&opcode).expect("the opcode is declared");

    // The premise: it pushes exactly one Boolean before the mutation.
    for case in spec.stack().success().cases() {
        assert_eq!(case.effect().computed_types(), vec![StackValueType::Bool]);
    }

    let cases = spec
        .stack()
        .success()
        .cases()
        .into_iter()
        .map(|case| {
            SuccessCase::new(
                case.condition(),
                SuccessStackEffect::new(case.effect().consumed_operands(), Vec::new()),
            )
        })
        .collect::<Vec<_>>();
    let stack = StackContract::new(
        spec.stack().operands().to_vec(),
        SuccessContract::Alternatives { cases },
        spec.stack().failure().clone(),
    );
    with_stack(&mut parts, opcode, stack);

    refused_by_the_signature_weld(parts);
}

/// `G11-R10`: an empty public key stays a rejection of its own.
///
/// Emptiness and unrecognized-nonempty are different facts — one is
/// refused outright, the other succeeds without verifying — and a
/// contract that dropped the empty-key abort would be saying the
/// forward-compatibility path swallows emptiness too.
#[test]
fn g11_r10_dropping_the_empty_public_key_rejection_is_refused() {
    let mut parts = parts();
    let opcode = OpcodeId::CheckSig;
    let spec = parts.opcodes.get(&opcode).expect("the opcode is declared");
    let effects = spec
        .stack()
        .failure()
        .effects()
        .iter()
        .filter(|effect| effect.cause() != FailureCause::EmptyPublicKey)
        .copied()
        .collect::<Vec<_>>();
    let stack = StackContract::new(
        spec.stack().operands().to_vec(),
        spec.stack().success().clone(),
        FailureContract::new(effects),
    );
    with_stack(&mut parts, opcode, stack);

    refused_by_the_signature_weld(parts);
}

/// Restates one primitive's resource cost.
fn with_resources(
    parts: &mut TargetDefinitionParts,
    opcode: OpcodeId,
    resources: OpcodeResourceCost,
) {
    let spec = parts.opcodes.get(&opcode).expect("the opcode is declared");
    let replacement = OpcodeSpec::new(
        spec.id(),
        spec.code(),
        spec.domains().iter().copied(),
        spec.stack().clone(),
        resources,
        spec.evidence().iter().copied(),
    );
    parts.opcodes.insert(opcode, replacement);
}

/// The reviewed cost of one primitive, with its stack growth replaced.
fn growth_replaced(
    parts: &TargetDefinitionParts,
    opcode: OpcodeId,
    growth: i64,
) -> OpcodeResourceCost {
    let cost = parts
        .opcodes
        .get(&opcode)
        .expect("the opcode is declared")
        .resources();
    OpcodeResourceCost::new(
        cost.script_bytes(),
        cost.operation_cost(),
        cost.validation_budget(),
        growth,
        cost.maximum_altstack_growth(),
    )
}

/// Runs the validator and requires the stack-growth weld to refuse.
fn refused_by_the_stack_growth_weld(parts: TargetDefinitionParts) {
    let errors = validate_target_definition(TargetDefinition::new(parts))
        .expect_err("the mutation must be rejected");
    assert!(errors.contains(&TargetError::StackGrowthContractMismatch));
}

/// `G11-H01`: every declared stack-growth row is the growth its own
/// stack contract implies.
///
/// The census, not a sample. The finding was that the field was welded
/// to nothing at all, so the guarantee has to be that every row is
/// reached, and the count is asserted so that a primitive dropped from
/// the registry cannot quietly shrink the census this test walks.
#[test]
fn g11_h01_every_reviewed_row_matches_its_derived_stack_growth() {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let opcodes = reviewed.definition().opcodes();
    assert_eq!(opcodes.len(), 55);

    for (id, spec) in opcodes {
        assert_eq!(
            spec.resources().maximum_stack_growth(),
            crate::weld::derived_stack_growth(*id, spec.stack()),
            "{id:?} declares a stack growth its stack contract does not imply"
        );
        assert_eq!(
            spec.resources().maximum_altstack_growth(),
            0,
            "{id:?} claims an alternate-stack growth no reviewed primitive produces"
        );
    }
}

/// `G11-H01`: the transient term applies to the verifying forms and to
/// nothing else.
///
/// This is the test that would have caught the wave's open question in
/// either direction. Fifty-three primitives settle at the depth they
/// declare; the two verifying signature forms declare one item above
/// where they settle, because the target pushes a truth value and only
/// then consumes it. If a later reviewer decided the declared rows were
/// wrong instead, this test — not the weld's silence — is what would
/// have to be argued with.
#[test]
fn g11_h01_only_the_verifying_forms_carry_a_transient_above_their_surviving_depth() {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let verifying = [
        (OpcodeId::CheckSigVerify, -2, -1),
        (OpcodeId::CheckSigFromStackVerify, -3, -2),
    ];

    for (id, spec) in reviewed.definition().opcodes() {
        let surviving = crate::weld::surviving_stack_growth(spec.stack());
        let derived = crate::weld::derived_stack_growth(*id, spec.stack());
        match verifying.iter().find(|(opcode, ..)| opcode == id) {
            Some((_, expected_surviving, expected_declared)) => {
                assert_eq!(surviving, *expected_surviving, "{id:?} surviving depth");
                assert_eq!(derived, *expected_declared, "{id:?} transient peak");
                assert_eq!(
                    derived,
                    surviving + 1,
                    "{id:?} carries exactly one transient"
                );
            }
            None => assert_eq!(
                derived, surviving,
                "{id:?} carries a transient the reviewed target does not justify"
            ),
        }
    }
}

/// `G11-H01`: the branching counterparts settle where they peak.
///
/// The transient rule is stated over the verifying forms, so the claim
/// that the branching forms need no such term is asserted rather than
/// left implied by the census test's non-verifying arm.
#[test]
fn g11_h01_the_branching_counterparts_declare_their_settling_depth() {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    for (id, expected) in [(OpcodeId::CheckSig, -1), (OpcodeId::CheckSigFromStack, -2)] {
        let spec = reviewed
            .definition()
            .opcodes()
            .get(&id)
            .expect("the opcode is declared");
        assert_eq!(crate::weld::surviving_stack_growth(spec.stack()), expected);
        assert_eq!(spec.resources().maximum_stack_growth(), expected);
    }
}

/// `G11-H01`: a growth row inconsistent with its own stack contract is
/// refused.
///
/// The finding itself. No weld read the field, so a row transcribed
/// from the wrong primitive validated.
#[test]
fn g11_h01_a_growth_row_inconsistent_with_its_stack_contract_is_refused() {
    let mut parts = parts();
    let cost = growth_replaced(&parts, OpcodeId::CheckSig, 3);
    with_resources(&mut parts, OpcodeId::CheckSig, cost);

    refused_by_the_stack_growth_weld(parts);
}

/// `G11-H01`: a verifying form declaring only its surviving depth is
/// refused.
///
/// The adjudication, made enforceable. The surviving figure is the one
/// a derivation without the transient rule produces, and it is exactly
/// the value a scheduler must not size the stack from.
#[test]
fn g11_h01_a_verifying_form_declaring_its_surviving_growth_is_refused() {
    for (opcode, surviving) in [
        (OpcodeId::CheckSigVerify, -2),
        (OpcodeId::CheckSigFromStackVerify, -3),
    ] {
        let mut parts = parts();
        let cost = growth_replaced(&parts, opcode, surviving);
        with_resources(&mut parts, opcode, cost);

        refused_by_the_stack_growth_weld(parts);
    }
}

/// `G11-H01`: moving a primitive's stack arithmetic without its growth
/// row is refused.
///
/// The mirror of the row mutation. The weld has to hold whichever side
/// of the repetition moves, or it only checks that one view was left
/// alone.
#[test]
fn g11_h01_changing_a_success_form_without_its_growth_row_is_refused() {
    let mut parts = parts();
    let opcode = OpcodeId::CheckSig;
    let spec = parts.opcodes.get(&opcode).expect("the opcode is declared");
    let stack = StackContract::new(
        spec.stack().operands().to_vec(),
        SuccessContract::Fixed {
            consumed_operands: 2,
            results: vec![StackValueType::Bool, StackValueType::Bool],
        },
        spec.stack().failure().clone(),
    );
    with_stack(&mut parts, opcode, stack);

    refused_by_the_stack_growth_weld(parts);
}

/// `G11-H01`: an alternate-stack claim no reviewed primitive produces
/// is refused.
///
/// Altstack growth stays zero until a reviewed primitive changes it,
/// and "stays zero" is a checked property here rather than a sentence
/// in the guide.
#[test]
fn g11_h01_a_nonzero_altstack_growth_row_is_refused() {
    let mut parts = parts();
    let cost = parts
        .opcodes
        .get(&OpcodeId::CheckSig)
        .expect("the opcode is declared")
        .resources();
    with_resources(
        &mut parts,
        OpcodeId::CheckSig,
        OpcodeResourceCost::new(
            cost.script_bytes(),
            cost.operation_cost(),
            cost.validation_budget(),
            cost.maximum_stack_growth(),
            1,
        ),
    );

    refused_by_the_stack_growth_weld(parts);
}

/// `G11-H01`: the reviewed contract remains valid under the new weld.
///
/// The control. A weld that refused the reviewed contract would be a
/// finding about the weld, not about the target.
#[test]
fn g11_h01_the_reviewed_contract_remains_valid() {
    validate_target_definition(TargetDefinition::new(parts()))
        .expect("the reviewed contract validates unmutated");
}
