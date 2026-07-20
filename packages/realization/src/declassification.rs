//! Dependency-derived public-availability and declassification analysis.

use std::collections::{BTreeMap, BTreeSet};

use crate::{FactId, RelationId};

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
