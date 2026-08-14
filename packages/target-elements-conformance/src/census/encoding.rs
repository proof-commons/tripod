//! The execution domain, the leaf version, instruction bytes, literal
//! pushes, and the resource boundaries.
//!
//! # An unreviewed leaf version does not refuse the script
//!
//! It does something more interesting: the reviewed semantics are not
//! applied at all. A script that the reviewed leaf would abort — a
//! primitive with nothing to consume — is accepted under a leaf version
//! nobody has assigned semantics to, because nothing executes it. That
//! is the observation the leaf-version row needs, and no fixture stated
//! at the reviewed version can make it (Guide-9 §13.1).
//!
//! # Reviewed extension bytes are not success opcodes
//!
//! In the reviewed domain a range of bytes is reserved so that future
//! rules can assign them, and a script using an unassigned one succeeds
//! outright. The reviewed extension primitives occupy bytes inside that
//! range, so a case where one of them *fails* is what establishes that
//! the target executes it rather than treating it as a licence to
//! succeed.
//!
//! # Consensus and relay are different rules
//!
//! A nonminimally encoded literal is a valid spend that nodes decline to
//! forward. The reviewed push contract says so — it classes the defect
//! as a relay rule and the oversized and truncated ones as the target's
//! own — and these cases state the consensus verdict, which is
//! acceptance. An executor answering at relay level would disagree, and
//! the fixture says which layer it is stating so the disagreement is
//! legible rather than mysterious.

use tapscript::TapscriptInstruction;
use target_elements::OpcodeId;

use crate::census::author::{Case, CensusAuthor, falsity, op, push, truth};
use crate::census::context::census_transaction;
use crate::census::material::{
    SIGNATURE_VECTOR_KEY, SIGNATURE_VECTOR_MESSAGE, SIGNATURE_VECTOR_SIGNATURE,
};
use crate::fixture::{EnforcementLayer, ExpectedPrimitiveOutcome, NativeCaseGroup};
use crate::protocol::ObservedFailureClass;

/// A leaf version byte the reviewed contract does not describe.
///
/// The value another chain assigns to its own script leaf, which is
/// exactly the byte a careless transcription would have used for this
/// target's.
const UNREVIEWED_LEAF_VERSION: u8 = 0xc0;

/// The widest literal the target admits.
const MAXIMUM_LITERAL: usize = 520;

/// The widest literal an opcode's own byte can state the width of.
const LARGEST_DIRECT_LITERAL: usize = 75;

/// The widest literal a one-byte width can state.
const LARGEST_ONE_BYTE_LITERAL: usize = 255;

/// The byte the wide literals are filled with.
const FILLER: u8 = 0x2a;

/// The opcode introducing a one-byte width.
///
/// Stated here only to build bytes no typed program will produce; the
/// reviewed contract owns the fact, and these cases are refused or
/// relay-refused precisely because they use it wrongly.
const ONE_BYTE_WIDTH_OPCODE: u8 = 0x4c;

/// The opcode introducing a two-byte width.
const TWO_BYTE_WIDTH_OPCODE: u8 = 0x4d;

/// The opcode introducing a four-byte width.
const FOUR_BYTE_WIDTH_OPCODE: u8 = 0x4e;

/// A byte the target executes nothing for.
const INVALID_OPCODE: u8 = 0xff;

/// Every domain, leaf, instruction, push, and resource case.
pub fn cases(author: &mut CensusAuthor<'_>) {
    execution_domain(author);
    leaf_version(author);
    instruction_bytes(author);
    push_forms(author);
    resource_boundaries(author);
}

/// One case with no transaction context.
const fn bare(
    group: NativeCaseGroup,
    opcode: Option<OpcodeId>,
    script: &[TapscriptInstruction],
) -> Case<'_> {
    Case {
        group,
        opcode,
        script,
        stack: &[],
        context: None,
    }
}

/// That the reviewed primitives execute in the reviewed domain.
fn execution_domain(author: &mut CensusAuthor<'_>) {
    let group = NativeCaseGroup::ExecutionDomain;

    // A reviewed extension primitive.
    let script = [
        push(author.le64(2)),
        push(author.le64(3)),
        op(OpcodeId::Add64),
        op(OpcodeId::ScriptNumToLe64),
        op(OpcodeId::GreaterThan64),
    ];
    author.accept(bare(group, Some(OpcodeId::Add64), &script), vec![truth()]);

    // A standard signature primitive, over the published vector.
    let stack = [
        author.item(SIGNATURE_VECTOR_SIGNATURE.to_vec()),
        author.item(SIGNATURE_VECTOR_MESSAGE.to_vec()),
        author.item(SIGNATURE_VECTOR_KEY.to_vec()),
    ];
    let script = [op(OpcodeId::CheckSigFromStack)];
    let case = Case {
        group,
        opcode: Some(OpcodeId::CheckSigFromStack),
        script: &script,
        stack: &stack,
        context: None,
    };
    author.accept(case, vec![truth()]);

    // A standard timelock primitive.
    let stack = [author.number(1)];
    let script = [op(OpcodeId::CheckSequenceVerify)];
    let case = Case {
        group,
        opcode: Some(OpcodeId::CheckSequenceVerify),
        script: &script,
        stack: &stack,
        context: Some(crate::census::context::timelock_transaction(2, 10)),
    };
    let expected = vec![author.number_bytes(1)];
    author.accept(case, expected);
}

/// That the reviewed leaf version is what selects those semantics.
fn leaf_version(author: &mut CensusAuthor<'_>) {
    let group = NativeCaseGroup::LeafVersion;

    // Under the reviewed leaf, the primitive executes and fails for want
    // of an operand.
    let script = [op(OpcodeId::Add64)];
    author.reject(
        bare(group, Some(OpcodeId::Add64), &script),
        &[ObservedFailureClass::StackUnderflow],
    );

    // Under an unreviewed leaf, the same script is not executed at all,
    // and the spend stands.
    author.unreviewed_leaf(
        bare(group, Some(OpcodeId::Add64), &script),
        UNREVIEWED_LEAF_VERSION,
        ExpectedPrimitiveOutcome::Accept {
            static_final_stack: None,
            static_final_altstack: None,
        },
    );

    // And a script the reviewed leaf accepts is accepted there too.
    let script = [
        push(author.le64(3)),
        push(author.le64(1)),
        op(OpcodeId::GreaterThan64),
    ];
    author.accept(
        bare(group, Some(OpcodeId::GreaterThan64), &script),
        vec![truth()],
    );
    author.unreviewed_leaf(
        bare(group, Some(OpcodeId::GreaterThan64), &script),
        UNREVIEWED_LEAF_VERSION,
        ExpectedPrimitiveOutcome::Accept {
            static_final_stack: None,
            static_final_altstack: None,
        },
    );
}

/// That reviewed bytes decode as reviewed primitives.
fn instruction_bytes(author: &mut CensusAuthor<'_>) {
    let group = NativeCaseGroup::InstructionEncoding;

    let script = [
        push(author.le64(2)),
        push(author.le64(3)),
        op(OpcodeId::Add64),
        op(OpcodeId::ScriptNumToLe64),
        op(OpcodeId::GreaterThan64),
    ];
    author.accept(bare(group, Some(OpcodeId::Add64), &script), vec![truth()]);

    // Each of these fails for want of an operand, which is only possible
    // if the byte was executed rather than treated as a success opcode.
    for id in [
        OpcodeId::Add64,
        OpcodeId::Sha256Initialize,
        OpcodeId::Le64ToScriptNum,
        OpcodeId::EcMulScalarVerify,
    ] {
        let script = [op(id)];
        author.reject(
            bare(group, Some(id), &script),
            &[ObservedFailureClass::StackUnderflow],
        );
    }

    // A byte the target executes nothing for.
    author.malformed(
        group,
        None,
        vec![INVALID_OPCODE],
        ExpectedPrimitiveOutcome::reject([ObservedFailureClass::UnknownOpcode], None),
        EnforcementLayer::Consensus,
    );

    // An introspection primitive still needs its context to be the
    // transaction, which is what a case carrying one establishes.
    let script = [op(OpcodeId::InspectNumInputs)];
    let case = Case {
        group,
        opcode: Some(OpcodeId::InspectNumInputs),
        script: &script,
        stack: &[],
        context: Some(census_transaction()),
    };
    let expected = vec![author.number_bytes(crate::census::context::INPUT_COUNT)];
    author.accept(case, expected);
}

/// The literal forms, each carrying a payload that is its own result.
fn push_forms(author: &mut CensusAuthor<'_>) {
    let group = NativeCaseGroup::PushEncoding;

    for payload in [
        vec![0x05],
        vec![0x10],
        vec![0x81],
        vec![0x20],
        vec![FILLER; 2],
        vec![FILLER; LARGEST_DIRECT_LITERAL],
        vec![FILLER; LARGEST_DIRECT_LITERAL + 1],
        vec![FILLER; LARGEST_ONE_BYTE_LITERAL],
        vec![FILLER; LARGEST_ONE_BYTE_LITERAL + 1],
        vec![FILLER; MAXIMUM_LITERAL],
    ] {
        let item = author.item(payload.clone());
        let script = [push(item)];
        author.accept(bare(group, None, &script), vec![payload]);
    }

    // The empty literal and a single zero byte are both the target's
    // false, so each is its own script's value.
    for payload in [Vec::new(), vec![0x00]] {
        let item = author.item(payload.clone());
        let script = [push(item)];
        author.evaluated_false(bare(group, None, &script), vec![payload]);
    }

    // Bytes no typed program produces: a payload that runs off the end,
    // a width byte with nothing after it, and a width that overruns.
    //
    // The target answers a push it cannot read and a byte it does not
    // execute with one code: both are the parser failing to produce an
    // instruction, and it does not record which. So the class the
    // reviewed contract names is admitted alongside the one the target
    // reports, which is what a class set is for.
    for script in [
        vec![0x02, 0xaa],
        vec![ONE_BYTE_WIDTH_OPCODE],
        vec![ONE_BYTE_WIDTH_OPCODE, 0x04, 0xaa, 0xbb],
        vec![TWO_BYTE_WIDTH_OPCODE, 0x02],
    ] {
        author.malformed(
            group,
            None,
            script,
            ExpectedPrimitiveOutcome::reject(
                [
                    ObservedFailureClass::MalformedPush,
                    ObservedFailureClass::UnknownOpcode,
                ],
                None,
            ),
            EnforcementLayer::Consensus,
        );
    }
}

/// The boundaries the target's own rules and a node's relay rules draw.
fn resource_boundaries(author: &mut CensusAuthor<'_>) {
    let group = NativeCaseGroup::Resource;

    // The widest literal the target accepts, and one byte more.
    let payload = vec![FILLER; MAXIMUM_LITERAL];
    let item = author.item(payload.clone());
    let script = [push(item)];
    author.accept(bare(group, None, &script), vec![payload]);

    let mut oversized = vec![TWO_BYTE_WIDTH_OPCODE, 0x09, 0x02];
    oversized.extend(std::iter::repeat_n(FILLER, MAXIMUM_LITERAL + 1));
    author.malformed(
        group,
        None,
        oversized,
        ExpectedPrimitiveOutcome::reject([ObservedFailureClass::MalformedPush], None),
        EnforcementLayer::Consensus,
    );

    // Three nonminimal forms of the same one-byte literal. The reviewed
    // push contract classes nonminimality as a relay rule, so the
    // target's own verdict is acceptance — which is exactly what these
    // cases state, at the consensus layer.
    for script in [
        vec![ONE_BYTE_WIDTH_OPCODE, 0x01, 0x05],
        vec![TWO_BYTE_WIDTH_OPCODE, 0x01, 0x00, 0x05],
        vec![FOUR_BYTE_WIDTH_OPCODE, 0x01, 0x00, 0x00, 0x00, 0x05],
    ] {
        author.malformed(
            group,
            None,
            script,
            ExpectedPrimitiveOutcome::accept(Some(vec![vec![0x05]])),
            EnforcementLayer::Consensus,
        );
    }

    // A nonminimal empty literal, whose payload is the target's false.
    author.malformed(
        group,
        None,
        vec![ONE_BYTE_WIDTH_OPCODE, 0x00],
        ExpectedPrimitiveOutcome::reject(
            [ObservedFailureClass::EvaluatedFalse],
            Some(vec![falsity()]),
        ),
        EnforcementLayer::Consensus,
    );
}
