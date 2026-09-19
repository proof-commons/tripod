//! The typed maturity-announcement request and its two censuses (§12.3,
//! §2.3).
//!
//! # A request is a list of choices, and the list is closed at both ends
//!
//! §12.3 states the request as three fields and §2.3 states the same
//! selection from the semantic side, as one announced cycle plus the
//! inherited sponsor interface. Both sections then state what a request
//! may *not* say, and that second list is the load-bearing half: a caller
//! who could name the successor program, the internal key, or the control
//! block could build a transaction no ABI derived, and the way to make
//! that impossible is not to check for it — it is to have no field to put
//! it in.
//!
//! So the two censuses here are the request's contract.
//! [`SelectableMaturityRequestFacet`] names the three choices the two
//! sections admit, and [`UnselectableMaturityRequestFacet`] names every
//! choice they refuse together with where the value is actually settled.
//! Both are walkable, which is what makes the absences auditable: a field
//! added later for one of the refused choices would contradict a census a
//! test reads rather than only a comment somebody could skip.
//!
//! # The form and the sponsor-change choice are inherited, not minted
//!
//! §12.3 says the sponsor fields follow the inherited interface, so
//! [`RequestedForm`] and [`SponsorChangeRequest`] are used here as they
//! stand and are not re-exported under this module's name. Inheritance is
//! the claim being made, and one type with one public path is how the
//! claim stays true: a second export would give one thing two names, and
//! a later change to one of the two paths would silently not be a change
//! to the other.
//!
//! # Both forms are representable, and only one of them is buildable
//!
//! A request can ask for the sponsored form even where nothing emits it
//! yet, because a request that could not say it wanted a sponsor region
//! could not be refused for wanting one. The refusal worth having names
//! the carrier the form would need and says the carrier is absent; a
//! request shape that made the question unaskable would replace that
//! refusal with silence.
//!
//! What the request does settle by itself is the one pair that
//! contradicts itself: the sponsorless form has no sponsor region, so
//! there is no residual for a sponsor-change output to carry, and that is
//! true of the request alone without reference to any deployment.

use realization::Cycle;

use crate::error::TransactionRefusal;
use crate::live_request::{RequestedForm, SponsorChangeRequest};

/// One choice §12.3 and §2.3 admit a request making.
///
/// Three members, which is the whole list: §12.3's three fields, in the
/// order it declares them, which is also §2.3's one selected cycle
/// followed by the inherited sponsor interface. Paired with
/// [`UnselectableMaturityRequestFacet`] so that the two censuses together
/// are the request's contract, readable without reading the struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SelectableMaturityRequestFacet {
    /// Which cycle the announcement names.
    AnnouncedCycle,
    /// Which of the two transaction forms is asked for.
    RequestedTransactionForm,
    /// Whether the sponsor takes a change output.
    OptionalSponsorChangeDestination,
}

impl SelectableMaturityRequestFacet {
    /// The complete census, in §12.3's own field order.
    pub const ALL: &'static [Self] = &[
        Self::AnnouncedCycle,
        Self::RequestedTransactionForm,
        Self::OptionalSponsorChangeDestination,
    ];
}

/// One choice §12.3 and §2.3 refuse a request, and where the value comes
/// from instead.
///
/// Fifteen members. §12.3's bullet list is twelve items and §2.3's is
/// thirteen, and ten pairs name one thing between them, so their union is
/// fifteen and not twenty-five. Counting the union rather than either
/// list is what the pairing is for: §2.3 refuses the choice semantically
/// and §12.3 refuses it again at the byte layout, and a census that
/// reported one section's total would understate the contract by exactly
/// the items only the other section names.
///
/// Every one of them is absent from [`MaturityAnnouncementRequest`]
/// structurally. There is no field, no constructor argument, and no
/// setter, which is the crate's standing preference over a refusal whose
/// failing branch could only be reasoned about. [`Self::origin`] states
/// where each value is settled instead, as a sentence a test can read
/// rather than only a comment a reader can.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnselectableMaturityRequestFacet {
    /// Which STATE output the announcement spends.
    ///
    /// §12.3 refuses the outpoint named independently of the current-root
    /// view and §2.3 refuses the predecessor STATE itself; they are one
    /// choice, because naming the outpoint is how a caller would name the
    /// predecessor.
    PredecessorStateInput,
    /// The predecessor's canonical semantic metadata.
    ///
    /// Supplied as a public fact and checked against the predecessor
    /// program rather than taken on the caller's word.
    PredecessorMetadata,
    /// The successor's canonical semantic metadata.
    ///
    /// Derived from the request and the predecessor, so that a successor
    /// and the announcement it answers cannot disagree.
    SuccessorMetadata,
    /// The successor's target program.
    ///
    /// The constructor's output over the derived metadata and the
    /// searched nonce, which is the only place the two can be combined
    /// exactly once.
    SuccessorProgram,
    /// Who the authorized maturity operator is.
    ///
    /// §12.3 refuses the operator key and §2.3 refuses the operator
    /// identity; a key a caller could choose would be an authorization a
    /// caller could choose.
    OperatorIdentity,
    /// The committed static subtree.
    StaticSubtree,
    /// The taproot internal key.
    InternalKey,
    /// The successor's canonical representation nonce.
    ///
    /// §12.3 refuses the nonce and §2.3 refuses its result; both are the
    /// search's, because leastness is a property of a search and not of a
    /// value handed in.
    RepresentationNonce,
    /// The target transaction version.
    TransactionVersion,
    /// The input sequence numbers.
    InputSequence,
    /// Which output carries the target's fee.
    ///
    /// §12.3's target fee role and §2.3's fee role are one choice, and
    /// §12.2 already fixes where it sits.
    TargetFeeRole,
    /// The control block of the executing leaf.
    ControlBlock,
    /// Where each role sits in the input and output censuses.
    ///
    /// §12.1 and §12.2 fix the positions, and §12.1 says in as many words
    /// that a request cannot select another coordinator.
    TransactionPositions,
    /// Which root the predecessor is current against.
    ///
    /// The cursor is the view's, and freshness against it is external
    /// evidence rather than a choice a request records.
    RootCursor,
    /// Whether the target accepts the result.
    ///
    /// Observed after the fact. A request that could state the verdict
    /// would be stating the answer to the question the candidate asks.
    TargetVerdict,
}

impl UnselectableMaturityRequestFacet {
    /// The complete census: §12.3's twelve in its order, then the three
    /// §2.3 adds, in §2.3's.
    pub const ALL: &'static [Self] = &[
        Self::PredecessorStateInput,
        Self::PredecessorMetadata,
        Self::SuccessorMetadata,
        Self::SuccessorProgram,
        Self::OperatorIdentity,
        Self::StaticSubtree,
        Self::InternalKey,
        Self::RepresentationNonce,
        Self::TransactionVersion,
        Self::InputSequence,
        Self::TargetFeeRole,
        Self::ControlBlock,
        Self::TransactionPositions,
        Self::RootCursor,
        Self::TargetVerdict,
    ];

    /// Where the value is settled, since the request does not say.
    ///
    /// Stated as a value rather than only as documentation, because a
    /// census whose origins live in comments stops being evidence the
    /// first time a member arrives without one. A test walks these, so a
    /// member added in silence fails rather than passes.
    #[must_use]
    pub const fn origin(self) -> &'static str {
        match self {
            Self::PredecessorStateInput => "the validated public current-STATE view (§12.4)",
            Self::PredecessorMetadata => "the same view's canonical predecessor metadata",
            Self::SuccessorMetadata => "the constructor search's derivation (§12.5)",
            Self::SuccessorProgram => "the linked bundle's constructor over that metadata",
            Self::OperatorIdentity => "the view's operator public identity",
            Self::StaticSubtree => "the linked bundle's committed static subtree",
            Self::InternalKey => "the deployment's internal key",
            Self::RepresentationNonce => "the constructor search from zero (§12.5)",
            Self::TransactionVersion => "the ABI's version for the form asked for",
            Self::InputSequence => "the ABI's sequence constraint",
            Self::TargetFeeRole => "the output layout's final fee role (§12.2)",
            Self::ControlBlock => "the committed tree, for the executing leaf",
            Self::TransactionPositions => "the input and output layouts (§12.1, §12.2)",
            Self::RootCursor => "the view's current-root binding",
            Self::TargetVerdict => "the reviewed target, observed after the fact",
        }
    }
}

/// One typed maturity-announcement request (§12.3, §2.3).
///
/// Three fields for three selections, under §12.3's own field names.
/// Every other fact the announcement needs is settled elsewhere, and
/// [`UnselectableMaturityRequestFacet`] says where.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MaturityAnnouncementRequest {
    announced_cycle: Cycle,
    requested_form: RequestedForm,
    sponsor_change: SponsorChangeRequest,
}

impl MaturityAnnouncementRequest {
    /// The request announcing `announced_cycle` in `requested_form`.
    ///
    /// A request is data, and nearly everything that could be wrong with
    /// one is wrong against something the request does not carry: whether
    /// the announced cycle falls inside the admissible lead window is a
    /// question about lead bounds and a current cycle, and whether a
    /// sponsor region can be funded at all is a question about a sponsor's
    /// carrier. Both belong to construction, and answering either here
    /// would mean a request validated against a deployment it never named.
    ///
    /// The sponsored form is admitted with and without change, so that a
    /// sponsored request reaches construction intact and is refused there
    /// by a refusal that can name what it lacks.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::SponsorChangeWithoutSponsoredForm`] when the
    /// sponsorless form asks for the sponsor-change output. That pair
    /// contradicts itself in the request alone: the form carries no
    /// sponsor region, so there is no residual for a change output to
    /// carry, and no deployment could supply one without changing the form
    /// the caller asked for.
    pub const fn new(
        announced_cycle: Cycle,
        requested_form: RequestedForm,
        sponsor_change: SponsorChangeRequest,
    ) -> Result<Self, TransactionRefusal> {
        if sponsor_change.requested() && !requested_form.sponsored() {
            return Err(TransactionRefusal::SponsorChangeWithoutSponsoredForm);
        }

        Ok(Self {
            announced_cycle,
            requested_form,
            sponsor_change,
        })
    }

    /// Which cycle the announcement names.
    #[must_use]
    pub const fn announced_cycle(self) -> Cycle {
        self.announced_cycle
    }

    /// Which of the two transaction forms the request asks for.
    #[must_use]
    pub const fn requested_form(self) -> RequestedForm {
        self.requested_form
    }

    /// Whether the request asks for the sponsor-change output.
    #[must_use]
    pub const fn sponsor_change(self) -> SponsorChangeRequest {
        self.sponsor_change
    }
}
