//! The harness's typed error root.
//!
//! # Every variant is a branch that runs
//!
//! There is no variant here for a condition the harness cannot reach and
//! no catch-all that absorbs an unclassified failure into a plausible
//! neighbour. A failure the harness has not classified is a failure it
//! does not understand, and reporting it as one it does understand is
//! how a rejection gets recorded as infrastructure trouble, or worse,
//! the other way round.
//!
//! # No child detail
//!
//! No variant carries the executor's path, its argv, its raw stderr, or
//! any environment value
//! `(´[ADR010-rule:output:data-classification]´)`. What a failing external
//! program is reported as is a fixed message, the protocol phase, the
//! process status, and the safe typed case identity — never the bytes it
//! chose to write.

use target_elements::TargetEvidenceRequirementId;

use crate::fixture::NativeCaseId;
use crate::protocol::{ProtocolPhase, ResponseShapeDefect};
use crate::prototype::PrototypeCaseId;
use crate::vocabulary::evidence_requirement_name;

/// A failure of the target-native conformance harness.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NativeConformanceError {
    /// The executor answered the handshake with a protocol schema this
    /// harness does not speak.
    #[error("the executor offered protocol schema {offered}, which this harness does not speak")]
    UnsupportedProtocolSchema {
        /// The schema the executor offered.
        offered: u32,
    },

    /// The executor process could not be started.
    #[error("the external executor could not be started")]
    ExecutorStartupFailed,

    /// The executor started but does not lead a process group of its
    /// own, so the run could not be supervised as one tree.
    ///
    /// Refused rather than run unsupervised: the alternative is a run
    /// whose timeout could only reach the first process, and whose
    /// cleanup contract would therefore be a claim the harness cannot
    /// keep (Guide-10 §5.8).
    #[error("the external executor could not be supervised as one process group")]
    ExecutorProcessGroupUnavailable,

    /// The executor did not complete the handshake.
    #[error("the external executor did not complete the handshake")]
    ExecutorHandshakeFailed,

    /// The executor's handshake does not cover the execution domain or
    /// leaf version the fixtures are stated against.
    #[error("the external executor does not support the required domain or leaf version")]
    ExecutorProtocolMismatch,

    /// The executor did not answer within its explicit typed timeout.
    ///
    /// Never a rejection: an executor that ran out of time observed
    /// nothing about the target (Guide-9 §11.7).
    #[error("the external executor exceeded its explicit timeout")]
    ExecutorTimeout,

    /// The executor exited before the protocol completed.
    #[error("the external executor exited before the protocol completed")]
    ExecutorExited {
        /// The child's exit status, where the host reported one.
        status: Option<i32>,
    },

    /// A protocol message could not be read as the typed message the
    /// phase requires.
    #[error("the executor sent a malformed message during the {phase} phase")]
    MalformedResponse {
        /// The phase the harness was in.
        phase: ProtocolPhase,
    },

    /// A protocol record exceeded the explicit bound for its phase.
    ///
    /// Reached without reading past the bound: a record that never
    /// terminates is refused here rather than allocated until the host
    /// intervenes.
    #[error("the executor sent a {phase} record larger than the {maximum}-byte bound")]
    ProtocolRecordTooLarge {
        /// The phase the harness was in.
        phase: ProtocolPhase,
        /// The bound that applied.
        maximum: usize,
    },

    /// The executor sent a blank or whitespace-only protocol record.
    ///
    /// The framing defines one nonempty JSON object per record, so an
    /// empty one is a failure rather than filler to be skipped.
    #[error("the executor sent a blank record during the {phase} phase")]
    BlankProtocolRecord {
        /// The phase the harness was in.
        phase: ProtocolPhase,
    },

    /// A response contradicted the interface its own executor
    /// advertised.
    #[error("the executor's response for case {case} is malformed: {defect}")]
    MalformedResponseShape {
        /// The case answered.
        case: NativeCaseId,
        /// How the response contradicts the advertised interface.
        defect: ResponseShapeDefect,
    },

    /// The executor never stated the environment it ran on.
    #[error("the external executor stated no environment observation")]
    MissingEnvironmentObservation,

    /// The environment the executor observed is not the one the binding
    /// names.
    #[error("the executor observed a different environment from the one the binding names")]
    EnvironmentBindingMismatch,

    /// The genesis the executor observed is not the bound one.
    #[error("the executor observed a different genesis identity from the bound one")]
    GenesisObservationMismatch,

    /// The reviewed domain or leaf version is not active where the
    /// executor ran.
    #[error("the reviewed domain or leaf version is not active where the executor ran")]
    ActivationObservationMismatch,

    /// The executor wrote further protocol data after the last response.
    #[error("the external executor wrote protocol data after its last response")]
    TrailingProtocolData,

    /// The executor answered one case twice.
    #[error("the external executor answered case {0} twice")]
    DuplicateCaseResponse(NativeCaseId),

    /// The executor never answered a case it was asked about.
    #[error("the external executor did not answer case {0}")]
    MissingCaseResponse(NativeCaseId),

    /// The executor answered a case it was never asked about.
    #[error("the external executor answered case {0}, which it was not asked about")]
    UnexpectedCaseResponse(NativeCaseId),

    /// The executor answered a different case from the outstanding one.
    #[error("the external executor answered out of order; case {expected} was outstanding")]
    ResponseOrderViolation {
        /// The case whose response was outstanding.
        expected: NativeCaseId,
    },

    /// The executor was asked for compound-prototype cases and said it
    /// does not read them.
    ///
    /// Refused before any case runs, and kept apart from the primitive
    /// mismatch above: an executor that speaks the primitive exchange
    /// perfectly and reads no prototype record has not failed the
    /// protocol, it has declined a workload
    /// `(´[PLAN-rule:guide10:schema-migration]´)`.
    #[error("the external executor does not read compound-prototype fixtures")]
    PrototypeFixturesUnsupported,

    /// A prototype response contradicted its executor's advertised
    /// interface.
    #[error("the executor's response for prototype case {case} is malformed: {defect}")]
    MalformedPrototypeResponseShape {
        /// The case answered.
        case: PrototypeCaseId,
        /// How the response contradicts the advertised interface.
        defect: ResponseShapeDefect,
    },

    /// The executor answered one prototype case twice.
    #[error("the external executor answered prototype case {0} twice")]
    DuplicatePrototypeResponse(PrototypeCaseId),

    /// The executor never answered a prototype case it was asked about.
    #[error("the external executor did not answer prototype case {0}")]
    MissingPrototypeResponse(PrototypeCaseId),

    /// The executor answered a prototype case it was never asked about.
    #[error("the external executor answered prototype case {0}, which it was not asked about")]
    UnexpectedPrototypeResponse(PrototypeCaseId),

    /// The executor answered a different prototype case from the
    /// outstanding one.
    #[error(
        "the external executor answered out of order; prototype case {expected} was outstanding"
    )]
    PrototypeResponseOrderViolation {
        /// The case whose response was outstanding.
        expected: PrototypeCaseId,
    },

    /// The executor reported infrastructure trouble for one case.
    #[error("the external executor reported infrastructure trouble for case {0}")]
    InfrastructureFailure(NativeCaseId),

    /// Two fixtures declared the same typed case identity.
    #[error("two fixtures declare case {0}")]
    DuplicateFixtureCase(NativeCaseId),

    /// A canonical fixture could not be stated against the reviewed
    /// contract at all.
    ///
    /// The census is first-party source, so this is a defect in it: a
    /// literal wider than the target admits, a script number outside
    /// the admissible range, or a program longer than the work limit.
    /// It is an error rather than a panic because the census is built
    /// by a command, and a command that cannot state its own fixtures
    /// must fail rather than abort the process.
    #[error("a canonical fixture could not be stated against the reviewed contract")]
    FixtureNotExpressible,

    /// A fixture's transaction context is not internally well shaped.
    ///
    /// A malformed fixture does not acquire a report subject: an index
    /// naming no input, an empty stated field, or a path with no script
    /// would leave the executor to invent what the fixture failed to
    /// state.
    #[error("the context of fixture {0} is not internally well shaped")]
    MalformedFixtureContext(NativeCaseId),

    /// A fixture is stated against a different contract revision, or a
    /// different execution domain or leaf version, from the run's.
    #[error("a fixture is stated against a different target contract from the run's")]
    TargetContractMismatch,

    /// A fixture is stated against a different development binding from
    /// the run's.
    #[error("a fixture is stated against a different development binding from the run's")]
    DevelopmentBindingMismatch,

    /// The evidence plan does not partition the target's requirement
    /// census exactly.
    #[error("the evidence plan does not partition the target evidence census exactly")]
    EvidenceCensusMismatch,

    /// A required evidence requirement has no case evidence at all.
    #[error("required target evidence {} has no case evidence", requirement_text(*.0))]
    RequiredEvidenceMissing(TargetEvidenceRequirementId),

    /// A required evidence requirement has failing case evidence.
    #[error("required target evidence {} failed", requirement_text(*.0))]
    RequiredEvidenceFailed(TargetEvidenceRequirementId),

    /// A required evidence requirement could not be established because
    /// the executor reported infrastructure trouble.
    #[error("required target evidence {} hit executor infrastructure trouble", requirement_text(*.0))]
    RequiredEvidenceInfrastructureError(TargetEvidenceRequirementId),

    /// The report was written under a revision this harness does not
    /// validate.
    #[error("the report offers schema {offered}, which this harness does not validate")]
    UnsupportedReportSchema {
        /// The schema the report offers.
        offered: u32,
    },

    /// The report's case census is not the fixture census.
    #[error("the report's case census is not the fixture census")]
    ReportCaseCensusMismatch,

    /// The report holds one case twice.
    #[error("the report holds case {0} twice")]
    DuplicateReportCase(NativeCaseId),

    /// The report's row for one case does not state the fixture that was
    /// executed.
    #[error("the report's row for case {0} does not state the executed fixture")]
    FixtureProjectionMismatch(NativeCaseId),

    /// The report's row for one case does not state what the executor
    /// observed, or what that observation compares to.
    #[error("the report's row for case {0} does not state the observed outcome")]
    ReportCaseOutcomeMismatch(NativeCaseId),

    /// The report's evidence rows are not the recomputed ones.
    #[error("the report's evidence rows are not the ones the plan and the run produce")]
    ReportEvidenceCensusMismatch,

    /// The report's claim rows are not the recomputed ones.
    #[error("the report's claim rows are not the ones the registry and the run produce")]
    ReportClaimCensusMismatch,

    /// The report states one claim twice.
    #[error("the report states one claim twice")]
    DuplicateEvidenceClaim(crate::claim::NativeEvidenceClaim),

    /// The report omits a claim the registry holds.
    #[error("the report omits a claim the registry holds")]
    MissingEvidenceClaim(crate::claim::NativeEvidenceClaim),

    /// The report states a claim the registry does not hold.
    #[error("the report states a claim the registry does not hold")]
    UnexpectedEvidenceClaim(crate::claim::NativeEvidenceClaim),

    /// The report's summary is not the recomputed one.
    #[error("the report's summary is not the one its own rows add up to")]
    ReportSummaryMismatch,

    /// The report's provenance or observed environment is not the
    /// transcript's.
    #[error("the report's provenance or observed environment is not the run's")]
    ReportProvenanceMismatch,

    /// A required claim has no passing case bearing on it.
    #[error("a required evidence claim has no passing case bearing on it")]
    RequiredClaimMissing(crate::claim::NativeEvidenceClaim),

    /// A required claim has failing case evidence.
    #[error("a required evidence claim has failing case evidence")]
    RequiredClaimFailed(crate::claim::NativeEvidenceClaim),

    /// The run selected a mock executor, which can never satisfy the
    /// target-native gate.
    ///
    /// A mock is admitted for protocol and failure-path tests only. The
    /// gate needs an explicit nonmock selection, and it is recorded as
    /// ordinary provenance rather than as a proof of anything
    /// (Guide-9 §1.6, §11.8).
    #[error("a mock executor cannot satisfy the target-native gate")]
    MockExecutorCannotSatisfyNativeGate,

    /// A matrix row does not belong to the relation being run.
    ///
    /// One report answers one relation. A row of the other relation in
    /// the same run would put its coverage under the wrong role, where a
    /// reader would count it as coverage of a relation nothing
    /// established.
    #[error("a matrix row does not belong to the relation being run")]
    PrototypeMatrixRelationMismatch {
        /// The offending case.
        case: PrototypeCaseId,
    },

    /// Two matrix rows declare the same compound case identity.
    #[error("two matrix rows declare one compound case")]
    DuplicatePrototypeCase(PrototypeCaseId),

    /// A matrix row does not state a coherent case.
    ///
    /// An incoherent fixture has no report subject: it could be
    /// satisfied by a transaction other than the one it means, and an
    /// executor's agreement with it would establish nothing.
    #[error("a matrix row does not state a coherent case")]
    IncoherentPrototypeFixture {
        /// The offending case.
        case: PrototypeCaseId,
        /// How the fixture is incoherent.
        ///
        /// Boxed because the defect carries a whole tree defect, and an
        /// error root every other variant of which is a few words would
        /// otherwise be as large as its largest member everywhere it is
        /// returned.
        defect: Box<crate::prototype::PrototypeFixtureDefect>,
    },

    /// The prototype report's role is not the one its relation answers
    /// for.
    #[error("the prototype report's role is not the one its relation answers for")]
    PrototypeReportRoleMismatch,

    /// The prototype report's case census is not the executed matrix.
    #[error("the prototype report's case census is not the executed matrix")]
    PrototypeReportCaseCensusMismatch,

    /// The prototype report's row for one case does not state the
    /// fixture that was executed.
    #[error("the prototype report's row for case {0} does not state the executed fixture")]
    PrototypeProjectionMismatch(PrototypeCaseId),

    /// The prototype report's row for one case does not state what the
    /// executor observed, or what that observation compares to.
    #[error("the prototype report's row for case {0} does not state the observed outcome")]
    PrototypeCaseOutcomeMismatch(PrototypeCaseId),

    /// The prototype report states one claim twice.
    #[error("the prototype report states one compound claim twice")]
    DuplicatePrototypeClaim(crate::prototype::PrototypeClaim),

    /// The prototype report omits a claim its relation holds.
    #[error("the prototype report omits a claim its relation holds")]
    MissingPrototypeClaim(crate::prototype::PrototypeClaim),

    /// The prototype report states a claim its relation does not hold.
    #[error("the prototype report states a claim its relation does not hold")]
    UnexpectedPrototypeClaim(crate::prototype::PrototypeClaim),

    /// The prototype report's claim rows are not the recomputed ones.
    #[error("the prototype report's claim rows are not the ones the census and the run produce")]
    PrototypeReportClaimCensusMismatch,

    /// The prototype report's summary is not the recomputed one.
    #[error("the prototype report's summary is not the one its own rows add up to")]
    PrototypeReportSummaryMismatch,

    /// The run executed no compound case at all.
    ///
    /// A run of nothing establishes nothing. Without this the gate would
    /// accept an empty matrix, whose every required claim is vacuously
    /// absent only because there are no claims to fail either.
    #[error("the run executed no compound case at all")]
    EmptyPrototypeMatrix,

    /// A required compound claim has no passing case bearing on it.
    #[error("a required compound claim has no passing case bearing on it")]
    RequiredPrototypeClaimMissing(crate::prototype::PrototypeClaim),

    /// A required compound claim has failing case evidence.
    #[error("a required compound claim has failing case evidence")]
    RequiredPrototypeClaimFailed(crate::prototype::PrototypeClaim),

    /// One compound case's observation was not what its fixture
    /// requires.
    #[error("compound case {0} did not observe what its fixture requires")]
    PrototypeCaseFailed(PrototypeCaseId),

    /// One compound case could not be run by the executor.
    #[error("compound case {0} hit executor infrastructure trouble")]
    PrototypeCaseInfrastructureError(PrototypeCaseId),
}

/// The safe spelling of one evidence requirement for a message.
///
/// A requirement outside the spelling table cannot reach a message
/// today — the vocabulary census test proves the table is total — and
/// the fallback names the shape of the gap rather than inventing a
/// spelling for it.
fn requirement_text(id: TargetEvidenceRequirementId) -> &'static str {
    evidence_requirement_name(id).unwrap_or("an unspelled target evidence requirement")
}
