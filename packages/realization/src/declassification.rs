//! Dependency-derived public-availability and declassification analysis.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};

use crate::{FactId, RelationId, RelationKind, RelationSubject, TransactionSide};

/// Reason one fact must be publicly available or is newly disclosed.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DisclosureReason {
    PublicState,
    PublicEvent,
    PublicInterface,
    PermissionlessConstructibility { relation: RelationId },
}

/// Node in a future direct Petgraph disclosure graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisclosureNode {
    Fact(FactId),
    Seed {
        fact: FactId,
        reason: DisclosureReason,
    },
}

/// Edge in a future direct Petgraph disclosure graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisclosureEdge {
    DependsOn,
    Seeds,
}

/// Required-public and newly-disclosed fact analysis.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeclassificationAnalysis {
    pub required_public: BTreeMap<FactId, BTreeSet<DisclosureReason>>,
    pub newly_disclosed: BTreeMap<FactId, BTreeSet<DisclosureReason>>,
    pub retained_private: BTreeSet<FactId>,
}

/// Phase-1 pilot disclosure analysis.
#[must_use]
pub fn phase1_declassification() -> DeclassificationAnalysis {
    let compact_constructibility = RelationId::new(
        OperationId::CompactAsh,
        RelationKind::Constructibility,
        RelationSubject::Operation,
    );
    let reason = DisclosureReason::PermissionlessConstructibility {
        relation: compact_constructibility,
    };
    let mut required_public = BTreeMap::new();

    for side in [TransactionSide::Input, TransactionSide::Output] {
        required_public.insert(
            FactId::FamilyAmount {
                operation: OperationId::CompactAsh,
                side,
                object: ObjectId::Ash,
            },
            BTreeSet::from([reason.clone()]),
        );
    }

    let retained_private = [TransactionSide::Input, TransactionSide::Output]
        .into_iter()
        .map(|side| FactId::FamilyAmount {
            operation: OperationId::TransferLive,
            side,
            object: ObjectId::ReceiptLive,
        })
        .collect();

    DeclassificationAnalysis {
        required_public,
        newly_disclosed: BTreeMap::new(),
        retained_private,
    }
}
