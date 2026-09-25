//! §14.8 names the resource fields of the two archived maturity submissions, and §18.3 compares predictions from their exact linked bundles and bytes with each submission's admitted target response.
//!
//! The operation boundary supplies no resource figure beyond the node's transaction weight here. A candidate the node did not judge supplies no target observation.

use std::collections::BTreeMap;

use linker::CandidateLinkedMaturityBundle;
use realization::{STATE_METADATA_BYTES, STATE_METADATA_VARIABLE_BYTES};
use tapscript::{
    AbstractLimits, StateLeafRole, StateProgramWitness, StateWitnessSchedule, TapscriptInstruction,
    program_stack_profile,
};
use target_elements::{OpcodeId, ResourceDimension, ReviewedElementsTapscriptDefinition};
use target_elements_conformance::executor::{OperationStep, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
};
use transaction::bytes::TargetTransaction;
use transaction::error::TransactionRefusal;

use crate::maturity_closure::{MaturityClosureRefusal, MaturityWitnessSelection, closure_target};
use crate::maturity_corpus::{
    MaturityCorpusImportRefusal, ValidatedMaturityCorpus, maturity_run_of_record,
    maturity_variable_run_of_record,
};
use crate::maturity_native::{MaturityAnnouncementPlanner, MaturityNativePlanRefusal};

/// The twenty-one resource-report fields of §14.8, in its published order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MaturityResourceField {
    /// Canonical predecessor metadata encoding width.
    MetadataEncodingBytes,
    /// The non-executing metadata leaf's script width.
    MetadataLeafBytes,
    /// Encoded programs in the committed static subtree.
    StaticSubtreeBytes,
    /// The executing announcement leaf's script width.
    AnnouncementLeafBytes,
    /// Constructor byte width, for which no emitter totals a figure.
    ConstructorBytes,
    /// The executing leaf's control-path depth.
    TreeDepth,
    /// Its exact control-block width.
    ControlBytes,
    /// Each witnessed role, leaf and control block separately.
    WitnessBytesByRole,
    /// The number of arguments before the leaf executes.
    InitialWitnessItemCount,
    /// Greatest main-stack depth in the abstract walk.
    PeakMainStack,
    /// Greatest alternate-stack depth in the abstract walk.
    PeakAlternateStack,
    /// An execution element width for which no emitter totals a maximum.
    LargestElement,
    /// Scheduled streaming-hash primitives.
    HashOperations,
    /// Scheduled curve and tweak verification primitives.
    CurveTweakOperations,
    /// Greatest validation-budget charge of the linked leaf.
    ValidationBudget,
    /// Exact weight of the submitted transaction.
    TransactionWeight,
    /// Exact virtual size of that transaction.
    VirtualSize,
    /// The target's observed consensus layer.
    TargetConsensusVerdict,
    /// The target's observed relay layer.
    RelayPolicyVerdict,
    /// Noncanonical nonce-search attempts.
    NonceSearchAttempts,
    /// Noncanonical construction and execution duration.
    ConstructionAndExecutionTime,
}

impl MaturityResourceField {
    /// The closed §14.8 field roster.
    pub const ALL: &'static [Self; 21] = &[
        Self::MetadataEncodingBytes,
        Self::MetadataLeafBytes,
        Self::StaticSubtreeBytes,
        Self::AnnouncementLeafBytes,
        Self::ConstructorBytes,
        Self::TreeDepth,
        Self::ControlBytes,
        Self::WitnessBytesByRole,
        Self::InitialWitnessItemCount,
        Self::PeakMainStack,
        Self::PeakAlternateStack,
        Self::LargestElement,
        Self::HashOperations,
        Self::CurveTweakOperations,
        Self::ValidationBudget,
        Self::TransactionWeight,
        Self::VirtualSize,
        Self::TargetConsensusVerdict,
        Self::RelayPolicyVerdict,
        Self::NonceSearchAttempts,
        Self::ConstructionAndExecutionTime,
    ];

    /// The field's stable report spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MetadataEncodingBytes => "metadata-encoding-bytes",
            Self::MetadataLeafBytes => "metadata-leaf-bytes",
            Self::StaticSubtreeBytes => "static-subtree-bytes",
            Self::AnnouncementLeafBytes => "announcement-leaf-bytes",
            Self::ConstructorBytes => "constructor-bytes",
            Self::TreeDepth => "tree-depth",
            Self::ControlBytes => "control-bytes",
            Self::WitnessBytesByRole => "witness-bytes-by-role",
            Self::InitialWitnessItemCount => "initial-witness-item-count",
            Self::PeakMainStack => "peak-main-stack",
            Self::PeakAlternateStack => "peak-alternate-stack",
            Self::LargestElement => "largest-element",
            Self::HashOperations => "hash-operations",
            Self::CurveTweakOperations => "curve-tweak-operations",
            Self::ValidationBudget => "validation-budget",
            Self::TransactionWeight => "transaction-weight",
            Self::VirtualSize => "virtual-size",
            Self::TargetConsensusVerdict => "target-consensus-verdict",
            Self::RelayPolicyVerdict => "relay-policy-verdict",
            Self::NonceSearchAttempts => "nonce-search-attempts",
            Self::ConstructionAndExecutionTime => "construction-and-execution-time",
        }
    }

    /// The matching target roster dimension, where §14.8 names one.
    #[must_use]
    pub const fn dimension(self) -> Option<ResourceDimension> {
        match self {
            Self::AnnouncementLeafBytes => Some(ResourceDimension::ScriptBytes),
            Self::TreeDepth => Some(ResourceDimension::ControlPathDepth),
            Self::WitnessBytesByRole => Some(ResourceDimension::WitnessBytes),
            Self::InitialWitnessItemCount => Some(ResourceDimension::InitialStackItems),
            Self::PeakMainStack => Some(ResourceDimension::PeakStackItems),
            Self::LargestElement => Some(ResourceDimension::StackElementBytes),
            Self::ValidationBudget => Some(ResourceDimension::ValidationBudget),
            Self::TransactionWeight => Some(ResourceDimension::TransactionWeight),
            _ => None,
        }
    }

    /// Whether §14.8 places this field outside canonical report bytes.
    #[must_use]
    pub const fn is_noncanonical_diagnostic(self) -> bool {
        matches!(
            self,
            Self::NonceSearchAttempts | Self::ConstructionAndExecutionTime
        )
    }
}

/// The two admitted submissions, in archive order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MaturitySubmission {
    /// The whole-metadata relay-refused historical submission.
    HistoricalWholeMetadata,
    /// The accepted variable-metadata submission.
    AcceptedVariableMetadata,
}

/// A position in the exact submitted script-path witness.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MaturityWitnessPosition {
    /// Successor output-key prefix.
    SuccessorOutputKeyPrefix,
    /// Successor representation nonce.
    SuccessorNonce,
    /// Requested cycle.
    RequestedCycle,
    /// Static subtree root.
    StaticSubtreeRoot,
    /// Whole or variable predecessor metadata, according to the schedule.
    PredecessorMetadata,
    /// Predecessor output-key prefix.
    PredecessorOutputKeyPrefix,
    /// Operator signature.
    OperatorSignature,
    /// Executed leaf program.
    AnnouncementLeaf,
    /// Script-path control block.
    ControlBlock,
}

/// Why a host prediction has no numeric figure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityPredictionAbsence {
    /// No emitter totals this quantity.
    NoEmitterTotalsIt,
    /// A single transaction is not a relay package.
    NoPackageInOneTransaction,
    /// This field names a target verdict rather than a host prediction.
    TheFieldIsATargetVerdict,
}

/// One host prediction, preserving absence and the reviewed zero separately.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturityPrediction {
    /// A measured scalar.
    Figure(u64),
    /// Exact widths of the ordered script-path witness items.
    ByPosition(Vec<(MaturityWitnessPosition, u64)>),
    /// The measured zero charged by the reviewed operation-cost contract.
    ZeroByTheReviewedContract,
    /// A value deliberately carried outside canonical resource bytes.
    NoncanonicalDiagnostic,
    /// No measured figure, with its reason.
    Absent(MaturityPredictionAbsence),
}

/// Why this operation boundary cannot provide an observation of a field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityUnobservedReason {
    /// The node exposes no interpreter accounting (`obs:upstream:eg-011`).
    TheNodeExposesNoInterpreterAccounting,
    /// This figure is a host or deployment property, not a target observation.
    TheFigureIsNotATargetObservation,
    /// The field belongs to noncanonical diagnostics.
    TheFieldIsANoncanonicalDiagnostic,
}

/// One field's comparison with the admitted submission response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityObservationStanding {
    /// Prediction and observation agree over the same bytes.
    Agree {
        /// Their common figure.
        figure: u64,
    },
    /// Both figures exist and differ.
    Mismatch {
        /// The host's figure.
        predicted: u64,
        /// The node's figure.
        observed: u64,
    },
    /// The response's observed verdict layer.
    Layer(ObservedOutcomeLayer),
    /// This boundary cannot expose this field's figure.
    NotObservableAtThisBoundary(MaturityUnobservedReason),
    /// This run supplied no weight for submitted bytes.
    NoObservationInThisRun,
}

/// A disagreement carrying both figures and the submitted subject.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityResourceFailure {
    /// The node and host weighed the same bytes differently.
    PredictionDisagreesWithObservation {
        /// Which archive supplied the submission.
        submission: MaturitySubmission,
        /// Which field disagreed.
        field: MaturityResourceField,
        /// The host's figure.
        predicted: u64,
        /// The node's figure.
        observed: u64,
    },
}

/// The complete field and roster predictions beside one admitted submission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturitySubmissionResources {
    /// The archive whose exact candidate bytes were measured.
    pub submission: MaturitySubmission,
    /// One prediction for every §14.8 field.
    pub fields: BTreeMap<MaturityResourceField, MaturityPrediction>,
    /// One prediction for every target resource dimension.
    pub roster: BTreeMap<ResourceDimension, MaturityPrediction>,
    /// One target standing for every §14.8 field.
    pub standings: BTreeMap<MaturityResourceField, MaturityObservationStanding>,
}

impl MaturitySubmissionResources {
    /// Number of field comparisons performed and found equal.
    #[must_use]
    pub fn agreements(&self) -> usize {
        self.standings
            .values()
            .filter(|standing| matches!(standing, MaturityObservationStanding::Agree { .. }))
            .count()
    }

    /// Every mismatch as a typed failure carrying both figures.
    #[must_use]
    pub fn failures(&self) -> Vec<MaturityResourceFailure> {
        self.standings
            .iter()
            .filter_map(|(&field, standing)| match standing {
                MaturityObservationStanding::Mismatch {
                    predicted,
                    observed,
                } => Some(
                    MaturityResourceFailure::PredictionDisagreesWithObservation {
                        submission: self.submission,
                        field,
                        predicted: *predicted,
                        observed: *observed,
                    },
                ),
                _ => None,
            })
            .collect()
    }
}

/// Why an archived prediction cannot be reconstructed from admitted evidence.
#[derive(Debug)]
pub enum MaturityResourcesRefusal {
    /// An archive failed its pinned admission.
    CorpusImport(MaturityCorpusImportRefusal),
    /// The replayed planner refused a request or response.
    Planner(MaturityNativePlanRefusal),
    /// The planner returned a refusal marker without retaining its reason.
    PlannerMarkerWithoutRefusal,
    /// No recorded exchange submitted transaction bytes.
    MissingSubmissionExchange(MaturitySubmission),
    /// Funding did not produce the planner's candidate serialization.
    MissingSubmissionBytes(MaturitySubmission),
    /// The linked bundle retained no metadata constructor.
    MissingRetainedConstructor(MaturitySubmission),
    /// The linked bundle supplied no announcement leaf.
    MissingAnnouncementLeaf(MaturitySubmission),
    /// The committed tree supplied no announcement depth.
    MissingAnnouncementDepth(MaturitySubmission),
    /// The transaction carried no sole input witness of nine items.
    UnexpectedWitnessShape(MaturitySubmission),
    /// The exact submitted bytes could not be decoded.
    TransactionDecode(TransactionRefusal),
    /// The reviewed target definition was unavailable.
    ReviewedTarget(MaturityClosureRefusal),
    /// A measured byte total could not fit the report's figure type.
    FigureOverflow,
}

/// Compare one predicted field with one admitted submission response.
///
/// A weight agreement is the only numeric target comparison this boundary
/// supports. Verdict layers are retained as observations without treating
/// them as predicted figures; every other field carries a typed reason.
#[must_use]
pub fn compare_submission(
    submission: MaturitySubmission,
    prediction: &MaturityPrediction,
    field: MaturityResourceField,
    response: &NativeOperationResponse,
) -> MaturityObservationStanding {
    use MaturityObservationStanding as Standing;
    use MaturityResourceField as Field;
    use MaturityUnobservedReason as Reason;
    let _ = submission;

    match field {
        Field::TransactionWeight => match prediction {
            MaturityPrediction::Figure(predicted) => response.resources.transaction_weight.map_or(
                Standing::NoObservationInThisRun,
                |observed| {
                    if *predicted == observed {
                        Standing::Agree { figure: observed }
                    } else {
                        Standing::Mismatch {
                            predicted: *predicted,
                            observed,
                        }
                    }
                },
            ),
            _ => Standing::NoObservationInThisRun,
        },
        Field::TargetConsensusVerdict | Field::RelayPolicyVerdict => {
            Standing::Layer(response.observed_layer)
        }
        Field::PeakMainStack
        | Field::PeakAlternateStack
        | Field::LargestElement
        | Field::ValidationBudget => {
            Standing::NotObservableAtThisBoundary(Reason::TheNodeExposesNoInterpreterAccounting)
        }
        Field::NonceSearchAttempts | Field::ConstructionAndExecutionTime => {
            Standing::NotObservableAtThisBoundary(Reason::TheFieldIsANoncanonicalDiagnostic)
        }
        _ => Standing::NotObservableAtThisBoundary(Reason::TheFigureIsNotATargetObservation),
    }
}

/// Read both admitted archives, replay their planners and compare exact bytes.
///
/// # Errors
/// Returns an archive, replay or byte-decoding refusal, or the typed absence
/// of a required submitted artifact; no missing figure is replaced by zero.
pub fn archived_submission_resources()
-> Result<[MaturitySubmissionResources; 2], MaturityResourcesRefusal> {
    let whole = maturity_run_of_record().map_err(MaturityResourcesRefusal::CorpusImport)?;
    let variable =
        maturity_variable_run_of_record().map_err(MaturityResourcesRefusal::CorpusImport)?;
    Ok([
        measure_archive(MaturitySubmission::HistoricalWholeMetadata, whole)?,
        measure_archive(MaturitySubmission::AcceptedVariableMetadata, variable)?,
    ])
}

fn replay_next(
    planner: &mut MaturityAnnouncementPlanner,
    previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
) -> Result<Option<OperationStep>, MaturityResourcesRefusal> {
    planner.next_step(previous).map_err(|_| {
        planner.refusal().map_or(
            MaturityResourcesRefusal::PlannerMarkerWithoutRefusal,
            |refusal| MaturityResourcesRefusal::Planner(refusal.clone()),
        )
    })
}

fn measure_archive(
    submission: MaturitySubmission,
    corpus: &ValidatedMaturityCorpus,
) -> Result<MaturitySubmissionResources, MaturityResourcesRefusal> {
    let evidence = corpus.evidence();
    let mut planner = MaturityAnnouncementPlanner::new(
        evidence.identity().clone(),
        evidence.branch(),
        MaturityWitnessSelection::Retained(evidence.schedule()),
    )
    .map_err(MaturityResourcesRefusal::Planner)?;
    let mut next = replay_next(&mut planner, None)?;
    for (position, (step, response)) in corpus.exchanges().iter().enumerate() {
        if next.as_ref() != Some(step) {
            return Err(MaturityResourcesRefusal::Planner(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position },
            ));
        }
        next = replay_next(&mut planner, Some((step.case(), response)))?;
    }
    if next.is_some() {
        return Err(MaturityResourcesRefusal::Planner(
            MaturityNativePlanRefusal::IncompleteTranscript,
        ));
    }
    planner
        .completed_transcript()
        .map_err(MaturityResourcesRefusal::Planner)?;
    let (_, response) = corpus
        .exchanges()
        .iter()
        .find(|(step, _)| matches!(step.subject(), OperationSubject::Submission(_)))
        .ok_or(MaturityResourcesRefusal::MissingSubmissionExchange(
            submission,
        ))?;
    let bytes = planner
        .submission_bytes()
        .ok_or(MaturityResourcesRefusal::MissingSubmissionBytes(submission))?;
    let transaction =
        TargetTransaction::decode(bytes).map_err(MaturityResourcesRefusal::TransactionDecode)?;
    let target = closure_target().map_err(MaturityResourcesRefusal::ReviewedTarget)?;
    predict_submission(
        submission,
        planner.bundle(),
        &transaction,
        &target,
        response,
    )
}

const fn witness_position(role: StateProgramWitness) -> MaturityWitnessPosition {
    use MaturityWitnessPosition as Position;
    match role {
        StateProgramWitness::SuccessorOutputKeyPrefix => Position::SuccessorOutputKeyPrefix,
        StateProgramWitness::SuccessorNonce => Position::SuccessorNonce,
        StateProgramWitness::RequestedCycle => Position::RequestedCycle,
        StateProgramWitness::StaticSubtreeRoot => Position::StaticSubtreeRoot,
        StateProgramWitness::PredecessorMetadata => Position::PredecessorMetadata,
        StateProgramWitness::PredecessorOutputKeyPrefix => Position::PredecessorOutputKeyPrefix,
        StateProgramWitness::OperatorSignature => Position::OperatorSignature,
    }
}

fn measured_total(
    bundle: &CandidateLinkedMaturityBundle,
    dimension: ResourceDimension,
) -> MaturityPrediction {
    bundle.resources().totals().total(dimension).map_or(
        MaturityPrediction::Absent(MaturityPredictionAbsence::NoEmitterTotalsIt),
        MaturityPrediction::Figure,
    )
}

struct MeasuredSubmission<'a> {
    bundle: &'a CandidateLinkedMaturityBundle,
    transaction: &'a TargetTransaction,
    metadata_bytes: u64,
    metadata_leaf_bytes: u64,
    static_bytes: u64,
    depth: u64,
    control_bytes: u64,
    widths: Vec<(MaturityWitnessPosition, u64)>,
    peak_main: u64,
    peak_alternate: u64,
    hash_ops: u64,
    curve_ops: u64,
}

impl<'a> MeasuredSubmission<'a> {
    fn derive(
        submission: MaturitySubmission,
        bundle: &'a CandidateLinkedMaturityBundle,
        transaction: &'a TargetTransaction,
        target: &ReviewedElementsTapscriptDefinition,
    ) -> Result<Self, MaturityResourcesRefusal> {
        let instance = bundle.instances().first().ok_or(
            MaturityResourcesRefusal::MissingRetainedConstructor(submission),
        )?;
        let leaf = bundle.program(StateLeafRole::Announcement).ok_or(
            MaturityResourcesRefusal::MissingAnnouncementLeaf(submission),
        )?;
        let depth = bundle
            .taptree()
            .depths()
            .get(&StateLeafRole::Announcement)
            .copied()
            .ok_or(MaturityResourcesRefusal::MissingAnnouncementDepth(
                submission,
            ))?;
        let (widths, control_bytes) = witness_widths(submission, bundle, transaction)?;
        let profile = program_stack_profile(
            target,
            leaf.program(),
            bundle.record().precondition(),
            AbstractLimits::for_target(target),
        );
        let static_bytes =
            bundle
                .taptree()
                .subtree()
                .leaves()
                .iter()
                .try_fold(0_u64, |total, entry| {
                    total
                        .checked_add(entry.leaf.program.encoded_length(target))
                        .ok_or(MaturityResourcesRefusal::FigureOverflow)
                })?;
        let mut hash_ops = 0_u64;
        let mut curve_ops = 0_u64;
        for instruction in leaf.program().instructions() {
            let TapscriptInstruction::Opcode(opcode) = instruction else {
                continue;
            };
            if matches!(
                *opcode,
                OpcodeId::Sha256Initialize | OpcodeId::Sha256Update | OpcodeId::Sha256Finalize
            ) {
                hash_ops += 1;
            }
            if matches!(*opcode, OpcodeId::EcMulScalarVerify | OpcodeId::TweakVerify) {
                curve_ops += 1;
            }
        }
        let metadata_bytes = match bundle.record().schedule() {
            StateWitnessSchedule::WholeMetadata => STATE_METADATA_BYTES,
            StateWitnessSchedule::VariableMetadata => STATE_METADATA_VARIABLE_BYTES,
        } as u64;
        Ok(Self {
            bundle,
            transaction,
            metadata_bytes,
            metadata_leaf_bytes: instance
                .constructor()
                .pattern()
                .program
                .encoded_length(target),
            static_bytes,
            depth: u64::from(depth),
            control_bytes,
            widths,
            peak_main: profile.peak_main(),
            peak_alternate: profile.peak_alternate(),
            hash_ops,
            curve_ops,
        })
    }

    fn fields(&self) -> BTreeMap<MaturityResourceField, MaturityPrediction> {
        use MaturityPrediction as Prediction;
        use MaturityPredictionAbsence as Absence;
        use MaturityResourceField as Field;
        use ResourceDimension as Dimension;
        BTreeMap::from([
            (
                Field::MetadataEncodingBytes,
                Prediction::Figure(self.metadata_bytes),
            ),
            (
                Field::MetadataLeafBytes,
                Prediction::Figure(self.metadata_leaf_bytes),
            ),
            (
                Field::StaticSubtreeBytes,
                Prediction::Figure(self.static_bytes),
            ),
            (
                Field::AnnouncementLeafBytes,
                measured_total(self.bundle, Dimension::ScriptBytes),
            ),
            (
                Field::ConstructorBytes,
                Prediction::Absent(Absence::NoEmitterTotalsIt),
            ),
            (Field::TreeDepth, Prediction::Figure(self.depth)),
            (Field::ControlBytes, Prediction::Figure(self.control_bytes)),
            (
                Field::WitnessBytesByRole,
                Prediction::ByPosition(self.widths.clone()),
            ),
            (
                Field::InitialWitnessItemCount,
                measured_total(self.bundle, Dimension::InitialStackItems),
            ),
            (Field::PeakMainStack, Prediction::Figure(self.peak_main)),
            (
                Field::PeakAlternateStack,
                Prediction::Figure(self.peak_alternate),
            ),
            (
                Field::LargestElement,
                Prediction::Absent(Absence::NoEmitterTotalsIt),
            ),
            (Field::HashOperations, Prediction::Figure(self.hash_ops)),
            (
                Field::CurveTweakOperations,
                Prediction::Figure(self.curve_ops),
            ),
            (
                Field::ValidationBudget,
                measured_total(self.bundle, Dimension::ValidationBudget),
            ),
            (
                Field::TransactionWeight,
                Prediction::Figure(self.transaction.weight()),
            ),
            (
                Field::VirtualSize,
                Prediction::Figure(self.transaction.virtual_size()),
            ),
            (
                Field::TargetConsensusVerdict,
                Prediction::Absent(Absence::TheFieldIsATargetVerdict),
            ),
            (
                Field::RelayPolicyVerdict,
                Prediction::Absent(Absence::TheFieldIsATargetVerdict),
            ),
            (
                Field::NonceSearchAttempts,
                Prediction::NoncanonicalDiagnostic,
            ),
            (
                Field::ConstructionAndExecutionTime,
                Prediction::NoncanonicalDiagnostic,
            ),
        ])
    }

    fn roster(&self) -> BTreeMap<ResourceDimension, MaturityPrediction> {
        use MaturityPrediction as Prediction;
        use MaturityPredictionAbsence as Absence;
        use ResourceDimension as Dimension;
        let mut roster = BTreeMap::new();
        for dimension in [
            Dimension::ScriptBytes,
            Dimension::OperationCost,
            Dimension::ValidationBudget,
            Dimension::InitialStackItems,
            Dimension::PeakStackItems,
            Dimension::InitialWitnessItemBytes,
        ] {
            let prediction = if dimension == Dimension::OperationCost
                && self.bundle.resources().totals().total(dimension) == Some(0)
            {
                Prediction::ZeroByTheReviewedContract
            } else {
                measured_total(self.bundle, dimension)
            };
            roster.insert(dimension, prediction);
        }
        roster.insert(
            Dimension::TransactionWeight,
            Prediction::Figure(self.transaction.weight()),
        );
        roster.insert(
            Dimension::WitnessBytes,
            Prediction::Figure(self.transaction.witness_bytes()),
        );
        roster.insert(Dimension::ControlPathDepth, Prediction::Figure(self.depth));
        roster.insert(
            Dimension::StackElementBytes,
            Prediction::Absent(Absence::NoEmitterTotalsIt),
        );
        roster.insert(
            Dimension::PackageLimit,
            Prediction::Absent(Absence::NoPackageInOneTransaction),
        );
        roster
    }
}

fn witness_widths(
    submission: MaturitySubmission,
    bundle: &CandidateLinkedMaturityBundle,
    transaction: &TargetTransaction,
) -> Result<(Vec<(MaturityWitnessPosition, u64)>, u64), MaturityResourcesRefusal> {
    let [witness] = transaction.witnesses() else {
        return Err(MaturityResourcesRefusal::UnexpectedWitnessShape(submission));
    };
    let items = witness.stack();
    if items.len() != 9 || bundle.record().witness().len() != 7 {
        return Err(MaturityResourcesRefusal::UnexpectedWitnessShape(submission));
    }
    let widths = bundle
        .record()
        .witness()
        .iter()
        .zip(items)
        .map(|((role, _), item)| (witness_position(*role), item.len() as u64))
        .chain([
            (
                MaturityWitnessPosition::AnnouncementLeaf,
                items[7].len() as u64,
            ),
            (MaturityWitnessPosition::ControlBlock, items[8].len() as u64),
        ])
        .collect();
    Ok((widths, items[8].len() as u64))
}

fn predict_submission(
    submission: MaturitySubmission,
    bundle: &CandidateLinkedMaturityBundle,
    transaction: &TargetTransaction,
    target: &ReviewedElementsTapscriptDefinition,
    response: &NativeOperationResponse,
) -> Result<MaturitySubmissionResources, MaturityResourcesRefusal> {
    let measured = MeasuredSubmission::derive(submission, bundle, transaction, target)?;
    let fields = measured.fields();
    let standings = MaturityResourceField::ALL
        .iter()
        .map(|&field| {
            let prediction = &fields[&field];
            (
                field,
                compare_submission(submission, prediction, field, response),
            )
        })
        .collect();
    Ok(MaturitySubmissionResources {
        submission,
        fields,
        roster: measured.roster(),
        standings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn submitted(corpus: &ValidatedMaturityCorpus) -> &[u8] {
        corpus
            .exchanges()
            .iter()
            .find_map(|(step, _)| match step.subject() {
                OperationSubject::Submission(subject) => Some(subject.transaction_bytes.as_slice()),
                _ => None,
            })
            .expect("the admitted archive has a submission exchange")
    }

    fn figure(resources: &MaturitySubmissionResources, field: MaturityResourceField) -> u64 {
        match resources
            .fields
            .get(&field)
            .expect("every field has a prediction")
        {
            MaturityPrediction::Figure(value) => *value,
            other => panic!("the field must carry a figure: {other:?}"),
        }
    }

    fn predicted_scalar(prediction: &MaturityPrediction) -> Option<u64> {
        match prediction {
            MaturityPrediction::Figure(value) => Some(*value),
            MaturityPrediction::ZeroByTheReviewedContract => Some(0),
            MaturityPrediction::ByPosition(_)
            | MaturityPrediction::NoncanonicalDiagnostic
            | MaturityPrediction::Absent(_) => None,
        }
    }

    fn observed_scalar(standing: &MaturityObservationStanding) -> Option<u64> {
        match standing {
            MaturityObservationStanding::Agree { figure } => Some(*figure),
            MaturityObservationStanding::Mismatch { observed, .. } => Some(*observed),
            MaturityObservationStanding::Layer(_)
            | MaturityObservationStanding::NotObservableAtThisBoundary(_)
            | MaturityObservationStanding::NoObservationInThisRun => None,
        }
    }

    #[test]
    fn the_fields_are_section_fourteen_eight_in_order() {
        use MaturityResourceField as Field;
        use ResourceDimension as Dimension;
        assert_eq!(Field::ALL.len(), 21);
        assert_eq!(
            Field::ALL
                .iter()
                .map(|field| field.name())
                .collect::<Vec<_>>(),
            [
                "metadata-encoding-bytes",
                "metadata-leaf-bytes",
                "static-subtree-bytes",
                "announcement-leaf-bytes",
                "constructor-bytes",
                "tree-depth",
                "control-bytes",
                "witness-bytes-by-role",
                "initial-witness-item-count",
                "peak-main-stack",
                "peak-alternate-stack",
                "largest-element",
                "hash-operations",
                "curve-tweak-operations",
                "validation-budget",
                "transaction-weight",
                "virtual-size",
                "target-consensus-verdict",
                "relay-policy-verdict",
                "nonce-search-attempts",
                "construction-and-execution-time",
            ]
        );
        assert_eq!(
            Field::ALL
                .iter()
                .filter(|field| field.is_noncanonical_diagnostic())
                .copied()
                .collect::<Vec<_>>(),
            [
                Field::NonceSearchAttempts,
                Field::ConstructionAndExecutionTime
            ]
        );
        assert_eq!(
            Field::ALL
                .iter()
                .filter_map(|field| field.dimension().map(|dimension| (*field, dimension)))
                .collect::<Vec<_>>(),
            [
                (Field::AnnouncementLeafBytes, Dimension::ScriptBytes),
                (Field::TreeDepth, Dimension::ControlPathDepth),
                (Field::WitnessBytesByRole, Dimension::WitnessBytes),
                (Field::InitialWitnessItemCount, Dimension::InitialStackItems),
                (Field::PeakMainStack, Dimension::PeakStackItems),
                (Field::LargestElement, Dimension::StackElementBytes),
                (Field::ValidationBudget, Dimension::ValidationBudget),
                (Field::TransactionWeight, Dimension::TransactionWeight),
            ]
        );
        for dimension in [
            Dimension::OperationCost,
            Dimension::InitialWitnessItemBytes,
            Dimension::PackageLimit,
        ] {
            assert_eq!(
                Field::ALL
                    .iter()
                    .filter(|field| field.dimension() == Some(dimension))
                    .count(),
                0
            );
        }
    }

    #[test]
    fn the_roster_stands_six_measured_three_elsewhere_one_unemitted_one_subjectless() {
        use MaturityPrediction as Prediction;
        use MaturityPredictionAbsence as Absence;
        use ResourceDimension as Dimension;
        let resources = archived_submission_resources().expect("both archives replay");
        for entry in &resources {
            assert_eq!(entry.roster.len(), 11);
            for dimension in [
                Dimension::ScriptBytes,
                Dimension::ValidationBudget,
                Dimension::InitialStackItems,
                Dimension::PeakStackItems,
                Dimension::InitialWitnessItemBytes,
            ] {
                assert!(matches!(
                    entry.roster.get(&dimension),
                    Some(Prediction::Figure(_))
                ));
            }
            assert_eq!(
                entry.roster.get(&Dimension::OperationCost),
                Some(&Prediction::ZeroByTheReviewedContract)
            );
            for dimension in [
                Dimension::TransactionWeight,
                Dimension::WitnessBytes,
                Dimension::ControlPathDepth,
            ] {
                assert!(matches!(
                    entry.roster.get(&dimension),
                    Some(Prediction::Figure(_))
                ));
            }
            assert_eq!(
                entry.roster.get(&Dimension::StackElementBytes),
                Some(&Prediction::Absent(Absence::NoEmitterTotalsIt))
            );
            assert_eq!(
                entry.roster.get(&Dimension::PackageLimit),
                Some(&Prediction::Absent(Absence::NoPackageInOneTransaction))
            );
            assert_eq!(
                entry.roster.get(&Dimension::ScriptBytes),
                entry
                    .fields
                    .get(&MaturityResourceField::AnnouncementLeafBytes)
            );
            assert_eq!(
                entry.roster.get(&Dimension::InitialStackItems),
                entry
                    .fields
                    .get(&MaturityResourceField::InitialWitnessItemCount)
            );
        }
    }

    #[test]
    fn the_archived_submissions_predict_their_own_bytes() {
        use MaturityResourceField as Field;
        use ResourceDimension as Dimension;
        let resources = archived_submission_resources().expect("both archives replay");
        let whole = TargetTransaction::decode(submitted(
            maturity_run_of_record().expect("whole archive admitted"),
        ))
        .expect("whole bytes decode");
        let variable = TargetTransaction::decode(submitted(
            maturity_variable_run_of_record().expect("variable archive admitted"),
        ))
        .expect("variable bytes decode");
        assert_eq!(whole.encode().len(), 1_694);
        assert_eq!(whole.encode_without_witness().len(), 130);
        assert_eq!(whole.weight(), 2_084);
        assert_eq!(variable.encode().len(), 1_701);
        assert_eq!(variable.encode_without_witness().len(), 130);
        assert_eq!(variable.weight(), 2_091);
        let whole_widths = whole.witnesses()[0]
            .stack()
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>();
        let variable_widths = variable.witnesses()[0]
            .stack()
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>();
        assert_eq!(whole_widths, [1, 4, 8, 32, 86, 1, 64, 1_286, 65]);
        assert_eq!(variable_widths, [1, 4, 8, 32, 53, 1, 64, 1_326, 65]);
        assert_eq!(figure(&resources[0], Field::MetadataEncodingBytes), 86);
        assert_eq!(figure(&resources[1], Field::MetadataEncodingBytes), 53);
        assert_eq!(figure(&resources[0], Field::MetadataLeafBytes), 90);
        assert_eq!(figure(&resources[1], Field::MetadataLeafBytes), 90);
        assert_eq!(figure(&resources[0], Field::AnnouncementLeafBytes), 1_286);
        assert_eq!(figure(&resources[1], Field::AnnouncementLeafBytes), 1_326);
        assert_eq!(
            figure(&resources[1], Field::AnnouncementLeafBytes)
                - figure(&resources[0], Field::AnnouncementLeafBytes),
            40
        );
        for entry in &resources {
            assert_eq!(figure(entry, Field::TreeDepth), 1);
            assert_eq!(figure(entry, Field::ControlBytes), 65);
            assert_eq!(figure(entry, Field::InitialWitnessItemCount), 7);
            assert_eq!(
                entry.fields.get(&Field::AnnouncementLeafBytes),
                entry.roster.get(&Dimension::ScriptBytes)
            );
            assert_eq!(
                entry.fields.get(&Field::StaticSubtreeBytes),
                entry.fields.get(&Field::AnnouncementLeafBytes)
            );
        }
        assert_eq!(
            resources[0].roster.get(&Dimension::InitialWitnessItemBytes),
            Some(&MaturityPrediction::Figure(86))
        );
        assert_eq!(
            resources[1].roster.get(&Dimension::InitialWitnessItemBytes),
            Some(&MaturityPrediction::Figure(64))
        );
        assert_eq!(
            resources[0].fields.get(&Field::WitnessBytesByRole),
            Some(&MaturityPrediction::ByPosition(vec![
                (MaturityWitnessPosition::SuccessorOutputKeyPrefix, 1),
                (MaturityWitnessPosition::SuccessorNonce, 4),
                (MaturityWitnessPosition::RequestedCycle, 8),
                (MaturityWitnessPosition::StaticSubtreeRoot, 32),
                (MaturityWitnessPosition::PredecessorMetadata, 86),
                (MaturityWitnessPosition::PredecessorOutputKeyPrefix, 1),
                (MaturityWitnessPosition::OperatorSignature, 64),
                (MaturityWitnessPosition::AnnouncementLeaf, 1_286),
                (MaturityWitnessPosition::ControlBlock, 65),
            ]))
        );
        assert_eq!(
            resources[0].roster.get(&Dimension::TransactionWeight),
            Some(&MaturityPrediction::Figure(2_084))
        );
        assert_eq!(
            resources[1].roster.get(&Dimension::TransactionWeight),
            Some(&MaturityPrediction::Figure(2_091))
        );
    }

    #[test]
    fn the_target_observed_one_dimension_and_it_agrees() {
        use MaturityObservationStanding as Standing;
        use MaturityResourceField as Field;
        use MaturityUnobservedReason as Reason;
        let resources = archived_submission_resources().expect("both archives replay");
        assert_eq!(
            resources[0].standings.get(&Field::TransactionWeight),
            Some(&Standing::Agree { figure: 2_084 })
        );
        assert_eq!(
            resources[1].standings.get(&Field::TransactionWeight),
            Some(&Standing::Agree { figure: 2_091 })
        );
        for entry in &resources {
            assert_eq!(entry.standings.len(), 21);
            for field in [
                Field::PeakMainStack,
                Field::PeakAlternateStack,
                Field::LargestElement,
                Field::ValidationBudget,
            ] {
                assert_eq!(
                    entry.standings.get(&field),
                    Some(&Standing::NotObservableAtThisBoundary(
                        Reason::TheNodeExposesNoInterpreterAccounting
                    ))
                );
            }
            for field in [
                Field::MetadataEncodingBytes,
                Field::MetadataLeafBytes,
                Field::StaticSubtreeBytes,
                Field::AnnouncementLeafBytes,
                Field::ConstructorBytes,
                Field::TreeDepth,
                Field::ControlBytes,
                Field::WitnessBytesByRole,
                Field::InitialWitnessItemCount,
                Field::HashOperations,
                Field::CurveTweakOperations,
                Field::VirtualSize,
            ] {
                assert_eq!(
                    entry.standings.get(&field),
                    Some(&Standing::NotObservableAtThisBoundary(
                        Reason::TheFigureIsNotATargetObservation
                    ))
                );
            }
            for field in [
                Field::NonceSearchAttempts,
                Field::ConstructionAndExecutionTime,
            ] {
                assert_eq!(
                    entry.standings.get(&field),
                    Some(&Standing::NotObservableAtThisBoundary(
                        Reason::TheFieldIsANoncanonicalDiagnostic
                    ))
                );
            }
            assert_eq!(entry.agreements(), 1);
            assert_eq!(entry.failures(), Vec::new());
        }
        for field in [Field::TargetConsensusVerdict, Field::RelayPolicyVerdict] {
            assert_eq!(
                resources[0].standings.get(&field),
                Some(&Standing::Layer(ObservedOutcomeLayer::RelayPolicyRejection))
            );
            assert_eq!(
                resources[1].standings.get(&field),
                Some(&Standing::Layer(ObservedOutcomeLayer::Accepted))
            );
        }
        assert_eq!(
            resources
                .iter()
                .map(MaturitySubmissionResources::agreements)
                .sum::<usize>(),
            2
        );
    }

    #[test]
    fn a_mismatch_records_both_figures_and_fails_by_type() {
        use MaturityObservationStanding as Standing;
        use MaturityResourceField as Field;
        let corpus = maturity_run_of_record().expect("whole archive admitted");
        let (_, archived) = corpus
            .exchanges()
            .iter()
            .find(|(step, _)| matches!(step.subject(), OperationSubject::Submission(_)))
            .expect("submission response");
        let mut response = archived.clone();
        response.resources.transaction_weight = Some(2_085);
        let mut measured =
            archived_submission_resources().expect("both archives replay")[0].clone();
        let prediction = measured
            .fields
            .get(&Field::TransactionWeight)
            .expect("weight prediction");
        let standing = compare_submission(
            measured.submission,
            prediction,
            Field::TransactionWeight,
            &response,
        );
        assert_eq!(
            standing,
            Standing::Mismatch {
                predicted: 2_084,
                observed: 2_085
            }
        );
        measured
            .standings
            .insert(Field::TransactionWeight, standing);
        assert_eq!(measured.agreements(), 0);
        assert_eq!(
            measured.failures(),
            [
                MaturityResourceFailure::PredictionDisagreesWithObservation {
                    submission: MaturitySubmission::HistoricalWholeMetadata,
                    field: Field::TransactionWeight,
                    predicted: 2_084,
                    observed: 2_085,
                }
            ]
        );
        response.resources.transaction_weight = None;
        let missing = compare_submission(
            measured.submission,
            prediction,
            Field::TransactionWeight,
            &response,
        );
        assert_eq!(missing, Standing::NoObservationInThisRun);
        assert_eq!(observed_scalar(&missing), None);
        measured.standings.insert(Field::TransactionWeight, missing);
        assert_eq!(measured.agreements(), 0);
        assert_eq!(measured.failures(), Vec::new());
    }

    #[test]
    fn no_absent_prediction_is_a_zero() {
        use MaturityObservationStanding as Standing;
        use MaturityPrediction as Prediction;
        let resources = archived_submission_resources().expect("both archives replay");
        for entry in &resources {
            let predictions = entry.fields.values().chain(entry.roster.values());
            assert_eq!(
                entry
                    .fields
                    .iter()
                    .filter_map(|(field, prediction)| match prediction {
                        Prediction::Figure(0) => Some(*field),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                [MaturityResourceField::PeakAlternateStack]
            );
            assert_eq!(
                entry
                    .roster
                    .values()
                    .filter(|prediction| matches!(prediction, Prediction::Figure(0)))
                    .count(),
                0
            );
            assert_eq!(
                predictions
                    .clone()
                    .filter(|prediction| {
                        matches!(prediction, Prediction::ZeroByTheReviewedContract)
                    })
                    .count(),
                1
            );
            for prediction in predictions {
                if matches!(
                    prediction,
                    Prediction::Absent(_) | Prediction::NoncanonicalDiagnostic
                ) {
                    assert_eq!(predicted_scalar(prediction), None);
                }
            }
            for standing in entry.standings.values() {
                if matches!(standing, Standing::NotObservableAtThisBoundary(_)) {
                    assert_eq!(observed_scalar(standing), None);
                }
            }
        }
    }
}
