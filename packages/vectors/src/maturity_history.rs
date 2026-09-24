//! Typed STATE root-history edges under Guide 14 §§14.6, 17.1, and 17.3 (`T11-114`).

use std::collections::BTreeMap;

use architecture::ids::{DeallocatorId, ObjectId, OperationId, ProjectionId, RootId, RootUse};
use architecture::spec::ARCHITECTURE;
use linker::CandidateLinkedMaturityBundle;
use linker::{StateLinkRefusal, state_bundle_continuity};
use realization::AnnouncementLeadBounds;
use tapscript::{StateInternalKeyPolicy, StateStaticSubtree, StateWitnessSchedule};
use target_elements::TargetContractVersion;
use target_elements_conformance::constructor::tagged::sha256;
use transaction::bytes::{Outpoint, TargetTransaction, Txid};
use transaction::operator_right::BranchContext;
use transaction::state_abi::CandidateMaturityAnnouncementAbi;
use transaction::taproot::Digest32;

use crate::maturity_continuity::{ValidatedMaturityContinuity, submitted_transaction_identities};
use crate::maturity_corpus::{
    MaturityDeclaredPremise, MaturityPremiseProvenance, ValidatedMaturityCorpus,
};
use crate::maturity_native::{MaturityAcceptanceObligation, MaturityAcceptanceRoute};
use crate::maturity_recovery::{
    MaturityRecoveryRefusal, PublicAnnouncementHandoff, recover_public_successor,
};

/// A transition certificate projected from a validated announcement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityTransitionCertificate {
    projection: ProjectionId,
    operation: OperationId,
    root: RootId,
    use_kind: RootUse,
    predecessor: Outpoint,
    successor: Outpoint,
    predecessor_static: StateStaticSubtree,
    successor_static: StateStaticSubtree,
}

impl MaturityTransitionCertificate {
    /// Bind the certificate endpoints to the validated target transaction. The certificate's
    /// agreement with that transaction is by construction: callers cannot supply its fields.
    ///
    /// # Panics
    /// Panics only if output index zero is outside the target's outpoint range, which zero cannot
    /// arrange.
    #[must_use]
    pub fn from_continuity(continuity: &ValidatedMaturityContinuity) -> Self {
        let transaction = continuity.transaction();
        let identity = submitted_transaction_identities(transaction, continuity.submitted_bytes());
        let Ok(successor) = Outpoint::new(identity.identity(), 0) else {
            unreachable!("zero is a valid output index");
        };
        Self {
            projection: ProjectionId::TransitionCertificate,
            operation: OperationId::AnnounceMaturity,
            root: RootId::State,
            use_kind: RootUse::Succession,
            predecessor: continuity.funded().outpoint,
            successor,
            predecessor_static: continuity.predecessor().static_subtree().clone(),
            successor_static: continuity.successor().static_subtree().clone(),
        }
    }

    #[must_use]
    pub const fn projection(&self) -> ProjectionId {
        self.projection
    }

    #[must_use]
    pub const fn operation(&self) -> OperationId {
        self.operation
    }

    #[must_use]
    pub const fn root(&self) -> RootId {
        self.root
    }

    #[must_use]
    pub const fn use_kind(&self) -> RootUse {
        self.use_kind
    }

    #[must_use]
    pub const fn predecessor(&self) -> Outpoint {
        self.predecessor
    }

    #[must_use]
    pub const fn successor(&self) -> Outpoint {
        self.successor
    }

    #[must_use]
    pub const fn predecessor_static(&self) -> &StateStaticSubtree {
        &self.predecessor_static
    }

    #[must_use]
    pub const fn successor_static(&self) -> &StateStaticSubtree {
        &self.successor_static
    }
}

/// One edge in an ordered STATE root-history sequence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateRootEdge {
    predecessor: Outpoint,
    successor: Outpoint,
    operation: OperationId,
    certificate: MaturityTransitionCertificate,
}

impl StateRootEdge {
    /// Project the edge and its certificate from validated continuity.
    #[must_use]
    pub fn from_continuity(continuity: &ValidatedMaturityContinuity) -> Self {
        let certificate = MaturityTransitionCertificate::from_continuity(continuity);
        Self {
            predecessor: certificate.predecessor(),
            successor: certificate.successor(),
            operation: certificate.operation(),
            certificate,
        }
    }

    /// Build the STATE root edge from one public announcement handoff.
    ///
    /// # Errors
    /// Refuses exactly when public recovery refuses the handoff.
    pub fn from_public_handoff(
        handoff: PublicAnnouncementHandoff,
    ) -> Result<Self, MaturityRecoveryRefusal> {
        let static_subtree = handoff.static_subtree().clone();
        let recovered = recover_public_successor(handoff)?;
        let certificate = MaturityTransitionCertificate {
            projection: ProjectionId::TransitionCertificate,
            operation: OperationId::AnnounceMaturity,
            root: RootId::State,
            use_kind: RootUse::Succession,
            predecessor: recovered.predecessor(),
            successor: recovered.successor(),
            predecessor_static: static_subtree.clone(),
            successor_static: static_subtree,
        };
        Ok(Self {
            predecessor: certificate.predecessor(),
            successor: certificate.successor(),
            operation: certificate.operation(),
            certificate,
        })
    }

    #[must_use]
    pub const fn predecessor(&self) -> Outpoint {
        self.predecessor
    }

    #[must_use]
    pub const fn successor(&self) -> Outpoint {
        self.successor
    }

    #[must_use]
    pub const fn operation(&self) -> OperationId {
        self.operation
    }

    #[must_use]
    pub const fn certificate(&self) -> &MaturityTransitionCertificate {
        &self.certificate
    }
}

/// The two endpoints of an edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeSide {
    Predecessor,
    Successor,
}

/// Where an outpoint was first seen in the sequence check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstSeen {
    StartingCursor,
    Edge { position: usize, side: EdgeSide },
}

/// The six ordered clauses of Guide 14 §17.1 and the §17.3 continuity rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryClause {
    PredecessorIsCursor,
    SuccessorIsUnique,
    CertificateOperation,
    CertificateAgreesWithTransaction,
    NoUnrelatedRootEffect,
    CursorAdvancesAfterValidation,
    BundleContinuity,
}

impl HistoryClause {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BundleContinuity => "rule:guide14-exec:bundle-continuity",
            _ => "rule:guide14-exec:edge-sequence",
        }
    }
}

/// Why a supplied STATE root-history sequence was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityRootHistoryRefusal {
    /// §17.1 expects one edge for one announcement. With no validated edge, no cursor can advance.
    EmptySequence,
    OutpointReused {
        position: usize,
        outpoint: Outpoint,
        first_seen: FirstSeen,
    },
    PredecessorIsNotTheCursor {
        position: usize,
        cursor: Outpoint,
        predecessor: Outpoint,
    },
    OperationIsNotMaturityAnnouncement {
        position: usize,
        operation: OperationId,
    },
    CertificateProjectionDisagrees {
        position: usize,
        projection: ProjectionId,
    },
    CertificateDisagreesWithEdge {
        position: usize,
        side: EdgeSide,
        edge: Outpoint,
        certificate: Outpoint,
    },
    UnrelatedRootEffect {
        position: usize,
        root: RootId,
        use_kind: RootUse,
        successor: Outpoint,
    },
    StaticSubtreeDiscontinuity {
        position: usize,
        refusal: StateLinkRefusal,
    },
}

impl MaturityRootHistoryRefusal {
    #[must_use]
    pub const fn clause(&self) -> HistoryClause {
        match self {
            Self::EmptySequence => HistoryClause::CursorAdvancesAfterValidation,
            Self::OutpointReused { .. } => HistoryClause::SuccessorIsUnique,
            Self::PredecessorIsNotTheCursor { .. } => HistoryClause::PredecessorIsCursor,
            Self::OperationIsNotMaturityAnnouncement { .. }
            | Self::CertificateProjectionDisagrees { .. } => HistoryClause::CertificateOperation,
            Self::CertificateDisagreesWithEdge { .. } => {
                HistoryClause::CertificateAgreesWithTransaction
            }
            Self::UnrelatedRootEffect { .. } => HistoryClause::NoUnrelatedRootEffect,
            Self::StaticSubtreeDiscontinuity { .. } => HistoryClause::BundleContinuity,
        }
    }

    #[must_use]
    pub const fn position(&self) -> Option<usize> {
        match self {
            Self::EmptySequence => None,
            Self::OutpointReused { position, .. }
            | Self::PredecessorIsNotTheCursor { position, .. }
            | Self::OperationIsNotMaturityAnnouncement { position, .. }
            | Self::CertificateProjectionDisagrees { position, .. }
            | Self::CertificateDisagreesWithEdge { position, .. }
            | Self::UnrelatedRootEffect { position, .. }
            | Self::StaticSubtreeDiscontinuity { position, .. } => Some(*position),
        }
    }
}

/// Validate an ordered sequence from a stated root cursor.
///
/// Uniqueness is checked over the whole sequence before walking it. Each edge then follows
/// Guide 14 §17.1's clauses in order. Clause five reads the operation's root and output
/// declarations; a termination-capable use is refused for STATE because its object declares
/// no deallocator. Guide 14 §17.3 is applied by the linker's existing continuity rule.
///
/// # Errors
/// Returns the first sequence or edge refusal.
#[expect(
    clippy::too_many_lines,
    reason = "The clause order remains visible in one function."
)]
pub fn validate_state_root_history(
    edges: &[StateRootEdge],
    cursor: Outpoint,
) -> Result<Outpoint, MaturityRootHistoryRefusal> {
    if edges.is_empty() {
        return Err(MaturityRootHistoryRefusal::EmptySequence);
    }

    let mut predecessors = BTreeMap::new();
    let mut successors = BTreeMap::new();
    for (position, edge) in edges.iter().enumerate() {
        if let Some(first_seen) = predecessors.insert(
            edge.predecessor,
            FirstSeen::Edge {
                position,
                side: EdgeSide::Predecessor,
            },
        ) {
            return Err(MaturityRootHistoryRefusal::OutpointReused {
                position,
                outpoint: edge.predecessor,
                first_seen,
            });
        }
        if edge.successor == cursor {
            return Err(MaturityRootHistoryRefusal::OutpointReused {
                position,
                outpoint: edge.successor,
                first_seen: FirstSeen::StartingCursor,
            });
        }
        if let Some(first_seen) = successors.insert(
            edge.successor,
            FirstSeen::Edge {
                position,
                side: EdgeSide::Successor,
            },
        ) {
            return Err(MaturityRootHistoryRefusal::OutpointReused {
                position,
                outpoint: edge.successor,
                first_seen,
            });
        }
    }

    let mut current = cursor;
    for (position, edge) in edges.iter().enumerate() {
        if edge.predecessor != current {
            return Err(MaturityRootHistoryRefusal::PredecessorIsNotTheCursor {
                position,
                cursor: current,
                predecessor: edge.predecessor,
            });
        }

        let certificate = &edge.certificate;
        if edge.operation != OperationId::AnnounceMaturity {
            return Err(
                MaturityRootHistoryRefusal::OperationIsNotMaturityAnnouncement {
                    position,
                    operation: edge.operation,
                },
            );
        }
        if certificate.operation != edge.operation {
            return Err(
                MaturityRootHistoryRefusal::OperationIsNotMaturityAnnouncement {
                    position,
                    operation: certificate.operation,
                },
            );
        }
        if certificate.projection != ProjectionId::TransitionCertificate {
            return Err(MaturityRootHistoryRefusal::CertificateProjectionDisagrees {
                position,
                projection: certificate.projection,
            });
        }

        if certificate.predecessor != edge.predecessor {
            return Err(MaturityRootHistoryRefusal::CertificateDisagreesWithEdge {
                position,
                side: EdgeSide::Predecessor,
                edge: edge.predecessor,
                certificate: certificate.predecessor,
            });
        }
        if certificate.successor != edge.successor {
            return Err(MaturityRootHistoryRefusal::CertificateDisagreesWithEdge {
                position,
                side: EdgeSide::Successor,
                edge: edge.successor,
                certificate: certificate.successor,
            });
        }

        let Some(operation_spec) = ARCHITECTURE.operation(edge.operation) else {
            return Err(MaturityRootHistoryRefusal::UnrelatedRootEffect {
                position,
                root: certificate.root,
                use_kind: certificate.use_kind,
                successor: certificate.successor,
            });
        };
        let declared_root = operation_spec
            .roots
            .iter()
            .any(|root| root.root == certificate.root && root.use_kind == certificate.use_kind);
        let state_output_index = operation_spec
            .outputs
            .iter()
            .position(|output| output.object == ObjectId::State);
        let successor_index = usize::try_from(certificate.successor.index()).ok();
        let state_has_no_deallocator = ARCHITECTURE.object(ObjectId::State).is_some_and(|object| {
            object
                .deallocators
                .iter()
                .all(|deallocator| *deallocator == DeallocatorId::None)
        });
        if !declared_root
            || (certificate.use_kind == RootUse::SuccessionOrTermination
                && state_has_no_deallocator)
            || state_output_index != successor_index
        {
            return Err(MaturityRootHistoryRefusal::UnrelatedRootEffect {
                position,
                root: certificate.root,
                use_kind: certificate.use_kind,
                successor: certificate.successor,
            });
        }

        state_bundle_continuity(
            &certificate.predecessor_static,
            &certificate.successor_static,
        )
        .map_err(
            |refusal| MaturityRootHistoryRefusal::StaticSubtreeDiscontinuity { position, refusal },
        )?;
        current = edge.successor;
    }
    Ok(current)
}

/// The ten facts named by Guide 14 §17.2 (`rule:guide14-exec:checkpoint`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityRootFact {
    NetworkIdentity,
    GenesisIdentity,
    BlockHash,
    BlockHeight,
    TransactionIdentity,
    PredecessorOutpoint,
    SuccessorOutpoint,
    TargetContract,
    LinkedCandidate,
    CandidateAbi,
}

/// How a checkpoint obtains a fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityRootFactKind {
    CarriedByTheAdmittedArchive,
    ReadFromThePublicBytes,
    DerivedFromThePublicBytesAndTheLocator,
    DeclaredPremise(MaturityPremiseProvenance),
}

impl MaturityRootFact {
    /// The checkpoint facts in Guide 14 §17.2 order.
    pub const ALL: &'static [Self; 10] = &[
        Self::NetworkIdentity,
        Self::GenesisIdentity,
        Self::BlockHash,
        Self::BlockHeight,
        Self::TransactionIdentity,
        Self::PredecessorOutpoint,
        Self::SuccessorOutpoint,
        Self::TargetContract,
        Self::LinkedCandidate,
        Self::CandidateAbi,
    ];

    /// Distinguish admitted facts, public-byte facts and deployment declarations.
    #[must_use]
    pub const fn kind(self) -> MaturityRootFactKind {
        match self {
            Self::NetworkIdentity
            | Self::GenesisIdentity
            | Self::BlockHash
            | Self::BlockHeight
            | Self::TransactionIdentity
            | Self::TargetContract => MaturityRootFactKind::CarriedByTheAdmittedArchive,
            Self::PredecessorOutpoint => MaturityRootFactKind::ReadFromThePublicBytes,
            Self::SuccessorOutpoint => MaturityRootFactKind::DerivedFromThePublicBytesAndTheLocator,
            Self::LinkedCandidate | Self::CandidateAbi => MaturityRootFactKind::DeclaredPremise(
                MaturityPremiseProvenance::DeploymentDeclaration,
            ),
        }
    }
}

/// A root checkpoint over the admitted announcement and its stated branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityRootCheckpoint {
    network_identity: String,
    genesis_identity: String,
    block_hash: Digest32,
    block_height: u32,
    transaction_identity: Txid,
    predecessor: Outpoint,
    successor: Outpoint,
    target_contract: String,
    linked_candidate: MaturityDeclaredPremise<CandidateLinkedMaturityBundle>,
    candidate_abi: MaturityDeclaredPremise<CandidateMaturityAnnouncementAbi>,
    branch: BranchContext,
}

/// A disagreement found while binding the admitted checkpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityRootCheckpointRefusal {
    AcceptanceIsOutstanding {
        routes: [MaturityAcceptanceRoute; 2],
    },
    TransactionIdentityDisagrees {
        checkpoint: Txid,
        handoff: Txid,
    },
    SubmittedBytesDisagree {
        checkpoint: Digest32,
        handoff: Digest32,
    },
    HandoffBytesUndecodable {
        offered: usize,
    },
    SuccessorOutpointDisagrees {
        checkpoint: Outpoint,
        edge: Outpoint,
    },
    SuccessorIsNotTheHandoffsStateOutput {
        checkpoint: Outpoint,
        identity: Txid,
        index: u32,
    },
    HandoffCarriesNoInput,
    PredecessorOutpointDisagrees {
        checkpoint: Outpoint,
        handoff: Outpoint,
    },
    LinkedCandidateDisagrees(MaturityPremiseDisagreement),
    CandidateAbiDisagrees(MaturityPremiseDisagreement),
}

/// The first published parameter on which a declared premise disagrees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityPremiseDisagreement {
    WitnessSchedule {
        declared: StateWitnessSchedule,
        published: StateWitnessSchedule,
    },
    LeadBounds {
        declared: AnnouncementLeadBounds,
        published: AnnouncementLeadBounds,
    },
    StaticRoot {
        declared: Digest32,
        published: Digest32,
    },
    InternalKey {
        declared: StateInternalKeyPolicy,
        published: StateInternalKeyPolicy,
    },
    ContractRevision {
        declared: TargetContractVersion,
        published: TargetContractVersion,
    },
}

impl MaturityRootCheckpointRefusal {
    /// The disagreeing fact, when acceptance supplied enough evidence to name one.
    #[must_use]
    pub const fn fact(&self) -> Option<MaturityRootFact> {
        match self {
            Self::AcceptanceIsOutstanding { .. } => None,
            Self::TransactionIdentityDisagrees { .. }
            | Self::SubmittedBytesDisagree { .. }
            | Self::HandoffBytesUndecodable { .. } => Some(MaturityRootFact::TransactionIdentity),
            Self::SuccessorOutpointDisagrees { .. }
            | Self::SuccessorIsNotTheHandoffsStateOutput { .. } => {
                Some(MaturityRootFact::SuccessorOutpoint)
            }
            Self::HandoffCarriesNoInput | Self::PredecessorOutpointDisagrees { .. } => {
                Some(MaturityRootFact::PredecessorOutpoint)
            }
            Self::LinkedCandidateDisagrees(..) => Some(MaturityRootFact::LinkedCandidate),
            Self::CandidateAbiDisagrees(..) => Some(MaturityRootFact::CandidateAbi),
        }
    }
}

/// One caller-supplied block in a modelled branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelledBranchBlock {
    height: u32,
    identity: Digest32,
    transactions: Vec<Txid>,
}

/// A modelled branch window identified by the caller's branch identifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelledBranchPrefix {
    identifier: Digest32,
    anchor_height: u32,
    blocks: Vec<ModelledBranchBlock>,
}

/// A checkpoint projected against a modelled prefix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityRootObservation {
    prefix: Digest32,
    context: BranchContext,
    block_height: u32,
    block_identity: Digest32,
    transaction: Txid,
}

/// A structural refusal while constructing or moving a modelled branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityModelledBranchRefusal {
    ZeroPrefixIdentifier { anchor: u32 },
    ZeroBlockIdentity { height: u32 },
    NonContiguousExtension { tip: u32, offered: u32 },
    RewoundBelowTheAnchor { anchor: u32, requested: u32 },
    RewoundAboveTheTip { tip: u32, requested: u32 },
}

/// A checkpoint observation that a modelled prefix no longer supports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityRootObservationRefusal {
    EvidenceFromAnotherPrefix {
        presented: Digest32,
        prefix: Digest32,
    },
    TransactionBlockIsNotInThePrefix {
        height: u32,
        anchor: u32,
        tip: u32,
    },
    BlockAtThatHeightChanged {
        height: u32,
        prefix: Digest32,
        checkpoint: Digest32,
    },
    TransactionAbsentFromThatBlock {
        height: u32,
        transaction: Txid,
        block: Digest32,
    },
}

impl MaturityRootCheckpoint {
    /// Bind the admitted readback and its ten checkpoint facts to a public handoff and edge.
    ///
    /// # Errors
    /// Refuses an outstanding acceptance or the first digest, identity, outpoint or premise
    /// disagreement.
    pub fn bind(
        edge: &StateRootEdge,
        handoff: &PublicAnnouncementHandoff,
        corpus: &ValidatedMaturityCorpus,
        linked_candidate: MaturityDeclaredPremise<CandidateLinkedMaturityBundle>,
        candidate_abi: MaturityDeclaredPremise<CandidateMaturityAnnouncementAbi>,
    ) -> Result<Self, MaturityRootCheckpointRefusal> {
        let readback = match corpus.evidence().acceptance_obligation() {
            MaturityAcceptanceObligation::Outstanding { routes } => {
                return Err(MaturityRootCheckpointRefusal::AcceptanceIsOutstanding {
                    routes: *routes,
                });
            }
            MaturityAcceptanceObligation::Established { readback, .. } => readback,
        };
        let checkpoint_digest = sha256(readback.bytes());
        let handoff_digest = sha256(handoff.bytes());
        if checkpoint_digest != handoff_digest {
            return Err(MaturityRootCheckpointRefusal::SubmittedBytesDisagree {
                checkpoint: checkpoint_digest,
                handoff: handoff_digest,
            });
        }
        let checkpoint = Self {
            network_identity: corpus.report().network_id().to_owned(),
            genesis_identity: corpus.report().genesis_id().to_owned(),
            block_hash: *readback.block_hash(),
            block_height: readback.block_height(),
            transaction_identity: readback.identity(),
            predecessor: edge.predecessor(),
            successor: edge.successor(),
            target_contract: corpus.report().target_contract().to_owned(),
            linked_candidate,
            candidate_abi,
            branch: corpus.evidence().branch(),
        };
        checkpoint.binds(edge, handoff)?;
        Ok(checkpoint)
    }

    /// Compare the checkpoint with the handoff's decoded bytes and published parameters.
    ///
    /// # Errors
    /// Returns the first undecodable bytes, identity, successor, predecessor, linked candidate
    /// or ABI disagreement, in that order.
    pub fn binds(
        &self,
        edge: &StateRootEdge,
        handoff: &PublicAnnouncementHandoff,
    ) -> Result<(), MaturityRootCheckpointRefusal> {
        let transaction = TargetTransaction::decode(handoff.bytes()).map_err(|_| {
            MaturityRootCheckpointRefusal::HandoffBytesUndecodable {
                offered: handoff.bytes().len(),
            }
        })?;
        let identity = submitted_transaction_identities(&transaction, handoff.bytes()).identity();
        if self.transaction_identity != identity {
            return Err(
                MaturityRootCheckpointRefusal::TransactionIdentityDisagrees {
                    checkpoint: self.transaction_identity,
                    handoff: identity,
                },
            );
        }
        if self.successor != edge.successor() {
            return Err(MaturityRootCheckpointRefusal::SuccessorOutpointDisagrees {
                checkpoint: self.successor,
                edge: edge.successor(),
            });
        }
        if self.successor.txid() != identity
            || self.successor.index() != handoff.state_output_index()
        {
            return Err(
                MaturityRootCheckpointRefusal::SuccessorIsNotTheHandoffsStateOutput {
                    checkpoint: self.successor,
                    identity,
                    index: handoff.state_output_index(),
                },
            );
        }
        let predecessor = transaction
            .inputs()
            .first()
            .ok_or(MaturityRootCheckpointRefusal::HandoffCarriesNoInput)?
            .outpoint();
        if self.predecessor != predecessor {
            return Err(
                MaturityRootCheckpointRefusal::PredecessorOutpointDisagrees {
                    checkpoint: self.predecessor,
                    handoff: predecessor,
                },
            );
        }
        if let Some(disagreement) =
            linked_candidate_disagreement(self.linked_candidate.value(), handoff)
        {
            return Err(MaturityRootCheckpointRefusal::LinkedCandidateDisagrees(
                disagreement,
            ));
        }
        if let Some(disagreement) = candidate_abi_disagreement(self.candidate_abi.value(), handoff)
        {
            return Err(MaturityRootCheckpointRefusal::CandidateAbiDisagrees(
                disagreement,
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn network_identity(&self) -> &str {
        &self.network_identity
    }

    #[must_use]
    pub fn genesis_identity(&self) -> &str {
        &self.genesis_identity
    }

    #[must_use]
    pub fn target_contract(&self) -> &str {
        &self.target_contract
    }

    #[must_use]
    pub const fn block_hash(&self) -> &Digest32 {
        &self.block_hash
    }

    #[must_use]
    pub const fn block_height(&self) -> u32 {
        self.block_height
    }

    #[must_use]
    pub const fn transaction_identity(&self) -> Txid {
        self.transaction_identity
    }

    #[must_use]
    pub const fn predecessor(&self) -> Outpoint {
        self.predecessor
    }

    #[must_use]
    pub const fn successor(&self) -> Outpoint {
        self.successor
    }

    #[must_use]
    pub const fn linked_candidate(
        &self,
    ) -> &MaturityDeclaredPremise<CandidateLinkedMaturityBundle> {
        &self.linked_candidate
    }

    #[must_use]
    pub const fn candidate_abi(
        &self,
    ) -> &MaturityDeclaredPremise<CandidateMaturityAnnouncementAbi> {
        &self.candidate_abi
    }

    #[must_use]
    pub const fn branch(&self) -> BranchContext {
        self.branch
    }
}

fn linked_candidate_disagreement(
    bundle: &CandidateLinkedMaturityBundle,
    handoff: &PublicAnnouncementHandoff,
) -> Option<MaturityPremiseDisagreement> {
    let declared = bundle.record().schedule();
    let published = handoff.schedule();
    if declared != published {
        return Some(MaturityPremiseDisagreement::WitnessSchedule {
            declared,
            published,
        });
    }
    let declared = bundle.deployment().lead_bounds().bounds();
    let published = handoff.bounds();
    if declared != published {
        return Some(MaturityPremiseDisagreement::LeadBounds {
            declared,
            published,
        });
    }
    let declared = *bundle.static_subtree().root();
    let published = *handoff.static_subtree().root();
    if declared != published {
        return Some(MaturityPremiseDisagreement::StaticRoot {
            declared,
            published,
        });
    }
    let declared = bundle.policy().internal_key();
    let published = handoff.internal_key();
    if declared != published {
        return Some(MaturityPremiseDisagreement::InternalKey {
            declared,
            published,
        });
    }
    let declared = bundle.contract();
    let published = handoff.contract();
    if declared != published {
        return Some(MaturityPremiseDisagreement::ContractRevision {
            declared,
            published,
        });
    }
    None
}

fn candidate_abi_disagreement(
    abi: &CandidateMaturityAnnouncementAbi,
    handoff: &PublicAnnouncementHandoff,
) -> Option<MaturityPremiseDisagreement> {
    let declared = abi.schedule();
    let published = handoff.schedule();
    if declared != published {
        return Some(MaturityPremiseDisagreement::WitnessSchedule {
            declared,
            published,
        });
    }
    let declared = abi.contract();
    let published = handoff.contract();
    if declared != published {
        return Some(MaturityPremiseDisagreement::ContractRevision {
            declared,
            published,
        });
    }
    None
}

impl ModelledBranchBlock {
    /// Admit a modelled block with a nonzero identity.
    ///
    /// # Errors
    /// Refuses an all-zero block identity with the offered height.
    pub fn new(
        height: u32,
        identity: Digest32,
        transactions: Vec<Txid>,
    ) -> Result<Self, MaturityModelledBranchRefusal> {
        if identity == [0; 32] {
            return Err(MaturityModelledBranchRefusal::ZeroBlockIdentity { height });
        }
        Ok(Self {
            height,
            identity,
            transactions,
        })
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub const fn identity(&self) -> &Digest32 {
        &self.identity
    }

    #[must_use]
    pub fn transactions(&self) -> &[Txid] {
        &self.transactions
    }
}

impl ModelledBranchPrefix {
    /// Admit a branch identifier and its first modelled block.
    ///
    /// # Errors
    /// Refuses an all-zero prefix identifier with the anchor height.
    pub fn anchored(
        identifier: Digest32,
        anchor: ModelledBranchBlock,
    ) -> Result<Self, MaturityModelledBranchRefusal> {
        if identifier == [0; 32] {
            return Err(MaturityModelledBranchRefusal::ZeroPrefixIdentifier {
                anchor: anchor.height,
            });
        }
        Ok(Self {
            identifier,
            anchor_height: anchor.height,
            blocks: vec![anchor],
        })
    }

    /// Begin at the checkpoint's block under its stated branch identifier.
    ///
    /// # Errors
    /// Preserves the block or prefix refusal if either identity is all zero.
    pub fn from_checkpoint(
        checkpoint: &MaturityRootCheckpoint,
    ) -> Result<Self, MaturityModelledBranchRefusal> {
        Self::anchored(
            *checkpoint.branch().identifier(),
            ModelledBranchBlock::new(
                checkpoint.block_height(),
                *checkpoint.block_hash(),
                vec![checkpoint.transaction_identity()],
            )?,
        )
    }

    /// Return a new prefix with one block at the tip's successor height.
    ///
    /// # Errors
    /// Refuses an offered height other than the tip's successor.
    pub fn extend(
        &self,
        block: ModelledBranchBlock,
    ) -> Result<Self, MaturityModelledBranchRefusal> {
        let tip = self.tip_height();
        if tip.checked_add(1) != Some(block.height) {
            return Err(MaturityModelledBranchRefusal::NonContiguousExtension {
                tip,
                offered: block.height,
            });
        }
        let mut extended = self.clone();
        extended.blocks.push(block);
        Ok(extended)
    }

    /// Return a prefix truncated at the requested block height.
    ///
    /// # Errors
    /// Refuses a requested height outside the anchor-to-tip window.
    pub fn rewind(&self, height: u32) -> Result<Self, MaturityModelledBranchRefusal> {
        if height < self.anchor_height {
            return Err(MaturityModelledBranchRefusal::RewoundBelowTheAnchor {
                anchor: self.anchor_height,
                requested: height,
            });
        }
        let tip = self.tip_height();
        if height > tip {
            return Err(MaturityModelledBranchRefusal::RewoundAboveTheTip {
                tip,
                requested: height,
            });
        }
        Ok(Self {
            identifier: self.identifier,
            anchor_height: self.anchor_height,
            blocks: self
                .blocks
                .iter()
                .take_while(|block| block.height <= height)
                .cloned()
                .collect(),
        })
    }

    /// Ask whether this prefix still supports the checkpoint's transaction block.
    ///
    /// # Errors
    /// Refuses another prefix, a missing block, a changed block or an absent transaction.
    pub fn reproject(
        &self,
        checkpoint: &MaturityRootCheckpoint,
    ) -> Result<MaturityRootObservation, MaturityRootObservationRefusal> {
        let prefix = *checkpoint.branch().identifier();
        if self.identifier != prefix {
            return Err(MaturityRootObservationRefusal::EvidenceFromAnotherPrefix {
                presented: self.identifier,
                prefix,
            });
        }
        let observation = MaturityRootObservation {
            prefix,
            context: self.context(),
            block_height: checkpoint.block_height(),
            block_identity: *checkpoint.block_hash(),
            transaction: checkpoint.transaction_identity(),
        };
        observation.binds(self)?;
        Ok(observation)
    }

    /// The branch identifier at this modelled tip height.
    ///
    /// # Panics
    /// Panics only if the identifier is all zero, which `anchored` refuses.
    #[must_use]
    pub fn context(&self) -> BranchContext {
        let Ok(context) = BranchContext::new(self.identifier, u64::from(self.tip_height())) else {
            unreachable!("anchored refuses a zero identifier");
        };
        context
    }

    #[must_use]
    pub const fn identifier(&self) -> &Digest32 {
        &self.identifier
    }

    #[must_use]
    pub const fn anchor_height(&self) -> u32 {
        self.anchor_height
    }

    /// The anchor height plus the number of successor blocks.
    #[must_use]
    pub fn tip_height(&self) -> u32 {
        let successors = self.blocks.len().saturating_sub(1);
        u32::try_from(successors)
            .map_or(u32::MAX, |offset| self.anchor_height.saturating_add(offset))
    }

    #[must_use]
    pub fn blocks(&self) -> &[ModelledBranchBlock] {
        &self.blocks
    }
}

impl MaturityRootObservation {
    /// Ask whether a modelled prefix still carries the observed transaction block.
    ///
    /// # Errors
    /// Refuses another prefix, a missing block, a changed block or an absent transaction.
    pub fn binds(
        &self,
        prefix: &ModelledBranchPrefix,
    ) -> Result<(), MaturityRootObservationRefusal> {
        if prefix.identifier != self.prefix {
            return Err(MaturityRootObservationRefusal::EvidenceFromAnotherPrefix {
                presented: prefix.identifier,
                prefix: self.prefix,
            });
        }
        let tip = prefix.tip_height();
        let Some(block) = prefix
            .blocks
            .iter()
            .find(|block| block.height == self.block_height)
        else {
            return Err(
                MaturityRootObservationRefusal::TransactionBlockIsNotInThePrefix {
                    height: self.block_height,
                    anchor: prefix.anchor_height,
                    tip,
                },
            );
        };
        if block.identity != self.block_identity {
            return Err(MaturityRootObservationRefusal::BlockAtThatHeightChanged {
                height: self.block_height,
                prefix: block.identity,
                checkpoint: self.block_identity,
            });
        }
        if !block.transactions.contains(&self.transaction) {
            return Err(
                MaturityRootObservationRefusal::TransactionAbsentFromThatBlock {
                    height: self.block_height,
                    transaction: self.transaction,
                    block: block.identity,
                },
            );
        }
        Ok(())
    }

    #[must_use]
    pub const fn prefix(&self) -> &Digest32 {
        &self.prefix
    }
    #[must_use]
    pub const fn context(&self) -> BranchContext {
        self.context
    }
    #[must_use]
    pub const fn block_height(&self) -> u32 {
        self.block_height
    }
    #[must_use]
    pub const fn block_identity(&self) -> &Digest32 {
        &self.block_identity
    }
    #[must_use]
    pub const fn transaction(&self) -> Txid {
        self.transaction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maturity_closure::closure_target;
    use crate::maturity_continuity::tests::{archived, variable_archived};
    use crate::maturity_continuity::{MaturityByteSource, project_maturity_continuity};
    use crate::maturity_corpus::{maturity_run_of_record, maturity_variable_run_of_record};
    use crate::maturity_evidence::{
        MaturityConstructorMaterial, MaturityConstructorMaterialAbsence,
        MaturityExecutorProvenanceExpectation, checkpoint_premises_of,
        derive_maturity_evidence_plan_with,
    };
    use crate::maturity_recovery::PublicAnnouncementLocator;
    use crate::maturity_recovery_report::accepted_public_handoff;
    use realization::Cycle;
    use std::sync::LazyLock;
    use tapscript::StateStaticNode;
    use transaction::bytes::Txid;

    static WHOLE_METADATA_PREMISES: LazyLock<(
        MaturityDeclaredPremise<CandidateLinkedMaturityBundle>,
        MaturityDeclaredPremise<CandidateMaturityAnnouncementAbi>,
    )> = LazyLock::new(|| {
        let plan = derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
            MaturityConstructorMaterial::Absent(
                MaturityConstructorMaterialAbsence::NotSuppliedToDerivation,
            ),
        )
        .expect("evidence plan");
        (
            MaturityDeclaredPremise::declared(
                plan.bundle().clone(),
                MaturityPremiseProvenance::DeploymentDeclaration,
            ),
            MaturityDeclaredPremise::declared(
                plan.abi().clone(),
                MaturityPremiseProvenance::DeploymentDeclaration,
            ),
        )
    });

    fn declarations(
        continuity: &ValidatedMaturityContinuity,
    ) -> (
        MaturityDeclaredPremise<CandidateLinkedMaturityBundle>,
        MaturityDeclaredPremise<CandidateMaturityAnnouncementAbi>,
    ) {
        checkpoint_premises_of(continuity).expect("accepted premises")
    }

    fn checkpoint(
        edge: &StateRootEdge,
        continuity: &ValidatedMaturityContinuity,
        handoff: &PublicAnnouncementHandoff,
        corpus: &ValidatedMaturityCorpus,
    ) -> MaturityRootCheckpoint {
        let (linked, abi) = declarations(continuity);
        MaturityRootCheckpoint::bind(edge, handoff, corpus, linked, abi)
            .expect("accepted checkpoint binds")
    }

    fn accepted_handoff() -> PublicAnnouncementHandoff {
        accepted_public_handoff(maturity_variable_run_of_record().expect("accepted corpus"))
            .expect("accepted handoff")
    }

    fn historical_handoff(
        historical: &ValidatedMaturityContinuity,
        accepted: &PublicAnnouncementHandoff,
    ) -> PublicAnnouncementHandoff {
        let corpus = maturity_run_of_record().expect("historical corpus");
        PublicAnnouncementHandoff::new(
            PublicAnnouncementLocator::new(
                MaturityByteSource::ArchivedSubmission {
                    run_address: corpus.report().run_address().to_owned(),
                },
                submitted_transaction_identities(
                    historical.transaction(),
                    historical.submitted_bytes(),
                )
                .identity(),
            ),
            archived().input().submitted_bytes.to_vec(),
            0,
            accepted.schedule(),
            accepted.bounds(),
            accepted.static_subtree().clone(),
            accepted.internal_key(),
            accepted.contract(),
        )
    }

    fn accepted_edge() -> StateRootEdge {
        let continuity = project_maturity_continuity(variable_archived().input())
            .expect("accepted archive projects");
        StateRootEdge::from_continuity(&continuity)
    }

    #[test]
    fn the_handoff_built_edge_agrees_with_the_projected_edge_field_by_field() {
        let handoff = accepted_handoff();
        let projected = accepted_edge();
        let edge = StateRootEdge::from_public_handoff(handoff.clone())
            .expect("accepted handoff recovers a root edge");
        assert_eq!(edge.predecessor(), projected.predecessor());
        assert_eq!(edge.successor(), projected.successor());
        assert_eq!(edge.operation(), projected.operation());

        let certificate = edge.certificate();
        let projected_certificate = projected.certificate();
        assert_eq!(certificate.projection(), projected_certificate.projection());
        assert_eq!(certificate.operation(), projected_certificate.operation());
        assert_eq!(certificate.root(), projected_certificate.root());
        assert_eq!(certificate.use_kind(), projected_certificate.use_kind());
        assert_eq!(
            certificate.predecessor(),
            projected_certificate.predecessor()
        );
        assert_eq!(certificate.successor(), projected_certificate.successor());
        assert_eq!(
            *certificate.predecessor_static().root(),
            *projected_certificate.predecessor_static().root()
        );
        assert_eq!(
            *certificate.successor_static().root(),
            *projected_certificate.successor_static().root()
        );

        let cursor = validate_state_root_history(std::slice::from_ref(&edge), edge.predecessor())
            .expect("handoff edge validates");
        assert_eq!(cursor, edge.successor());
        assert_eq!(cursor.index(), 0);

        let build: fn(PublicAnnouncementHandoff) -> Result<StateRootEdge, MaturityRecoveryRefusal> =
            StateRootEdge::from_public_handoff;
        assert!(build(accepted_handoff()).is_ok());
        let wrong_index = PublicAnnouncementHandoff::new(
            handoff.locator().clone(),
            handoff.bytes().to_vec(),
            1,
            handoff.schedule(),
            handoff.bounds(),
            handoff.static_subtree().clone(),
            handoff.internal_key(),
            handoff.contract(),
        );
        assert_eq!(
            build(wrong_index),
            Err(MaturityRecoveryRefusal::WrongOutputIndex {
                stated: 1,
                declared: 0,
            })
        );
    }

    fn other_outpoint(byte: u8) -> Outpoint {
        Outpoint::new(Txid::from_internal([byte; 32]), 0).expect("test outpoint")
    }

    fn retarget(edge: &StateRootEdge, predecessor: Outpoint, successor: Outpoint) -> StateRootEdge {
        let mut changed = edge.clone();
        changed.predecessor = predecessor;
        changed.successor = successor;
        changed.certificate.predecessor = predecessor;
        changed.certificate.successor = successor;
        changed
    }

    fn assert_refusal(
        edges: &[StateRootEdge],
        cursor: Outpoint,
        expected: &MaturityRootHistoryRefusal,
        clause: HistoryClause,
        position: Option<usize>,
    ) {
        let actual = validate_state_root_history(edges, cursor).expect_err("refused sequence");
        assert_eq!(&actual, expected);
        assert_eq!(actual.clause(), clause);
        assert_eq!(actual.position(), position);
    }

    #[test]
    fn the_checkpoint_binds_eight_facts_and_declares_two() {
        use MaturityRootFact as F;
        use MaturityRootFactKind as K;

        let continuity =
            project_maturity_continuity(variable_archived().input()).expect("accepted continuity");
        let edge = StateRootEdge::from_continuity(&continuity);
        let corpus = maturity_variable_run_of_record().expect("accepted corpus");
        let handoff = accepted_handoff();
        let checkpoint = checkpoint(&edge, &continuity, &handoff, corpus);
        let MaturityAcceptanceObligation::Established { readback, .. } =
            corpus.evidence().acceptance_obligation()
        else {
            panic!("accepted readback")
        };
        assert_eq!(checkpoint.network_identity(), corpus.report().network_id());
        assert_eq!(checkpoint.genesis_identity(), corpus.report().genesis_id());
        assert_eq!(checkpoint.block_hash(), readback.block_hash());
        assert_eq!(checkpoint.block_height(), readback.block_height());
        assert_eq!(checkpoint.transaction_identity(), readback.identity());
        assert_eq!(checkpoint.predecessor(), edge.predecessor());
        assert_eq!(checkpoint.predecessor(), continuity.funded().outpoint);
        assert_eq!(
            checkpoint.successor(),
            Outpoint::new(readback.identity(), 0).expect("outpoint")
        );
        assert_eq!(checkpoint.successor(), edge.successor());
        assert_eq!(
            checkpoint.target_contract(),
            corpus.report().target_contract()
        );
        assert_eq!(checkpoint.linked_candidate().value(), continuity.bundle());
        assert_eq!(
            checkpoint.candidate_abi().value().schedule(),
            handoff.schedule()
        );
        assert_eq!(
            checkpoint.linked_candidate().value().record().schedule(),
            handoff.schedule()
        );
        assert_eq!(handoff.schedule(), StateWitnessSchedule::VariableMetadata);
        assert_eq!(
            checkpoint.linked_candidate().provenance(),
            MaturityPremiseProvenance::DeploymentDeclaration
        );
        assert_eq!(
            checkpoint.candidate_abi().provenance(),
            MaturityPremiseProvenance::DeploymentDeclaration
        );
        assert_eq!(checkpoint.branch(), continuity.branch());
        assert_eq!(checkpoint.binds(&edge, &handoff), Ok(()));

        let expected = [
            (F::NetworkIdentity, K::CarriedByTheAdmittedArchive),
            (F::GenesisIdentity, K::CarriedByTheAdmittedArchive),
            (F::BlockHash, K::CarriedByTheAdmittedArchive),
            (F::BlockHeight, K::CarriedByTheAdmittedArchive),
            (F::TransactionIdentity, K::CarriedByTheAdmittedArchive),
            (F::PredecessorOutpoint, K::ReadFromThePublicBytes),
            (
                F::SuccessorOutpoint,
                K::DerivedFromThePublicBytesAndTheLocator,
            ),
            (F::TargetContract, K::CarriedByTheAdmittedArchive),
            (
                F::LinkedCandidate,
                K::DeclaredPremise(MaturityPremiseProvenance::DeploymentDeclaration),
            ),
            (
                F::CandidateAbi,
                K::DeclaredPremise(MaturityPremiseProvenance::DeploymentDeclaration),
            ),
        ];
        assert_eq!(F::ALL.len(), 10);
        for ((fact, kind), roster_fact) in expected.into_iter().zip(F::ALL) {
            assert_eq!(fact, *roster_fact);
            assert_eq!(fact.kind(), kind);
        }
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "each refusal retains a distinct binding operand"
    )]
    fn the_other_archives_facts_do_not_bind_the_accepted_handoff() {
        let accepted =
            project_maturity_continuity(variable_archived().input()).expect("accepted continuity");
        let accepted_edge = StateRootEdge::from_continuity(&accepted);
        let accepted_corpus = maturity_variable_run_of_record().expect("accepted corpus");
        let historical =
            project_maturity_continuity(archived().input()).expect("historical continuity");
        let historical_edge = StateRootEdge::from_continuity(&historical);
        let historical_corpus = maturity_run_of_record().expect("historical corpus");
        let handoff = accepted_handoff();
        let historical_handoff = historical_handoff(&historical, &handoff);
        let MaturityAcceptanceObligation::Outstanding { routes } =
            historical_corpus.evidence().acceptance_obligation()
        else {
            panic!("historical acceptance is outstanding")
        };
        let (linked, abi) = declarations(&historical);
        let outstanding = MaturityRootCheckpoint::bind(
            &historical_edge,
            &historical_handoff,
            historical_corpus,
            linked,
            abi,
        )
        .expect_err("historical acceptance refuses");
        assert_eq!(
            outstanding,
            MaturityRootCheckpointRefusal::AcceptanceIsOutstanding { routes: *routes }
        );
        assert_eq!(outstanding.fact(), None);

        let checkpoint = checkpoint(&accepted_edge, &accepted, &handoff, accepted_corpus);
        let accepted_identity = checkpoint.transaction_identity();
        let historical_identity = submitted_transaction_identities(
            historical.transaction(),
            historical.submitted_bytes(),
        )
        .identity();
        let identities_differ = accepted_identity != historical_identity;
        assert!(identities_differ);
        let identity_refusal = checkpoint
            .binds(&historical_edge, &historical_handoff)
            .expect_err("another handoff identity refuses");
        assert_eq!(
            identity_refusal,
            MaturityRootCheckpointRefusal::TransactionIdentityDisagrees {
                checkpoint: accepted_identity,
                handoff: historical_identity,
            }
        );
        assert_eq!(
            identity_refusal.fact(),
            Some(MaturityRootFact::TransactionIdentity)
        );
        let (linked, abi) = declarations(&historical);
        let bytes_refusal = MaturityRootCheckpoint::bind(
            &historical_edge,
            &historical_handoff,
            accepted_corpus,
            linked,
            abi,
        )
        .expect_err("another handoff's bytes refuse");
        let MaturityAcceptanceObligation::Established { readback, .. } =
            accepted_corpus.evidence().acceptance_obligation()
        else {
            panic!("accepted readback")
        };
        assert_eq!(
            bytes_refusal,
            MaturityRootCheckpointRefusal::SubmittedBytesDisagree {
                checkpoint: sha256(readback.bytes()),
                handoff: sha256(historical_handoff.bytes()),
            }
        );
        assert_eq!(
            bytes_refusal.fact(),
            Some(MaturityRootFact::TransactionIdentity)
        );

        let wrong_successor = retarget(
            &accepted_edge,
            accepted_edge.predecessor,
            other_outpoint(0x5b),
        );
        let (linked, abi) = declarations(&accepted);
        let successor_refusal =
            MaturityRootCheckpoint::bind(&wrong_successor, &handoff, accepted_corpus, linked, abi)
                .expect_err("changed successor refuses");
        assert_eq!(
            successor_refusal,
            MaturityRootCheckpointRefusal::SuccessorIsNotTheHandoffsStateOutput {
                checkpoint: wrong_successor.successor,
                identity: accepted_identity,
                index: 0,
            }
        );
        assert_eq!(
            successor_refusal.fact(),
            Some(MaturityRootFact::SuccessorOutpoint)
        );
        let edge_refusal = checkpoint
            .binds(&wrong_successor, &handoff)
            .expect_err("another edge successor refuses");
        assert_eq!(
            edge_refusal,
            MaturityRootCheckpointRefusal::SuccessorOutpointDisagrees {
                checkpoint: accepted_edge.successor,
                edge: wrong_successor.successor,
            }
        );
        assert_eq!(
            edge_refusal.fact(),
            Some(MaturityRootFact::SuccessorOutpoint)
        );
        let wrong_predecessor = retarget(
            &accepted_edge,
            other_outpoint(0x5c),
            accepted_edge.successor,
        );
        let (linked, abi) = declarations(&accepted);
        let predecessor_refusal = MaturityRootCheckpoint::bind(
            &wrong_predecessor,
            &handoff,
            accepted_corpus,
            linked,
            abi,
        )
        .expect_err("changed predecessor refuses");
        let transaction = TargetTransaction::decode(handoff.bytes()).expect("accepted bytes");
        let input_zero = transaction.inputs().first().expect("accepted input zero");
        assert_eq!(
            predecessor_refusal,
            MaturityRootCheckpointRefusal::PredecessorOutpointDisagrees {
                checkpoint: wrong_predecessor.predecessor,
                handoff: input_zero.outpoint(),
            }
        );
        assert_eq!(
            predecessor_refusal.fact(),
            Some(MaturityRootFact::PredecessorOutpoint)
        );
    }

    #[test]
    fn a_premise_that_did_not_produce_the_accepted_bytes_refuses_the_binding() {
        let continuity =
            project_maturity_continuity(variable_archived().input()).expect("accepted continuity");
        let edge = StateRootEdge::from_continuity(&continuity);
        let corpus = maturity_variable_run_of_record().expect("accepted corpus");
        let handoff = accepted_handoff();
        let (linked, abi) = declarations(&continuity);

        let linked_refusal = MaturityRootCheckpoint::bind(
            &edge,
            &handoff,
            corpus,
            WHOLE_METADATA_PREMISES.0.clone(),
            abi.clone(),
        )
        .expect_err("whole-metadata linked candidate refuses");
        assert_eq!(
            linked_refusal,
            MaturityRootCheckpointRefusal::LinkedCandidateDisagrees(
                MaturityPremiseDisagreement::WitnessSchedule {
                    declared: StateWitnessSchedule::WholeMetadata,
                    published: StateWitnessSchedule::VariableMetadata,
                }
            )
        );
        assert_eq!(
            linked_refusal.fact(),
            Some(MaturityRootFact::LinkedCandidate)
        );

        let abi_refusal = MaturityRootCheckpoint::bind(
            &edge,
            &handoff,
            corpus,
            linked.clone(),
            WHOLE_METADATA_PREMISES.1.clone(),
        )
        .expect_err("whole-metadata ABI refuses");
        assert_eq!(
            abi_refusal,
            MaturityRootCheckpointRefusal::CandidateAbiDisagrees(
                MaturityPremiseDisagreement::WitnessSchedule {
                    declared: StateWitnessSchedule::WholeMetadata,
                    published: StateWitnessSchedule::VariableMetadata,
                }
            )
        );
        assert_eq!(abi_refusal.fact(), Some(MaturityRootFact::CandidateAbi));

        let changed_bounds = AnnouncementLeadBounds::new(Cycle::new(4), Cycle::new(7))
            .expect("changed lead bounds are valid");
        let bounds_differ = changed_bounds != handoff.bounds();
        assert!(bounds_differ);
        let changed_handoff = PublicAnnouncementHandoff::new(
            handoff.locator().clone(),
            handoff.bytes().to_vec(),
            handoff.state_output_index(),
            handoff.schedule(),
            changed_bounds,
            handoff.static_subtree().clone(),
            handoff.internal_key(),
            handoff.contract(),
        );
        let bounds_refusal =
            MaturityRootCheckpoint::bind(&edge, &changed_handoff, corpus, linked, abi)
                .expect_err("changed published lead bounds refuse");
        assert_eq!(
            bounds_refusal,
            MaturityRootCheckpointRefusal::LinkedCandidateDisagrees(
                MaturityPremiseDisagreement::LeadBounds {
                    declared: handoff.bounds(),
                    published: changed_bounds,
                }
            )
        );
        assert_eq!(
            bounds_refusal.fact(),
            Some(MaturityRootFact::LinkedCandidate)
        );
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one modelled history exercises each stale outcome"
    )]
    fn a_rewind_stales_the_observation_and_another_prefix_refuses_its_reuse() {
        let continuity =
            project_maturity_continuity(variable_archived().input()).expect("accepted continuity");
        let edge = StateRootEdge::from_continuity(&continuity);
        let corpus = maturity_variable_run_of_record().expect("accepted corpus");
        let handoff = accepted_handoff();
        let checkpoint = checkpoint(&edge, &continuity, &handoff, corpus);
        let height = checkpoint.block_height();
        let earlier_height = height
            .checked_sub(1)
            .expect("accepted block follows an earlier height");
        let next_height = height.checked_add(1).expect("successor height");
        let later_height = next_height.checked_add(1).expect("later height");
        let branch = *checkpoint.branch().identifier();
        let prefix = ModelledBranchPrefix::from_checkpoint(&checkpoint).expect("checkpoint prefix");
        let observation = prefix.reproject(&checkpoint).expect("observed checkpoint");
        assert_eq!(observation.prefix(), &branch);
        assert_eq!(observation.block_height(), height);
        assert_eq!(observation.block_identity(), checkpoint.block_hash());
        assert_eq!(observation.transaction(), checkpoint.transaction_identity());
        assert_eq!(observation.context(), prefix.context());
        assert_eq!(observation.context().identifier(), &branch);
        assert_eq!(observation.context().checkpoint(), u64::from(height));
        assert_eq!(observation.binds(&prefix), Ok(()));
        assert_eq!(prefix.anchor_height(), height);
        assert_eq!(prefix.tip_height(), height);
        assert_eq!(prefix.blocks().len(), 1);

        let next =
            ModelledBranchBlock::new(next_height, [0x51; 32], Vec::new()).expect("next block");
        let extended = prefix.extend(next).expect("contiguous extension");
        assert_eq!(extended.tip_height(), next_height);
        let late =
            ModelledBranchBlock::new(later_height, [0x52; 32], Vec::new()).expect("late block");
        assert_eq!(
            prefix.extend(late),
            Err(MaturityModelledBranchRefusal::NonContiguousExtension {
                tip: height,
                offered: later_height,
            })
        );
        assert_eq!(
            prefix.rewind(earlier_height),
            Err(MaturityModelledBranchRefusal::RewoundBelowTheAnchor {
                anchor: height,
                requested: earlier_height,
            })
        );
        assert_eq!(
            prefix.rewind(next_height),
            Err(MaturityModelledBranchRefusal::RewoundAboveTheTip {
                tip: height,
                requested: next_height,
            })
        );

        let earlier = ModelledBranchBlock::new(earlier_height, [0x53; 32], Vec::new())
            .expect("earlier block");
        assert_eq!(
            ModelledBranchPrefix::anchored([0; 32], earlier.clone()),
            Err(MaturityModelledBranchRefusal::ZeroPrefixIdentifier {
                anchor: earlier_height
            })
        );
        assert_eq!(
            ModelledBranchBlock::new(height, [0; 32], Vec::new()),
            Err(MaturityModelledBranchRefusal::ZeroBlockIdentity { height })
        );
        let earlier_prefix =
            ModelledBranchPrefix::anchored(branch, earlier.clone()).expect("earlier prefix");
        let recorded = ModelledBranchBlock::new(
            height,
            *checkpoint.block_hash(),
            vec![checkpoint.transaction_identity()],
        )
        .expect("recorded block");
        let two_blocks = earlier_prefix
            .extend(recorded)
            .expect("checkpoint extension");
        assert_eq!(two_blocks.reproject(&checkpoint), Ok(observation.clone()));
        let rewound = two_blocks.rewind(earlier_height).expect("rewind to anchor");
        let removed = MaturityRootObservationRefusal::TransactionBlockIsNotInThePrefix {
            height,
            anchor: earlier_height,
            tip: earlier_height,
        };
        assert_eq!(rewound.reproject(&checkpoint), Err(removed.clone()));
        assert_eq!(observation.binds(&rewound), Err(removed));

        let changed_identity = [0x54; 32];
        let substituted = rewound
            .extend(
                ModelledBranchBlock::new(
                    height,
                    changed_identity,
                    vec![checkpoint.transaction_identity()],
                )
                .expect("substituted block"),
            )
            .expect("substituted extension");
        assert_eq!(
            substituted.reproject(&checkpoint),
            Err(MaturityRootObservationRefusal::BlockAtThatHeightChanged {
                height,
                prefix: changed_identity,
                checkpoint: *checkpoint.block_hash(),
            },)
        );
        let another_transaction = Txid::from_internal([0x55; 32]);
        let transaction_removed = rewound
            .extend(
                ModelledBranchBlock::new(
                    height,
                    *checkpoint.block_hash(),
                    vec![another_transaction],
                )
                .expect("block without transaction"),
            )
            .expect("replacement extension");
        assert_eq!(
            transaction_removed.reproject(&checkpoint),
            Err(
                MaturityRootObservationRefusal::TransactionAbsentFromThatBlock {
                    height,
                    transaction: checkpoint.transaction_identity(),
                    block: *checkpoint.block_hash(),
                },
            )
        );

        let foreign = ModelledBranchPrefix::anchored([0x5a; 32], earlier)
            .expect("foreign prefix without checkpoint height");
        assert_eq!(foreign.tip_height(), earlier_height);
        assert_eq!(
            foreign.reproject(&checkpoint),
            Err(MaturityRootObservationRefusal::EvidenceFromAnotherPrefix {
                presented: [0x5a; 32],
                prefix: branch,
            },)
        );
    }

    #[test]
    fn accepted_archive_single_edge_validates_and_advances_the_cursor() {
        let edge = accepted_edge();
        assert_eq!(edge.successor.index(), 0);
        assert_eq!(
            validate_state_root_history(std::slice::from_ref(&edge), edge.predecessor),
            Ok(edge.successor)
        );
    }

    #[test]
    fn stale_predecessor_refuses_under_clause_one() {
        let edge = accepted_edge();
        let stale_cursor = other_outpoint(0x91);
        assert_refusal(
            std::slice::from_ref(&edge),
            stale_cursor,
            &MaturityRootHistoryRefusal::PredecessorIsNotTheCursor {
                position: 0,
                cursor: stale_cursor,
                predecessor: edge.predecessor,
            },
            HistoryClause::PredecessorIsCursor,
            Some(0),
        );
    }

    #[test]
    fn wrong_current_root_view_refuses_under_clause_one() {
        let edge = accepted_edge();
        let view_cursor = other_outpoint(0x92);
        assert_refusal(
            std::slice::from_ref(&edge),
            view_cursor,
            &MaturityRootHistoryRefusal::PredecessorIsNotTheCursor {
                position: 0,
                cursor: view_cursor,
                predecessor: edge.predecessor,
            },
            HistoryClause::PredecessorIsCursor,
            Some(0),
        );
    }

    #[test]
    fn two_predecessors_refuses_under_clause_two() {
        let edge = accepted_edge();
        let second = retarget(&edge, edge.predecessor, other_outpoint(0x5a));
        assert_refusal(
            &[edge.clone(), second],
            edge.predecessor,
            &MaturityRootHistoryRefusal::OutpointReused {
                position: 1,
                outpoint: edge.predecessor,
                first_seen: FirstSeen::Edge {
                    position: 0,
                    side: EdgeSide::Predecessor,
                },
            },
            HistoryClause::SuccessorIsUnique,
            Some(1),
        );
    }

    #[test]
    fn two_successors_refuses_under_clause_two() {
        let edge = accepted_edge();
        let second = retarget(&edge, edge.successor, edge.successor);
        assert_refusal(
            &[edge.clone(), second],
            edge.predecessor,
            &MaturityRootHistoryRefusal::OutpointReused {
                position: 1,
                outpoint: edge.successor,
                first_seen: FirstSeen::Edge {
                    position: 0,
                    side: EdgeSide::Successor,
                },
            },
            HistoryClause::SuccessorIsUnique,
            Some(1),
        );
    }

    #[test]
    fn missing_edge_refuses_the_empty_sequence() {
        let cursor = accepted_edge().predecessor;
        assert_refusal(
            &[],
            cursor,
            &MaturityRootHistoryRefusal::EmptySequence,
            HistoryClause::CursorAdvancesAfterValidation,
            None,
        );
    }

    #[test]
    fn duplicate_edge_refuses_under_clause_two() {
        let honest = accepted_edge();
        let edges = [honest.clone(), honest.clone()];
        assert_refusal(
            &edges,
            honest.predecessor,
            &MaturityRootHistoryRefusal::OutpointReused {
                position: 1,
                outpoint: honest.predecessor,
                first_seen: FirstSeen::Edge {
                    position: 0,
                    side: EdgeSide::Predecessor,
                },
            },
            HistoryClause::SuccessorIsUnique,
            Some(1),
        );
    }

    #[test]
    fn successor_cursor_restored_after_invalid_intermediate_edge_refuses_under_clause_four() {
        let honest = accepted_edge();
        let first_successor = other_outpoint(0xa1);
        let middle_successor = other_outpoint(0xa2);
        let other_certificate_successor = other_outpoint(0xa3);
        let first = retarget(&honest, honest.predecessor, first_successor);
        let mut middle = retarget(&honest, first_successor, middle_successor);
        middle.certificate.successor = other_certificate_successor;
        let last = retarget(&honest, middle_successor, honest.successor);
        let edges = [first, middle, last];
        assert_eq!(
            edges.last().map(StateRootEdge::successor),
            Some(honest.successor)
        );
        assert_eq!(edges[1].successor, middle_successor);
        assert_refusal(
            &edges,
            honest.predecessor,
            &MaturityRootHistoryRefusal::CertificateDisagreesWithEdge {
                position: 1,
                side: EdgeSide::Successor,
                edge: middle_successor,
                certificate: other_certificate_successor,
            },
            HistoryClause::CertificateAgreesWithTransaction,
            Some(1),
        );
    }

    #[test]
    fn state_termination_refuses_under_clause_five() {
        let mut edge = accepted_edge();
        edge.certificate.use_kind = RootUse::SuccessionOrTermination;
        assert_refusal(
            std::slice::from_ref(&edge),
            edge.predecessor,
            &MaturityRootHistoryRefusal::UnrelatedRootEffect {
                position: 0,
                root: RootId::State,
                use_kind: RootUse::SuccessionOrTermination,
                successor: edge.successor,
            },
            HistoryClause::NoUnrelatedRootEffect,
            Some(0),
        );
    }

    #[test]
    fn old_static_subtree_to_new_subtree_without_migration_refuses_under_bundle_continuity() {
        let mut edge = accepted_edge();
        let original = edge.certificate.predecessor_static();
        let target = closure_target().expect("constructor target");
        let rebuilt = StateStaticSubtree::new(
            &target,
            Some(StateStaticNode::Leaf {
                identity: 1,
                leaf: original.leaves()[0].leaf.clone(),
            }),
        )
        .expect("second descriptor");
        let subtrees_differ = original != &rebuilt;
        assert!(subtrees_differ);
        let predecessor_root = *original.root();
        let successor_root = *rebuilt.root();
        edge.certificate.successor_static = rebuilt;
        assert_refusal(
            std::slice::from_ref(&edge),
            edge.predecessor,
            &MaturityRootHistoryRefusal::StaticSubtreeDiscontinuity {
                position: 0,
                refusal: StateLinkRefusal::StaticSubtreeDiscontinuity {
                    predecessor: predecessor_root,
                    successor: successor_root,
                },
            },
            HistoryClause::BundleContinuity,
            Some(0),
        );
    }

    #[test]
    fn unrelated_state_shaped_input_refuses_under_clause_four() {
        let mut edge = accepted_edge();
        let unrelated = other_outpoint(0xb1);
        let certificate = edge.certificate.predecessor;
        edge.predecessor = unrelated;
        assert_refusal(
            std::slice::from_ref(&edge),
            unrelated,
            &MaturityRootHistoryRefusal::CertificateDisagreesWithEdge {
                position: 0,
                side: EdgeSide::Predecessor,
                edge: unrelated,
                certificate,
            },
            HistoryClause::CertificateAgreesWithTransaction,
            Some(0),
        );
    }

    #[test]
    fn root_cursor_points_to_sponsor_change_refuses_under_clause_five() {
        let mut edge = accepted_edge();
        let sponsor = Outpoint::new(edge.successor.txid(), 1).expect("sponsor output index");
        edge.successor = sponsor;
        edge.certificate.successor = sponsor;
        assert_refusal(
            std::slice::from_ref(&edge),
            edge.predecessor,
            &MaturityRootHistoryRefusal::UnrelatedRootEffect {
                position: 0,
                root: RootId::State,
                use_kind: RootUse::Succession,
                successor: sponsor,
            },
            HistoryClause::NoUnrelatedRootEffect,
            Some(0),
        );
    }

    #[test]
    fn resv_edge_refuses_under_clause_five() {
        let mut edge = accepted_edge();
        edge.certificate.root = RootId::Resv;
        assert_refusal(
            std::slice::from_ref(&edge),
            edge.predecessor,
            &MaturityRootHistoryRefusal::UnrelatedRootEffect {
                position: 0,
                root: RootId::Resv,
                use_kind: RootUse::Succession,
                successor: edge.successor,
            },
            HistoryClause::NoUnrelatedRootEffect,
            Some(0),
        );
    }

    #[test]
    fn pace_edge_refuses_under_clause_five() {
        let mut edge = accepted_edge();
        edge.certificate.root = RootId::Pace;
        assert_refusal(
            std::slice::from_ref(&edge),
            edge.predecessor,
            &MaturityRootHistoryRefusal::UnrelatedRootEffect {
                position: 0,
                root: RootId::Pace,
                use_kind: RootUse::Succession,
                successor: edge.successor,
            },
            HistoryClause::NoUnrelatedRootEffect,
            Some(0),
        );
    }

    #[test]
    fn authority_edge_refuses_under_clause_five() {
        for root in [RootId::EntAuth, RootId::DistAuth] {
            let mut edge = accepted_edge();
            edge.certificate.root = root;
            assert_refusal(
                std::slice::from_ref(&edge),
                edge.predecessor,
                &MaturityRootHistoryRefusal::UnrelatedRootEffect {
                    position: 0,
                    root,
                    use_kind: RootUse::Succession,
                    successor: edge.successor,
                },
                HistoryClause::NoUnrelatedRootEffect,
                Some(0),
            );
        }
    }

    #[test]
    fn transition_certificate_names_another_predecessor_refuses_under_clause_four() {
        let mut edge = accepted_edge();
        let other = other_outpoint(0xc1);
        edge.certificate.predecessor = other;
        assert_refusal(
            std::slice::from_ref(&edge),
            edge.predecessor,
            &MaturityRootHistoryRefusal::CertificateDisagreesWithEdge {
                position: 0,
                side: EdgeSide::Predecessor,
                edge: edge.predecessor,
                certificate: other,
            },
            HistoryClause::CertificateAgreesWithTransaction,
            Some(0),
        );
    }

    #[test]
    fn transition_certificate_names_another_successor_refuses_under_clause_four() {
        let mut edge = accepted_edge();
        let other = other_outpoint(0xc2);
        edge.certificate.successor = other;
        assert_refusal(
            std::slice::from_ref(&edge),
            edge.predecessor,
            &MaturityRootHistoryRefusal::CertificateDisagreesWithEdge {
                position: 0,
                side: EdgeSide::Successor,
                edge: edge.successor,
                certificate: other,
            },
            HistoryClause::CertificateAgreesWithTransaction,
            Some(0),
        );
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "One exhaustive match covers every refusal shape."
    )]
    fn cursor_is_returned_only_on_the_validating_path() {
        let edge = accepted_edge();
        assert_eq!(
            validate_state_root_history(std::slice::from_ref(&edge), edge.predecessor),
            Ok(edge.successor)
        );

        let mut wrong_operation = edge.clone();
        wrong_operation.operation = OperationId::Cycle;
        assert_refusal(
            &[wrong_operation],
            edge.predecessor,
            &MaturityRootHistoryRefusal::OperationIsNotMaturityAnnouncement {
                position: 0,
                operation: OperationId::Cycle,
            },
            HistoryClause::CertificateOperation,
            Some(0),
        );
        let mut wrong_certificate_operation = edge.clone();
        wrong_certificate_operation.certificate.operation = OperationId::Cycle;
        assert_refusal(
            &[wrong_certificate_operation],
            edge.predecessor,
            &MaturityRootHistoryRefusal::OperationIsNotMaturityAnnouncement {
                position: 0,
                operation: OperationId::Cycle,
            },
            HistoryClause::CertificateOperation,
            Some(0),
        );
        let mut wrong_projection = edge.clone();
        wrong_projection.certificate.projection = ProjectionId::BurnEvent;
        assert_refusal(
            &[wrong_projection],
            edge.predecessor,
            &MaturityRootHistoryRefusal::CertificateProjectionDisagrees {
                position: 0,
                projection: ProjectionId::BurnEvent,
            },
            HistoryClause::CertificateOperation,
            Some(0),
        );
        let mut starting_cursor_successor = edge.clone();
        starting_cursor_successor.successor = edge.predecessor;
        assert_refusal(
            &[starting_cursor_successor],
            edge.predecessor,
            &MaturityRootHistoryRefusal::OutpointReused {
                position: 0,
                outpoint: edge.predecessor,
                first_seen: FirstSeen::StartingCursor,
            },
            HistoryClause::SuccessorIsUnique,
            Some(0),
        );

        let refusals = [
            MaturityRootHistoryRefusal::EmptySequence,
            MaturityRootHistoryRefusal::OutpointReused {
                position: 0,
                outpoint: edge.successor,
                first_seen: FirstSeen::StartingCursor,
            },
            MaturityRootHistoryRefusal::PredecessorIsNotTheCursor {
                position: 0,
                cursor: edge.predecessor,
                predecessor: edge.successor,
            },
            MaturityRootHistoryRefusal::OperationIsNotMaturityAnnouncement {
                position: 0,
                operation: OperationId::Cycle,
            },
            MaturityRootHistoryRefusal::CertificateProjectionDisagrees {
                position: 0,
                projection: ProjectionId::BurnEvent,
            },
            MaturityRootHistoryRefusal::CertificateDisagreesWithEdge {
                position: 0,
                side: EdgeSide::Successor,
                edge: edge.successor,
                certificate: edge.predecessor,
            },
            MaturityRootHistoryRefusal::UnrelatedRootEffect {
                position: 0,
                root: RootId::Resv,
                use_kind: RootUse::Succession,
                successor: edge.successor,
            },
            MaturityRootHistoryRefusal::StaticSubtreeDiscontinuity {
                position: 0,
                refusal: StateLinkRefusal::StaticSubtreeDiscontinuity {
                    predecessor: *edge.certificate.predecessor_static.root(),
                    successor: *edge.certificate.successor_static.root(),
                },
            },
        ];
        for refusal in refusals {
            let has_returned_cursor = match &refusal {
                MaturityRootHistoryRefusal::EmptySequence
                | MaturityRootHistoryRefusal::OutpointReused { .. }
                | MaturityRootHistoryRefusal::PredecessorIsNotTheCursor { .. }
                | MaturityRootHistoryRefusal::OperationIsNotMaturityAnnouncement { .. }
                | MaturityRootHistoryRefusal::CertificateProjectionDisagrees { .. }
                | MaturityRootHistoryRefusal::CertificateDisagreesWithEdge { .. }
                | MaturityRootHistoryRefusal::UnrelatedRootEffect { .. }
                | MaturityRootHistoryRefusal::StaticSubtreeDiscontinuity { .. } => false,
            };
            assert!(!has_returned_cursor);
            assert_eq!(
                refusal.position().is_none(),
                matches!(refusal, MaturityRootHistoryRefusal::EmptySequence)
            );
        }
    }
}
