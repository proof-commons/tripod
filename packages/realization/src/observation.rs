//! Primitive target-independent observation of one concrete operation.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    AssetId, BoundId, DeltaKind, ObjectId, OpenFlowKind, OperationId, ProjectionId, RootId,
    RootUse, TagId,
};

use crate::{Count, OwnerId, ProtocolAmount, RealizationError, RepresentationMode};

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

/// One canonical closed-asset delta derived from a transition certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedCanonicalDelta {
    pub asset: AssetId,
    pub kind: DeltaKind,
    pub amount: ProtocolAmount,
    pub sources: Vec<ObservedObjectRef>,
    pub destinations: Vec<ObservedObjectRef>,
    pub destruction_tag: Option<TagId>,
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
    pub canonical_deltas: Vec<ObservedCanonicalDelta>,
    pub open_flows: Vec<ObservedOpenFlow>,
    pub root_effects: Vec<ObservedRootEffect>,
    pub projections: BTreeSet<ProjectionId>,
    pub bounds: BTreeMap<BoundId, Count>,
}

impl OperationObservation {
    /// Validate local reference consistency and canonicalize repeated collections.
    pub fn validate_and_normalize(mut self) -> Result<Self, RealizationError> {
        self.objects.sort_by_key(|object| object.reference);

        for pair in self.objects.windows(2) {
            if pair[0].reference == pair[1].reference {
                return Err(RealizationError::DuplicateObservedObject(pair[0].reference));
            }
        }

        let known = self
            .objects
            .iter()
            .map(|object| object.reference)
            .collect::<BTreeSet<_>>();

        for delta in &mut self.canonical_deltas {
            delta.sources.sort();
            delta.destinations.sort();

            if has_duplicates(&delta.sources) || has_duplicates(&delta.destinations) {
                return Err(RealizationError::DuplicateObservedReference);
            }

            for source in &delta.sources {
                if source.side != ObservedSide::Input {
                    return Err(RealizationError::WrongObservedReferenceSide(*source));
                }

                if !known.contains(source) {
                    return Err(RealizationError::UnknownObservedObject(*source));
                }
            }

            for destination in &delta.destinations {
                if destination.side != ObservedSide::Output {
                    return Err(RealizationError::WrongObservedReferenceSide(*destination));
                }

                if !known.contains(destination) {
                    return Err(RealizationError::UnknownObservedObject(*destination));
                }
            }
        }

        self.canonical_deltas.sort_by(|left, right| {
            (
                left.asset.code(),
                left.kind.code(),
                left.amount,
                &left.sources,
                &left.destinations,
                left.destruction_tag.map(TagId::code),
            )
                .cmp(&(
                    right.asset.code(),
                    right.kind.code(),
                    right.amount,
                    &right.sources,
                    &right.destinations,
                    right.destruction_tag.map(TagId::code),
                ))
        });

        validate_delta_reference_partition(&self.canonical_deltas)?;

        for flow in &mut self.open_flows {
            flow.sources.sort();
            flow.destinations.sort();

            if has_duplicates(&flow.sources) || has_duplicates(&flow.destinations) {
                return Err(RealizationError::DuplicateObservedReference);
            }

            for reference in flow.sources.iter().chain(&flow.destinations) {
                if !known.contains(reference) {
                    return Err(RealizationError::UnknownObservedObject(*reference));
                }
            }
        }

        self.open_flows.sort_by(|left, right| {
            (
                left.kind.code(),
                left.fee,
                &left.sources,
                &left.destinations,
            )
                .cmp(&(
                    right.kind.code(),
                    right.fee,
                    &right.sources,
                    &right.destinations,
                ))
        });

        self.root_effects.sort_by_key(|effect| effect.root.code());

        for pair in self.root_effects.windows(2) {
            if pair[0].root == pair[1].root {
                return Err(RealizationError::DuplicateObservedRoot(pair[0].root));
            }
        }

        Ok(self)
    }

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

fn has_duplicates<T: Ord>(values: &[T]) -> bool {
    values.windows(2).any(|pair| pair[0] == pair[1])
}

fn validate_delta_reference_partition(
    deltas: &[ObservedCanonicalDelta],
) -> Result<(), RealizationError> {
    let mut source_uses = BTreeSet::new();
    let mut destination_uses = BTreeSet::new();

    for delta in deltas {
        for source in &delta.sources {
            if !source_uses.insert(*source) {
                return Err(RealizationError::ObservedCanonicalPartitionOverlap);
            }
        }

        for destination in &delta.destinations {
            if !destination_uses.insert(*destination) {
                return Err(RealizationError::ObservedCanonicalPartitionOverlap);
            }
        }
    }

    Ok(())
}

/// Validate and canonicalize an operation observation.
pub fn validate_observation(
    observation: OperationObservation,
) -> Result<OperationObservation, RealizationError> {
    observation.validate_and_normalize()
}
