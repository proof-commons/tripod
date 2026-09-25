//! Per archive, this module builds maturity candidates at the minimum, interior and maximum leads over the archive's recorded funding.
//!
//! It names §18.2's fourteen cases with their standings. No run judged any candidate's bytes in this measurement, including the two interior candidates whose bytes equal archived submissions, so every observation stands absent. Equality across leads is a measurement of this generation's fixed-width encoding.

use realization::Cycle;
use target_elements_conformance::executor::{OperationStep, TargetOperationPlanner};
use target_elements_conformance::protocol::OperationSubject;
use transaction::bytes::TargetTransaction;
use transaction::error::TransactionRefusal;

use crate::maturity_closure::{MaturityClosureRefusal, MaturityWitnessSelection, closure_target};
use crate::maturity_corpus::{
    MaturityCorpusImportRefusal, ValidatedMaturityCorpus, maturity_run_of_record,
    maturity_variable_run_of_record,
};
use crate::maturity_native::{
    MaturityAnnouncementPlanner, MaturityLead, MaturityNativePlanRefusal,
};
use crate::maturity_resources::{
    MaturityResourcesRefusal, MaturitySubmission, MaturitySubmissionResources,
    predict_unjudged_submission,
};

/// The fourteen complete-transaction cases of §18.2, in its published order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityResourceCase {
    /// The earliest admissible lead.
    MinimumLead,
    /// The latest admissible lead.
    MaximumLead,
    /// An admissible lead inside the window.
    RepresentativeInteriorLead,
    /// A predecessor with distinct nonzero quantities.
    NontrivialMetadata,
    /// A successor found at nonce zero.
    NonceZero,
    /// A successor found at a nonzero nonce.
    NonceNonzero,
    /// The greatest search attempt found in the finite corpus.
    GreatestNonceAttemptFoundInFiniteCorpus,
    /// The sponsorless form.
    Sponsorless,
    /// A sponsored form without a change output.
    SponsoredWithoutChange,
    /// A sponsored form with a change output where supported.
    SponsoredWithChangeWhereSupported,
    /// The deepest executing control path.
    DeepestControlPath,
    /// The largest witnessed predecessor metadata item.
    LargestMetadataWitness,
    /// The operator signature item.
    OperatorSignature,
    /// A maximum sponsor shape, if one has a subject.
    CandidateMaximumSponsorShape,
}

impl MaturityResourceCase {
    /// Every §18.2 case in the guide's order.
    pub const ALL: &'static [Self; 14] = &[
        Self::MinimumLead,
        Self::MaximumLead,
        Self::RepresentativeInteriorLead,
        Self::NontrivialMetadata,
        Self::NonceZero,
        Self::NonceNonzero,
        Self::GreatestNonceAttemptFoundInFiniteCorpus,
        Self::Sponsorless,
        Self::SponsoredWithoutChange,
        Self::SponsoredWithChangeWhereSupported,
        Self::DeepestControlPath,
        Self::LargestMetadataWitness,
        Self::OperatorSignature,
        Self::CandidateMaximumSponsorShape,
    ];

    /// The case's stable report spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MinimumLead => "minimum-lead",
            Self::MaximumLead => "maximum-lead",
            Self::RepresentativeInteriorLead => "representative-interior-lead",
            Self::NontrivialMetadata => "nontrivial-metadata",
            Self::NonceZero => "nonce-zero",
            Self::NonceNonzero => "nonce-nonzero",
            Self::GreatestNonceAttemptFoundInFiniteCorpus => {
                "greatest-nonce-attempt-found-in-the-finite-corpus"
            }
            Self::Sponsorless => "sponsorless",
            Self::SponsoredWithoutChange => "sponsored-without-change",
            Self::SponsoredWithChangeWhereSupported => "sponsored-with-change-where-supported",
            Self::DeepestControlPath => "deepest-control-path",
            Self::LargestMetadataWitness => "largest-metadata-witness",
            Self::OperatorSignature => "operator-signature",
            Self::CandidateMaximumSponsorShape => "candidate-maximum-sponsor-shape",
        }
    }

    /// Whether the case is built, a corpus reading, refused, or subjectless.
    #[must_use]
    pub const fn standing(self) -> MaturityCaseStanding {
        match self {
            Self::GreatestNonceAttemptFoundInFiniteCorpus => {
                MaturityCaseStanding::ReadFromTheNonceCorpus
            }
            Self::SponsoredWithoutChange | Self::SponsoredWithChangeWhereSupported => {
                MaturityCaseStanding::SponsoredFormRefusedAtConstruction
            }
            Self::CandidateMaximumSponsorShape => {
                MaturityCaseStanding::NoSponsorShapeInThisGeneration
            }
            _ => MaturityCaseStanding::MeasuredOverBuiltCandidates,
        }
    }
}

/// How one §18.2 case has a subject in this generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityCaseStanding {
    /// A complete candidate supplies its measurement.
    MeasuredOverBuiltCandidates,
    /// The finite nonce corpus supplies the reading.
    ReadFromTheNonceCorpus,
    /// Construction refuses the sponsored form before a transaction exists.
    SponsoredFormRefusedAtConstruction,
    /// No maximum sponsor shape has a subject for this operation.
    NoSponsorShapeInThisGeneration,
}

/// One complete candidate predicted from its linked bundle and exact bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityMeasuredCandidate {
    /// The archive whose funding responses built this candidate.
    pub submission: MaturitySubmission,
    /// The requested position in the admissible lead window.
    pub lead: MaturityLead,
    /// The cycle carried by the finalized announcement.
    pub announced_cycle: Cycle,
    /// The exact signed candidate serialization.
    pub bytes: Vec<u8>,
    /// Predictions with no observation from this run.
    pub resources: MaturitySubmissionResources,
}

/// A typed refusal while building or predicting a candidate.
#[derive(Debug)]
pub enum MaturityMeasurementRefusal {
    /// An archive failed admission.
    Corpus(MaturityCorpusImportRefusal),
    /// A lead planner refused a funding exchange or construction.
    Planner(MaturityNativePlanRefusal),
    /// The planner's refusal marker carried no retained reason.
    PlannerMarkerWithoutRefusal,
    /// The two funding responses did not leave a submission to measure.
    MissingSubmission(MaturitySubmission),
    /// The candidate's exact bytes did not decode.
    TransactionDecode(TransactionRefusal),
    /// The reviewed target definition was unavailable.
    Target(MaturityClosureRefusal),
    /// The candidate's bundle or transaction refused resource derivation.
    Resources(MaturityResourcesRefusal),
}

/// Build and predict the three admissible leads over each archive's funding.
///
/// # Errors
/// Returns the typed archive, planner, byte, target or prediction refusal; an absent submission is never substituted with an archived one.
pub fn measure_maturity_cases() -> Result<Vec<MaturityMeasuredCandidate>, MaturityMeasurementRefusal>
{
    let whole = maturity_run_of_record().map_err(MaturityMeasurementRefusal::Corpus)?;
    let variable = maturity_variable_run_of_record().map_err(MaturityMeasurementRefusal::Corpus)?;
    let target = closure_target().map_err(MaturityMeasurementRefusal::Target)?;
    let mut candidates = Vec::with_capacity(6);
    for (submission, corpus) in [
        (MaturitySubmission::HistoricalWholeMetadata, whole),
        (MaturitySubmission::AcceptedVariableMetadata, variable),
    ] {
        for lead in [
            MaturityLead::Minimum,
            MaturityLead::Interior,
            MaturityLead::Maximum,
        ] {
            let planner = replay_candidate(submission, corpus, lead)?;
            let bytes = planner
                .submission_bytes()
                .ok_or(MaturityMeasurementRefusal::MissingSubmission(submission))?
                .to_vec();
            let announced_cycle = planner
                .announcement()
                .ok_or(MaturityMeasurementRefusal::MissingSubmission(submission))?
                .construction()
                .request()
                .announced_cycle();
            let transaction = TargetTransaction::decode(&bytes)
                .map_err(MaturityMeasurementRefusal::TransactionDecode)?;
            let resources =
                predict_unjudged_submission(submission, planner.bundle(), &transaction, &target)
                    .map_err(MaturityMeasurementRefusal::Resources)?;
            candidates.push(MaturityMeasuredCandidate {
                submission,
                lead,
                announced_cycle,
                bytes,
                resources,
            });
        }
    }
    Ok(candidates)
}

fn replay_candidate(
    submission: MaturitySubmission,
    corpus: &ValidatedMaturityCorpus,
    lead: MaturityLead,
) -> Result<MaturityAnnouncementPlanner, MaturityMeasurementRefusal> {
    let evidence = corpus.evidence();
    let mut planner = MaturityAnnouncementPlanner::at_lead(
        evidence.identity().clone(),
        evidence.branch(),
        MaturityWitnessSelection::Retained(evidence.schedule()),
        lead,
    )
    .map_err(MaturityMeasurementRefusal::Planner)?;
    let mut next = planner
        .next_step(None)
        .map_err(|_| marker_refusal(&planner))?;
    for (position, (step, response)) in corpus.exchanges().iter().take(2).enumerate() {
        if next.as_ref() != Some(step) {
            return Err(MaturityMeasurementRefusal::Planner(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position },
            ));
        }
        next = planner
            .next_step(Some((step.case(), response)))
            .map_err(|_| marker_refusal(&planner))?;
    }
    if !matches!(
        next.as_ref().map(OperationStep::subject),
        Some(OperationSubject::Submission(_))
    ) {
        return Err(MaturityMeasurementRefusal::MissingSubmission(submission));
    }
    Ok(planner)
}

fn marker_refusal(planner: &MaturityAnnouncementPlanner) -> MaturityMeasurementRefusal {
    planner.refusal().map_or(
        MaturityMeasurementRefusal::PlannerMarkerWithoutRefusal,
        |refusal| MaturityMeasurementRefusal::Planner(refusal.clone()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maturity_closure::OracleStateCurve;
    use crate::maturity_resources::{
        MaturityObservationStanding, MaturityPrediction, MaturityResourceField,
        MaturityWitnessPosition,
    };
    use realization::announce_maturity;
    use tapscript::StateWitnessSchedule;
    use transaction::live_request::{RequestedForm, SponsorChangeRequest};
    use transaction::state_construct::construct_maturity_announcement;
    use transaction::state_request::MaturityAnnouncementRequest;

    fn figure(candidate: &MaturityMeasuredCandidate, field: MaturityResourceField) -> u64 {
        match candidate.resources.fields.get(&field) {
            Some(MaturityPrediction::Figure(value)) => *value,
            other => panic!("the measured field must carry a figure: {other:?}"),
        }
    }

    fn metadata_item_width(candidate: &MaturityMeasuredCandidate) -> u64 {
        let Some(MaturityPrediction::ByPosition(widths)) = candidate
            .resources
            .fields
            .get(&MaturityResourceField::WitnessBytesByRole)
        else {
            panic!("the candidate has an ordered witness-width prediction");
        };
        widths
            .iter()
            .find_map(|(position, width)| {
                (*position == MaturityWitnessPosition::PredecessorMetadata).then_some(*width)
            })
            .expect("the witnessed predecessor metadata has a width")
    }

    #[test]
    fn the_fourteen_cases_are_section_eighteen_two_in_order() {
        use MaturityCaseStanding as Standing;
        use MaturityResourceCase as Case;
        assert_eq!(Case::ALL.len(), 14);
        assert_eq!(
            Case::ALL.iter().map(|case| case.name()).collect::<Vec<_>>(),
            [
                "minimum-lead",
                "maximum-lead",
                "representative-interior-lead",
                "nontrivial-metadata",
                "nonce-zero",
                "nonce-nonzero",
                "greatest-nonce-attempt-found-in-the-finite-corpus",
                "sponsorless",
                "sponsored-without-change",
                "sponsored-with-change-where-supported",
                "deepest-control-path",
                "largest-metadata-witness",
                "operator-signature",
                "candidate-maximum-sponsor-shape",
            ]
        );
        for (standing, count) in [
            (Standing::MeasuredOverBuiltCandidates, 10),
            (Standing::ReadFromTheNonceCorpus, 1),
            (Standing::SponsoredFormRefusedAtConstruction, 2),
            (Standing::NoSponsorShapeInThisGeneration, 1),
        ] {
            assert_eq!(
                Case::ALL
                    .iter()
                    .filter(|case| case.standing() == standing)
                    .count(),
                count
            );
        }
        assert_eq!(
            measure_maturity_cases()
                .expect("six candidates build")
                .len(),
            6
        );
    }

    #[test]
    fn each_lead_announces_its_window_edge() {
        let candidates = measure_maturity_cases().expect("six candidates build");
        for (submission, corpus, group) in [
            (
                MaturitySubmission::HistoricalWholeMetadata,
                maturity_run_of_record().expect("whole archive admitted"),
                &candidates[0..3],
            ),
            (
                MaturitySubmission::AcceptedVariableMetadata,
                maturity_variable_run_of_record().expect("variable archive admitted"),
                &candidates[3..6],
            ),
        ] {
            for ((candidate, lead), expected) in group
                .iter()
                .zip([
                    MaturityLead::Minimum,
                    MaturityLead::Interior,
                    MaturityLead::Maximum,
                ])
                .zip([Cycle::new(9), Cycle::new(10), Cycle::new(11)])
            {
                assert_eq!(candidate.submission, submission);
                assert_eq!(candidate.lead, lead);
                assert_eq!(candidate.announced_cycle, expected);
                let planner = replay_candidate(submission, corpus, lead)
                    .expect("recorded funding builds the candidate");
                let announcement = planner
                    .announcement()
                    .expect("funding builds an announcement");
                let construction = announcement.construction();
                let predecessor = construction.validated_view().view().predecessor_metadata();
                assert_eq!(predecessor.cycle, Cycle::new(5));
                assert_eq!(construction.request().announced_cycle(), expected);
                assert_eq!(
                    construction.successor_metadata(),
                    announce_maturity(
                        &predecessor,
                        expected,
                        planner.bundle().deployment().lead_bounds().bounds()
                    )
                    .expect("each announced cycle is inside the window")
                );
            }
        }
    }

    #[test]
    fn the_interior_lead_is_the_archived_submission() {
        let candidates = measure_maturity_cases().expect("six candidates build");
        for (corpus, candidate) in [
            (
                maturity_run_of_record().expect("whole archive admitted"),
                &candidates[1],
            ),
            (
                maturity_variable_run_of_record().expect("variable archive admitted"),
                &candidates[4],
            ),
        ] {
            let archived = corpus
                .exchanges()
                .iter()
                .find_map(|(step, _)| match step.subject() {
                    OperationSubject::Submission(subject) => {
                        Some(subject.transaction_bytes.as_slice())
                    }
                    _ => None,
                })
                .expect("the archive has submitted request bytes");
            assert_eq!(candidate.lead, MaturityLead::Interior);
            assert_eq!(candidate.bytes.as_slice(), archived);
            assert_eq!(candidate.resources.standings.len(), 21);
            assert_eq!(
                candidate
                    .resources
                    .standings
                    .values()
                    .copied()
                    .collect::<Vec<_>>(),
                vec![MaturityObservationStanding::NoObservationInThisRun; 21]
            );
        }
    }

    #[test]
    fn every_candidate_of_one_schedule_costs_the_same() {
        use MaturityResourceField as Field;
        let candidates = measure_maturity_cases().expect("six candidates build");
        assert_eq!(candidates.len(), 6);
        for group in [&candidates[0..3], &candidates[3..6]] {
            for candidate in &group[1..] {
                assert_eq!(candidate.bytes.len(), group[0].bytes.len());
                assert_eq!(
                    candidate
                        .resources
                        .fields
                        .iter()
                        .filter(|(field, _)| **field != Field::NonceSearchAttempts)
                        .collect::<Vec<_>>(),
                    group[0]
                        .resources
                        .fields
                        .iter()
                        .filter(|(field, _)| **field != Field::NonceSearchAttempts)
                        .collect::<Vec<_>>()
                );
                assert_eq!(candidate.resources.roster, group[0].resources.roster);
                assert_eq!(candidate.resources.standings, group[0].resources.standings);
            }
        }
        assert_eq!(candidates[0].bytes.len(), 1_694);
        assert_eq!(candidates[3].bytes.len(), 1_701);
        assert_eq!(
            (
                metadata_item_width(&candidates[0]),
                metadata_item_width(&candidates[3])
            ),
            (86, 53)
        );
        assert_eq!(
            (
                figure(&candidates[0], Field::AnnouncementLeafBytes),
                figure(&candidates[3], Field::AnnouncementLeafBytes)
            ),
            (1_286, 1_326)
        );
        assert_eq!(
            (
                figure(&candidates[0], Field::TransactionWeight),
                figure(&candidates[3], Field::TransactionWeight)
            ),
            (2_084, 2_091)
        );
    }

    #[test]
    fn both_sponsored_cases_are_refused_where_wave_eleven_pins_them() {
        let corpus = maturity_variable_run_of_record().expect("variable archive admitted");
        let planner = replay_candidate(
            MaturitySubmission::AcceptedVariableMetadata,
            corpus,
            MaturityLead::Interior,
        )
        .expect("interior candidate builds over recorded funding");
        let construction = planner
            .announcement()
            .expect("funding builds the interior announcement")
            .construction();
        let target = closure_target().expect("the reviewed target exists");
        for change in [
            SponsorChangeRequest::NotRequested,
            SponsorChangeRequest::Requested,
        ] {
            let request = MaturityAnnouncementRequest::new(
                construction.request().announced_cycle(),
                RequestedForm::Sponsored,
                change,
            )
            .expect("both sponsored requests are representable");
            assert_eq!(
                construct_maturity_announcement(
                    &target,
                    construction.abi(),
                    construction.validated_view(),
                    &request,
                    &OracleStateCurve
                ),
                Err(TransactionRefusal::SponsoredMaturityFormHasNoCarrier)
            );
        }
    }

    #[test]
    fn the_largest_metadata_witness_is_the_replay_only_schedule() {
        let candidates = measure_maturity_cases().expect("six candidates build");
        assert_eq!(metadata_item_width(&candidates[0]), 86);
        assert_eq!(metadata_item_width(&candidates[3]), 53);
        assert_eq!(
            metadata_item_width(&candidates[0]).cmp(&80),
            std::cmp::Ordering::Greater
        );
        assert!(StateWitnessSchedule::WholeMetadata.is_replay_only());
        assert!(!StateWitnessSchedule::VariableMetadata.is_replay_only());
        assert_eq!(
            maturity_run_of_record()
                .expect("whole archive admitted")
                .schedule(),
            StateWitnessSchedule::WholeMetadata
        );
        assert_eq!(
            maturity_variable_run_of_record()
                .expect("variable archive admitted")
                .schedule(),
            StateWitnessSchedule::VariableMetadata
        );
    }
}
