//! The adapter from compiler-owned abstract target requirements to
//! Elements target obligations.
//!
//! # Boundary
//!
//! This crate is the join, and only the join. The compiler owns what an
//! approved analysis requires of *some* target, abstractly; the target
//! package owns what one reviewed Elements tapscript contract offers,
//! concretely; neither may name the other. The join has to live
//! somewhere, and it lives here because this is the one package whose
//! contract admits both.
//!
//! It owns no target program, no instruction, no stack schedule, no
//! transaction layout, and no bundle. It emits nothing.
//!
//! # Dependencies
//!
//! `compiler` and `target-elements`, and nothing else — not first-party
//! and not third-party. The crate serializes nothing, hashes nothing,
//! parses nothing, and opens no file.
//!
//! # State
//!
//! Implemented: the package boundary, the typed instruction core — a
//! typed instruction, a checked stack item, an exact serializer, and a
//! parser over the reviewed subset — the abstract stack validator, which
//! keeps every successful alternative and every non-aborting failure
//! state apart from the aborting causes — the static capability adapter
//! —
//! a multi-state assessment of each compiler capability against the
//! reviewed static target contract — and the external-evidence-role
//! adapter, with exact census equality in both directions against the
//! two censuses the analysis published.
//!
//! Not assessed here: anything a deployment declares. The static
//! contract, the development binding, and target-native evidence are
//! three different values, and nothing in this crate accepts one while
//! answering for another. A deployment-aware assessment is deferred
//! until a consumer for one exists.
//!
//! Not implemented: the stack scheduler, backend proof patterns,
//! constructors, and the relocatable bundle.
//!
//! Not claimed: anything about a real node. No target program has been
//! emitted, no transaction has been built, every evidence requirement
//! the target contract names remains unresolved, and no complete
//! backend proof pattern exists — the last of those is enforced by an
//! uninhabited pattern identity rather than by convention.
//!
//! # Public modules
//!
//! - [`instruction`] — [`TapscriptInstruction`], the typed instruction,
//!   and [`StackItem`], the checked literal it pushes. There is no raw
//!   opcode, no raw instruction, and no raw program in the safe API.
//! - [`program`] — [`TapscriptProgram`], the validated instruction
//!   sequence, with the exact serializer
//!   ([`TapscriptProgram::encode`]) and the reviewed-subset parser
//!   ([`TapscriptProgram::decode`]) that is this crate's one entry
//!   point for untrusted bytes.
//! - [`stack`] — the abstract stack validator: [`validate_program`],
//!   its [`AbstractStackState`] input, its [`AbstractLimits`] work
//!   budget, its three-set [`AbstractExecutionResult`], and
//!   [`resource_projection`].
//! - [`capability`] — the adapter proper:
//!   [`assess_static_capability`], [`assess_evidence_role`],
//!   [`assess_requirements`], and [`assess_complete_census`], returning
//!   the multi-state [`StaticCapabilityAssessment`] and
//!   [`ExternalEvidenceAssessment`] carried together in a
//!   [`TargetAssessmentSet`].
//! - [`error`] — [`TapscriptError`], the crate's single error root.
//!
//! Every entry point in the crate takes the reviewed static contract,
//! `target_elements::ReviewedElementsTapscriptDefinition`, obtained from
//! `target_elements::reviewed_elements_tapscript`. That type has no
//! public constructor, so a caller cannot substitute a contract of its
//! own at this boundary.
//!
//! # Quickstart: build a program, round-trip it, validate it
//!
//! ```
//! use tapscript::{
//!     AbstractLimits, AbstractStackState, StackItem, TapscriptInstruction, TapscriptProgram,
//!     validate_program,
//! };
//! use target_elements::{FailureCause, OpcodeId, StackValueType, reviewed_elements_tapscript};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // The reviewed static contract is the input of everything here.
//! let target = reviewed_elements_tapscript().expect("the reviewed contract validates");
//!
//! // Programs are built from reviewed primitive identities and checked
//! // literals. No raw byte enters by this path.
//! let program = TapscriptProgram::new(vec![
//!     TapscriptInstruction::Push(StackItem::script_number(&target, 7)?),
//!     TapscriptInstruction::Push(StackItem::script_number(&target, 7)?),
//!     TapscriptInstruction::Opcode(OpcodeId::Equal),
//! ])?;
//!
//! // The serializer resolves every opcode byte and push form from the
//! // contract, and the parser accepts exactly the reviewed subset back.
//! let bytes = program.encode(&target);
//! assert_eq!(TapscriptProgram::decode(&target, &bytes)?, program);
//!
//! // The validator answers three sets, never one Boolean.
//! let result = validate_program(
//!     &target,
//!     &program,
//!     &AbstractStackState::from_main(Vec::new()),
//!     AbstractLimits::for_target(&target),
//! )?;
//!
//! // One clean state, holding the Boolean the comparison pushed.
//! assert_eq!(result.success().len(), 1);
//! assert_eq!(
//!     result.success().iter().next().expect("one state").main(),
//!     &[StackValueType::Bool],
//! );
//!
//! // No failure pushed a false and carried on here.
//! assert!(result.nonaborting_failure().is_empty());
//!
//! // Abort causes are retained rather than dismissed: nothing in the
//! // abstract state rules out the target refusing the execution
//! // domain, so the cause stays in the answer.
//! assert!(
//!     result
//!         .aborts()
//!         .contains(&FailureCause::UnsupportedExecutionDomain),
//! );
//! assert!(!result.always_aborts());
//! # Ok(())
//! # }
//! ```
//!
//! # Quickstart: assess what this target obliges a backend to do
//!
//! ```
//! use compiler::target::RequiredCapability;
//! use tapscript::{AssessmentDisposition, assess_complete_census};
//! use target_elements::reviewed_elements_tapscript;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let target = reviewed_elements_tapscript().expect("the reviewed contract validates");
//! let assessed = assess_complete_census(&target)?;
//!
//! let owner = assessed
//!     .capability_assessment(RequiredCapability::OwnerAuthorization)
//!     .expect("the census covers every compiler capability");
//!
//! // A multi-state answer: the sighash construction was not reached by
//! // the review, so the prerequisites are not established.
//! assert_eq!(
//!     owner.disposition(),
//!     AssessmentDisposition::MissingTargetPrimitives,
//! );
//! # Ok(())
//! # }
//! ```
//!
//! # Errors
//!
//! Every fallible operation returns [`TapscriptError`], which is a
//! `std::error::Error`. Its variants fall into four families: literal
//! and encoding refusals from the [`StackItem`] constructors, parse
//! refusals from [`TapscriptProgram::decode`], abstract-validation and
//! work-budget refusals from [`validate_program`], and census
//! disagreements from the assessment entry points. Each entry point's
//! own `# Errors` section names the exact variants it can return.
//!
//! Worked examples, the full public-API tour, and the boundary
//! discussion are in the package README.

#![forbid(unsafe_code)]

pub mod capability;
pub mod error;
pub mod instruction;
pub mod program;
pub mod stack;

pub use capability::{
    AssessmentDisposition, AssessmentProjection, BackendFoundationRequirement, BackendPatternId,
    EvidenceAssessmentDisposition, EvidenceAssessmentProjection, ExternalEvidenceAssessment,
    StaticCapabilityAssessment, TargetAssessmentSet, UnsupportedReason, assess_complete_census,
    assess_evidence_role, assess_requirements, assess_static_capability,
};
pub use error::TapscriptError;
pub use instruction::{StackItem, TapscriptInstruction};
pub use program::{MAXIMUM_PROGRAM_INSTRUCTIONS, TapscriptProgram};
pub use stack::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, resource_projection,
    validate_program,
};

#[cfg(test)]
mod tests;
