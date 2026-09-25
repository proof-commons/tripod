//! This corpus records the eight real-curve constructor derivations performed by the measured maturity candidates under the reviewed nonce budget.
//!
//! It reports each selection, its attempts and rejected lower candidates, and the greatest attempt count. An independent scan tests host leastness for these eight derivations. The residual permits a later admissible nonce beyond the budget; host leastness is not target leastness, these derivations say nothing of other metadata, and the register cell belongs to the closure row.

use linker::StateLinkRefusal;
use realization::{StateMetadata, StateRepresentationNonce};
use tapscript::{StateConstructorRefusal, StateNonceBudget};
use target_elements::ReviewedElementsTapscriptDefinition;

use crate::maturity_closure::{OracleStateCurve, closure_target};
use crate::maturity_corpus::{maturity_run_of_record, maturity_variable_run_of_record};
use crate::maturity_measurements::{MaturityMeasurementRefusal, replay_candidate};
use crate::maturity_native::{MaturityAnnouncementPlanner, MaturityLead};
use crate::maturity_resources::MaturitySubmission;

/// Which side's metadata the linked constructor derives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityConstructorSide {
    /// The archive's funded predecessor, read from its interior planner.
    Predecessor,
    /// A successor at one of the measured leads.
    Successor,
}

/// One real-curve selection and every lower refusal, in nonce order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityNonceDerivation {
    /// The archive whose funding supplies this derivation.
    pub submission: MaturitySubmission,
    /// The metadata side committed by this derivation.
    pub side: MaturityConstructorSide,
    /// The lead for a successor; absent for the predecessor.
    pub lead: Option<MaturityLead>,
    /// The first admitted representation nonce.
    pub selected: StateRepresentationNonce,
    /// The number of attempted nonces, including the selection.
    pub attempts: u32,
    /// Every lower nonce and its retryable refusal, in scan order.
    pub rejected: Vec<(StateRepresentationNonce, StateConstructorRefusal)>,
}

/// The two archives' eight derivations under their linked constructor budget.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityNonceCorpus {
    /// Per archive: predecessor, then minimum, interior and maximum successors.
    pub derivations: [MaturityNonceDerivation; 8],
    /// The reviewed search bound read from the interior planner's linked policy.
    pub budget: StateNonceBudget,
}

impl MaturityNonceCorpus {
    /// The greatest attempt count, choosing the first derivation on a tie.
    #[must_use]
    pub fn greatest(&self) -> &MaturityNonceDerivation {
        self.derivations[1..]
            .iter()
            .fold(&self.derivations[0], |greatest, next| {
                if next.attempts > greatest.attempts {
                    next
                } else {
                    greatest
                }
            })
    }
}

/// The finite corpus's residual, embedding `StateNonceEvidence::RESIDUAL` verbatim.
pub const NONCE_CORPUS_RESIDUAL: &str = "Eight named real-curve derivations succeeded within the reviewed 4,096-attempt budget; host leastness only; a later admissible nonce may exist beyond the search budget; empirical success over these eight does not establish that every metadata admits a nonce within the budget.";

/// A typed refusal from replay, target selection or linked construction.
#[derive(Debug)]
pub enum MaturityNonceCorpusRefusal {
    /// A measured candidate's archive, replay or target refused.
    Measurement(MaturityMeasurementRefusal),
    /// Applying a linked constructor to one metadata refused.
    Link(StateLinkRefusal),
}

fn metadata_for(
    submission: MaturitySubmission,
    side: MaturityConstructorSide,
    planner: &MaturityAnnouncementPlanner,
) -> Result<StateMetadata, MaturityNonceCorpusRefusal> {
    let construction = planner
        .announcement()
        .ok_or(MaturityNonceCorpusRefusal::Measurement(
            MaturityMeasurementRefusal::MissingSubmission(submission),
        ))?
        .construction();
    Ok(match side {
        MaturityConstructorSide::Predecessor => {
            construction.validated_view().view().predecessor_metadata()
        }
        MaturityConstructorSide::Successor => construction.successor_metadata(),
    })
}

fn derive(
    submission: MaturitySubmission,
    side: MaturityConstructorSide,
    lead: Option<MaturityLead>,
    planner: &MaturityAnnouncementPlanner,
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<MaturityNonceDerivation, MaturityNonceCorpusRefusal> {
    let metadata = metadata_for(submission, side, planner)?;
    let built = planner
        .bundle()
        .apply_constructor(target, &metadata, &OracleStateCurve)
        .map_err(MaturityNonceCorpusRefusal::Link)?;
    let evidence = built.evidence();
    Ok(MaturityNonceDerivation {
        submission,
        side,
        lead,
        selected: evidence.selected,
        attempts: evidence.selected.get() + 1,
        rejected: evidence.rejected.clone(),
    })
}

fn archive_derivations(
    submission: MaturitySubmission,
    corpus: &crate::maturity_corpus::ValidatedMaturityCorpus,
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<([MaturityNonceDerivation; 4], StateNonceBudget), MaturityNonceCorpusRefusal> {
    let minimum = replay_candidate(submission, corpus, MaturityLead::Minimum)
        .map_err(MaturityNonceCorpusRefusal::Measurement)?;
    let interior = replay_candidate(submission, corpus, MaturityLead::Interior)
        .map_err(MaturityNonceCorpusRefusal::Measurement)?;
    let maximum = replay_candidate(submission, corpus, MaturityLead::Maximum)
        .map_err(MaturityNonceCorpusRefusal::Measurement)?;
    let budget = interior.bundle().policy().budget();
    Ok((
        [
            derive(
                submission,
                MaturityConstructorSide::Predecessor,
                None,
                &interior,
                target,
            )?,
            derive(
                submission,
                MaturityConstructorSide::Successor,
                Some(MaturityLead::Minimum),
                &minimum,
                target,
            )?,
            derive(
                submission,
                MaturityConstructorSide::Successor,
                Some(MaturityLead::Interior),
                &interior,
                target,
            )?,
            derive(
                submission,
                MaturityConstructorSide::Successor,
                Some(MaturityLead::Maximum),
                &maximum,
                target,
            )?,
        ],
        budget,
    ))
}

/// Derive the two archives' predecessor and three successor selections each.
///
/// # Errors
/// Returns a typed archive, planner, target or linked-constructor refusal.
pub fn maturity_nonce_corpus() -> Result<MaturityNonceCorpus, MaturityNonceCorpusRefusal> {
    let whole = maturity_run_of_record().map_err(|refusal| {
        MaturityNonceCorpusRefusal::Measurement(MaturityMeasurementRefusal::Corpus(refusal))
    })?;
    let variable = maturity_variable_run_of_record().map_err(|refusal| {
        MaturityNonceCorpusRefusal::Measurement(MaturityMeasurementRefusal::Corpus(refusal))
    })?;
    let target = closure_target().map_err(|refusal| {
        MaturityNonceCorpusRefusal::Measurement(MaturityMeasurementRefusal::Target(refusal))
    })?;
    let ([h0, h1, h2, h3], budget) =
        archive_derivations(MaturitySubmission::HistoricalWholeMetadata, whole, &target)?;
    let ([v0, v1, v2, v3], _) = archive_derivations(
        MaturitySubmission::AcceptedVariableMetadata,
        variable,
        &target,
    )?;
    Ok(MaturityNonceCorpus {
        derivations: [h0, h1, h2, h3, v0, v1, v2, v3],
        budget,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use linker::CandidateLinkedMaturityBundle;
    use realization::EncodedStateMetadata;
    use tapscript::{CandidateStateConstructor, StateNonceEvidence, state_output_program_at_nonce};

    fn planner_for(derivation: &MaturityNonceDerivation) -> MaturityAnnouncementPlanner {
        let corpus = match derivation.submission {
            MaturitySubmission::HistoricalWholeMetadata => {
                maturity_run_of_record().expect("whole archive admitted")
            }
            MaturitySubmission::AcceptedVariableMetadata => {
                maturity_variable_run_of_record().expect("variable archive admitted")
            }
        };
        let lead = match (derivation.side, derivation.lead) {
            (MaturityConstructorSide::Predecessor, None) => MaturityLead::Interior,
            (MaturityConstructorSide::Successor, Some(lead)) => lead,
            other => panic!("the derivation has a side and lead: {other:?}"),
        };
        replay_candidate(derivation.submission, corpus, lead)
            .expect("recorded funding builds this lead")
    }

    #[test]
    fn the_corpus_is_eight_named_derivations() {
        use MaturityConstructorSide as Side;
        use MaturityLead as Lead;
        use MaturitySubmission as Submission;
        let corpus = maturity_nonce_corpus().expect("eight real-curve derivations succeed");
        assert_eq!(corpus.derivations.len(), 8);
        assert_eq!(corpus.budget, StateNonceBudget::default());
        assert_eq!(corpus.budget.attempts(), 4_096);
        assert_eq!(
            corpus
                .derivations
                .iter()
                .map(|entry| (entry.submission, entry.side, entry.lead))
                .collect::<Vec<_>>(),
            [
                (Submission::HistoricalWholeMetadata, Side::Predecessor, None),
                (
                    Submission::HistoricalWholeMetadata,
                    Side::Successor,
                    Some(Lead::Minimum)
                ),
                (
                    Submission::HistoricalWholeMetadata,
                    Side::Successor,
                    Some(Lead::Interior)
                ),
                (
                    Submission::HistoricalWholeMetadata,
                    Side::Successor,
                    Some(Lead::Maximum)
                ),
                (
                    Submission::AcceptedVariableMetadata,
                    Side::Predecessor,
                    None
                ),
                (
                    Submission::AcceptedVariableMetadata,
                    Side::Successor,
                    Some(Lead::Minimum)
                ),
                (
                    Submission::AcceptedVariableMetadata,
                    Side::Successor,
                    Some(Lead::Interior)
                ),
                (
                    Submission::AcceptedVariableMetadata,
                    Side::Successor,
                    Some(Lead::Maximum)
                ),
            ]
        );
        assert_eq!(
            corpus
                .derivations
                .iter()
                .map(|entry| (entry.selected.get(), entry.attempts))
                .collect::<Vec<_>>(),
            [
                (1, 2),
                (1, 2),
                (0, 1),
                (2, 3),
                (1, 2),
                (1, 2),
                (0, 1),
                (2, 3)
            ]
        );
    }

    #[test]
    fn every_selection_is_the_least_admitted_nonce() {
        let corpus = maturity_nonce_corpus().expect("eight real-curve derivations succeed");
        let target = closure_target().expect("reviewed target exists");
        for derivation in &corpus.derivations {
            let planner = planner_for(derivation);
            let metadata = metadata_for(derivation.submission, derivation.side, &planner)
                .expect("the replay retains its construction");
            let bundle: &CandidateLinkedMaturityBundle = planner.bundle();
            assert_eq!(bundle.policy().budget(), corpus.budget);
            let built = bundle
                .apply_constructor(&target, &metadata, &OracleStateCurve)
                .expect("this metadata has a reviewed selection");
            assert_eq!(derivation.selected, built.evidence().selected);
            assert_eq!(derivation.rejected, built.evidence().rejected);
            assert_eq!(derivation.attempts, derivation.selected.get() + 1);
            assert_eq!(
                derivation.rejected.len(),
                usize::try_from(derivation.selected.get()).expect("nonce fits usize")
            );
            for (attempt, (recorded_nonce, refusal)) in derivation.rejected.iter().enumerate() {
                let nonce = StateRepresentationNonce::new(
                    u32::try_from(attempt).expect("attempt fits u32"),
                );
                assert_eq!(*recorded_nonce, nonce);
                assert!(refusal.retryable());
                assert_eq!(
                    state_output_program_at_nonce(
                        &target,
                        &EncodedStateMetadata {
                            semantic: metadata,
                            representation: nonce
                        },
                        bundle.static_subtree(),
                        bundle.policy().internal_key(),
                        &OracleStateCurve,
                    ),
                    Err(*refusal)
                );
            }
            assert_eq!(
                state_output_program_at_nonce(
                    &target,
                    &EncodedStateMetadata {
                        semantic: metadata,
                        representation: derivation.selected
                    },
                    bundle.static_subtree(),
                    bundle.policy().internal_key(),
                    &OracleStateCurve,
                ),
                Ok(built.output_program())
            );
        }
    }

    #[test]
    fn the_archived_derivations_are_the_archives_own() {
        let corpus = maturity_nonce_corpus().expect("eight real-curve derivations succeed");
        assert_eq!(
            (
                corpus.derivations[0].selected.get(),
                corpus.derivations[2].selected.get()
            ),
            (1, 0)
        );
        assert_eq!(
            (
                corpus.derivations[0].attempts,
                corpus.derivations[2].attempts
            ),
            (2, 1)
        );
    }

    #[test]
    fn the_greatest_attempt_is_stated_beside_the_budget() {
        let corpus = maturity_nonce_corpus().expect("eight real-curve derivations succeed");
        let greatest = corpus.greatest();
        assert_eq!(corpus.budget, StateNonceBudget::default());
        assert_eq!(corpus.budget.attempts(), 4_096);
        assert_eq!(
            (
                greatest.submission,
                greatest.side,
                greatest.lead,
                greatest.selected.get(),
                greatest.attempts
            ),
            (
                MaturitySubmission::HistoricalWholeMetadata,
                MaturityConstructorSide::Successor,
                Some(MaturityLead::Maximum),
                2,
                3
            )
        );
        assert!(greatest.attempts <= corpus.budget.attempts());
        let shortened = StateNonceBudget::new(greatest.attempts - 1);
        assert_eq!(
            shortened,
            Ok(StateNonceBudget::new(2).expect("two attempts fit the reviewed budget"))
        );
        let planner = planner_for(greatest);
        let metadata = metadata_for(greatest.submission, greatest.side, &planner)
            .expect("the greatest replay retains its construction");
        let target = closure_target().expect("reviewed target exists");
        assert_eq!(
            CandidateStateConstructor::derive(
                &target,
                &metadata,
                planner.bundle().static_subtree(),
                planner.bundle().policy().internal_key(),
                shortened.expect("the greatest exceeds one attempt"),
                &OracleStateCurve,
            ),
            Err(StateConstructorRefusal::RepresentationSearchExhausted)
        );
    }

    #[test]
    fn the_residual_claims_no_totality() {
        let corpus = maturity_nonce_corpus().expect("eight real-curve derivations succeed");
        assert!(NONCE_CORPUS_RESIDUAL.contains(StateNonceEvidence::RESIDUAL));
        assert!(NONCE_CORPUS_RESIDUAL.contains("Eight named real-curve derivations"));
        assert_eq!(corpus.derivations.len(), 8);
        assert_eq!(
            corpus
                .derivations
                .iter()
                .filter(|entry| {
                    !matches!(
                        (entry.side, entry.lead),
                        (MaturityConstructorSide::Predecessor, None)
                            | (MaturityConstructorSide::Successor, Some(_))
                    )
                })
                .count(),
            0
        );
    }
}
