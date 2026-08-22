//! The typed live-transfer request and its destinations (§12.3, §12.4).
//!
//! # A request is a list of choices, and the list is closed at both ends
//!
//! §12.3 gives two lists: the eight things a request may select and the
//! fourteen it may not. Both are censuses here, and they are censuses
//! rather than prose because the second one is the load-bearing half. A
//! request that could name the coordinator, the family positions, or the
//! constructor bytes would be a request that could build a transaction
//! the ABI never authorized, and the way to make that impossible is not
//! to check for it — it is to have no field to put it in.
//!
//! So [`UnselectableRequestFacet`] names each refused choice and says
//! where it actually comes from, and every one of them is absent from
//! [`LiveTransferRequest`] structurally. The census is what makes that
//! absence auditable: a field added later for one of those fourteen
//! would contradict a doc comment that names it, and the test that pairs
//! the two censuses is what a reviewer reads instead of the struct.
//!
//! # The sponsor is asked for, not described
//!
//! A request selects an *optional sponsor capability*, and a capability
//! is an adapter rather than a bag of values. So a request carries no
//! sponsor outpoints, no fee, and no sponsor-change program: it records
//! the ask, and the capability supplies the rest at construction time.
//! §1.9 is why the sponsor-change program in particular is not here — a
//! request that named it could redirect a sponsor's own change to
//! somewhere the sponsor never agreed to.
//!
//! # A destination is an owner and a value, and nothing else
//!
//! §12.4 keeps blinding and signing material out of semantic
//! destination identity, and §1.10 keeps it out of this crate
//! altogether. [`LiveReceiptDestination`] therefore has exactly two
//! fields. What a private representation needs beyond them arrives as
//! explicit public test randomness, which is a fixture and says so.

use std::collections::{BTreeMap, BTreeSet};

use linker::OwnerParameter;
use linker::live_backend::LiveTransferRepresentationPlan;

use crate::bytes::Outpoint;
use crate::error::TransactionRefusal;

/// One destination's semantic value.
///
/// A newtype over the target's explicit amount width rather than
/// `realization::ProtocolAmount`, which is the semantic layer's own type
/// and belongs to a package this one does not depend on. The two agree
/// on what a value is; they disagree on who is allowed to say so, and
/// the transaction layer is not.
///
/// Zero is refused. §6.4 establishes positive semantic value by valid
/// object construction rather than by disclosing an amount, and a
/// destination worth nothing is not an object anybody constructed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolValue(u64);

impl ProtocolValue {
    /// The value one positive amount names.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::DestinationValueIsZero`] for zero.
    pub const fn new(amount: u64) -> Result<Self, TransactionRefusal> {
        if amount == 0 {
            return Err(TransactionRefusal::DestinationValueIsZero);
        }
        Ok(Self(amount))
    }

    /// The amount, at the target's explicit width.
    #[must_use]
    pub const fn amount(self) -> u64 {
        self.0
    }
}

/// One live receipt destination (§12.4).
///
/// Two fields, and the pair is the semantic identity the projection
/// compares. Target-specific blinding and signing material lives in
/// separate authorized or test-only capabilities (§1.10) and has no
/// field here, so a destination cannot carry one by accident and two
/// destinations cannot differ by one.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiveReceiptDestination {
    owner: OwnerParameter,
    value: ProtocolValue,
}

impl LiveReceiptDestination {
    /// The destination creating `value` for `owner`.
    #[must_use]
    pub const fn new(owner: OwnerParameter, value: ProtocolValue) -> Self {
        Self { owner, value }
    }

    /// The owner the receipt is created for.
    #[must_use]
    pub const fn owner(&self) -> &OwnerParameter {
        &self.owner
    }

    /// The semantic value the receipt carries.
    #[must_use]
    pub const fn value(&self) -> ProtocolValue {
        self.value
    }
}

/// Which of the two transaction forms a request asks for (§12.5).
///
/// A named pair rather than a boolean. The sponsored and sponsorless
/// forms differ in their input census, their output census, and their
/// target version, and a boolean parameter at a call site says which one
/// only to a reader holding the signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RequestedForm {
    /// A sponsor suffix funds the target fee role.
    Sponsored,
    /// No sponsor region exists, and the request funds no fee.
    Sponsorless,
}

impl RequestedForm {
    /// Whether the form asks for a sponsor region.
    #[must_use]
    pub const fn sponsored(self) -> bool {
        matches!(self, Self::Sponsored)
    }
}

/// Whether a request asks for the optional sponsor-change role (§12.2).
///
/// The presence is the selection and the program is not. §1.9 keeps
/// sponsor values and sponsor destinations opaque to protocol data, so
/// what the change output actually pays to comes from the sponsor's own
/// capability and is compared against the deployment's admitted program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SponsorChangeRequest {
    /// The sponsor's residual gets its own output.
    Requested,
    /// The sponsor keeps no change in this transaction.
    NotRequested,
}

impl SponsorChangeRequest {
    /// Whether a change output is asked for.
    #[must_use]
    pub const fn requested(self) -> bool {
        matches!(self, Self::Requested)
    }
}

/// The explicit public randomness a private test construction consumes.
///
/// Public test material in the sense
/// `(´[ADR015-rule:security:test-material]´)` fixes, and named for what
/// it is at every use. §12.8's expected model is central public-fixture
/// construction, which is reproducible exactly because the randomness is
/// an input the caller has already published rather than something this
/// crate generates.
///
/// There is no constructor that takes randomness from a source a
/// production secret could come from, no generation, and no storage.
/// §1.10 refuses a first-party production interface for blinding
/// material, and this is not the beginning of one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicTestRandomness([u8; 32]);

impl PublicTestRandomness {
    /// The randomness one published fixture value names.
    #[must_use]
    pub const fn from_published_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The published bytes.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// One choice §12.3 admits a request making.
///
/// Eight members, which is the whole list. Paired with
/// [`UnselectableRequestFacet`] so that the two censuses together are
/// the request's contract, readable without reading the struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SelectableRequestFacet {
    /// Which live receipts are consumed.
    ReceiptInputOutpoints,
    /// The ordered destination entries.
    OrderedDestinationEntries,
    /// Each destination's owner.
    DestinationOwners,
    /// Each destination's semantic value.
    DestinationSemanticValues,
    /// One admitted representation plan.
    RepresentationPlan,
    /// Whether a sponsor capability is offered.
    OptionalSponsorCapability,
    /// Whether the sponsor takes a change output.
    OptionalSponsorChangeDestination,
    /// Explicit public test randomness, where the representation needs
    /// it.
    ExplicitPublicTestRandomness,
}

impl SelectableRequestFacet {
    /// The complete census, in §12.3's own order.
    pub const ALL: &'static [Self] = &[
        Self::ReceiptInputOutpoints,
        Self::OrderedDestinationEntries,
        Self::DestinationOwners,
        Self::DestinationSemanticValues,
        Self::RepresentationPlan,
        Self::OptionalSponsorCapability,
        Self::OptionalSponsorChangeDestination,
        Self::ExplicitPublicTestRandomness,
    ];
}

/// One choice §12.3 refuses a request, and where it comes from instead.
///
/// Fourteen members. §12.3's own bullet list is fourteen items long, and
/// each is here under the name the guide gives it, so that a later
/// reading of the guide can be checked against this census rather than
/// against the struct's field list.
///
/// Every one of them is absent from [`LiveTransferRequest`]
/// structurally. There is no field, no builder argument, and no setter,
/// which is the crate's standing preference over a refusal whose failing
/// branch could only be reasoned about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnselectableRequestFacet {
    /// Who owned a consumed receipt. Authenticated from the input's own
    /// spending predicate.
    PredecessorOwner,
    /// A consumed receipt's class. Structural, from its constructor.
    PredecessorClass,
    /// A consumed receipt's asset. The deployment's protocol asset.
    PredecessorAsset,
    /// A created receipt's class. Structural, from its constructor.
    OutputClass,
    /// A created receipt's asset. The deployment's protocol asset.
    OutputAsset,
    /// The exact bytes of an output constructor. Derived from the linked
    /// constructor the destination owner selects.
    ConstructorBytes,
    /// The target program an output pays to. Derived from the same
    /// constructor's committed tree.
    TargetProgram,
    /// Which input coordinates the family. Input zero, by the ABI's
    /// coordinator rule.
    Coordinator,
    /// Where a family's run of positions sits. From the shape's family
    /// ranges.
    FamilyPositions,
    /// The order of a witness's components. From the linked leaf's
    /// witness census.
    WitnessOrder,
    /// Which output carries the target's fee. From the layout's fee
    /// role.
    TargetFeeRole,
    /// An issuance. The live-transfer relation has none.
    Issuance,
    /// A destruction. The live-transfer relation has none.
    Destruction,
    /// A narrowed semantic projection. The evidence layer's, not a
    /// caller's.
    SpecializedProjection,
}

impl UnselectableRequestFacet {
    /// The complete census, in §12.3's own order.
    pub const ALL: &'static [Self] = &[
        Self::PredecessorOwner,
        Self::PredecessorClass,
        Self::PredecessorAsset,
        Self::OutputClass,
        Self::OutputAsset,
        Self::ConstructorBytes,
        Self::TargetProgram,
        Self::Coordinator,
        Self::FamilyPositions,
        Self::WitnessOrder,
        Self::TargetFeeRole,
        Self::Issuance,
        Self::Destruction,
        Self::SpecializedProjection,
    ];
}

/// One typed live-transfer request (§12.3).
///
/// Six fields for eight selections: the destination entries carry three
/// of the eight between them, because §12.4 makes an owner and a value
/// one object rather than two parallel lists that could come to differ
/// in length.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveTransferRequest {
    receipts: BTreeSet<Outpoint>,
    destinations: Vec<LiveReceiptDestination>,
    representation: LiveTransferRepresentationPlan,
    form: RequestedForm,
    sponsor_change: SponsorChangeRequest,
    randomness: Option<PublicTestRandomness>,
}

impl LiveTransferRequest {
    /// The request transferring exactly these receipts to exactly these
    /// destinations.
    ///
    /// Duplicate receipt outpoints are rejected before sorting rather
    /// than collapsed by it (§12.1). A set built by insertion would
    /// silently accept a caller who named one outpoint twice and consume
    /// a smaller family than the caller asked for.
    ///
    /// Destination entries are *not* deduplicated, and the difference is
    /// the point: two entries with the same owner and the same value are
    /// a split into equal halves, which is an ordinary transfer, where
    /// one outpoint named twice is a family nobody has.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::EmptyReceiptSelection`] for an empty
    /// consumed family; [`TransactionRefusal::DuplicateReceiptOutpoint`]
    /// when one outpoint is named more than once;
    /// [`TransactionRefusal::EmptyDestinationCensus`] when nothing is
    /// created; [`TransactionRefusal::SponsorChangeWithoutSponsoredForm`]
    /// for a sponsorless request asking for sponsor change (§12.5);
    /// [`TransactionRefusal::PublicTestRandomnessWithoutPrivateForm`]
    /// when randomness is offered to an explicit request; and
    /// [`TransactionRefusal::PrivateFormWithoutPublicTestRandomness`]
    /// when a private-committed request offers none.
    pub fn new(
        receipts: impl IntoIterator<Item = Outpoint>,
        destinations: impl IntoIterator<Item = LiveReceiptDestination>,
        representation: LiveTransferRepresentationPlan,
        form: RequestedForm,
        sponsor_change: SponsorChangeRequest,
        randomness: Option<PublicTestRandomness>,
    ) -> Result<Self, TransactionRefusal> {
        let mut selected = BTreeSet::new();
        for outpoint in receipts {
            if !selected.insert(outpoint) {
                return Err(TransactionRefusal::DuplicateReceiptOutpoint(outpoint));
            }
        }
        if selected.is_empty() {
            return Err(TransactionRefusal::EmptyReceiptSelection);
        }

        let destinations: Vec<_> = destinations.into_iter().collect();
        if destinations.is_empty() {
            return Err(TransactionRefusal::EmptyDestinationCensus);
        }

        if sponsor_change.requested() && !form.sponsored() {
            return Err(TransactionRefusal::SponsorChangeWithoutSponsoredForm);
        }

        match (representation, randomness) {
            (LiveTransferRepresentationPlan::Explicit, Some(_)) => {
                return Err(TransactionRefusal::PublicTestRandomnessWithoutPrivateForm);
            }
            (LiveTransferRepresentationPlan::PrivateCommitted, None) => {
                return Err(TransactionRefusal::PrivateFormWithoutPublicTestRandomness);
            }
            _ => {}
        }

        Ok(Self {
            receipts: selected,
            destinations,
            representation,
            form,
            sponsor_change,
            randomness,
        })
    }

    /// The selected receipt outpoints, in canonical order.
    ///
    /// Canonical order is the set's own order, which is ascending by
    /// transaction identifier and then by index — §12.1's declared
    /// ordering, reached by construction rather than by a later sort
    /// somebody could forget.
    #[must_use]
    pub const fn receipts(&self) -> &BTreeSet<Outpoint> {
        &self.receipts
    }

    /// The destination entries, in typed request order.
    ///
    /// Request order is ABI presentation order (§12.2) and is committed
    /// by every owner signature. It is not semantic object identity;
    /// [`Self::destination_multiset`] is what a semantic comparison
    /// reads.
    #[must_use]
    pub fn destinations(&self) -> &[LiveReceiptDestination] {
        &self.destinations
    }

    /// The destination multiset the semantic projection compares
    /// (§12.2).
    ///
    /// A count per distinct entry rather than a set, because two
    /// destinations of the same owner and value are two receipts and a
    /// set would report them as one.
    #[must_use]
    pub fn destination_multiset(&self) -> BTreeMap<LiveReceiptDestination, usize> {
        let mut counts = BTreeMap::new();
        for destination in &self.destinations {
            *counts.entry(destination.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// The distinct destination owners, in canonical order.
    #[must_use]
    pub fn destination_owners(&self) -> BTreeSet<OwnerParameter> {
        self.destinations
            .iter()
            .map(|destination| destination.owner.clone())
            .collect()
    }

    /// The total semantic value the destinations carry.
    ///
    /// `None` on overflow of the target's explicit width, which is a
    /// request nothing can build rather than a sum to report.
    #[must_use]
    pub fn destination_total(&self) -> Option<u64> {
        self.destinations
            .iter()
            .try_fold(0_u64, |total, destination| {
                total.checked_add(destination.value.amount())
            })
    }

    /// The admitted representation plan.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        self.representation
    }

    /// Which transaction form the request asks for.
    #[must_use]
    pub const fn form(&self) -> RequestedForm {
        self.form
    }

    /// Whether the request asks for the sponsor-change role.
    #[must_use]
    pub const fn sponsor_change(&self) -> SponsorChangeRequest {
        self.sponsor_change
    }

    /// The explicit public test randomness, where the representation
    /// needs it.
    #[must_use]
    pub const fn public_test_randomness(&self) -> Option<PublicTestRandomness> {
        self.randomness
    }

    /// Which of §12.3's admitted choices this request actually makes.
    ///
    /// Every request makes the first five; the last three are optional
    /// and appear only when exercised. A reader comparing this against
    /// [`SelectableRequestFacet::ALL`] learns what a particular request
    /// left to the ABI.
    #[must_use]
    pub fn selected(&self) -> BTreeSet<SelectableRequestFacet> {
        let mut selected = BTreeSet::from([
            SelectableRequestFacet::ReceiptInputOutpoints,
            SelectableRequestFacet::OrderedDestinationEntries,
            SelectableRequestFacet::DestinationOwners,
            SelectableRequestFacet::DestinationSemanticValues,
            SelectableRequestFacet::RepresentationPlan,
        ]);
        if self.form.sponsored() {
            selected.insert(SelectableRequestFacet::OptionalSponsorCapability);
        }
        if self.sponsor_change.requested() {
            selected.insert(SelectableRequestFacet::OptionalSponsorChangeDestination);
        }
        if self.randomness.is_some() {
            selected.insert(SelectableRequestFacet::ExplicitPublicTestRandomness);
        }
        selected
    }
}
