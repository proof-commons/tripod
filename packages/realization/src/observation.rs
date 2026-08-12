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
    pub value: ObservedValue,
    pub owner: Option<OwnerId>,
    pub representation: RepresentationMode,
}

/// The amount an observed object carries, if the protocol may read it.
///
/// Sponsor erasure is structural here rather than advisory (S3). An
/// ordinary sponsor L-BTC amount is sponsor-local data: its security
/// role is exact membership, owner authorization, isolation, and
/// substrate-enforced conservation, none of which is an amount. The
/// projection therefore does not carry one, so no protocol relation
/// can read what is not present, and a future backend cannot add a
/// read merely because its target exposes an introspection primitive.
///
/// Protocol object amounts remain exact and readable: conservation of
/// *protocol* value is this layer's own obligation, unlike sponsor
/// conservation, which belongs to the substrate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObservedValue {
    /// A protocol-owned amount, and an operand of protocol relations.
    Protocol(ProtocolAmount),
    /// A sponsor-local amount, erased by the protocol projection. The
    /// amount exists on chain; it is not this layer's to read.
    SponsorOpaque,
}

impl ObservedValue {
    /// The protocol amount, or `None` when the value is sponsor-local.
    ///
    /// Callers must handle the erased case explicitly; there is
    /// deliberately no defaulting accessor, because silently reading a
    /// sponsor amount as zero would reintroduce exactly the read this
    /// type removes.
    #[must_use]
    pub const fn protocol(self) -> Option<ProtocolAmount> {
        match self {
            Self::Protocol(amount) => Some(amount),
            Self::SponsorOpaque => None,
        }
    }

    /// Whether this is a readable protocol amount equal to `amount`.
    #[must_use]
    pub fn is(self, amount: ProtocolAmount) -> bool {
        self.protocol() == Some(amount)
    }

    /// Whether this is a readable protocol amount of zero.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.is(ProtocolAmount::ZERO)
    }
}

/// One observed issuance: an authority mints `amount` of `asset` into
/// the destination objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedIssuance {
    pub asset: AssetId,
    pub authority: AssetId,
    pub authority_input: ObservedObjectRef,
    pub amount: ProtocolAmount,
    pub destinations: Vec<ObservedObjectRef>,
}

/// One observed destruction leg inside a flow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedDestructionLeg {
    pub tag: TagId,
    pub amount: ProtocolAmount,
}

/// One exact observed canonical flow: a source set, a destination set,
/// an optional movement kind, and zero or more destruction legs.
///
/// A flow may carry both a movement and destruction legs (partial clear,
/// terminal settlement); they intentionally share this flow's single
/// source set. Across flows, no source is reused — that is the
/// distinction the flattened delta rows could not express.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedCanonicalFlow {
    pub asset: AssetId,
    pub sources: Vec<ObservedObjectRef>,
    pub destinations: Vec<ObservedObjectRef>,
    pub movement_kind: Option<DeltaKind>,
    pub destructions: Vec<ObservedDestructionLeg>,
}

/// The exact observed canonical partition of one operation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ObservedCanonicalPartition {
    pub issuances: Vec<ObservedIssuance>,
    pub flows: Vec<ObservedCanonicalFlow>,
}

/// One exact open-value flow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedOpenFlow {
    pub kind: OpenFlowKind,
    pub sources: Vec<ObservedObjectRef>,
    pub destinations: Vec<ObservedObjectRef>,
    pub fee: ProtocolAmount,
}

/// What actually happened to one root in one observed operation.
///
/// This is an *event*, not a policy. `architecture::RootUse` describes
/// what an operation is *allowed* to do to a root; an operation either
/// succeeds a root or terminates it, and "forbidden" is not something
/// that can be observed — it is the absence of any effect. Encoding an
/// actual effect in the policy enum made a
/// `RootUse::SuccessionOrTermination` policy accept only whichever
/// single value the adapter happened to choose, so ordinary
/// (non-sealing) redemption would have failed conformance against a
/// policy that explicitly permits succession.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObservedRootEffectKind {
    Succession,
    Termination,
}

impl ObservedRootEffectKind {
    /// Whether one root-use policy admits this actual effect.
    ///
    /// `Forbidden` admits no effect at all; the absence of an effect is
    /// checked by the caller, because no value of this type can express
    /// it.
    #[must_use]
    pub const fn permitted_by(self, policy: RootUse) -> bool {
        match policy {
            RootUse::Forbidden => false,
            RootUse::Succession => matches!(self, Self::Succession),
            RootUse::SuccessionOrTermination => true,
        }
    }
}

/// One actual root effect derived from a transition certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedRootEffect {
    pub root: RootId,
    pub effect: ObservedRootEffectKind,
}

/// Complete primitive observation of one operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationObservation {
    pub operation: OperationId,
    pub objects: Vec<ObservedObject>,
    pub protocol_signers: BTreeSet<OwnerId>,
    pub sponsor_signers: BTreeSet<OwnerId>,
    pub canonical_partition: ObservedCanonicalPartition,
    pub open_flows: Vec<ObservedOpenFlow>,
    pub root_effects: Vec<ObservedRootEffect>,
    pub projections: BTreeSet<ProjectionId>,
    pub bounds: BTreeMap<BoundId, Count>,
}

impl OperationObservation {
    /// Validate local reference consistency and canonicalize repeated
    /// collections.
    ///
    /// # The open-flow boundary (SR2-07)
    ///
    /// Normalization establishes *reference structure*; relation
    /// evaluation establishes operation-specific role semantics. So
    /// this function requires that an open-flow source is an input
    /// reference and a destination an output reference, that no
    /// reference is claimed by two open flows, and that no CPFP anchor
    /// joins an open flow — the same three facts the canonical
    /// partition already has to satisfy, and all three properties of
    /// how the observation refers to its own objects rather than
    /// judgements about the transaction.
    ///
    /// Anchor exclusion belongs here because it is family-structural
    /// and kind-independent: the model kernel rejects an anchor as an
    /// open-flow source or destination unconditionally, whatever the
    /// flow kind, since the anchor family's whole purpose is to stand
    /// outside the open-value partition.
    ///
    /// Complete L-BTC partitioning deliberately does *not* belong here,
    /// and stays with the relations:
    ///
    /// - it is operation-specific. The kernel's completeness rule runs
    ///   against the branch's admitted open-flow kinds; this function
    ///   holds no architecture and cannot know which kinds the observed
    ///   operation admits, or whether it has an open-value region at
    ///   all;
    /// - it is already owned. For the current pilots the claimant is
    ///   the fee-sponsor region, and `SponsorIsolation` requires exact
    ///   membership alongside the family, asset, and owner-authorization
    ///   facts about those same members. Absorbing completeness here
    ///   would make that relation partly vacuous and quietly remove a
    ///   semantic obligation from the relation census the compiler
    ///   consumes;
    /// - it would destroy the verdict. An unclaimed sponsor member is a
    ///   real transaction breaking a real rule, and must be *evaluable*
    ///   and reported as a sponsor-isolation failure. Rejecting it here
    ///   would return a malformed-observation error instead, which says
    ///   the observation could not be read rather than that the
    ///   operation does not conform.
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

        for issuance in &mut self.canonical_partition.issuances {
            check_reference(issuance.authority_input, ObservedSide::Input, &known)?;

            issuance.destinations.sort();

            if has_duplicates(&issuance.destinations) {
                return Err(RealizationError::DuplicateObservedReference);
            }

            for destination in &issuance.destinations {
                check_reference(*destination, ObservedSide::Output, &known)?;
            }
        }

        for flow in &mut self.canonical_partition.flows {
            flow.sources.sort();
            flow.destinations.sort();

            if has_duplicates(&flow.sources) || has_duplicates(&flow.destinations) {
                return Err(RealizationError::DuplicateObservedReference);
            }

            for source in &flow.sources {
                check_reference(*source, ObservedSide::Input, &known)?;
            }

            for destination in &flow.destinations {
                check_reference(*destination, ObservedSide::Output, &known)?;
            }

            flow.destructions.sort_by_key(|leg| leg.tag.code());

            for pair in flow.destructions.windows(2) {
                if pair[0].tag == pair[1].tag {
                    return Err(RealizationError::DuplicateObservedReference);
                }
            }
        }

        self.canonical_partition.flows.sort_by(|left, right| {
            (
                left.asset.code(),
                &left.sources,
                &left.destinations,
                left.movement_kind.map(DeltaKind::code),
            )
                .cmp(&(
                    right.asset.code(),
                    &right.sources,
                    &right.destinations,
                    right.movement_kind.map(DeltaKind::code),
                ))
        });

        self.canonical_partition
            .issuances
            .sort_by_key(|issuance| (issuance.asset.code(), issuance.amount));

        validate_partition_reference_layout(&self.canonical_partition)?;

        // The CPFP anchor family stands outside every open-value
        // partition, for every flow kind, so it is collected once here
        // rather than decided per operation.
        let anchors = self
            .objects
            .iter()
            .filter(|object| object.kind == ObservedObjectKind::Declared(ObjectId::CpfpAnchor))
            .map(|object| object.reference)
            .collect::<BTreeSet<_>>();

        normalize_open_flows(&mut self.open_flows, &known, &anchors)?;

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

fn check_reference(
    reference: ObservedObjectRef,
    side: ObservedSide,
    known: &BTreeSet<ObservedObjectRef>,
) -> Result<(), RealizationError> {
    if reference.side != side {
        return Err(RealizationError::WrongObservedReferenceSide(reference));
    }

    if !known.contains(&reference) {
        return Err(RealizationError::UnknownObservedObject(reference));
    }

    Ok(())
}

/// The open-flow reference structure (SR2-07): canonical order within
/// each flow, exact sides, no reference claimed by two flows, and no
/// CPFP anchor inside any of them.
///
/// See the boundary note on
/// [`OperationObservation::validate_and_normalize`] for why complete
/// partitioning is deliberately not established here.
fn normalize_open_flows(
    open_flows: &mut [ObservedOpenFlow],
    known: &BTreeSet<ObservedObjectRef>,
    anchors: &BTreeSet<ObservedObjectRef>,
) -> Result<(), RealizationError> {
    let mut claimed_sources = BTreeSet::new();
    let mut claimed_destinations = BTreeSet::new();

    for flow in open_flows {
        flow.sources.sort();
        flow.destinations.sort();

        if has_duplicates(&flow.sources) || has_duplicates(&flow.destinations) {
            return Err(RealizationError::DuplicateObservedReference);
        }

        for (references, side, claimed) in [
            (&flow.sources, ObservedSide::Input, &mut claimed_sources),
            (
                &flow.destinations,
                ObservedSide::Output,
                &mut claimed_destinations,
            ),
        ] {
            for reference in references {
                check_reference(*reference, side, known)?;

                if anchors.contains(reference) {
                    return Err(RealizationError::AnchorInObservedOpenFlow(*reference));
                }

                if !claimed.insert(*reference) {
                    return Err(RealizationError::ObservedOpenFlowOverlap(*reference));
                }
            }
        }
    }

    Ok(())
}

/// The exact between-partition reference rule: a source reference is
/// used by exactly one flow, and a destination reference is used by
/// exactly one flow or issuance. Within a single flow, a movement and
/// its destruction legs share that flow's one source set — which is not
/// a reuse, because the flow has a single source list.
fn validate_partition_reference_layout(
    partition: &ObservedCanonicalPartition,
) -> Result<(), RealizationError> {
    let mut source_uses = BTreeSet::new();
    let mut destination_uses = BTreeSet::new();

    for flow in &partition.flows {
        for source in &flow.sources {
            if !source_uses.insert(*source) {
                return Err(RealizationError::ObservedCanonicalPartitionOverlap);
            }
        }

        for destination in &flow.destinations {
            if !destination_uses.insert(*destination) {
                return Err(RealizationError::ObservedCanonicalPartitionOverlap);
            }
        }
    }

    for issuance in &partition.issuances {
        for destination in &issuance.destinations {
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
