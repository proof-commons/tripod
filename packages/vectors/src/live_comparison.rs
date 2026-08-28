//! §18.4's prediction laid beside the target's observation.
//!
//! §18.4 asks for backend, linker, and ABI predictions to be compared
//! with target observations *over the same exact bytes*, and states what
//! a mismatch does: it records both figures, returns a typed planner
//! failure, stops further steps, fails the resource report, and cannot
//! coexist with a successful result. It also states the rule that governs
//! everything else here — no absent observation is read as zero or as
//! agreement.
//!
//! # One of the seventeen dimensions is observable, and sixteen are not
//!
//! That is the honest shape of this boundary, and saying so precisely is
//! most of what this module does.
//!
//! The node exposes no per-script accounting. Its adapter says as much in
//! its own words: for an operation step the peaks, the widest element and
//! the validation budget stay null, because inventing them would be the
//! adapter reporting its own arithmetic as the target's. So every
//! interpreter dimension of §18.3 has no observation to compare against —
//! and it is typed as *unobservable at this boundary* rather than left
//! out, because a dimension quietly missing from a comparison table reads
//! exactly like a dimension that agreed.
//!
//! The exception is the transaction's weight, and it is a real one. The
//! executor reads it back from the node's own `decoderawtransaction`, and
//! it does so for *refused* bytes too. That matters more here than
//! anywhere else in this guide: no candidate transfer can be accepted
//! (§1.7), so a comparison that needed an accepted transaction would have
//! nothing to compare — and this one does not. The candidate's real bytes
//! went to a real node, the node weighed them, and this workspace weighed
//! the same bytes. Those two figures are §18.4's comparison, performed.
//!
//! # The prediction is decoded, not remembered
//!
//! [`crate::live_native::PredictedTransferResources`] is taken by
//! decoding the submitted serialization and weighing the result, so both
//! sides of the comparison are functions of one byte string. A figure the
//! builder handed forward would have been a prediction about the value
//! that produced the bytes, which is a different claim and a weaker one.

use std::collections::BTreeMap;

use tapscript::upstream::LiveTransferRepresentationPlan;

use crate::live_evidence::LiveInfrastructureBlocker;
use crate::live_measurements::LiveResourceRecord;
use crate::live_native::{LiveNativeStep, LiveNativeTranscript};

/// Why one dimension has no observation to be compared against.
///
/// Four reasons, and they call for four different repairs. A reader
/// deciding what to build next needs to know whether the figure is
/// missing because the node does not expose it, because it is not a
/// property of a submission at all, because its subject cannot exist, or
/// because it is a layer rather than a number.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum UnobservedReason {
    /// The node publishes no per-script interpreter accounting.
    ///
    /// The adapter writes null for the peaks, the widest element and the
    /// validation budget on every operation step, on the stated ground
    /// that computing them itself would be reporting its own arithmetic
    /// as the target's.
    TheNodeExposesNoInterpreterAccounting,
    /// The figure is a property of the deployment, not of a submission.
    ///
    /// A leaf's encoded length and a committed tree's depth are settled
    /// before any transaction exists; a node handed one transaction has
    /// no opinion about them.
    TheFigureIsNotATargetObservation,
    /// The figure's subject cannot exist in this workspace.
    ///
    /// The owner signature (§1.7) and the confidential proof (§12.8's
    /// model serializes none) are both slots rather than things, so there
    /// is nothing for a target to have observed.
    TheSubjectCannotExist,
    /// The dimension is a §1.11 layer rather than a figure.
    ///
    /// It *is* observed — the transcript carries the node's own layer and
    /// its own words — and it is not compared here, because comparing a
    /// layer against a predicted layer would be the discharge-by-intent
    /// §13.2 refuses.
    TheVerdictIsALayerRatherThanAFigure,
}

impl UnobservedReason {
    /// The reason's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::TheNodeExposesNoInterpreterAccounting => {
                "the-node-exposes-no-interpreter-accounting"
            }
            Self::TheFigureIsNotATargetObservation => "the-figure-is-not-a-target-observation",
            Self::TheSubjectCannotExist => "the-subject-cannot-exist",
            Self::TheVerdictIsALayerRatherThanAFigure => {
                "the-verdict-is-a-layer-rather-than-a-figure"
            }
        }
    }
}

/// Why an observable weight still has no comparison in this run.
///
/// Submitted bytes whose target observation omitted a weight are a local
/// evidence absence. A form that never had bytes to submit instead keeps
/// the infrastructure blocker that prevented those bytes from existing.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ComparisonAbsenceReason {
    /// These bytes were submitted, but the target supplied no weight.
    NoObservedWeightForTheseBytes,
    /// The run stopped before this form had bytes to submit.
    RunBlockedBeforeBytesExisted(LiveInfrastructureBlocker),
}

impl ComparisonAbsenceReason {
    /// The reason's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoObservedWeightForTheseBytes => "no-observed-weight-for-these-bytes",
            Self::RunBlockedBeforeBytesExisted(_) => "run-blocked-before-bytes-existed",
        }
    }
}

/// Where one dimension's comparison stands (§18.4).
///
/// Four arms, and exactly one of them is agreement. The two that carry no
/// comparison are separate on purpose: a boundary that cannot expose a
/// figure and a run that did not produce one are different findings, and
/// only the second is repaired by running again.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ComparisonStanding {
    /// Both figures exist over the same bytes, and they are equal.
    Agree {
        /// The one figure both sides reported.
        figure: u64,
    },
    /// Both figures exist over the same bytes, and they differ.
    ///
    /// §18.4 requires both to be recorded, and both are: a comparison
    /// that kept only the difference would have destroyed the evidence
    /// the mismatch is about.
    Mismatch {
        /// What this workspace computed.
        predicted: u64,
        /// What the target reported.
        observed: u64,
    },
    /// This boundary exposes no observation of this dimension.
    NotObservableAtThisBoundary(UnobservedReason),
    /// The dimension is observable, and this run observed nothing.
    ///
    /// The reason distinguishes submitted bytes whose target observation
    /// omitted a weight from a form whose blocker prevented any bytes from
    /// existing. Distinguished from the arm above because this one is
    /// cleared by another run and that one is not.
    NoObservationInThisRun(ComparisonAbsenceReason),
}

impl ComparisonStanding {
    /// Whether this standing is a mismatch.
    ///
    /// The predicate the resource report reads. §18.4 says a mismatch
    /// cannot coexist with a successful result, so the question has to be
    /// askable of a standing without matching on the rest of it.
    #[must_use]
    pub const fn is_mismatch(self) -> bool {
        matches!(self, Self::Mismatch { .. })
    }

    /// Whether this standing is a performed comparison that agreed.
    #[must_use]
    pub const fn is_agreement(self) -> bool {
        matches!(self, Self::Agree { .. })
    }

    /// The standing's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Agree { .. } => "agree",
            Self::Mismatch { .. } => "mismatch",
            Self::NotObservableAtThisBoundary(_) => "not-observable-at-this-boundary",
            Self::NoObservationInThisRun(_) => "no-observation-in-this-run",
        }
    }
}

/// The typed planner failure §18.4 requires a mismatch to return.
///
/// It carries both figures and the dimension they disagree about, because
/// a failure that said only "resources disagreed" would be a failure
/// nobody could act on. There is one variant, and its existence is the
/// point: a mismatch has a value to *be*, so a caller cannot report one
/// by returning success with a note.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ResourcePlannerFailure {
    /// One dimension's prediction and observation disagree.
    PredictionDisagreesWithObservation {
        /// Which submitted form.
        plan: LiveTransferRepresentationPlan,
        /// Which dimension.
        dimension: LiveResourceRecord,
        /// What this workspace computed.
        predicted: u64,
        /// What the target reported.
        observed: u64,
    },
}

/// One submitted form's comparison across §18.3's seventeen dimensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanResourceComparison {
    plan: LiveTransferRepresentationPlan,
    standings: BTreeMap<LiveResourceRecord, ComparisonStanding>,
}

impl PlanResourceComparison {
    /// Which submitted form this is about.
    #[must_use]
    pub const fn plan(&self) -> LiveTransferRepresentationPlan {
        self.plan
    }

    /// Where each dimension stands.
    #[must_use]
    pub const fn standings(&self) -> &BTreeMap<LiveResourceRecord, ComparisonStanding> {
        &self.standings
    }

    /// One dimension's standing.
    #[must_use]
    pub fn standing(&self, dimension: LiveResourceRecord) -> Option<ComparisonStanding> {
        self.standings.get(&dimension).copied()
    }

    /// Every dimension this comparison actually performed and agreed on.
    #[must_use]
    pub fn agreements(&self) -> BTreeMap<LiveResourceRecord, u64> {
        self.standings
            .iter()
            .filter_map(|(dimension, standing)| match standing {
                ComparisonStanding::Agree { figure } => Some((*dimension, *figure)),
                _ => None,
            })
            .collect()
    }

    /// Every typed failure this comparison found.
    ///
    /// Empty for a comparison in which nothing disagreed — which is not
    /// the same as a comparison in which nothing was compared, and
    /// [`Self::agreements`] is what tells the two apart.
    #[must_use]
    pub fn failures(&self) -> Vec<ResourcePlannerFailure> {
        self.standings
            .iter()
            .filter_map(|(dimension, standing)| match standing {
                ComparisonStanding::Mismatch {
                    predicted,
                    observed,
                } => Some(ResourcePlannerFailure::PredictionDisagreesWithObservation {
                    plan: self.plan,
                    dimension: *dimension,
                    predicted: *predicted,
                    observed: *observed,
                }),
                _ => None,
            })
            .collect()
    }
}

/// Why one dimension carries no observation at this boundary.
///
/// Stated once, and read by the comparison rather than restated at each
/// of seventeen call sites. `None` for the one dimension this boundary
/// does expose.
const fn unobservable(dimension: LiveResourceRecord) -> Option<UnobservedReason> {
    use LiveResourceRecord as Record;
    use UnobservedReason as Reason;

    match dimension {
        // The one figure the node publishes about a submission, refused
        // or not.
        Record::CompleteWeight => None,
        // Settled by the deployment before any transaction existed.
        Record::CoordinatorBytes
        | Record::MemberBytes
        | Record::ConstructorBytes
        | Record::TaptreeDepth
        | Record::ControlBytes
        // The node reports a weight and not a virtual size. The identity
        // between them is documented and this study uses it elsewhere,
        // but deriving one observation from another is not observing it,
        // and §18.4's rule is about exactly that kind of substitution.
        | Record::VirtualSize
        | Record::ConstructionAndExecutionTime => Some(Reason::TheFigureIsNotATargetObservation),
        // The adapter writes null for every interpreter figure on an
        // operation step, and says why in its own words.
        Record::InitialWitnessItems
        | Record::PeakMainStack
        | Record::PeakAlternateStack
        | Record::LargestElement
        | Record::ValidationBudget => Some(Reason::TheNodeExposesNoInterpreterAccounting),
        Record::OwnerSignatureWitnessBytes | Record::ConfidentialProofBytes => {
            Some(Reason::TheSubjectCannotExist)
        }
        Record::ConsensusVerdict | Record::RelayPolicyVerdict => {
            Some(Reason::TheVerdictIsALayerRatherThanAFigure)
        }
    }
}

/// Which step submits one representation's transfer.
const fn submission_step(plan: LiveTransferRepresentationPlan) -> LiveNativeStep {
    match plan {
        LiveTransferRepresentationPlan::Explicit => LiveNativeStep::SubmitExplicitTransfer,
        LiveTransferRepresentationPlan::PrivateCommitted => LiveNativeStep::SubmitPrivateTransfer,
    }
}

/// Compare one run's predictions with what the target observed (§18.4).
///
/// One entry per representation plan, whatever the run managed to submit.
/// A form the run never offered still gets a comparison, and every
/// dimension of it stands at [`ComparisonStanding::NoObservationInThisRun`]
/// naming the blocker that stopped it — because a form silently absent
/// from this table would be a form whose figures nobody had checked,
/// reported as a table with no disagreements in it.
#[must_use]
pub fn compare_run(transcript: &LiveNativeTranscript) -> Vec<PlanResourceComparison> {
    [
        LiveTransferRepresentationPlan::Explicit,
        LiveTransferRepresentationPlan::PrivateCommitted,
    ]
    .into_iter()
    .map(|plan| compare_plan(transcript, plan))
    .collect()
}

/// Compare one submitted form.
fn compare_plan(
    transcript: &LiveNativeTranscript,
    plan: LiveTransferRepresentationPlan,
) -> PlanResourceComparison {
    let predicted = transcript
        .predicted()
        .get(&plan)
        .and_then(|predicted| predicted.weight());
    let observed = transcript
        .observation(submission_step(plan))
        .and_then(crate::live_native::LiveNativeObservation::observed_weight);

    // What stopped this form from producing an observation, where one is
    // missing. The gap the run itself recorded is the sharper answer; a
    // form the run did submit and got no weight for carries that local
    // absence instead of borrowing a blocker from another pipeline.
    //
    // The gap's word is scoped to this pipeline's own funding step and is
    // not a claim that no confidential predecessor exists anywhere: one
    // has been funded, mined, and read back through the confidential arm.
    // What this run recorded is that ITS private form was never
    // submitted, which is the fact the standing rests on and which the
    // narrower funding arm does not change.
    let absence_reason = transcript.gap_for(plan).map_or(
        ComparisonAbsenceReason::NoObservedWeightForTheseBytes,
        |_| {
            ComparisonAbsenceReason::RunBlockedBeforeBytesExisted(
                LiveInfrastructureBlocker::NoConfidentialPredecessorCanBeFunded,
            )
        },
    );

    let standings = LiveResourceRecord::ALL
        .iter()
        .map(|dimension| {
            let standing = unobservable(*dimension).map_or_else(
                || compare_figure(predicted, observed, absence_reason),
                ComparisonStanding::NotObservableAtThisBoundary,
            );
            (*dimension, standing)
        })
        .collect();

    PlanResourceComparison { plan, standings }
}

/// Compare one dimension's two figures, where the boundary exposes it.
///
/// The whole of §18.4's arithmetic, in one place. Both figures present
/// and equal is the only agreement; both present and unequal is the
/// mismatch that carries them both; and anything else is an unobserved
/// run, never a zero and never an agreement.
const fn compare_figure(
    predicted: Option<u64>,
    observed: Option<u64>,
    absence_reason: ComparisonAbsenceReason,
) -> ComparisonStanding {
    match (predicted, observed) {
        (Some(predicted), Some(observed)) if predicted == observed => {
            ComparisonStanding::Agree { figure: observed }
        }
        (Some(predicted), Some(observed)) => ComparisonStanding::Mismatch {
            predicted,
            observed,
        },
        _ => ComparisonStanding::NoObservationInThisRun(absence_reason),
    }
}

/// Every typed failure one run's comparisons found (§18.4).
///
/// The value the resource report reads to decide whether it may report a
/// result at all. Empty is the only standing a successful report may sit
/// beside, and emptiness here does not by itself say a comparison
/// happened — [`PlanResourceComparison::agreements`] is what does.
#[must_use]
pub fn run_failures(comparisons: &[PlanResourceComparison]) -> Vec<ResourcePlannerFailure> {
    comparisons
        .iter()
        .flat_map(PlanResourceComparison::failures)
        .collect()
}

/// How many dimensions one run actually compared and agreed on.
#[must_use]
pub fn run_agreements(comparisons: &[PlanResourceComparison]) -> usize {
    comparisons
        .iter()
        .map(|comparison| comparison.agreements().len())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{UnobservedReason, unobservable};
    use crate::live_measurements::LiveResourceRecord;
    use std::collections::BTreeSet;

    #[test]
    fn exactly_one_of_the_seventeen_dimensions_is_observable_at_this_boundary() {
        // The honest shape of §18.4 here, asserted rather than described.
        // Sixteen dimensions have a reason they cannot be observed and
        // one does not, and a later wave that widened the boundary would
        // have to change this count in order to claim the widening.
        let observable: BTreeSet<_> = LiveResourceRecord::ALL
            .iter()
            .filter(|dimension| unobservable(**dimension).is_none())
            .copied()
            .collect();

        assert_eq!(
            observable,
            BTreeSet::from([LiveResourceRecord::CompleteWeight]),
        );
        assert_eq!(
            LiveResourceRecord::ALL.len() - observable.len(),
            16,
            "sixteen dimensions must each carry a reason they are unobserved",
        );
    }

    #[test]
    fn every_unobserved_dimension_names_which_kind_of_absence_it_is() {
        // §18.4's rule made structural: a dimension with no observation
        // carries a typed reason, so it cannot be read as agreement. All
        // four reasons are used, because a taxonomy with a dead arm is a
        // taxonomy that has stopped describing the boundary.
        let mut used: BTreeSet<UnobservedReason> = BTreeSet::new();
        for dimension in LiveResourceRecord::ALL {
            if let Some(reason) = unobservable(*dimension) {
                used.insert(reason);
            }
        }
        assert_eq!(used.len(), 4);
    }
}
