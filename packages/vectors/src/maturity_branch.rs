//! Branch-relative STATE root-edge realizations under Guide 14 §§14.6 and 17.1 (`T11-122`).

use realization::domain::Cycle;
use realization::state::StateMetadata;
use tapscript::OperatorKey;
use transaction::bytes::{Outpoint, Txid};
use transaction::taproot::Digest32;

use crate::maturity_continuity::ValidatedMaturityContinuity;
use crate::maturity_history::{
    MaturityModelledBranchRefusal, MaturityRootHistoryRefusal, ModelledBranchBlock,
    ModelledBranchPrefix, StateRootEdge, validate_state_root_history,
};
use crate::maturity_recovery::RecoveredSuccessor;

/// Compare the committed root-edge facts without comparing in-memory subtree data.
fn same_root_edge(left: &StateRootEdge, right: &StateRootEdge) -> bool {
    let left_certificate = left.certificate();
    let right_certificate = right.certificate();
    left.predecessor() == right.predecessor()
        && left.successor() == right.successor()
        && left.operation() == right.operation()
        && left_certificate.projection() == right_certificate.projection()
        && left_certificate.root() == right_certificate.root()
        && left_certificate.use_kind() == right_certificate.use_kind()
        && left_certificate.predecessor() == right_certificate.predecessor()
        && left_certificate.successor() == right_certificate.successor()
        && left_certificate.predecessor_static().root()
            == right_certificate.predecessor_static().root()
        && left_certificate.successor_static().root() == right_certificate.successor_static().root()
}

/// The semantic transition an announcement intends, apart from any branch realization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturitySemanticEdge {
    predecessor: StateMetadata,
    requested_cycle: Cycle,
    successor: StateMetadata,
}

impl MaturitySemanticEdge {
    /// Read the predecessor, request and successor from public recovery.
    #[must_use]
    pub const fn from_recovered(recovered: &RecoveredSuccessor) -> Self {
        Self {
            predecessor: recovered.predecessor_metadata().semantic,
            requested_cycle: recovered.requested_cycle(),
            successor: *recovered.successor_semantics(),
        }
    }

    /// Read the predecessor, request and successor from validated continuity.
    #[must_use]
    pub const fn from_continuity(continuity: &ValidatedMaturityContinuity) -> Self {
        Self {
            predecessor: continuity.predecessor().encoded_metadata().semantic,
            requested_cycle: continuity.requested_cycle(),
            successor: *continuity.expected_successor(),
        }
    }

    /// The predecessor's semantic metadata.
    #[must_use]
    pub const fn predecessor(&self) -> StateMetadata {
        self.predecessor
    }

    /// The witnessed announcement request.
    #[must_use]
    pub const fn requested_cycle(&self) -> Cycle {
        self.requested_cycle
    }

    /// The derived successor's semantic metadata.
    #[must_use]
    pub const fn successor(&self) -> StateMetadata {
        self.successor
    }
}

/// A caller-stated placement of one root edge in a modelled branch block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityRealization {
    height: u32,
    edge: StateRootEdge,
    operator: OperatorKey,
}

impl MaturityRealization {
    /// State the block height, edge and operator key for one realization.
    #[must_use]
    pub const fn new(height: u32, edge: StateRootEdge, operator: OperatorKey) -> Self {
        Self {
            height,
            edge,
            operator,
        }
    }

    /// The height of the block carrying this realization.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The realized STATE root edge.
    #[must_use]
    pub const fn edge(&self) -> &StateRootEdge {
        &self.edge
    }

    /// The caller-stated operator key.
    #[must_use]
    pub const fn operator(&self) -> &OperatorKey {
        &self.operator
    }
}

/// One modelled branch prefix and the root edges placed on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityBranchView {
    prefix: ModelledBranchPrefix,
    realizations: Vec<MaturityRealization>,
}

impl MaturityBranchView {
    /// Start a branch view at the supplied prefix's anchor block.
    #[must_use]
    pub const fn forked(prefix: ModelledBranchPrefix) -> Self {
        Self {
            prefix,
            realizations: Vec::new(),
        }
    }

    /// Extend the view by one block and the realizations stated for that block.
    ///
    /// Each distinct offered edge's successor transaction must appear in the block. A repeated
    /// edge has the same successor transaction and is retained in the offered order.
    ///
    /// # Errors
    /// Refuses a realization at another height or with a successor transaction absent from the
    /// block before attempting to extend the prefix. Carries the prefix's extension refusal.
    pub fn realize(
        &self,
        block: ModelledBranchBlock,
        realizations: Vec<MaturityRealization>,
    ) -> Result<Self, MaturityBranchRefusal> {
        let mut checked_edges: Vec<&StateRootEdge> = Vec::new();
        for realization in &realizations {
            if realization.height() != block.height() {
                return Err(MaturityBranchRefusal::RealizationHeightDiffers {
                    realization: realization.height(),
                    block: block.height(),
                });
            }
            if checked_edges
                .iter()
                .any(|checked| same_root_edge(checked, realization.edge()))
            {
                continue;
            }
            let transaction = realization.edge().successor().txid();
            if !block.transactions().contains(&transaction) {
                return Err(MaturityBranchRefusal::RealizationAbsentFromItsBlock {
                    height: block.height(),
                    transaction,
                });
            }
            checked_edges.push(realization.edge());
        }

        let prefix = self
            .prefix
            .extend(block)
            .map_err(MaturityBranchRefusal::Prefix)?;
        let mut extended = self.realizations.clone();
        extended.extend(realizations);
        Ok(Self {
            prefix,
            realizations: extended,
        })
    }

    /// Remove blocks above the requested height and name their realization suffix.
    ///
    /// # Errors
    /// Carries the prefix's refusal when the requested height is below its anchor or above its tip.
    pub fn rewind(
        &self,
        height: u32,
    ) -> Result<(Self, MaturityRealizationSuffixInvalidation), MaturityBranchRefusal> {
        let prefix = self
            .prefix
            .rewind(height)
            .map_err(MaturityBranchRefusal::Prefix)?;
        let (retained, invalidated) = self
            .realizations
            .iter()
            .cloned()
            .partition(|realization| realization.height() <= height);
        Ok((
            Self {
                prefix,
                realizations: retained,
            },
            MaturityRealizationSuffixInvalidation {
                branch: *self.prefix.identifier(),
                rewound_to: height,
                invalidated,
            },
        ))
    }

    /// The caller-stated modelled branch prefix.
    #[must_use]
    pub const fn prefix(&self) -> &ModelledBranchPrefix {
        &self.prefix
    }

    /// Root-edge realizations in the order they were placed.
    #[must_use]
    pub fn realizations(&self) -> &[MaturityRealization] {
        &self.realizations
    }
}

/// The exact realization suffix removed by one rewind of a branch view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityRealizationSuffixInvalidation {
    branch: Digest32,
    rewound_to: u32,
    invalidated: Vec<MaturityRealization>,
}

impl MaturityRealizationSuffixInvalidation {
    /// The branch whose view was rewound.
    #[must_use]
    pub const fn branch(&self) -> &Digest32 {
        &self.branch
    }

    /// The last retained block height.
    #[must_use]
    pub const fn rewound_to(&self) -> u32 {
        self.rewound_to
    }

    /// Removed realizations in their original order.
    #[must_use]
    pub fn invalidated(&self) -> &[MaturityRealization] {
        &self.invalidated
    }
}

/// One intended semantic step beside the root edge built to realize it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityIntendedStep {
    semantic: MaturitySemanticEdge,
    edge: StateRootEdge,
}

impl MaturityIntendedStep {
    /// The semantic transition independent of a branch placement.
    #[must_use]
    pub const fn semantic(&self) -> MaturitySemanticEdge {
        self.semantic
    }

    /// The root edge built for the intended transition.
    #[must_use]
    pub const fn edge(&self) -> &StateRootEdge {
        &self.edge
    }
}

/// An intended root-history sequence whose steps are held apart from any branch view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityThread {
    starting_cursor: Outpoint,
    operator: OperatorKey,
    steps: Vec<MaturityIntendedStep>,
}

impl MaturityThread {
    /// Start an intended history at a cursor under the stated operator key.
    #[must_use]
    pub const fn anchored(starting_cursor: Outpoint, operator: OperatorKey) -> Self {
        Self {
            starting_cursor,
            operator,
            steps: Vec::new(),
        }
    }

    /// Return a new thread with one semantic step and its constructed root edge appended.
    #[must_use]
    pub fn record(&self, semantic: MaturitySemanticEdge, edge: StateRootEdge) -> Self {
        let mut recorded = self.clone();
        recorded.steps.push(MaturityIntendedStep { semantic, edge });
        recorded
    }

    /// Read the thread's realized chain and remaining semantics on one modelled branch.
    ///
    /// Matching realizations advance a running cursor in view order. The collected root edges are
    /// checked by the Guide 14 §17.1 validator from the thread's starting cursor.
    ///
    /// # Errors
    /// Boxes a root-history refusal if the collected sequence fails validation.
    pub fn project(
        &self,
        view: &MaturityBranchView,
    ) -> Result<MaturityThreadProjection, MaturityBranchRefusal> {
        let mut cursor = self.starting_cursor;
        let mut realized = Vec::new();
        let mut collected = Vec::new();
        for realization in &view.realizations {
            let intended = self
                .steps
                .iter()
                .any(|step| same_root_edge(realization.edge(), step.edge()));
            if intended && realization.edge().predecessor() == cursor {
                cursor = realization.edge().successor();
                collected.push(realization.edge().clone());
                realized.push(realization.clone());
            }
        }
        if !collected.is_empty() {
            cursor = validate_state_root_history(&collected, self.starting_cursor).map_err(
                |refusal| MaturityBranchRefusal::ThreadSequenceRefused(Box::new(refusal)),
            )?;
        }
        let unrealized = self
            .steps
            .iter()
            .filter(|step| {
                !realized
                    .iter()
                    .any(|realization| same_root_edge(realization.edge(), step.edge()))
            })
            .map(MaturityIntendedStep::semantic)
            .collect();
        Ok(MaturityThreadProjection {
            branch: *view.prefix.identifier(),
            realized,
            cursor,
            unrealized,
        })
    }

    /// The cursor from which a branch projection starts.
    #[must_use]
    pub const fn starting_cursor(&self) -> Outpoint {
        self.starting_cursor
    }

    /// The thread's operator key.
    #[must_use]
    pub const fn operator(&self) -> &OperatorKey {
        &self.operator
    }

    /// Intended steps retained independently of any branch view.
    #[must_use]
    pub fn steps(&self) -> &[MaturityIntendedStep] {
        &self.steps
    }
}

/// A thread's realized chain and remaining semantic steps on one modelled branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityThreadProjection {
    branch: Digest32,
    realized: Vec<MaturityRealization>,
    cursor: Outpoint,
    unrealized: Vec<MaturitySemanticEdge>,
}

impl MaturityThreadProjection {
    /// The branch on which the thread was projected.
    #[must_use]
    pub const fn branch(&self) -> &Digest32 {
        &self.branch
    }

    /// Matching realizations that advance the thread's cursor.
    #[must_use]
    pub fn realized(&self) -> &[MaturityRealization] {
        &self.realized
    }

    /// The last cursor reached on this branch.
    #[must_use]
    pub const fn cursor(&self) -> Outpoint {
        self.cursor
    }

    /// Semantic steps whose root edges were not collected on this branch.
    #[must_use]
    pub fn unrealized(&self) -> &[MaturitySemanticEdge] {
        &self.unrealized
    }
}

/// Why a branch move or a thread projection was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityBranchRefusal {
    /// The modelled prefix refused extension or rewind.
    Prefix(MaturityModelledBranchRefusal),
    /// A realization named another block height.
    RealizationHeightDiffers { realization: u32, block: u32 },
    /// The block did not list the edge's successor transaction.
    RealizationAbsentFromItsBlock { height: u32, transaction: Txid },
    /// The collected realized edges did not form a valid root-history sequence.
    ThreadSequenceRefused(Box<MaturityRootHistoryRefusal>),
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;
    use crate::live_capability::OracleLiveCurve;
    use crate::maturity_closure::{MaturityWitnessSelection, OracleStateCurve, closure_target};
    use crate::maturity_continuity::tests::variable_archived;
    use crate::maturity_continuity::{
        MaturityByteSource, MaturityProjectionInput, project_maturity_continuity,
    };
    use crate::maturity_corpus::maturity_variable_run_of_record;
    use crate::maturity_history::{EdgeSide, FirstSeen};
    use crate::maturity_native::{
        MaturityAcceptanceObligation, MaturityAcceptedReadback, MaturityAnnouncementPlanner,
    };
    use crate::maturity_operator::{OPERATOR_HANDLE, OperatorVerifier};
    use tapscript::StateWitnessSchedule;
    use target_elements_conformance::executor::TargetOperationPlanner;
    use target_elements_conformance::protocol::OperationSubject;
    use transaction::bytes::{AssetField, ValueField};
    use transaction::live_request::{RequestedForm, SponsorChangeRequest};
    use transaction::operator_right::OperatorRightRegistry;
    use transaction::operator_signing::{
        OPERATOR_SIGHASH_TYPE_BYTE, OperatorSigningResponse, ScriptPathSignatureVerifier,
    };
    use transaction::state_abi::derive_maturity_announcement_abi;
    use transaction::state_construct::construct_maturity_announcement;
    use transaction::state_finalize::{
        FinalizedMaturityAnnouncement, finalize_maturity_announcement,
    };
    use transaction::state_request::MaturityAnnouncementRequest;
    use transaction::state_signing::OperatorSigningStarted;
    use transaction::state_view::{MaturityViewStatement, PublicMaturityStateView};

    fn accepted() -> &'static ValidatedMaturityContinuity {
        static ACCEPTED: LazyLock<ValidatedMaturityContinuity> = LazyLock::new(|| {
            project_maturity_continuity(variable_archived().input())
                .expect("accepted archive projects")
        });
        &ACCEPTED
    }

    fn readback() -> &'static MaturityAcceptedReadback {
        let corpus = maturity_variable_run_of_record().expect("accepted corpus");
        let MaturityAcceptanceObligation::Established { readback, .. } =
            corpus.evidence().acceptance_obligation()
        else {
            panic!("accepted archive carries a readback")
        };
        readback
    }

    fn signed_competing_bytes(input: &MaturityProjectionInput<'_>) -> Vec<u8> {
        let target = closure_target().expect("target");
        let predecessor = input
            .bundle
            .instances()
            .first()
            .expect("retained predecessor");
        let metadata = predecessor.metadata();
        let view = PublicMaturityStateView::new([
            MaturityViewStatement::CurrentStateOutpoint(input.funded.outpoint),
            MaturityViewStatement::AssetAndAmount(
                AssetField::Explicit(input.funded.asset),
                ValueField::Explicit(input.funded.amount),
            ),
            MaturityViewStatement::PredecessorMetadata(metadata.semantic),
            MaturityViewStatement::PredecessorRepresentationNonce(metadata.representation),
            MaturityViewStatement::CurrentRootBinding(input.branch),
            MaturityViewStatement::PredecessorProgram(input.funded.program.clone()),
            MaturityViewStatement::AcceptedLinkedBundle(Box::new((*input.bundle).clone())),
        ])
        .expect("competing view")
        .validate(&target, &OracleStateCurve)
        .expect("validated competing view");
        let abi = derive_maturity_announcement_abi(&target, &view).expect("competing ABI");
        let (earliest, latest) = input
            .bundle
            .deployment()
            .lead_bounds()
            .bounds()
            .window(metadata.semantic.cycle)
            .expect("announcement window");
        assert!(earliest <= latest);
        let request = MaturityAnnouncementRequest::new(
            earliest,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
        )
        .expect("earliest sponsorless request");
        let finalized: FinalizedMaturityAnnouncement = finalize_maturity_announcement(
            construct_maturity_announcement(&target, &abi, &view, &request, &OracleStateCurve)
                .expect("competing construction"),
        );

        let protected = finalized.protected_bytes().to_vec();
        let started = OperatorSigningStarted::open(
            &finalized,
            &target,
            &OracleLiveCurve::new(target.clone()),
        )
        .expect("freeze competing announcement");
        let signing_request = started.request();
        let signature = OPERATOR_HANDLE
            .material()
            .expect("operator material")
            .sign(signing_request.message().with_vector_grown(), &[0; 32])
            .expect("sign competing announcement");
        OperatorVerifier
            .verify(
                signing_request.binding().key().bytes(),
                signing_request.message().with_vector_grown(),
                &signature,
            )
            .expect("verify competing signature");
        let answer = OperatorSigningResponse::new(
            signing_request.input_index(),
            signature.to_vec(),
            OPERATOR_SIGHASH_TYPE_BYTE,
            signing_request.frozen_bytes().to_vec(),
            signing_request.binding().key().clone(),
            signing_request.binding().deployment().clone(),
            signing_request.binding().capability_revision(),
        );
        let mut registry = OperatorRightRegistry::default();
        let right = registry
            .issue(
                started.construction_right_scope(),
                started.protected_bytes(),
            )
            .expect("competing construction right");
        let authorized = started
            .authorize(&mut registry, right, [answer], &OperatorVerifier)
            .expect("authorize competing announcement");
        assert_eq!(authorized.protected_bytes(), protected);
        let ready = authorized
            .bind_for_submission(&target)
            .expect("competing submit ready");
        assert_eq!(ready.protected_bytes(), protected);
        assert_ne!(ready.bytes(), protected);
        ready.bytes().to_vec()
    }

    fn competing() -> &'static ValidatedMaturityContinuity {
        static COMPETING: LazyLock<ValidatedMaturityContinuity> = LazyLock::new(|| {
            let input = variable_archived().input();
            let bytes = signed_competing_bytes(&input);
            project_maturity_continuity(MaturityProjectionInput {
                source: MaturityByteSource::NodeFreeSubmitReady,
                submitted_bytes: &bytes,
                funded: input.funded,
                branch: input.branch,
                bundle: input.bundle,
                identity: input.identity,
            })
            .expect("signed earliest-cycle candidate projects")
        });
        &COMPETING
    }

    fn forked_views(height: u32) -> (MaturityBranchView, MaturityBranchView) {
        let anchor_height = height.checked_sub(1).expect("earlier anchor height");
        assert_eq!(anchor_height, 4);
        let anchor = ModelledBranchBlock::new(anchor_height, [0x60; 32], Vec::new())
            .expect("modelled anchor");
        let a = ModelledBranchPrefix::anchored([0xa1; 32], anchor.clone()).expect("branch A");
        let b = ModelledBranchPrefix::anchored([0xb1; 32], anchor).expect("branch B");
        (MaturityBranchView::forked(a), MaturityBranchView::forked(b))
    }

    fn accepted_block(edge: &StateRootEdge) -> ModelledBranchBlock {
        let readback = readback();
        assert_eq!(readback.block_height(), 5);
        assert_eq!(edge.successor().txid(), readback.identity());
        ModelledBranchBlock::new(
            readback.block_height(),
            *readback.block_hash(),
            vec![readback.identity()],
        )
        .expect("accepted block")
    }

    fn realization(
        height: u32,
        edge: &StateRootEdge,
        continuity: &ValidatedMaturityContinuity,
    ) -> MaturityRealization {
        MaturityRealization::new(
            height,
            edge.clone(),
            continuity.bundle().deployment().operator().key().clone(),
        )
    }

    #[test]
    fn a_fork_after_one_predecessor_realizes_competing_successors_on_their_own_branches() {
        let accepted_edge = StateRootEdge::from_continuity(accepted());
        let competing_edge = StateRootEdge::from_continuity(competing());
        let accepted_semantic = MaturitySemanticEdge::from_continuity(accepted());
        let competing_semantic = MaturitySemanticEdge::from_continuity(competing());
        assert_eq!(accepted_edge.predecessor(), competing_edge.predecessor());
        assert_eq!(
            accepted_semantic.predecessor(),
            competing_semantic.predecessor()
        );
        assert_ne!(
            accepted_semantic.requested_cycle(),
            competing_semantic.requested_cycle()
        );
        assert_ne!(
            accepted_semantic.successor(),
            competing_semantic.successor()
        );
        assert_ne!(accepted_edge.successor(), competing_edge.successor());

        let height = readback().block_height();
        let (a, b) = forked_views(height);
        let accepted_realization = realization(height, &accepted_edge, accepted());
        let competing_realization = realization(height, &competing_edge, competing());
        let a = a
            .realize(accepted_block(&accepted_edge), vec![accepted_realization])
            .expect("A realizes accepted edge");
        let competing_block =
            ModelledBranchBlock::new(height, [0xb2; 32], vec![competing_edge.successor().txid()])
                .expect("competing block");
        let b = b
            .realize(competing_block, vec![competing_realization])
            .expect("B realizes competing edge");
        assert_eq!(a.prefix().identifier(), &[0xa1; 32]);
        assert_eq!(b.prefix().identifier(), &[0xb1; 32]);
        assert_eq!(a.prefix().anchor_height(), 4);
        assert_eq!(b.prefix().anchor_height(), 4);

        let accepted_thread = MaturityThread::anchored(
            accepted_edge.predecessor(),
            accepted().bundle().deployment().operator().key().clone(),
        )
        .record(accepted_semantic, accepted_edge.clone());
        let on_a = accepted_thread.project(&a).expect("accepted edge on A");
        assert_eq!(on_a.branch(), a.prefix().identifier());
        assert_eq!(on_a.cursor(), accepted_edge.successor());
        assert_eq!(on_a.realized().len(), 1);
        assert!(same_root_edge(
            on_a.realized()
                .first()
                .expect("accepted realization")
                .edge(),
            &accepted_edge
        ));
        assert_eq!(on_a.unrealized().len(), 0);

        let on_b = accepted_thread.project(&b).expect("accepted thread on B");
        assert_eq!(on_b.branch(), b.prefix().identifier());
        assert_eq!(on_b.cursor(), accepted_thread.starting_cursor());
        assert_eq!(on_b.realized().len(), 0);
        assert_eq!(on_b.unrealized(), &[accepted_semantic]);

        let competing_thread = MaturityThread::anchored(
            competing_edge.predecessor(),
            competing().bundle().deployment().operator().key().clone(),
        )
        .record(competing_semantic, competing_edge.clone());
        let competing_on_b = competing_thread.project(&b).expect("competing edge on B");
        assert_eq!(competing_on_b.cursor(), competing_edge.successor());
        assert_eq!(competing_on_b.realized().len(), 1);
        assert_eq!(competing_on_b.unrealized().len(), 0);

        assert_eq!(
            validate_state_root_history(
                &[accepted_edge.clone(), competing_edge],
                accepted_edge.predecessor(),
            ),
            Err(MaturityRootHistoryRefusal::OutpointReused {
                position: 1,
                outpoint: accepted_edge.predecessor(),
                first_seen: FirstSeen::Edge {
                    position: 0,
                    side: EdgeSide::Predecessor,
                },
            })
        );
    }

    #[test]
    fn rewinding_one_branch_invalidates_only_its_realization_suffix_and_keeps_semantic_history() {
        let accepted_edge = StateRootEdge::from_continuity(accepted());
        let competing_edge = StateRootEdge::from_continuity(competing());
        let semantic = MaturitySemanticEdge::from_continuity(accepted());
        let height = readback().block_height();
        let anchor_height = height.checked_sub(1).expect("anchor height");
        let next_height = height.checked_add(1).expect("empty successor height");
        let (a, b) = forked_views(height);
        let accepted_realization = realization(height, &accepted_edge, accepted());
        let a = a
            .realize(
                accepted_block(&accepted_edge),
                vec![accepted_realization.clone()],
            )
            .expect("accepted edge on A");
        let b = b
            .realize(
                ModelledBranchBlock::new(
                    height,
                    [0xb2; 32],
                    vec![competing_edge.successor().txid()],
                )
                .expect("competing block"),
                vec![realization(height, &competing_edge, competing())],
            )
            .expect("competing edge on B");
        let b_before = (
            *b.prefix().identifier(),
            b.prefix().tip_height(),
            b.realizations().len(),
        );
        let thread = MaturityThread::anchored(
            accepted_edge.predecessor(),
            accepted().bundle().deployment().operator().key().clone(),
        )
        .record(semantic, accepted_edge);
        let steps_before = thread.steps().to_vec();
        let extended = a
            .realize(
                ModelledBranchBlock::new(next_height, [0x61; 32], Vec::new())
                    .expect("empty successor block"),
                Vec::new(),
            )
            .expect("A extends by an empty block");
        let (rewound, event) = extended.rewind(anchor_height).expect("rewind A to anchor");

        assert_eq!(event.branch(), &[0xa1; 32]);
        assert_eq!(event.rewound_to(), anchor_height);
        assert_eq!(event.invalidated().len(), 1);
        let removed = event.invalidated().first().expect("one removed edge");
        assert_eq!(removed.height(), accepted_realization.height());
        assert_eq!(removed.operator(), accepted_realization.operator());
        assert!(same_root_edge(removed.edge(), accepted_realization.edge()));
        assert_eq!(rewound.prefix().tip_height(), anchor_height);
        assert_eq!(rewound.realizations().len(), 0);

        assert_eq!(
            b_before,
            (
                *b.prefix().identifier(),
                b.prefix().tip_height(),
                b.realizations().len(),
            )
        );
        let b_after_edge = b.realizations().first().expect("B realization");
        assert_eq!(b_after_edge.height(), height);
        assert_eq!(
            b_after_edge.operator(),
            competing().bundle().deployment().operator().key()
        );
        assert!(same_root_edge(b_after_edge.edge(), &competing_edge));
        assert_eq!(thread.steps().len(), steps_before.len());
        for (step, original) in thread.steps().iter().zip(&steps_before) {
            assert_eq!(step.semantic(), original.semantic());
            assert!(same_root_edge(step.edge(), original.edge()));
        }

        let projected = thread.project(&rewound).expect("thread on rewound A");
        assert_eq!(projected.cursor(), thread.starting_cursor());
        assert_eq!(projected.realized().len(), 0);
        assert_eq!(projected.unrealized(), &[semantic]);
        let above_tip = next_height.checked_add(1).expect("above tip");
        assert_eq!(
            extended.rewind(above_tip),
            Err(MaturityBranchRefusal::Prefix(
                MaturityModelledBranchRefusal::RewoundAboveTheTip {
                    tip: next_height,
                    requested: above_tip,
                }
            ))
        );
    }

    #[test]
    fn re_realizing_the_same_block_reproduces_identical_bytes_and_an_identical_view() {
        let corpus = maturity_variable_run_of_record().expect("accepted corpus");
        let identity = corpus.evidence().identity().clone();
        let branch = corpus.evidence().branch();
        let mut planner = MaturityAnnouncementPlanner::new(
            identity.clone(),
            branch,
            MaturityWitnessSelection::Retained(StateWitnessSchedule::VariableMetadata),
        )
        .expect("fresh variable planner");
        assert_eq!(corpus.exchanges().len(), 3);
        let mut next = planner.next_step(None).expect("issue");
        for (step, response) in corpus.exchanges().iter().take(2) {
            assert_eq!(next.as_ref(), Some(step));
            next = planner
                .next_step(Some((step.case(), response)))
                .expect("replay accepted exchange");
        }
        let (submission_step, _) = corpus.exchanges().get(2).expect("third exchange");
        assert_eq!(next.as_ref(), Some(submission_step));
        let OperationSubject::Submission(submission) = submission_step.subject() else {
            panic!("third exchange submits the announcement")
        };
        let replayed_bytes = planner
            .submission_bytes()
            .expect("replayed submission bytes");
        assert_eq!(replayed_bytes, submission.transaction_bytes.as_slice());
        assert_eq!(replayed_bytes, readback().bytes());

        let source = variable_archived().input();
        let replayed = project_maturity_continuity(MaturityProjectionInput {
            source: MaturityByteSource::NodeFreeSubmitReady,
            submitted_bytes: replayed_bytes,
            funded: source.funded,
            branch,
            bundle: planner.bundle(),
            identity: &identity,
        })
        .expect("replayed bytes project");
        let original_edge = StateRootEdge::from_continuity(accepted());
        let replayed_edge = StateRootEdge::from_continuity(&replayed);
        assert_eq!(replayed_edge, original_edge);

        let height = readback().block_height();
        let anchor_height = height.checked_sub(1).expect("anchor height");
        let (a, _) = forked_views(height);
        let block = accepted_block(&original_edge);
        let realized = a
            .realize(
                block.clone(),
                vec![realization(height, &original_edge, accepted())],
            )
            .expect("original A realization");
        let (rewound, event) = realized.rewind(anchor_height).expect("rewound A");
        assert_eq!(event.invalidated().len(), 1);
        assert_eq!(
            replayed.bundle().deployment().operator().key(),
            accepted().bundle().deployment().operator().key()
        );
        let reproduced = rewound
            .realize(block, vec![realization(height, &replayed_edge, &replayed)])
            .expect("reproduced A realization");
        assert_eq!(reproduced, realized);
    }

    #[test]
    fn a_realization_outside_its_block_refuses_before_the_prefix_moves() {
        let edge = StateRootEdge::from_continuity(accepted());
        let height = readback().block_height();
        let next_height = height.checked_add(1).expect("next height");
        let (view, _) = forked_views(height);
        let original = view.clone();
        let key = accepted().bundle().deployment().operator().key().clone();
        assert_eq!(
            view.realize(
                accepted_block(&edge),
                vec![MaturityRealization::new(
                    next_height,
                    edge.clone(),
                    key.clone()
                )],
            ),
            Err(MaturityBranchRefusal::RealizationHeightDiffers {
                realization: next_height,
                block: height,
            })
        );
        assert_eq!(view.prefix(), original.prefix());
        assert_eq!(view.realizations().len(), 0);

        let without_transaction =
            ModelledBranchBlock::new(height, *readback().block_hash(), Vec::new())
                .expect("block without successor transaction");
        assert_eq!(
            view.realize(
                without_transaction,
                vec![MaturityRealization::new(height, edge.clone(), key)],
            ),
            Err(MaturityBranchRefusal::RealizationAbsentFromItsBlock {
                height,
                transaction: edge.successor().txid(),
            })
        );
        assert_eq!(view.prefix(), original.prefix());
        assert_eq!(view.realizations().len(), 0);

        let skipped =
            ModelledBranchBlock::new(next_height, [0x62; 32], Vec::new()).expect("skipped block");
        assert_eq!(
            view.realize(skipped, Vec::new()),
            Err(MaturityBranchRefusal::Prefix(
                MaturityModelledBranchRefusal::NonContiguousExtension {
                    tip: view.prefix().tip_height(),
                    offered: next_height,
                }
            ))
        );
        assert_eq!(view.prefix(), original.prefix());
        assert_eq!(view.realizations().len(), 0);
    }
}
