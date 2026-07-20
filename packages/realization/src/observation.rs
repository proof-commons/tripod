//! Primitive target-independent observation of one concrete operation.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    AssetId, BoundId, ObjectId, OpenFlowKind, OperationId, ProjectionId, RootId, RootUse,
};

use crate::{Count, OwnerId, ProtocolAmount, RepresentationMode};

/// Object side in an observed transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObservedSide {
    Input,
    Output,
}

/// Stable reference local to one semantic observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObservedObjectRef {
    pub side: ObservedSide,
    pub ordinal: u32,
}

/// Observed asset identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObservedAsset {
    Declared(AssetId),
    Foreign(u32),
}

/// Observed object family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObservedObjectKind {
    Declared(ObjectId),
    Unrecognized,
}

/// Primitive semantic object observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedObject {
    pub reference: ObservedObjectRef,
    pub kind: ObservedObjectKind,
    pub asset: ObservedAsset,
    pub value: ProtocolAmount,
    pub owner: Option<OwnerId>,
    pub representation: RepresentationMode,
}

/// One exact open-value flow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedOpenFlow {
    pub kind: OpenFlowKind,
    pub sources: Vec<ObservedObjectRef>,
    pub destinations: Vec<ObservedObjectRef>,
    pub fee: ProtocolAmount,
}

/// Root use derived from a transition certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedRootEffect {
    pub root: RootId,
    pub use_kind: RootUse,
}

/// Complete primitive observation of one operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationObservation {
    pub operation: OperationId,
    pub objects: Vec<ObservedObject>,
    pub protocol_signers: BTreeSet<OwnerId>,
    pub sponsor_signers: BTreeSet<OwnerId>,
    pub open_flows: Vec<ObservedOpenFlow>,
    pub root_effects: Vec<ObservedRootEffect>,
    pub projections: BTreeSet<ProjectionId>,
    pub bounds: BTreeMap<BoundId, Count>,
}

impl OperationObservation {
    /// Resolve one semantic object reference.
    #[must_use]
    pub fn object(&self, reference: ObservedObjectRef) -> Option<&ObservedObject> {
        self.objects
            .iter()
            .find(|object| object.reference == reference)
    }

    /// Declared objects on one side and in one family.
    pub fn declared_objects(
        &self,
        side: ObservedSide,
        kind: ObjectId,
    ) -> impl Iterator<Item = &ObservedObject> {
        self.objects.iter().filter(move |object| {
            object.reference.side == side && object.kind == ObservedObjectKind::Declared(kind)
        })
    }
}
