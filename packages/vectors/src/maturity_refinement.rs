//! A concrete STATE output relates to one semantic STATE when its witnessed metadata decodes and the reviewed constructor reproduces its program at the witnessed nonce.
//!
//! The completeness function checks the predecessor's semantic lead window; constructibility at each lead is witnessed by the measured candidates in the test.
//!
//! A finite corpus cannot establish a general forward-simulation theorem over every possible concrete step.
//!
//! Completeness here holds only under the six named representability assumptions, including the nonce budget, sponsorless form and signer-held operator key.
//!
//! The one accepted step is the admitted corpus's accepted variable-schedule step; no acceptance is inferred for another step.
//!
//! These checks establish no lifecycle-wide coinduction and no freshness of the caller-stated branch context.
//!
//! Neither a target-chain observation beyond the admitted acceptance nor construction of a sponsored form follows from this relation.

use linker::CandidateDeploymentIdentity;
use realization::{
    AnnouncementLeadBounds, Cycle, MaturityTransitionRefusal, StateMetadata, StateMetadataRefusal,
    announce_maturity, decode_state_metadata,
};
use tapscript::{
    StateConstructorRefusal, StateInternalKeyPolicy, StateStaticSubtree,
    state_output_program_at_nonce,
};
use target_elements::ReviewedElementsTapscriptDefinition;
use transaction::operator_right::BranchContext;

use crate::maturity_closure::{MaturityClosureRefusal, OracleStateCurve, closure_target};
use crate::maturity_continuity::{MaturityConstructorProjection, ValidatedMaturityContinuity};
use crate::maturity_continuity_report::MaturityContinuityReportRole;
use crate::maturity_history_report::MaturityRootHistoryReportRole;
use crate::maturity_native::MaturityAcceptanceObligation;
use crate::maturity_recovery_report::MaturityPublicRecoveryReportRole;
use crate::maturity_report::MaturitySafetyReportRole;

/// The retained constructor context and caller-stated deployment context against which a STATE output is abstracted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityBoundEnvironment {
    /// The inclusive announcement lead bounds.
    pub bounds: AnnouncementLeadBounds,
    /// The linked static subtree used by both constructor sides.
    pub static_subtree: StateStaticSubtree,
    /// The linked internal-key policy.
    pub internal_key_policy: StateInternalKeyPolicy,
    /// The independently reviewed target definition.
    pub target: ReviewedElementsTapscriptDefinition,
    /// The deployment identity verified by the continuity projection.
    pub identity: CandidateDeploymentIdentity,
    /// The caller-stated branch and checkpoint.
    pub branch: BranchContext,
}

impl MaturityBoundEnvironment {
    /// Read the retained context and the reviewed target used by the constructor.
    ///
    /// # Errors
    /// Returns the reviewed-target refusal if the target definition cannot be obtained.
    pub fn from_projection(
        projection: &ValidatedMaturityContinuity,
    ) -> Result<Self, MaturityClosureRefusal> {
        Ok(Self {
            bounds: projection.bounds(),
            static_subtree: projection.bundle().static_subtree().clone(),
            internal_key_policy: projection.bundle().policy().internal_key(),
            target: closure_target()?,
            identity: projection.identity().clone(),
            branch: projection.branch(),
        })
    }
}

/// One STATE output program and the exact encoded metadata witnessed for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityConcreteState {
    /// The output program to reproduce byte for byte.
    pub program: Vec<u8>,
    /// The canonical metadata bytes, including the representation nonce.
    pub metadata_bytes: Vec<u8>,
}

impl MaturityConcreteState {
    /// Copy the exact program and metadata bytes from one constructor projection.
    #[must_use]
    pub fn from_projection(projection: &MaturityConstructorProjection) -> Self {
        Self {
            program: projection.output_program().to_vec(),
            metadata_bytes: projection.metadata_bytes().to_vec(),
        }
    }
}

/// A failed step of the concrete-to-semantic abstraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaturityAbstractionRefusal {
    /// The witnessed metadata bytes do not decode canonically.
    MetadataDoesNotDecode(StateMetadataRefusal),
    /// The reviewed constructor refuses the witnessed nonce and environment.
    ReconstructionRefused(StateConstructorRefusal),
    /// The reconstructed program differs from the concrete output program.
    ProgramDiffers,
}

/// Decode and reconstruct a concrete STATE output before returning its nonce-erased semantic state.
///
/// # Errors
/// Returns the metadata, reconstruction or exact-program refusal at the first failed check.
pub fn abstract_state(
    concrete: &MaturityConcreteState,
    environment: &MaturityBoundEnvironment,
) -> Result<StateMetadata, MaturityAbstractionRefusal> {
    let encoded = decode_state_metadata(&concrete.metadata_bytes)
        .map_err(MaturityAbstractionRefusal::MetadataDoesNotDecode)?;
    let reconstructed = state_output_program_at_nonce(
        &environment.target,
        &encoded,
        &environment.static_subtree,
        environment.internal_key_policy,
        &OracleStateCurve,
    )
    .map_err(MaturityAbstractionRefusal::ReconstructionRefused)?;
    if reconstructed != concrete.program {
        return Err(MaturityAbstractionRefusal::ProgramDiffers);
    }
    Ok(encoded.semantic)
}

/// Acceptance standing derives only from the projection's obligation: Established gives accepted target identity and Outstanding gives host projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityStepStanding {
    /// Exact submitted bytes carry a verified established acceptance obligation.
    AcceptedWithTargetIdentity,
    /// Constructor continuity holds without an established acceptance obligation.
    HostProjectedOnly,
}

impl MaturityStepStanding {
    /// Classify a validated projection by its verified or passed-through acceptance obligation.
    #[must_use]
    pub fn from_projection(projection: &ValidatedMaturityContinuity) -> Self {
        match projection.acceptance_obligation() {
            MaturityAcceptanceObligation::Established { .. } => Self::AcceptedWithTargetIdentity,
            MaturityAcceptanceObligation::Outstanding { .. } => Self::HostProjectedOnly,
        }
    }
}

/// The uniquely derived semantic transition and its acceptance standing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturityForwardWitness {
    /// The abstracted concrete predecessor W.
    pub predecessor: StateMetadata,
    /// The transition result W′, equal to the abstracted concrete successor.
    pub successor: StateMetadata,
    /// The projection's acceptance standing.
    pub standing: MaturityStepStanding,
}

/// The check that prevents one concrete projection from simulating the semantic transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaturityForwardRefusal {
    /// The predecessor cannot be abstracted under the bound environment.
    PredecessorAbstraction(MaturityAbstractionRefusal),
    /// The requested semantic transition is refused.
    SemanticTransition(MaturityTransitionRefusal),
    /// The successor cannot be abstracted under the bound environment.
    SuccessorAbstraction(MaturityAbstractionRefusal),
    /// The abstracted successor differs from the unique transition result.
    SuccessorDiffers {
        /// The transition result W′.
        derived: StateMetadata,
        /// The abstracted concrete successor.
        abstracted: StateMetadata,
    },
}

/// Require both concrete constructor sides to simulate the one requested semantic announcement.
///
/// # Errors
/// Returns the first predecessor abstraction, transition, successor abstraction or successor-equality refusal.
pub fn simulate_forward(
    projection: &ValidatedMaturityContinuity,
    environment: &MaturityBoundEnvironment,
) -> Result<MaturityForwardWitness, MaturityForwardRefusal> {
    let predecessor = abstract_state(
        &MaturityConcreteState::from_projection(projection.predecessor()),
        environment,
    )
    .map_err(MaturityForwardRefusal::PredecessorAbstraction)?;
    let successor = announce_maturity(
        &predecessor,
        projection.requested_cycle(),
        projection.bounds(),
    )
    .map_err(MaturityForwardRefusal::SemanticTransition)?;
    let abstracted = abstract_state(
        &MaturityConcreteState::from_projection(projection.successor()),
        environment,
    )
    .map_err(MaturityForwardRefusal::SuccessorAbstraction)?;
    if abstracted != successor {
        return Err(MaturityForwardRefusal::SuccessorDiffers {
            derived: successor,
            abstracted,
        });
    }
    Ok(MaturityForwardWitness {
        predecessor,
        successor,
        standing: MaturityStepStanding::from_projection(projection),
    })
}

/// The five stated clauses of the scoped maturity refinement question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityTheoremClause {
    /// Refused concrete steps have no allowed semantic step at their declared boundary.
    SoundnessOfRefusedSteps,
    /// The admitted accepted concrete step has the unique semantic announcement successor.
    ForwardSimulationOfAcceptedSteps,
    /// The accepted step's retained root history has ordered edges.
    EdgeOrderingOfTheRootHistory,
    /// The accepted successor is reconstructible from public bytes.
    ReconstructionFromPublicBytes,
    /// Every request in the admitted predecessor's lead window succeeds under named assumptions.
    SupportedStepCompleteness,
}

/// The report role or local check witnessing one theorem clause.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityClauseWitness {
    /// The maturity-announcement safety report role.
    SafetyReport(MaturitySafetyReportRole),
    /// The state-constructor continuity report role.
    ContinuityReport(MaturityContinuityReportRole),
    /// The state-root history report role.
    RootHistoryReport(MaturityRootHistoryReportRole),
    /// The state-public recovery report role.
    PublicRecoveryReport(MaturityPublicRecoveryReportRole),
    /// This refinement module's semantic-window check.
    ThisModule,
}

impl MaturityTheoremClause {
    /// Every clause in declaration order.
    pub const ALL: [Self; 5] = [
        Self::SoundnessOfRefusedSteps,
        Self::ForwardSimulationOfAcceptedSteps,
        Self::EdgeOrderingOfTheRootHistory,
        Self::ReconstructionFromPublicBytes,
        Self::SupportedStepCompleteness,
    ];

    /// Name the report role or local check that witnesses this clause.
    #[must_use]
    pub const fn witness(self) -> MaturityClauseWitness {
        match self {
            Self::SoundnessOfRefusedSteps => MaturityClauseWitness::SafetyReport(
                MaturitySafetyReportRole::MaturityAnnouncementSafety,
            ),
            Self::ForwardSimulationOfAcceptedSteps => MaturityClauseWitness::ContinuityReport(
                MaturityContinuityReportRole::StateConstructorContinuity,
            ),
            Self::EdgeOrderingOfTheRootHistory => MaturityClauseWitness::RootHistoryReport(
                MaturityRootHistoryReportRole::StateRootHistory,
            ),
            Self::ReconstructionFromPublicBytes => MaturityClauseWitness::PublicRecoveryReport(
                MaturityPublicRecoveryReportRole::StatePublicRecovery,
            ),
            Self::SupportedStepCompleteness => MaturityClauseWitness::ThisModule,
        }
    }
}

/// Conditions under which the admitted predecessor's window is represented by built candidates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityRepresentabilityAssumption {
    /// The selected representation nonce lies within the reviewed search budget.
    NonceWithinTheReviewedBudget,
    /// The candidate has the sponsorless transaction form.
    SponsorlessForm,
    /// Both constructor sides use the retained singleton static subtree.
    SingletonStaticSubtree,
    /// The request lies in the predecessor's inclusive lead window.
    RequestInsideTheLeadWindow,
    /// The retained schedule is admitted by the target's width bound.
    ScheduleAdmittedByTheTargetsWidthBound,
    /// The published signer holds the operator key required by the candidate.
    OperatorKeyHeldByThePublishedSigner,
}

impl MaturityRepresentabilityAssumption {
    /// The complete named assumption set.
    pub const ALL: [Self; 6] = [
        Self::NonceWithinTheReviewedBudget,
        Self::SponsorlessForm,
        Self::SingletonStaticSubtree,
        Self::RequestInsideTheLeadWindow,
        Self::ScheduleAdmittedByTheTargetsWidthBound,
        Self::OperatorKeyHeldByThePublishedSigner,
    ];
}

/// Limits that remain explicit beside the admitted finite witness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityRefinementNonClaim {
    /// Finite tests do not prove the relation for every possible concrete step.
    NoGeneralTheoremFromAFiniteCorpus,
    /// The window check says nothing about construction outside its six assumptions.
    NoCompletenessBeyondTheNamedAssumptions,
    /// No other concrete step receives accepted standing from this corpus.
    NoAcceptedStepBeyondTheAdmittedCorpus,
    /// A checked transition does not establish a lifecycle-wide coinductive property.
    NoLifecycleWideCoinduction,
}

impl MaturityRefinementNonClaim {
    /// Every stated limit in declaration order.
    pub const ALL: [Self; 4] = [
        Self::NoGeneralTheoremFromAFiniteCorpus,
        Self::NoCompletenessBeyondTheNamedAssumptions,
        Self::NoAcceptedStepBeyondTheAdmittedCorpus,
        Self::NoLifecycleWideCoinduction,
    ];

    /// State the exact limit of this finite witness.
    #[must_use]
    pub const fn sentence(self) -> &'static str {
        match self {
            Self::NoGeneralTheoremFromAFiniteCorpus => {
                "A finite corpus cannot prove forward simulation for every concrete step."
            }
            Self::NoCompletenessBeyondTheNamedAssumptions => {
                "The window check cannot establish construction beyond the six named assumptions."
            }
            Self::NoAcceptedStepBeyondTheAdmittedCorpus => {
                "The admitted corpus cannot establish acceptance of another concrete step."
            }
            Self::NoLifecycleWideCoinduction => {
                "One checked transition cannot establish lifecycle-wide coinduction."
            }
        }
    }
}

/// The inclusive semantic lead window and the assumptions attached to its construction evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityCompletenessWitness {
    /// Every requested cycle in the inclusive window, in ascending order.
    pub cycles: Vec<Cycle>,
    /// The six assumptions under which the measured candidates witness construction.
    pub assumptions: [MaturityRepresentabilityAssumption; 6],
}

/// A failure to establish the complete supported semantic lead window and its boundary refusals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaturityCompletenessRefusal {
    /// The predecessor cannot be abstracted under the bound environment.
    PredecessorAbstraction(MaturityAbstractionRefusal),
    /// The lead bounds cannot produce an inclusive window from this predecessor cycle.
    Window(MaturityTransitionRefusal),
    /// The cycle immediately below the window is outside the cycle domain.
    BelowWindowUnavailable,
    /// The cycle immediately above the window is outside the cycle domain.
    AboveWindowUnavailable,
    /// An inside-window request was refused by the semantic transition.
    InsideWindow {
        /// The request that should have succeeded.
        cycle: Cycle,
        /// The actual transition refusal.
        refusal: MaturityTransitionRefusal,
    },
    /// An outside-window request did not yield its required bound refusal.
    OutsideWindow {
        /// The cycle immediately beside the window.
        cycle: Cycle,
        /// The bound refusal required at this side.
        expected: MaturityTransitionRefusal,
        /// The actual refusal, or none when the request succeeded.
        actual: Option<MaturityTransitionRefusal>,
    },
}

/// Check every semantic request of one predecessor's lead window and both neighboring refusals.
///
/// # Errors
/// Returns the first abstraction, window, inside-window or boundary refusal that prevents this finite check.
pub fn check_supported_step_completeness(
    projection: &ValidatedMaturityContinuity,
    environment: &MaturityBoundEnvironment,
) -> Result<MaturityCompletenessWitness, MaturityCompletenessRefusal> {
    let predecessor = abstract_state(
        &MaturityConcreteState::from_projection(projection.predecessor()),
        environment,
    )
    .map_err(MaturityCompletenessRefusal::PredecessorAbstraction)?;
    let (earliest, latest) = environment
        .bounds
        .window(predecessor.cycle)
        .map_err(MaturityCompletenessRefusal::Window)?;
    let mut cycles = Vec::new();
    for ordinal in earliest.get()..=latest.get() {
        let cycle = Cycle::new(ordinal);
        announce_maturity(&predecessor, cycle, environment.bounds)
            .map_err(|refusal| MaturityCompletenessRefusal::InsideWindow { cycle, refusal })?;
        cycles.push(cycle);
    }
    let below = Cycle::new(
        earliest
            .get()
            .checked_sub(1)
            .ok_or(MaturityCompletenessRefusal::BelowWindowUnavailable)?,
    );
    let above = Cycle::new(
        latest
            .get()
            .checked_add(1)
            .ok_or(MaturityCompletenessRefusal::AboveWindowUnavailable)?,
    );
    for (cycle, expected) in [
        (below, MaturityTransitionRefusal::AnnouncementBelowMinimum),
        (above, MaturityTransitionRefusal::AnnouncementAboveMaximum),
    ] {
        let actual = announce_maturity(&predecessor, cycle, environment.bounds).err();
        if actual != Some(expected) {
            return Err(MaturityCompletenessRefusal::OutsideWindow {
                cycle,
                expected,
                actual,
            });
        }
    }
    Ok(MaturityCompletenessWitness {
        cycles,
        assumptions: MaturityRepresentabilityAssumption::ALL,
    })
}

/// The admitted corpus carries exactly one accepted concrete step.
pub const ADMITTED_ACCEPTED_STEPS: usize = 1;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maturity_continuity::tests::{archived, variable_archived};
    use crate::maturity_continuity::{
        MaturityProjectionInput, project_maturity_continuity,
        project_maturity_continuity_with_acceptance,
    };
    use crate::maturity_corpus::maturity_variable_run_of_record;
    use crate::maturity_measurements::measure_maturity_cases;
    use crate::maturity_resources::MaturitySubmission;
    use tapscript::{StateLeafRole, StateStaticNode, TapscriptInstruction, TapscriptProgram};
    use target_elements::OpcodeId;

    fn accepted_projection() -> ValidatedMaturityContinuity {
        let corpus = maturity_variable_run_of_record().expect("accepted corpus");
        project_maturity_continuity_with_acceptance(
            variable_archived().input(),
            corpus.evidence().acceptance_obligation(),
        )
        .expect("accepted projection")
    }

    fn environment(projection: &ValidatedMaturityContinuity) -> MaturityBoundEnvironment {
        MaturityBoundEnvironment::from_projection(projection).expect("reviewed environment")
    }

    #[test]
    fn the_accepted_step_forward_simulates_to_exactly_one_semantic_successor() {
        let projection = accepted_projection();
        let bound = environment(&projection);
        let first = simulate_forward(&projection, &bound).expect("accepted forward simulation");
        let second = simulate_forward(&projection, &bound).expect("same forward simulation");
        let expected = announce_maturity(
            &first.predecessor,
            projection.requested_cycle(),
            projection.bounds(),
        )
        .expect("admitted semantic transition");
        assert_eq!(
            first.standing,
            MaturityStepStanding::AcceptedWithTargetIdentity
        );
        assert_eq!(
            first.predecessor,
            projection.predecessor().encoded_metadata().semantic
        );
        assert_eq!(first.predecessor.cycle, Cycle::new(5));
        assert_eq!(projection.requested_cycle(), Cycle::new(10));
        assert_eq!(
            (projection.bounds().minimum(), projection.bounds().maximum()),
            (Cycle::new(4), Cycle::new(6))
        );
        assert_eq!(first.successor, expected);
        assert_eq!(&first.successor, projection.expected_successor());
        assert_eq!(first, second);
    }

    #[test]
    fn the_abstraction_is_a_function_of_the_concrete_state() {
        let projection = accepted_projection();
        let bound = environment(&projection);
        let concrete = MaturityConcreteState::from_projection(projection.successor());
        let leaf = bound
            .static_subtree
            .leaves()
            .iter()
            .find(|entry| entry.leaf.role == StateLeafRole::Announcement)
            .expect("retained announcement leaf");
        let encoded = decode_state_metadata(&concrete.metadata_bytes).expect("witnessed metadata");
        let mut instructions = leaf.leaf.program.instructions().to_vec();
        let mut other_program = None;
        for _ in 0..32 {
            instructions.push(TapscriptInstruction::Opcode(OpcodeId::Verify));
            let mut changed_leaf = leaf.leaf.clone();
            changed_leaf.program =
                TapscriptProgram::new(instructions.clone()).expect("altered leaf within limit");
            let other_tree = StateStaticSubtree::new(
                &bound.target,
                Some(StateStaticNode::Leaf {
                    identity: leaf.identity,
                    leaf: changed_leaf,
                }),
            )
            .expect("alternate complete static subtree");
            if let Ok(program) = state_output_program_at_nonce(
                &bound.target,
                &encoded,
                &other_tree,
                bound.internal_key_policy,
                &OracleStateCurve,
            ) {
                other_program = Some(program);
                break;
            }
        }
        let other_program = other_program.expect("alternate subtree admits the witnessed nonce");
        assert_ne!(other_program, concrete.program);
        let changed = MaturityConcreteState {
            program: other_program,
            metadata_bytes: concrete.metadata_bytes.clone(),
        };
        assert_eq!(
            abstract_state(&changed, &bound),
            Err(MaturityAbstractionRefusal::ProgramDiffers)
        );
        let undecodable = MaturityConcreteState {
            program: concrete.program.clone(),
            metadata_bytes: vec![0],
        };
        assert!(matches!(
            abstract_state(&undecodable, &bound),
            Err(MaturityAbstractionRefusal::MetadataDoesNotDecode(_))
        ));
        let first = abstract_state(&concrete, &bound).expect("witnessed program abstracts");
        let second = abstract_state(&concrete, &bound).expect("same program abstracts again");
        assert_eq!(first, second);
    }

    #[test]
    fn the_historical_step_is_host_projected_and_not_accepted() {
        let projection =
            project_maturity_continuity(archived().input()).expect("historical host projection");
        let witness = simulate_forward(&projection, &environment(&projection))
            .expect("historical forward simulation");
        assert_eq!(witness.standing, MaturityStepStanding::HostProjectedOnly);
        assert_eq!(&witness.successor, projection.expected_successor());
    }

    #[test]
    fn every_request_inside_the_window_is_constructible_under_the_named_assumptions() {
        let measured = measure_maturity_cases().expect("six constructed candidates");
        assert_eq!(measured.len(), 6);
        for (source, submission) in [
            (archived(), MaturitySubmission::HistoricalWholeMetadata),
            (
                variable_archived(),
                MaturitySubmission::AcceptedVariableMetadata,
            ),
        ] {
            let projection =
                project_maturity_continuity(source.input()).expect("archive projection");
            let bound = environment(&projection);
            let witness = check_supported_step_completeness(&projection, &bound)
                .expect("complete semantic window");
            assert_eq!(
                witness.cycles,
                [Cycle::new(9), Cycle::new(10), Cycle::new(11)]
            );
            assert_eq!(witness.assumptions.len(), 6);
            let predecessor = abstract_state(
                &MaturityConcreteState::from_projection(projection.predecessor()),
                &bound,
            )
            .expect("abstract predecessor");
            assert_eq!(
                announce_maturity(&predecessor, Cycle::new(8), bound.bounds),
                Err(MaturityTransitionRefusal::AnnouncementBelowMinimum)
            );
            assert_eq!(
                announce_maturity(&predecessor, Cycle::new(12), bound.bounds),
                Err(MaturityTransitionRefusal::AnnouncementAboveMaximum)
            );
            let candidates: Vec<_> = measured
                .iter()
                .filter(|candidate| candidate.submission == submission)
                .collect();
            assert_eq!(candidates.len(), 3);
            for candidate in candidates {
                assert!(witness.cycles.contains(&candidate.announced_cycle));
                let mut input: MaturityProjectionInput<'_> = source.input();
                input.submitted_bytes = &candidate.bytes;
                let candidate_projection =
                    project_maturity_continuity(input).expect("measured candidate projects");
                let simulated =
                    simulate_forward(&candidate_projection, &environment(&candidate_projection))
                        .expect("measured candidate forward simulates");
                assert_eq!(
                    simulated.successor.maturity,
                    realization::Maturity::Announced {
                        cycle: candidate.announced_cycle
                    }
                );
            }
        }
        assert_eq!(MaturityRepresentabilityAssumption::ALL.len(), 6);
    }

    #[test]
    fn each_report_witnesses_exactly_one_clause() {
        assert_eq!(MaturityTheoremClause::ALL.len(), 5);
        assert_eq!(
            MaturityTheoremClause::SoundnessOfRefusedSteps.witness(),
            MaturityClauseWitness::SafetyReport(
                MaturitySafetyReportRole::MaturityAnnouncementSafety
            )
        );
        assert_eq!(
            MaturityTheoremClause::ForwardSimulationOfAcceptedSteps.witness(),
            MaturityClauseWitness::ContinuityReport(
                MaturityContinuityReportRole::StateConstructorContinuity
            )
        );
        assert_eq!(
            MaturityTheoremClause::EdgeOrderingOfTheRootHistory.witness(),
            MaturityClauseWitness::RootHistoryReport(
                MaturityRootHistoryReportRole::StateRootHistory
            )
        );
        assert_eq!(
            MaturityTheoremClause::ReconstructionFromPublicBytes.witness(),
            MaturityClauseWitness::PublicRecoveryReport(
                MaturityPublicRecoveryReportRole::StatePublicRecovery
            )
        );
        assert_eq!(
            MaturityTheoremClause::SupportedStepCompleteness.witness(),
            MaturityClauseWitness::ThisModule
        );
    }

    #[test]
    fn the_non_claims_state_what_a_finite_corpus_cannot() {
        assert_eq!(MaturityRefinementNonClaim::ALL.len(), 4);
        assert!(
            MaturityRefinementNonClaim::ALL
                .iter()
                .all(|claim| !claim.sentence().is_empty())
        );
        assert_eq!(ADMITTED_ACCEPTED_STEPS, 1);
    }
}
