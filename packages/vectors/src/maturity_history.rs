//! Typed STATE root-history edges under Guide 14 §§14.6, 17.1, and 17.3 (`T11-114`).

use std::collections::BTreeMap;

use architecture::ids::{DeallocatorId, ObjectId, OperationId, ProjectionId, RootId, RootUse};
use architecture::spec::ARCHITECTURE;
use linker::{StateLinkRefusal, state_bundle_continuity};
use tapscript::StateStaticSubtree;
use transaction::bytes::Outpoint;

use crate::maturity_continuity::{ValidatedMaturityContinuity, submitted_transaction_identities};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maturity_closure::closure_target;
    use crate::maturity_continuity::project_maturity_continuity;
    use crate::maturity_continuity::tests::variable_archived;
    use tapscript::StateStaticNode;
    use transaction::bytes::Txid;

    fn accepted_edge() -> StateRootEdge {
        let continuity = project_maturity_continuity(variable_archived().input())
            .expect("accepted archive projects");
        StateRootEdge::from_continuity(&continuity)
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
        assert_refusal(
            &[edge.clone(), edge.clone()],
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
        assert_ne!(original, &rebuilt);
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
