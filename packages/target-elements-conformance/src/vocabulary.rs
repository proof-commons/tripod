//! Wire spellings for the reviewed target identities that appear in
//! protocol and report data.
//!
//! # Why a table rather than a derived name
//!
//! Protocol and report data crosses a process boundary and is compared
//! for evidence, so the spelling of a primitive has to be a decision
//! this package made and can test, not a by-product of a `Debug`
//! implementation that a rename in another crate would silently change.
//! Each table below is therefore explicit, and a census test proves it
//! covers its owning crate's complete census exactly, with no duplicate
//! spelling and no spelling naming two identities.
//!
//! # These are names, not semantics
//!
//! A spelling here carries no target byte, no stack effect, no width,
//! and no gating. It is the string a reviewed identity travels under.
//! Nothing may read semantics out of it, and nothing may reconstruct a
//! target fact from it: the reviewed contract remains the only source of
//! those, and this table would be an unreviewed second one if it carried
//! any.

use target_elements::confidential::OpeningBlocker;
use target_elements::{ElementsCapability, OpcodeId, TargetEvidenceRequirementId};

/// The wire spelling of every reviewed opening blocker.
///
/// The blockers are Wave 5's own review results, and a disposition record
/// names them rather than restating them. They reach a report through
/// this table for the same reason every other identity here does: the
/// owning crate is standard-library-only by decision (Guide 11 §6.1), so
/// it derives no serialization, and a spelling invented at the point of
/// use would be one this package could not test.
const OPENING_BLOCKER_NAMES: &[(OpeningBlocker, &str)] = &[
    (
        OpeningBlocker::GeneratorNotDerivableOnScript,
        "generator_not_derivable_on_script",
    ),
    (
        OpeningBlocker::EncodingDomainMismatch,
        "encoding_domain_mismatch",
    ),
    (
        OpeningBlocker::SuppliedParityUnbound,
        "supplied_parity_unbound",
    ),
];

/// The wire spelling of every reviewed primitive.
const OPCODE_NAMES: &[(OpcodeId, &str)] = &[
    (OpcodeId::Sha256Initialize, "sha256_initialize"),
    (OpcodeId::Sha256Update, "sha256_update"),
    (OpcodeId::Sha256Finalize, "sha256_finalize"),
    (OpcodeId::InspectInputOutpoint, "inspect_input_outpoint"),
    (OpcodeId::InspectInputAsset, "inspect_input_asset"),
    (OpcodeId::InspectInputValue, "inspect_input_value"),
    (
        OpcodeId::InspectInputScriptPubKey,
        "inspect_input_script_pub_key",
    ),
    (OpcodeId::InspectInputSequence, "inspect_input_sequence"),
    (OpcodeId::InspectInputIssuance, "inspect_input_issuance"),
    (OpcodeId::PushCurrentInputIndex, "push_current_input_index"),
    (OpcodeId::InspectOutputAsset, "inspect_output_asset"),
    (OpcodeId::InspectOutputValue, "inspect_output_value"),
    (OpcodeId::InspectOutputNonce, "inspect_output_nonce"),
    (
        OpcodeId::InspectOutputScriptPubKey,
        "inspect_output_script_pub_key",
    ),
    (OpcodeId::InspectVersion, "inspect_version"),
    (OpcodeId::InspectLockTime, "inspect_lock_time"),
    (OpcodeId::InspectNumInputs, "inspect_num_inputs"),
    (OpcodeId::InspectNumOutputs, "inspect_num_outputs"),
    (OpcodeId::TxWeight, "tx_weight"),
    (OpcodeId::Add64, "add64"),
    (OpcodeId::Sub64, "sub64"),
    (OpcodeId::Mul64, "mul64"),
    (OpcodeId::Div64, "div64"),
    (OpcodeId::Neg64, "neg64"),
    (OpcodeId::LessThan64, "less_than64"),
    (OpcodeId::LessThanOrEqual64, "less_than_or_equal64"),
    (OpcodeId::GreaterThan64, "greater_than64"),
    (OpcodeId::GreaterThanOrEqual64, "greater_than_or_equal64"),
    (OpcodeId::ScriptNumToLe64, "script_num_to_le64"),
    (OpcodeId::Le64ToScriptNum, "le64_to_script_num"),
    (OpcodeId::Le32ToLe64, "le32_to_le64"),
    (OpcodeId::EcMulScalarVerify, "ec_mul_scalar_verify"),
    (OpcodeId::TweakVerify, "tweak_verify"),
    (OpcodeId::CheckSig, "check_sig"),
    (OpcodeId::CheckSigVerify, "check_sig_verify"),
    (OpcodeId::CheckSigFromStack, "check_sig_from_stack"),
    (
        OpcodeId::CheckSigFromStackVerify,
        "check_sig_from_stack_verify",
    ),
    (OpcodeId::CheckSequenceVerify, "check_sequence_verify"),
    (OpcodeId::Duplicate, "duplicate"),
    (OpcodeId::DuplicateTwo, "duplicate_two"),
    (OpcodeId::CopyOver, "copy_over"),
    (OpcodeId::Swap, "swap"),
    (OpcodeId::Rotate, "rotate"),
    (OpcodeId::RemoveSecond, "remove_second"),
    (OpcodeId::Tuck, "tuck"),
    (OpcodeId::Drop, "drop"),
    (OpcodeId::DropTwo, "drop_two"),
    (OpcodeId::Equal, "equal"),
    (OpcodeId::EqualVerify, "equal_verify"),
    (OpcodeId::Verify, "verify"),
    (OpcodeId::Concatenate, "concatenate"),
    (OpcodeId::Size, "size"),
    (OpcodeId::Substring, "substring"),
    (OpcodeId::BitwiseAnd, "bitwise_and"),
    (OpcodeId::BitwiseXor, "bitwise_xor"),
];

/// The wire spelling of every target capability.
const CAPABILITY_NAMES: &[(ElementsCapability, &str)] = &[
    (
        ElementsCapability::TapscriptExecution,
        "tapscript_execution",
    ),
    (
        ElementsCapability::RequiredLeafVersion,
        "required_leaf_version",
    ),
    (
        ElementsCapability::InputCountInspection,
        "input_count_inspection",
    ),
    (
        ElementsCapability::OutputCountInspection,
        "output_count_inspection",
    ),
    (
        ElementsCapability::CurrentInputIndexInspection,
        "current_input_index_inspection",
    ),
    (
        ElementsCapability::InputOutpointInspection,
        "input_outpoint_inspection",
    ),
    (
        ElementsCapability::InputAssetInspection,
        "input_asset_inspection",
    ),
    (
        ElementsCapability::InputValueInspection,
        "input_value_inspection",
    ),
    (
        ElementsCapability::InputProgramInspection,
        "input_program_inspection",
    ),
    (
        ElementsCapability::InputSequenceInspection,
        "input_sequence_inspection",
    ),
    (
        ElementsCapability::InputIssuanceInspection,
        "input_issuance_inspection",
    ),
    (
        ElementsCapability::OutputAssetInspection,
        "output_asset_inspection",
    ),
    (
        ElementsCapability::OutputValueInspection,
        "output_value_inspection",
    ),
    (
        ElementsCapability::OutputNonceInspection,
        "output_nonce_inspection",
    ),
    (
        ElementsCapability::OutputProgramInspection,
        "output_program_inspection",
    ),
    (
        ElementsCapability::TransactionVersionInspection,
        "transaction_version_inspection",
    ),
    (
        ElementsCapability::TransactionLockTimeInspection,
        "transaction_lock_time_inspection",
    ),
    (
        ElementsCapability::TransactionWeightInspection,
        "transaction_weight_inspection",
    ),
    (
        ElementsCapability::SignedFixedWidthArithmetic,
        "signed_fixed_width_arithmetic",
    ),
    (
        ElementsCapability::SignedFixedWidthComparison,
        "signed_fixed_width_comparison",
    ),
    (
        ElementsCapability::ScriptNumberConversion,
        "script_number_conversion",
    ),
    (ElementsCapability::StreamingSha256, "streaming_sha256"),
    (
        ElementsCapability::EcScalarVerification,
        "ec_scalar_verification",
    ),
    (ElementsCapability::TweakVerification, "tweak_verification"),
    (
        ElementsCapability::SignatureVerification,
        "signature_verification",
    ),
    (
        ElementsCapability::StackMessageSignatureVerification,
        "stack_message_signature_verification",
    ),
    (
        ElementsCapability::OutputCommittingSighash,
        "output_committing_sighash",
    ),
    (
        ElementsCapability::InputCommitmentControl,
        "input_commitment_control",
    ),
    (ElementsCapability::RelativeTimelock, "relative_timelock"),
    (
        ElementsCapability::StackRearrangement,
        "stack_rearrangement",
    ),
    (
        ElementsCapability::ByteStringEquality,
        "byte_string_equality",
    ),
    (
        ElementsCapability::BooleanVerification,
        "boolean_verification",
    ),
    (
        ElementsCapability::ByteStringConcatenation,
        "byte_string_concatenation",
    ),
    (ElementsCapability::ByteStringWidth, "byte_string_width"),
    (ElementsCapability::ByteStringSlicing, "byte_string_slicing"),
    (ElementsCapability::BitwiseByteLogic, "bitwise_byte_logic"),
    (
        ElementsCapability::CanonicalByteOrdering,
        "canonical_byte_ordering",
    ),
    (
        ElementsCapability::ConfidentialValueConservation,
        "confidential_value_conservation",
    ),
    (
        ElementsCapability::CommitmentEquality,
        "commitment_equality",
    ),
    (
        ElementsCapability::AuthenticatedValueOpening,
        "authenticated_value_opening",
    ),
    (
        ElementsCapability::ExplicitValueInspection,
        "explicit_value_inspection",
    ),
    (
        ElementsCapability::IssuanceIntrospection,
        "issuance_introspection",
    ),
    (
        ElementsCapability::ReissuanceIntrospection,
        "reissuance_introspection",
    ),
    (
        ElementsCapability::ConsensusResourceLimits,
        "consensus_resource_limits",
    ),
    (
        ElementsCapability::PolicyResourceLimits,
        "policy_resource_limits",
    ),
];

/// The wire spelling of every target evidence requirement.
const EVIDENCE_NAMES: &[(TargetEvidenceRequirementId, &str)] = &[
    (
        TargetEvidenceRequirementId::TapscriptExecutionDomain,
        "tapscript_execution_domain",
    ),
    (
        TargetEvidenceRequirementId::LeafVersionActivation,
        "leaf_version_activation",
    ),
    (
        TargetEvidenceRequirementId::OpcodeSemantics,
        "opcode_semantics",
    ),
    (
        TargetEvidenceRequirementId::EncodingSemantics,
        "encoding_semantics",
    ),
    (
        TargetEvidenceRequirementId::PushEncodingSemantics,
        "push_encoding_semantics",
    ),
    (
        TargetEvidenceRequirementId::InputIntrospectionSemantics,
        "input_introspection_semantics",
    ),
    (
        TargetEvidenceRequirementId::OutputIntrospectionSemantics,
        "output_introspection_semantics",
    ),
    (
        TargetEvidenceRequirementId::TransactionIntrospectionSemantics,
        "transaction_introspection_semantics",
    ),
    (
        TargetEvidenceRequirementId::ArithmeticSemantics,
        "arithmetic_semantics",
    ),
    (
        TargetEvidenceRequirementId::ComparisonSemantics,
        "comparison_semantics",
    ),
    (
        TargetEvidenceRequirementId::ConversionSemantics,
        "conversion_semantics",
    ),
    (
        TargetEvidenceRequirementId::StreamingHashSemantics,
        "streaming_hash_semantics",
    ),
    (
        TargetEvidenceRequirementId::SignatureSemantics,
        "signature_semantics",
    ),
    (
        TargetEvidenceRequirementId::SighashSemantics,
        "sighash_semantics",
    ),
    (
        TargetEvidenceRequirementId::RelativeTimelockSemantics,
        "relative_timelock_semantics",
    ),
    (
        TargetEvidenceRequirementId::EllipticCurveSemantics,
        "elliptic_curve_semantics",
    ),
    (
        TargetEvidenceRequirementId::ConfidentialValueConservation,
        "confidential_value_conservation",
    ),
    (
        TargetEvidenceRequirementId::CommitmentEquality,
        "commitment_equality",
    ),
    (
        TargetEvidenceRequirementId::IssuanceIntrospection,
        "issuance_introspection",
    ),
    (
        TargetEvidenceRequirementId::StackRearrangementSemantics,
        "stack_rearrangement_semantics",
    ),
    (
        TargetEvidenceRequirementId::ByteStringSemantics,
        "byte_string_semantics",
    ),
    (
        TargetEvidenceRequirementId::VerificationSemantics,
        "verification_semantics",
    ),
    (
        TargetEvidenceRequirementId::ConsensusResourceLimits,
        "consensus_resource_limits",
    ),
    (
        TargetEvidenceRequirementId::PolicyResourceLimits,
        "policy_resource_limits",
    ),
];

/// The spelling one reviewed primitive travels under.
///
/// `None` cannot arise for a member of the reviewed census: the census
/// test proves the table states a spelling for every one of them. The
/// option exists because the identity type is non-exhaustive, so a
/// primitive admitted upstream after this table was written would
/// otherwise be answered with a guess.
#[must_use]
pub fn opcode_name(id: OpcodeId) -> Option<&'static str> {
    lookup_name(OPCODE_NAMES, id)
}

/// The reviewed primitive one spelling names.
#[must_use]
pub fn opcode_from_name(name: &str) -> Option<OpcodeId> {
    lookup_value(OPCODE_NAMES, name)
}

/// The spelling one target capability travels under.
#[must_use]
pub fn capability_name(capability: ElementsCapability) -> Option<&'static str> {
    lookup_name(CAPABILITY_NAMES, capability)
}

/// The target capability one spelling names.
#[must_use]
pub fn capability_from_name(name: &str) -> Option<ElementsCapability> {
    lookup_value(CAPABILITY_NAMES, name)
}

/// The spelling one evidence requirement travels under.
#[must_use]
pub fn evidence_requirement_name(id: TargetEvidenceRequirementId) -> Option<&'static str> {
    lookup_name(EVIDENCE_NAMES, id)
}

/// The evidence requirement one spelling names.
#[must_use]
pub fn evidence_requirement_from_name(name: &str) -> Option<TargetEvidenceRequirementId> {
    lookup_value(EVIDENCE_NAMES, name)
}

/// The spelling one reviewed opening blocker travels under.
#[must_use]
pub fn opening_blocker_name(blocker: OpeningBlocker) -> Option<&'static str> {
    lookup_name(OPENING_BLOCKER_NAMES, blocker)
}

/// The reviewed opening blocker one spelling names.
#[must_use]
pub fn opening_blocker_from_name(name: &str) -> Option<OpeningBlocker> {
    lookup_value(OPENING_BLOCKER_NAMES, name)
}

/// The spelling a table states for one identity.
fn lookup_name<T: Copy + PartialEq>(
    table: &[(T, &'static str)],
    wanted: T,
) -> Option<&'static str> {
    table
        .iter()
        .find(|(value, _)| *value == wanted)
        .map(|(_, name)| *name)
}

/// The identity a table states for one spelling.
fn lookup_value<T: Copy>(table: &[(T, &'static str)], wanted: &str) -> Option<T> {
    table
        .iter()
        .find(|(_, name)| *name == wanted)
        .map(|(value, _)| *value)
}
