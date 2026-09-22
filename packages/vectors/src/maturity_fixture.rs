//! Canonical semantic fixtures for the maturity announcement, and the
//! successors an independent layer derives for them.
//!
//! Guide-14 §15.4 states what a semantic fixture may not contain: no
//! target program, no control block, no committed leaf, no transaction
//! byte, no witness authorization. This module takes that list
//! literally. A [`MaturitySemanticCase`] is a predecessor world, a lead
//! window, a typed request, an order context and one derived successor
//! — there is no field a target fact could be put in, and a test checks
//! the rendering for the vocabulary anyway. That test composes the terms
//! it searches for out of pieces, because a text scan cannot tell a
//! module that names a term in order to exclude it from one that carries
//! it, and this file has to read as free of them under either reading.
//!
//! # Where the expected successor comes from
//!
//! §15.3 names three producers of an expected STATE and puts the backend
//! constructor outside all three. Every successor here is what
//! [`announce_maturity`] returned for the case's own predecessor,
//! announced cycle and lead window, and never what a construction
//! settled on: the construction's answer is compared against this one
//! later, and if the two were the same computation there would be
//! nothing to compare. The transition is also the layer that owns the
//! window, so a world it refuses is refused here in its own five-member
//! vocabulary rather than by a second opinion written in this module.
//!
//! # The two producers this registry cannot call
//!
//! The other two producers §15.3 names are recorded as outstanding
//! rather than left out, because an absent producer and an unreachable
//! one are different findings and only one of them is work. The
//! executable model's own transition is public, but this package's
//! manifest names no dependency edge to the crate that holds it, and
//! whether it should is a question about this package's dependencies
//! rather than about one registry — a registry that answered it by
//! adding an edge would have settled it in passing. The typed
//! projection adapter cannot be called by anything: it is a pair of
//! private functions in a module its crate compiles only under test, so
//! no dependency edge would reach it, and making it reachable is a
//! change in that crate.
//!
//! What stands in their place is evidence rather than a promise. That
//! crate's own conformance tests run both transitions over the same
//! worlds and compare the results at the minimum and the maximum lead,
//! one cycle below the minimum and one above the maximum, and over the
//! already-announced and the complete predecessor, while requiring both
//! semantic censuses to carry the same six fields. The realization's
//! answer here is therefore the model's answer until one of those tests
//! fails, which is a statement a reader can check rather than one this
//! module makes about itself.
//!
//! # What a semantic-only world can state
//!
//! §15.1 asks a positive fixture to begin from one operational STATE
//! with maturity unannounced, nontrivial values in every semantic field,
//! a current cycle, an authorized operator, canonical root history, no
//! RESV participation, and optional sponsor context where selected. The
//! metadata census carries the quantities, the maturity status and the
//! current cycle, and the typed request carries the sponsor context. The
//! operator, the root history and the RESV non-participation are carried
//! by neither, and writing them here would mean authoring facts this
//! layer does not hold: the request's own census of what a caller cannot
//! select settles the operator identity in the public current-STATE view
//! and the root cursor in that view's current-root binding, both of them
//! external evidence about a deployment rather than fields of a world. A
//! registry that stated them anyway would be evidence for its own
//! authorship.
//!
//! One asked-for retention has no producer at all. §15.1 retains an
//! appended transition certificate beside the predecessor world, the
//! request, the canonical order and the successor world; the transition
//! returns a successor and nothing else, and a certificate written here
//! would be this module certifying its own expectation. It is therefore
//! carried as [`OutstandingRetention::AppendedTransitionCertificate`],
//! so that a reader counting what §15.1 asks for finds it named rather
//! than missing.
//!
//! # The row vocabulary has one home
//!
//! §16.1's fourteen positive rows are named by the STATE safety matrix,
//! which transcribes §16's tables in full, and this registry reads its
//! names out of that transcription rather than restating them. A name
//! written twice is two statements that agree until one of them is
//! edited, and the wording a reader checks against the guide should be
//! the wording every consumer reads. The ordinals here are the table's
//! own 1 to 14, and a row's position in the transcription is what
//! [`PositiveRow::name`] resolves.
//!
//! # A class is a predicate and not a name
//!
//! §15.5 asks a positive class for an executable typed predicate and
//! closes by saying that a class name alone cannot make a fixture
//! canonical. A name is a claim about a world; a function is a claim a
//! test can refute. Every class whose subject this layer holds is
//! therefore a [`MaturityClassWitness`] member whose predicate is a
//! function over values a case already carries, run over the registry's
//! own cases by a test, so that filing a case under a class and
//! exhibiting that class are two separate things and the second can
//! fail.
//!
//! Three of the six classes §15.5 exemplifies have no case as their
//! subject, and they say so in their own type rather than by being
//! absent. The nonzero representation nonce is a property of the
//! constructor's bounded search: a semantic world carries no nonce at
//! all, so the predicate reads a link's retained search instead, and it
//! reads one passed to it rather than one held here — this registry
//! stores no link. The sponsored class has no carrier in this operation
//! to run a predicate against, and public recovery compares against
//! material a semantic fixture excludes by construction. A class
//! carried as a typed member with its reason is one a later wave can
//! discharge by name, while a class left out is one nobody can find.

use std::collections::BTreeSet;

use architecture::{ObjectId, OperationId};
use linker::{CandidateLinkedMaturityBundle, StateRetainedConstructor};
use realization::{
    AnnouncementLeadBounds, Cycle, Maturity, ProtocolAmount, StateMetadata, announce_maturity,
};
use tapscript::StateNonceEvidence;
use transaction::{MaturityAnnouncementRequest, RequestedForm, SponsorChangeRequest};

use crate::error::VectorError;
use crate::fixture::SemanticFixtureId;
use crate::maturity_safety::MATURITY_SAFETY_ROWS;

/// The exact operation identity every fixture in this module claims.
///
/// Taken from the architecture's own identifier rather than spelled as a
/// name here, so that the operation a fixture claims and the operation
/// the architecture declares cannot drift apart.
pub const OPERATION: OperationId = OperationId::AnnounceMaturity;

/// The one semantic subject this operation moves.
pub const SUBJECT: ObjectId = ObjectId::State;

/// The four pool quantities the maturity link's own sources state.
///
/// Reused deliberately: the link fixes one predecessor world, so a row
/// stated over these numbers is a row a link could one day carry, and a
/// row that needs another world says which fact of its own forced it.
const LINKED_POOL: [u64; 4] = [1, 2, 3, 4];

/// Pool quantities spread across the domain's magnitudes.
///
/// Pairwise distinct and far apart, so a copy-through that truncated a
/// quantity or swapped two of them cannot coincide with the original.
const SPREAD_POOL: [u64; 4] = [3, 65_537, 4_294_967_291, 1_125_899_906_842_597];

/// The current cycle the link's own sources state.
const LINKED_CYCLE: u64 = 5;

/// The three lead windows the fixture deployments fix.
const FIRST_WINDOW: (u64, u64) = (2, 4);
const SECOND_WINDOW: (u64, u64) = (3, 5);
const THIRD_WINDOW: (u64, u64) = (4, 6);

/// One row of §16.1's positive case table.
///
/// The ordinal is the table's own position, and the name is read out of
/// the §16 transcription, so that a reader can check a row against the
/// table and the wording has one home rather than two.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PositiveRow {
    /// An announcement at exactly the minimum admissible lead.
    MinimumValidLead,
    /// An announcement at exactly the maximum admissible lead.
    MaximumValidLead,
    /// An announcement strictly inside the window.
    RepresentativeInteriorLead,
    /// A world whose current cycle is the least ordinal there is.
    SmallestCurrentCycle,
    /// A world at the last current cycle whose window still resolves.
    CurrentCycleNearCheckedUpperDomain,
    /// A world whose unaffected quantities are nontrivial and distinct.
    NontrivialUnaffectedFields,
    /// A successor found at representation nonce zero.
    RepresentationNonceZero,
    /// A successor found at a nonzero representation nonce.
    RepresentationNonceNonzero,
    /// An announcement in the sponsorless form.
    Sponsorless,
    /// A sponsored announcement returning no residual.
    SponsoredWithoutChange,
    /// A sponsored announcement returning a residual.
    SponsoredWithChange,
    /// A successor program recovered from public material alone.
    PublicSuccessorRecovery,
    /// Two equal constructions producing equal candidate bytes.
    RepeatedEqualConstruction,
    /// An accepted spend whose four projections all match.
    AcceptedByTheTarget,
}

impl PositiveRow {
    /// Every row of §16.1, in the table's order.
    pub const ALL: &'static [Self] = &[
        Self::MinimumValidLead,
        Self::MaximumValidLead,
        Self::RepresentativeInteriorLead,
        Self::SmallestCurrentCycle,
        Self::CurrentCycleNearCheckedUpperDomain,
        Self::NontrivialUnaffectedFields,
        Self::RepresentationNonceZero,
        Self::RepresentationNonceNonzero,
        Self::Sponsorless,
        Self::SponsoredWithoutChange,
        Self::SponsoredWithChange,
        Self::PublicSuccessorRecovery,
        Self::RepeatedEqualConstruction,
        Self::AcceptedByTheTarget,
    ];

    /// The row's position in the table, counting from one.
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        match self {
            Self::MinimumValidLead => 1,
            Self::MaximumValidLead => 2,
            Self::RepresentativeInteriorLead => 3,
            Self::SmallestCurrentCycle => 4,
            Self::CurrentCycleNearCheckedUpperDomain => 5,
            Self::NontrivialUnaffectedFields => 6,
            Self::RepresentationNonceZero => 7,
            Self::RepresentationNonceNonzero => 8,
            Self::Sponsorless => 9,
            Self::SponsoredWithoutChange => 10,
            Self::SponsoredWithChange => 11,
            Self::PublicSuccessorRecovery => 12,
            Self::RepeatedEqualConstruction => 13,
            Self::AcceptedByTheTarget => 14,
        }
    }

    /// The row's entry in the §16 transcription, counting from zero.
    ///
    /// The one place a position lives. The transcription holds §16's
    /// tables contiguously in the guide's order with §16.1 first, so the
    /// fourteen positive rows are its first fourteen entries and a row's
    /// ordinal is this position plus one; a test holds both statements
    /// to the transcription itself rather than leaving the indexing to
    /// an argument.
    const fn position(self) -> usize {
        match self {
            Self::MinimumValidLead => 0,
            Self::MaximumValidLead => 1,
            Self::RepresentativeInteriorLead => 2,
            Self::SmallestCurrentCycle => 3,
            Self::CurrentCycleNearCheckedUpperDomain => 4,
            Self::NontrivialUnaffectedFields => 5,
            Self::RepresentationNonceZero => 6,
            Self::RepresentationNonceNonzero => 7,
            Self::Sponsorless => 8,
            Self::SponsoredWithoutChange => 9,
            Self::SponsoredWithChange => 10,
            Self::PublicSuccessorRecovery => 11,
            Self::RepeatedEqualConstruction => 12,
            Self::AcceptedByTheTarget => 13,
        }
    }

    /// The row's stable name, in the table's own wording.
    ///
    /// Read out of the §16 transcription rather than restated here. That
    /// transcription is what a reader checks against the guide line by
    /// line, and a second copy of the wording would be a second
    /// statement of it — one that agrees until either is edited, and
    /// whose disagreement nothing would report.
    #[must_use]
    pub const fn name(self) -> &'static str {
        MATURITY_SAFETY_ROWS[self.position()].name()
    }

    /// The §15.5 class this row keys to, where §15.5 names one.
    ///
    /// The two lead rows key to the two lead classes and the
    /// unaffected-fields row to the single-field transition. Both nonce
    /// rows key to the one nonce class, because the class's statement —
    /// that every earlier nonce was checked and refused for a retryable
    /// reason — is the same at a zero and at a nonzero selection, and is
    /// what makes either of them least. The sponsored rows and the
    /// recovery row key to the two classes whose subjects sit outside
    /// this layer, so that a reader counting §15.5's classes finds them
    /// named.
    ///
    /// The interior-lead row, the two domain-edge rows and the
    /// sponsorless row key to none, which is not an omission. §15.5
    /// exemplifies classes, and an interior lead, a domain-edge current
    /// cycle and the sponsorless form are not among them; minting a
    /// class name to cover a row is the move §15.5's closing sentence
    /// refuses. Those rows' own assertions are checked where every
    /// row's is, when its case is built.
    #[must_use]
    pub const fn witness(self) -> Option<MaturityClassWitness> {
        match self {
            Self::MinimumValidLead => Some(MaturityClassWitness::MinimumLead),
            Self::MaximumValidLead => Some(MaturityClassWitness::MaximumLead),
            Self::NontrivialUnaffectedFields => Some(MaturityClassWitness::SingleFieldTransition),
            Self::RepresentationNonceZero | Self::RepresentationNonceNonzero => {
                Some(MaturityClassWitness::NonzeroRepresentationNonce)
            }
            Self::SponsoredWithoutChange | Self::SponsoredWithChange => {
                Some(MaturityClassWitness::Sponsored)
            }
            Self::PublicSuccessorRecovery => Some(MaturityClassWitness::PublicRecovery),
            Self::RepresentativeInteriorLead
            | Self::SmallestCurrentCycle
            | Self::CurrentCycleNearCheckedUpperDomain
            | Self::Sponsorless
            | Self::RepeatedEqualConstruction
            | Self::AcceptedByTheTarget => None,
        }
    }

    /// Why no semantic fixture here answers this row, where none does.
    ///
    /// A total function of the row rather than a comment beside the
    /// census, so that a row moving between the two halves is an edit to
    /// one place a reader can find, and so that a carried row can never
    /// be read as an obligation nobody noticed.
    #[must_use]
    pub const fn carried_reason(self) -> Option<UnfixedReason> {
        match self {
            Self::MinimumValidLead
            | Self::MaximumValidLead
            | Self::RepresentativeInteriorLead
            | Self::SmallestCurrentCycle
            | Self::CurrentCycleNearCheckedUpperDomain
            | Self::NontrivialUnaffectedFields
            | Self::Sponsorless => None,
            Self::RepresentationNonceZero | Self::RepresentationNonceNonzero => {
                Some(UnfixedReason::RepresentationSearchFact)
            }
            Self::SponsoredWithoutChange | Self::SponsoredWithChange => {
                Some(UnfixedReason::SponsoredFormHasNoCarrier)
            }
            Self::PublicSuccessorRecovery => Some(UnfixedReason::RecoveryComparesTargetMaterial),
            Self::RepeatedEqualConstruction => Some(UnfixedReason::PropertyOfRepeatedConstruction),
            Self::AcceptedByTheTarget => Some(UnfixedReason::PropertyOfAnAcceptedRun),
        }
    }
}

/// Why a §16.1 row is carried here rather than answered by a fixture.
///
/// Typed rather than written as prose, because a reason a reader can
/// match on is a reason a later wave can discharge by name, and because
/// a row another producer answers is a different thing from a row
/// nobody has looked at.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UnfixedReason {
    /// The row states a fact of the constructor's representation search.
    RepresentationSearchFact,
    /// The row states a sponsored form nothing in this operation carries.
    SponsoredFormHasNoCarrier,
    /// The row compares against material a semantic fixture excludes.
    RecoveryComparesTargetMaterial,
    /// The row states a property of running one construction twice.
    PropertyOfRepeatedConstruction,
    /// The row states a property of an accepted run against a target.
    PropertyOfAnAcceptedRun,
}

impl UnfixedReason {
    /// Every reason, in declaration order.
    pub const ALL: &'static [Self] = &[
        Self::RepresentationSearchFact,
        Self::SponsoredFormHasNoCarrier,
        Self::RecoveryComparesTargetMaterial,
        Self::PropertyOfRepeatedConstruction,
        Self::PropertyOfAnAcceptedRun,
    ];

    /// The reason, argued from what the tree holds.
    #[must_use]
    pub const fn reason(self) -> &'static str {
        match self {
            Self::RepresentationSearchFact => {
                "a representation nonce is a property of the constructor's search from zero and not of a semantic world: the semantic metadata census admits no nonce at all, and the request's own census of unselectable facets settles the nonce in that search, so a semantic fixture has nothing to state either way"
            }
            Self::SponsoredFormHasNoCarrier => {
                "the sponsored form has no carrier in this operation: the reduced announcement leaf carries no sponsor check, and the sponsor relations hold no region of the transaction until a later refit gives them one, so a sponsored row would state a form that neither the leaf nor the model constrains"
            }
            Self::RecoveryComparesTargetMaterial => {
                "public recovery is an equality between an independent reconstruction and the program at output zero, and an output program is target material a semantic fixture excludes by construction, so the row is answered where that reconstruction is performed and not here"
            }
            Self::PropertyOfRepeatedConstruction => {
                "repeatability is a property of running one construction twice rather than of a fixture: a registry asserting it would be asserting it about itself, while the statement worth having is about the builder a fixture is handed to"
            }
            Self::PropertyOfAnAcceptedRun => {
                "an acceptance and the four projections that must match it are facts of a run against a real target: a fixture can state what an accepted run would have to exhibit, and it cannot state that one occurred"
            }
        }
    }
}

/// One §15.5 positive class, as the predicate that class is.
///
/// §15.5 lists six classes by example and closes by saying that a class
/// name alone cannot make a fixture canonical. Each member here carries
/// the predicate its class is: stated in §15.5's words by
/// [`Self::predicate`], and computed by [`Self::holds`] wherever the
/// subject is a case. A member whose subject is not a case says which
/// reason puts it elsewhere rather than answering `false` about a
/// question it was never asked.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityClassWitness {
    /// The announced cycle sits exactly one minimum lead after the
    /// current one.
    ///
    /// Read as the case's stored distance against the window's own lower
    /// endpoint, so the class is decided by the bounds that admitted the
    /// world rather than by arithmetic repeated here.
    MinimumLead,
    /// The announced cycle sits exactly one maximum lead after the
    /// current one.
    ///
    /// Read the same way against the upper endpoint. More than one
    /// world can exhibit it, because a class is a property and not an
    /// identity.
    MaximumLead,
    /// Every semantic field but maturity compares equal across the
    /// transition, and maturity is the field that moved.
    ///
    /// Read over the case's predecessor and the successor its own layer
    /// derived. The moved field is part of the statement: a predicate
    /// the identity satisfies would witness no transition at all.
    SingleFieldTransition,
    /// Every earlier nonce is checked and refused for a retryable
    /// reason.
    ///
    /// The subject is a link's retained search rather than a case, and
    /// the predicate is [`nonce_search_was_least_first`]. A semantic
    /// world carries no nonce: the selection belongs to the
    /// constructor's bounded scan from zero, and the only first-party
    /// account of what that scan checked is the evidence the scan itself
    /// retained.
    NonzeroRepresentationNonce,
    /// The exact sponsor suffix and fee role are present.
    ///
    /// Carried with its predicate stated and nothing to run it against.
    /// The reduced announcement leaf carries no sponsor check, and the
    /// sponsor relations hold no region of the transaction until a later
    /// refit gives them one, so a predicate run today would be deciding
    /// a form that neither the leaf nor the model constrains.
    Sponsored,
    /// An independent reconstruction equals the program of output zero.
    ///
    /// Carried because that comparison is against material a semantic
    /// fixture excludes by construction, so the class is exercised where
    /// the reconstruction is performed and belongs with the recovery
    /// deliverables rather than here.
    PublicRecovery,
}

impl MaturityClassWitness {
    /// Every class §15.5 names, in the order it lists them.
    pub const ALL: &'static [Self] = &[
        Self::MinimumLead,
        Self::MaximumLead,
        Self::SingleFieldTransition,
        Self::NonzeroRepresentationNonce,
        Self::Sponsored,
        Self::PublicRecovery,
    ];

    /// The predicate the class is, in §15.5's own words.
    #[must_use]
    pub const fn predicate(self) -> &'static str {
        match self {
            Self::MinimumLead => "the announced cycle is the current cycle plus the minimum lead",
            Self::MaximumLead => "the announced cycle is the current cycle plus the maximum lead",
            Self::SingleFieldTransition => "all semantic fields but maturity compare equal",
            Self::NonzeroRepresentationNonce => {
                "every earlier nonce is checked and refused for a retryable reason"
            }
            Self::Sponsored => "the exact sponsor suffix and fee role are present",
            Self::PublicRecovery => {
                "an independent reconstruction equals the program of output zero"
            }
        }
    }

    /// Why a semantic case is not this class's subject, where it is not.
    ///
    /// Decided here and nowhere else, so that [`Self::holds`] and a
    /// reader counting the classes a case can answer cannot disagree.
    /// The reasons are the registry's own: a class with no case subject
    /// and a row no fixture answers are one fact read from two sides,
    /// and a reason with one home is what lets a later wave discharge it
    /// by name.
    #[must_use]
    pub const fn no_case_subject(self) -> Option<UnfixedReason> {
        match self {
            Self::MinimumLead | Self::MaximumLead | Self::SingleFieldTransition => None,
            Self::NonzeroRepresentationNonce => Some(UnfixedReason::RepresentationSearchFact),
            Self::Sponsored => Some(UnfixedReason::SponsoredFormHasNoCarrier),
            Self::PublicRecovery => Some(UnfixedReason::RecoveryComparesTargetMaterial),
        }
    }

    /// Whether one case exhibits this class.
    #[must_use]
    pub fn holds(self, case: &MaturitySemanticCase) -> ClassVerdict {
        if let Some(reason) = self.no_case_subject() {
            return ClassVerdict::NoCaseSubject(reason);
        }
        let holds = match self {
            Self::MinimumLead => case.lead() == case.bounds().minimum(),
            Self::MaximumLead => case.lead() == case.bounds().maximum(),
            Self::SingleFieldTransition => moves_maturity_alone(case),
            // Answered above, by the one function that decides which
            // classes a case is the subject of. Listed rather than left
            // to a catch-all so that a class added later stops this
            // match instead of silently reading as refuted.
            Self::NonzeroRepresentationNonce | Self::Sponsored | Self::PublicRecovery => false,
        };
        if holds {
            ClassVerdict::Holds
        } else {
            ClassVerdict::Fails
        }
    }
}

/// What a class's predicate says about one case.
///
/// A sum rather than a `bool`, because a class whose subject is not a
/// case has no answer to give about one, and reporting `false` would
/// make an inapplicable predicate indistinguishable from a refuted one.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ClassVerdict {
    /// The case exhibits the class.
    Holds,
    /// The case does not exhibit it.
    Fails,
    /// The class's subject is not a case, for this reason.
    NoCaseSubject(UnfixedReason),
}

/// Whether a link's retained search checked every earlier nonce and
/// refused each for a reason a nonce change can repair.
///
/// The subject §15.5's nonce class needs, and the reason that subject is
/// not a case: the selection is the constructor's, made by a bounded
/// scan from zero, and what the scan checked is stated only by the
/// evidence the scan retained. Every retained instance is read, because
/// a link retains one per semantic metadata it applied its recipe to and
/// a claim about the search is a claim about each of them.
///
/// Four conditions, over each instance's own evidence record. The bundle
/// retains at least one instance, since a statement about every search
/// would otherwise hold where no search ran. The refused candidates are,
/// in scan order, exactly the nonces below the selected one. Each was
/// refused for a retryable reason, which is the class's own wording.
/// And the selected nonce is the first the scan did not refuse, which is
/// the count of those it did.
///
/// The leastness this establishes is leastness inside the scan's budget
/// and no more. The evidence record carries that residual itself, and a
/// predicate claiming more would be claiming it about candidates nobody
/// evaluated.
#[must_use]
pub fn nonce_search_was_least_first(bundle: &CandidateLinkedMaturityBundle) -> bool {
    let instances = bundle.instances();
    !instances.is_empty()
        && instances
            .iter()
            .map(StateRetainedConstructor::constructor)
            .all(|constructor| search_was_least_first(constructor.evidence()))
}

/// Whether the transition moved maturity and left every other semantic
/// field where it was.
///
/// The five unaffected fields compare equal across the case's own
/// derivation, and maturity is read on both sides rather than only on
/// one: that it moved from unannounced to the announced cycle the request
/// names is what makes this a transition, and a predicate the identity
/// satisfies would be a statement about nothing having happened.
fn moves_maturity_alone(case: &MaturitySemanticCase) -> bool {
    let predecessor = case.predecessor();
    let successor = case.expected();
    let announced = Maturity::Announced {
        cycle: case.announced_cycle(),
    };
    predecessor.omega == successor.omega
        && predecessor.y_l == successor.y_l
        && predecessor.y_t == successor.y_t
        && predecessor.q == successor.q
        && predecessor.cycle == successor.cycle
        && predecessor.maturity == Maturity::Unannounced
        && successor.maturity == announced
}

/// Whether one retained search settled on the least nonce it admitted.
fn search_was_least_first(evidence: &StateNonceEvidence) -> bool {
    evidence
        .rejected
        .iter()
        .zip(0_u32..)
        .all(|((nonce, refusal), earlier)| refusal.retryable() && nonce.get() == earlier)
        && u32::try_from(evidence.rejected.len())
            .is_ok_and(|checked| evidence.selected.get() == checked)
}

/// The canonical order §15.1 retains, for one STATE in one operation.
///
/// The operation's semantic census is one predecessor and one successor,
/// so the order over its subjects is the one-element order and there is
/// nothing to normalize. Stated as a subject and a count a test can read
/// rather than as a sentence, because a trivial order asserted in prose
/// and a trivial order a reader can check are different kinds of claim.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalOrder {
    subject: ObjectId,
    subjects: u32,
}

impl CanonicalOrder {
    /// The order over one STATE subject.
    pub const SINGLE_STATE_SUBJECT: Self = Self {
        subject: SUBJECT,
        subjects: 1,
    };

    /// Which object the order is over.
    #[must_use]
    pub const fn subject(self) -> ObjectId {
        self.subject
    }

    /// How many subjects the order ranges over.
    #[must_use]
    pub const fn subjects(self) -> u32 {
        self.subjects
    }

    /// Whether the order is the trivial one.
    #[must_use]
    pub const fn is_trivial(self) -> bool {
        self.subjects <= 1
    }
}

/// One positive semantic case: a model-valid announcement world, its
/// request, and the successor the realization layer's own transition
/// derived from them.
///
/// Every field is a semantic fact. None is a byte, a program, an index
/// or a witness, because comparing those later would compare a
/// construction with itself rather than with a world.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaturitySemanticCase {
    id: SemanticFixtureId,
    row: PositiveRow,
    predecessor: StateMetadata,
    bounds: AnnouncementLeadBounds,
    request: MaturityAnnouncementRequest,
    order: CanonicalOrder,
    lead: Cycle,
    expected: StateMetadata,
}

impl MaturitySemanticCase {
    /// The fixture's identity.
    #[must_use]
    pub const fn id(&self) -> SemanticFixtureId {
        self.id
    }

    /// The §16.1 row this fixture answers.
    #[must_use]
    pub const fn row(&self) -> PositiveRow {
        self.row
    }

    /// The predecessor world, whose maturity is unannounced.
    #[must_use]
    pub const fn predecessor(&self) -> StateMetadata {
        self.predecessor
    }

    /// The inclusive lead window the announcement is judged against.
    #[must_use]
    pub const fn bounds(&self) -> AnnouncementLeadBounds {
        self.bounds
    }

    /// The typed request, carrying the three facets a caller selects.
    #[must_use]
    pub const fn request(&self) -> MaturityAnnouncementRequest {
        self.request
    }

    /// The canonical order retained with the world.
    #[must_use]
    pub const fn order(&self) -> CanonicalOrder {
        self.order
    }

    /// The distance between the current cycle and the announced one.
    #[must_use]
    pub const fn lead(&self) -> Cycle {
        self.lead
    }

    /// The successor the realization's transition derived.
    #[must_use]
    pub const fn expected(&self) -> StateMetadata {
        self.expected
    }

    /// Which cycle the request announces.
    #[must_use]
    pub const fn announced_cycle(&self) -> Cycle {
        self.request.announced_cycle()
    }
}

/// What the census says about one §16.1 row.
///
/// A sum rather than two optional members, so that a row can be neither
/// answered twice nor left with no statement at all.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RowAnswer {
    /// The row is answered by this fixture.
    Fixed(MaturitySemanticCase),
    /// The row is carried, for this reason.
    Carried(UnfixedReason),
}

/// One row of the positive census and its answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CensusEntry {
    row: PositiveRow,
    answer: RowAnswer,
}

impl CensusEntry {
    /// Which §16.1 row this entry is.
    #[must_use]
    pub const fn row(&self) -> PositiveRow {
        self.row
    }

    /// The entry's answer, fixed or carried.
    #[must_use]
    pub const fn answer(&self) -> &RowAnswer {
        &self.answer
    }

    /// The fixture answering this row, where one does.
    #[must_use]
    pub const fn fixture(&self) -> Option<&MaturitySemanticCase> {
        match &self.answer {
            RowAnswer::Fixed(case) => Some(case),
            RowAnswer::Carried(_) => None,
        }
    }

    /// Why this row is carried, where it is.
    #[must_use]
    pub const fn carried(&self) -> Option<UnfixedReason> {
        match self.answer {
            RowAnswer::Fixed(_) => None,
            RowAnswer::Carried(reason) => Some(reason),
        }
    }
}

/// A producer of an expected STATE that this registry cannot call.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OutstandingProducer {
    /// The executable model's own maturity transition.
    ExecutableModel,
    /// The typed adapter projecting one layer's world onto the other's.
    TypedProjectionAdapter,
}

impl OutstandingProducer {
    /// Both producers, in §15.3's order.
    pub const ALL: &'static [Self] = &[Self::ExecutableModel, Self::TypedProjectionAdapter];

    /// What the producer is.
    #[must_use]
    pub const fn producer(self) -> &'static str {
        match self {
            Self::ExecutableModel => "the executable model's maturity transition",
            Self::TypedProjectionAdapter => "the typed projection adapter",
        }
    }

    /// Why this registry does not call it.
    #[must_use]
    pub const fn reason(self) -> &'static str {
        match self {
            Self::ExecutableModel => {
                "the transition is public, and this package's manifest names no dependency edge to the crate holding it; whether admitting that edge is right is a question about this package's dependencies, and a registry that answered it by adding one would have settled it in passing"
            }
            Self::TypedProjectionAdapter => {
                "the adapter is a pair of private functions in a module its crate compiles only under test, so no dependency edge would reach it at all, and making it reachable is a change in that crate rather than in this one"
            }
        }
    }

    /// What stands as evidence in its place, and can be checked.
    #[must_use]
    pub const fn standing_evidence(self) -> &'static str {
        match self {
            Self::ExecutableModel => {
                "that crate's conformance tests run both transitions over the same worlds and compare the results at the minimum and the maximum lead, one cycle below the minimum and one above the maximum, and over the already-announced and the complete predecessor"
            }
            Self::TypedProjectionAdapter => {
                "the same tests exercise the adapter in both directions over whole worlds and require both semantic censuses to carry the same six fields"
            }
        }
    }
}

/// Something §15.1 retains that no layer in this tree produces.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OutstandingRetention {
    /// The transition certificate §15.1 appends to a retained world.
    AppendedTransitionCertificate,
}

impl OutstandingRetention {
    /// Every outstanding retention, in declaration order.
    pub const ALL: &'static [Self] = &[Self::AppendedTransitionCertificate];

    /// What §15.1 asks to be retained.
    #[must_use]
    pub const fn retained(self) -> &'static str {
        match self {
            Self::AppendedTransitionCertificate => "the appended transition certificate",
        }
    }

    /// Why it is carried rather than stated.
    #[must_use]
    pub const fn reason(self) -> &'static str {
        match self {
            Self::AppendedTransitionCertificate => {
                "no layer in this tree appends one: the transition returns a semantic successor and nothing else, and a certificate written in this module would be the registry certifying its own expectation"
            }
        }
    }
}

/// The stated facts of one fixture world, before they are typed.
///
/// Held as plain magnitudes so that every typed value is built in one
/// place, and so that a domain refusal is reported against the fixture
/// whose facts asked for it rather than against the census as a whole.
#[derive(Clone, Copy, Debug)]
struct StatedWorld {
    pool: [u64; 4],
    cycle: u64,
    lead: (u64, u64),
    announced_cycle: u64,
}

impl StatedWorld {
    /// One stated world.
    const fn new(pool: [u64; 4], cycle: u64, lead: (u64, u64), announced_cycle: u64) -> Self {
        Self {
            pool,
            cycle,
            lead,
            announced_cycle,
        }
    }
}

/// The world a row's own name asks for, where a fixture answers it.
///
/// The worlds are pairwise distinct, and a test says so: two rows
/// carrying one world under two names would make a run of either row
/// equally good evidence for both, which is the one way a census of
/// named rows can be well formed and still witness nothing.
const fn stated_world(row: PositiveRow) -> Option<StatedWorld> {
    match row {
        PositiveRow::MinimumValidLead => Some(StatedWorld::new(
            LINKED_POOL,
            LINKED_CYCLE,
            FIRST_WINDOW,
            LINKED_CYCLE + FIRST_WINDOW.0,
        )),
        PositiveRow::MaximumValidLead => Some(StatedWorld::new(
            LINKED_POOL,
            LINKED_CYCLE,
            FIRST_WINDOW,
            LINKED_CYCLE + FIRST_WINDOW.1,
        )),
        PositiveRow::RepresentativeInteriorLead => Some(StatedWorld::new(
            LINKED_POOL,
            LINKED_CYCLE,
            FIRST_WINDOW,
            LINKED_CYCLE + FIRST_WINDOW.0 + 1,
        )),
        // The least ordinal there is. §15.1 asks every semantic field to
        // be nontrivial and this row asks the current cycle to be the
        // smallest one; where the two meet, the row's own name decides,
        // and the quantities stay nontrivial.
        PositiveRow::SmallestCurrentCycle => Some(StatedWorld::new(
            LINKED_POOL,
            0,
            FIRST_WINDOW,
            FIRST_WINDOW.0 + 1,
        )),
        // The last current cycle whose window still resolves: one cycle
        // later, deriving the upper endpoint leaves the domain. The
        // property is checked against the transition's own window rather
        // than against this arithmetic.
        PositiveRow::CurrentCycleNearCheckedUpperDomain => Some(StatedWorld::new(
            LINKED_POOL,
            u64::MAX - FIRST_WINDOW.1,
            FIRST_WINDOW,
            u64::MAX,
        )),
        PositiveRow::NontrivialUnaffectedFields => Some(StatedWorld::new(
            SPREAD_POOL,
            LINKED_CYCLE,
            THIRD_WINDOW,
            LINKED_CYCLE + THIRD_WINDOW.0 + 1,
        )),
        PositiveRow::Sponsorless => Some(StatedWorld::new(
            LINKED_POOL,
            LINKED_CYCLE,
            SECOND_WINDOW,
            LINKED_CYCLE + SECOND_WINDOW.1 - 1,
        )),
        PositiveRow::RepresentationNonceZero
        | PositiveRow::RepresentationNonceNonzero
        | PositiveRow::SponsoredWithoutChange
        | PositiveRow::SponsoredWithChange
        | PositiveRow::PublicSuccessorRecovery
        | PositiveRow::RepeatedEqualConstruction
        | PositiveRow::AcceptedByTheTarget => None,
    }
}

/// Type one stated world and build the case answering its row.
fn case_for(row: PositiveRow, world: StatedWorld) -> Result<MaturitySemanticCase, VectorError> {
    let id = SemanticFixtureId::new(row.name(), row.ordinal());
    let amount = |value: u64| {
        ProtocolAmount::new(value)
            .map_err(|cause| VectorError::ExpectationNotDerivable { fixture: id, cause })
    };
    let predecessor = StateMetadata {
        omega: amount(world.pool[0])?,
        y_l: amount(world.pool[1])?,
        y_t: amount(world.pool[2])?,
        q: amount(world.pool[3])?,
        cycle: Cycle::new(world.cycle),
        maturity: Maturity::Unannounced,
    };
    let bounds = AnnouncementLeadBounds::new(Cycle::new(world.lead.0), Cycle::new(world.lead.1))
        .map_err(|cause| VectorError::ExpectationNotDerivable { fixture: id, cause })?;
    // Every row here states the sponsorless form with no change asked
    // for, which the request type admits; the mapping exists because the
    // constructor is total over a pair this module happens to hold
    // constant, and a world whose request its own layer refuses is not a
    // stated world at all.
    let request = MaturityAnnouncementRequest::new(
        Cycle::new(world.announced_cycle),
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    )
    .map_err(|_| VectorError::InvalidSemanticFixture(id))?;

    semantic_case(id, row, predecessor, bounds, request)
}

/// Build one positive semantic case from stated, typed facts.
///
/// The expectation is derived before the row's own name is checked, and
/// the order matters: a world the transition refuses is not a positive
/// world at all, and that finding belongs to the layer that owns the
/// window. Only a world the transition admitted is then asked whether it
/// exhibits what its row is named for.
///
/// # Errors
///
/// [`VectorError::ExpectedSuccessorRefused`] when the realization
/// refuses the transition, carrying the refusal whole, and
/// [`VectorError::FixtureContradictsItsClass`] when the stated facts do
/// not witness the row they are filed under — including every row the
/// census carries, which no fixture built here can answer.
pub fn semantic_case(
    id: SemanticFixtureId,
    row: PositiveRow,
    predecessor: StateMetadata,
    bounds: AnnouncementLeadBounds,
    request: MaturityAnnouncementRequest,
) -> Result<MaturitySemanticCase, VectorError> {
    let announced_cycle = request.announced_cycle();
    let expected = announce_maturity(&predecessor, announced_cycle, bounds).map_err(|refusal| {
        VectorError::ExpectedSuccessorRefused {
            fixture: id,
            refusal,
        }
    })?;
    // The transition admitted the world, so the announced cycle is at
    // least one whole minimum lead after the current one and there is
    // nothing here to saturate at; saturating rather than wrapping keeps
    // the distance total without a panicking path.
    let lead = Cycle::new(
        announced_cycle
            .get()
            .saturating_sub(predecessor.cycle.get()),
    );

    witnesses_its_row(id, row, predecessor, bounds, request, lead)?;

    Ok(MaturitySemanticCase {
        id,
        row,
        predecessor,
        bounds,
        request,
        order: CanonicalOrder::SINGLE_STATE_SUBJECT,
        lead,
        expected,
    })
}

/// Whether a case's stated facts witness the row it is filed under.
///
/// A row name is an assertion about the world, and a census of rows
/// whose names nothing checks is a census that passes whatever it was
/// given. Only the rows whose names assert a fact these values carry are
/// listed as holding; every row the census carries for a reason is
/// listed as refused, because a row no semantic fixture answers cannot
/// be answered by one built here.
///
/// # Errors
///
/// [`VectorError::FixtureContradictsItsClass`], naming the fixture and
/// the row whose assertion its facts do not make.
fn witnesses_its_row(
    id: SemanticFixtureId,
    row: PositiveRow,
    predecessor: StateMetadata,
    bounds: AnnouncementLeadBounds,
    request: MaturityAnnouncementRequest,
    lead: Cycle,
) -> Result<(), VectorError> {
    let holds = match row {
        PositiveRow::MinimumValidLead => lead == bounds.minimum(),
        PositiveRow::MaximumValidLead => lead == bounds.maximum(),
        PositiveRow::RepresentativeInteriorLead => {
            lead > bounds.minimum() && lead < bounds.maximum()
        }
        PositiveRow::SmallestCurrentCycle => predecessor.cycle == Cycle::ZERO,
        // Read off the transition's own window rather than recomputed
        // here: the claim is that this is the last current cycle the
        // realization admits, and only the realization can say so.
        PositiveRow::CurrentCycleNearCheckedUpperDomain => {
            bounds.window(predecessor.cycle).is_ok()
                && predecessor
                    .cycle
                    .checked_add(Cycle::new(1))
                    .is_ok_and(|next| bounds.window(next).is_err())
        }
        PositiveRow::NontrivialUnaffectedFields => {
            let pool = [
                predecessor.omega,
                predecessor.y_l,
                predecessor.y_t,
                predecessor.q,
            ];
            pool.iter().all(|amount| !amount.is_zero())
                && are_distinct(&pool)
                && predecessor.cycle != Cycle::ZERO
        }
        PositiveRow::Sponsorless => {
            !request.requested_form().sponsored() && !request.sponsor_change().requested()
        }
        PositiveRow::RepresentationNonceZero
        | PositiveRow::RepresentationNonceNonzero
        | PositiveRow::SponsoredWithoutChange
        | PositiveRow::SponsoredWithChange
        | PositiveRow::PublicSuccessorRecovery
        | PositiveRow::RepeatedEqualConstruction
        | PositiveRow::AcceptedByTheTarget => false,
    };

    if holds {
        Ok(())
    } else {
        Err(VectorError::FixtureContradictsItsClass {
            fixture: id,
            class: row.name(),
        })
    }
}

/// Whether the stated quantities are pairwise distinct.
fn are_distinct(pool: &[ProtocolAmount]) -> bool {
    pool.iter().copied().collect::<BTreeSet<_>>().len() == pool.len()
}

/// The §16.1 positive census: every row of the table, answered or
/// carried.
///
/// The census is the registry: the fixtures are read out of it rather
/// than kept in a second list, so a row's answer and the fixture
/// answering it cannot drift apart.
///
/// # Errors
///
/// [`VectorError::MatrixCoverageMismatch`] when a row states both a
/// world and a reason to carry it, or neither, which is the one way the
/// two halves could stop covering the table; and any refusal
/// [`semantic_case`] raises, which for a row this module states is a
/// regression in the layer that refused rather than a fixture defect.
pub fn positive_semantic_census() -> Result<Vec<CensusEntry>, VectorError> {
    let mut census = Vec::with_capacity(PositiveRow::ALL.len());
    for row in PositiveRow::ALL.iter().copied() {
        let answer = match (stated_world(row), row.carried_reason()) {
            (Some(world), None) => RowAnswer::Fixed(case_for(row, world)?),
            (None, Some(reason)) => RowAnswer::Carried(reason),
            _ => return Err(VectorError::MatrixCoverageMismatch { class: row.name() }),
        };
        census.push(CensusEntry { row, answer });
    }
    Ok(census)
}

/// Every fixture the census carries, in the table's order.
///
/// # Errors
///
/// Propagates whatever [`positive_semantic_census`] refuses.
pub fn fixtures() -> Result<Vec<MaturitySemanticCase>, VectorError> {
    Ok(positive_semantic_census()?
        .iter()
        .filter_map(|entry| entry.fixture().copied())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{
        CanonicalOrder, CensusEntry, ClassVerdict, MaturityClassWitness, MaturitySemanticCase,
        OPERATION, OutstandingProducer, OutstandingRetention, PositiveRow, SUBJECT, UnfixedReason,
        fixtures, nonce_search_was_least_first, positive_semantic_census, search_was_least_first,
        semantic_case,
    };
    use crate::error::VectorError;
    use crate::fixture::SemanticFixtureId;
    use crate::maturity_closure::{MaturityDeployment, linked_maturity_bundle};
    use crate::maturity_safety::{
        MATURITY_SAFETY_ROWS, MaturitySafetyRow, MaturitySafetySection, rows_of,
    };
    use architecture::{ObjectId, OperationId};
    use realization::{
        AnnouncementLeadBounds, Cycle, Maturity, MaturityTransitionRefusal, ProtocolAmount,
        StateMetadata, StateRepresentationNonce, announce_maturity,
    };
    use std::collections::{BTreeSet, HashSet};
    use tapscript::{StateConstructorRefusal, StateNonceEvidence};
    use transaction::{MaturityAnnouncementRequest, RequestedForm, SponsorChangeRequest};

    /// Every fixture the census carries.
    fn all_fixtures() -> Vec<MaturitySemanticCase> {
        fixtures().expect("the positive census builds")
    }

    /// The fixture answering one row.
    fn named(row: PositiveRow) -> MaturitySemanticCase {
        all_fixtures()
            .into_iter()
            .find(|case| case.row() == row)
            .unwrap_or_else(|| panic!("the census fixes {}", row.name()))
    }

    /// The census entry for one row.
    fn entry(row: PositiveRow) -> CensusEntry {
        positive_semantic_census()
            .expect("the positive census builds")
            .into_iter()
            .find(|entry| entry.row() == row)
            .unwrap_or_else(|| panic!("the census answers {}", row.name()))
    }

    #[test]
    fn the_census_answers_every_positive_row_exactly_once() {
        let census = positive_semantic_census().expect("the positive census builds");
        assert_eq!(census.len(), PositiveRow::ALL.len());
        assert_eq!(census.len(), 14);

        let answered: BTreeSet<PositiveRow> = census.iter().map(CensusEntry::row).collect();
        let named: BTreeSet<PositiveRow> = PositiveRow::ALL.iter().copied().collect();
        assert_eq!(answered, named);

        let fixed = census
            .iter()
            .filter(|entry| entry.fixture().is_some())
            .count();
        let carried = census
            .iter()
            .filter(|entry| entry.carried().is_some())
            .count();
        assert_eq!(fixed, 7);
        assert_eq!(carried, 7);
        assert_eq!(fixed + carried, PositiveRow::ALL.len());
        assert_eq!(all_fixtures().len(), fixed);

        let ordinals: BTreeSet<u32> = PositiveRow::ALL.iter().map(|row| row.ordinal()).collect();
        assert_eq!(ordinals.len(), PositiveRow::ALL.len());
        assert_eq!(ordinals.iter().copied().min(), Some(1));
        assert_eq!(ordinals.iter().copied().max(), Some(14));
    }

    /// Recomputed by the same layer through a second call, so the
    /// assertion compares the stored expectation with the transition's
    /// answer rather than restating what was stored.
    #[test]
    fn each_successor_is_the_realization_transitions_own_answer() {
        for case in &all_fixtures() {
            let predecessor = case.predecessor();
            let derived = announce_maturity(&predecessor, case.announced_cycle(), case.bounds())
                .expect("the stated world is one the transition admits");
            assert_eq!(derived, case.expected(), "{:?}", case.id());
        }
    }

    #[test]
    fn the_successor_differs_from_its_predecessor_in_maturity_alone() {
        for case in &all_fixtures() {
            let predecessor = case.predecessor();
            let successor = case.expected();
            assert_eq!(predecessor.omega, successor.omega, "{:?}", case.id());
            assert_eq!(predecessor.y_l, successor.y_l, "{:?}", case.id());
            assert_eq!(predecessor.y_t, successor.y_t, "{:?}", case.id());
            assert_eq!(predecessor.q, successor.q, "{:?}", case.id());
            assert_eq!(predecessor.cycle, successor.cycle, "{:?}", case.id());
            assert_eq!(predecessor.maturity, Maturity::Unannounced);
            assert_eq!(
                successor.maturity,
                Maturity::Announced {
                    cycle: case.announced_cycle()
                }
            );
        }
    }

    #[test]
    fn the_lead_rows_sit_at_the_windows_bounds_and_inside_it() {
        let minimum = named(PositiveRow::MinimumValidLead);
        assert_eq!(minimum.lead(), minimum.bounds().minimum());

        let maximum = named(PositiveRow::MaximumValidLead);
        assert_eq!(maximum.lead(), maximum.bounds().maximum());

        let interior = named(PositiveRow::RepresentativeInteriorLead);
        assert!(interior.lead() > interior.bounds().minimum());
        assert!(interior.lead() < interior.bounds().maximum());

        for case in &all_fixtures() {
            assert_eq!(
                case.predecessor()
                    .cycle
                    .checked_add(case.lead())
                    .expect("the lead is a distance inside the cycle domain"),
                case.announced_cycle(),
                "{:?}",
                case.id()
            );
        }
    }

    #[test]
    fn the_domain_edge_rows_sit_where_the_realization_still_admits_them() {
        let smallest = named(PositiveRow::SmallestCurrentCycle);
        assert_eq!(smallest.predecessor().cycle, Cycle::ZERO);

        let near_top = named(PositiveRow::CurrentCycleNearCheckedUpperDomain);
        let current = near_top.predecessor().cycle;
        near_top
            .bounds()
            .window(current)
            .expect("the window resolves at the fixture's own current cycle");
        let next = current
            .checked_add(Cycle::new(1))
            .expect("one cycle past the fixture is still an ordinal");
        assert_eq!(
            near_top.bounds().window(next),
            Err(MaturityTransitionRefusal::CycleArithmeticOverflow)
        );
        assert_eq!(near_top.announced_cycle(), Cycle::MAX);
    }

    /// §15.1 asks every semantic field to be nontrivial, and §16.1's
    /// fourth row asks the current cycle to be the smallest there is.
    /// The two meet at that row alone, where its own name decides.
    #[test]
    fn every_unaffected_field_is_nontrivial_and_distinct() {
        for case in &all_fixtures() {
            let predecessor = case.predecessor();
            let pool = [
                predecessor.omega,
                predecessor.y_l,
                predecessor.y_t,
                predecessor.q,
            ];
            for amount in pool {
                assert_ne!(amount, ProtocolAmount::ZERO, "{:?}", case.id());
            }
            let distinct: BTreeSet<ProtocolAmount> = pool.iter().copied().collect();
            assert_eq!(distinct.len(), pool.len(), "{:?}", case.id());

            if case.row() == PositiveRow::SmallestCurrentCycle {
                assert_eq!(predecessor.cycle, Cycle::ZERO);
            } else {
                assert_ne!(predecessor.cycle, Cycle::ZERO, "{:?}", case.id());
            }
        }

        let spread = named(PositiveRow::NontrivialUnaffectedFields);
        let linked = named(PositiveRow::MinimumValidLead);
        assert_ne!(spread.predecessor().omega, linked.predecessor().omega);
        assert_ne!(spread.predecessor().q, linked.predecessor().q);
    }

    /// The exclusion list, checked against the rendering rather than
    /// trusted to the field list, because a field added later would pass
    /// a field-list review and fail this. Six terms are composed out of
    /// pieces: a text scan over this file counts them wherever they
    /// appear, and a module that spelled them here to exclude them would
    /// read exactly like one that carried them.
    #[test]
    fn a_case_renders_no_target_vocabulary() {
        let census = positive_semantic_census().expect("the positive census builds");
        let rendered = format!("{census:?}").to_lowercase();
        for forbidden in [
            concat!("control", "_block"),
            concat!("tap", "leaf"),
            concat!("leaf", "_script"),
            concat!("transaction", "_bytes"),
            concat!("signa", "ture"),
            concat!("target", "transaction"),
            "controlblock",
            "witness",
            "outpoint",
            "txid",
            "scriptpubkey",
            "program",
            "wallet",
            "input_index",
            "output_index",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "a semantic fixture rendered the target term {forbidden}"
            );
        }
    }

    #[test]
    fn every_carried_row_names_the_reason_its_ground_implies() {
        for (row, reason) in [
            (
                PositiveRow::RepresentationNonceZero,
                UnfixedReason::RepresentationSearchFact,
            ),
            (
                PositiveRow::RepresentationNonceNonzero,
                UnfixedReason::RepresentationSearchFact,
            ),
            (
                PositiveRow::SponsoredWithoutChange,
                UnfixedReason::SponsoredFormHasNoCarrier,
            ),
            (
                PositiveRow::SponsoredWithChange,
                UnfixedReason::SponsoredFormHasNoCarrier,
            ),
            (
                PositiveRow::PublicSuccessorRecovery,
                UnfixedReason::RecoveryComparesTargetMaterial,
            ),
            (
                PositiveRow::RepeatedEqualConstruction,
                UnfixedReason::PropertyOfRepeatedConstruction,
            ),
            (
                PositiveRow::AcceptedByTheTarget,
                UnfixedReason::PropertyOfAnAcceptedRun,
            ),
        ] {
            assert_eq!(row.carried_reason(), Some(reason));
            let carried = entry(row);
            assert_eq!(carried.carried(), Some(reason));
            assert_eq!(carried.fixture(), None);
        }

        for reason in UnfixedReason::ALL.iter().copied() {
            assert!(reason.reason().len() > 40, "{reason:?}");
            assert!(
                PositiveRow::ALL
                    .iter()
                    .any(|row| row.carried_reason() == Some(reason)),
                "{reason:?}"
            );
        }
    }

    #[test]
    fn the_outstanding_members_are_present_with_their_reasons() {
        assert_eq!(OutstandingProducer::ALL.len(), 2);
        for producer in OutstandingProducer::ALL.iter().copied() {
            assert!(producer.producer().len() > 10, "{producer:?}");
            assert!(producer.reason().len() > 40, "{producer:?}");
            assert!(producer.standing_evidence().len() > 40, "{producer:?}");
        }
        assert_ne!(
            OutstandingProducer::ExecutableModel.reason(),
            OutstandingProducer::TypedProjectionAdapter.reason()
        );

        assert_eq!(OutstandingRetention::ALL.len(), 1);
        for retention in OutstandingRetention::ALL.iter().copied() {
            assert!(retention.retained().len() > 10, "{retention:?}");
            assert!(retention.reason().len() > 40, "{retention:?}");
        }
    }

    /// The refusal is the realization's, carried whole: the fixture is
    /// filed under a row whose name it would otherwise contradict, and
    /// the transition refuses first because a world outside the window
    /// is not a positive world at all.
    #[test]
    fn a_lead_below_the_window_is_refused_by_the_realizations_own_refusal() {
        let case = named(PositiveRow::MinimumValidLead);
        let bounds = case.bounds();
        let below = Cycle::new(case.predecessor().cycle.get() + bounds.minimum().get() - 1);
        let id = SemanticFixtureId::new("one-cycle-below-the-minimum-lead", 0);
        let request = MaturityAnnouncementRequest::new(
            below,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
        )
        .expect("a sponsorless request asking for no change is admitted");

        assert_eq!(
            semantic_case(
                id,
                PositiveRow::MinimumValidLead,
                case.predecessor(),
                bounds,
                request
            ),
            Err(VectorError::ExpectedSuccessorRefused {
                fixture: id,
                refusal: MaturityTransitionRefusal::AnnouncementBelowMinimum,
            })
        );
    }

    /// Nothing is wrong with the world: it is the census's own, and the
    /// transition admits it. What is refused is the pairing, which is a
    /// different defect from an unbuildable world and is reported as one.
    #[test]
    fn a_carried_row_cannot_be_built_as_a_fixture() {
        let case = named(PositiveRow::MinimumValidLead);
        for row in [
            PositiveRow::RepresentationNonceZero,
            PositiveRow::SponsoredWithChange,
            PositiveRow::AcceptedByTheTarget,
        ] {
            let id = SemanticFixtureId::new(row.name(), row.ordinal());
            assert_eq!(
                semantic_case(id, row, case.predecessor(), case.bounds(), case.request()),
                Err(VectorError::FixtureContradictsItsClass {
                    fixture: id,
                    class: row.name(),
                }),
                "{}",
                row.name()
            );
        }
    }

    #[test]
    fn every_fixture_orders_one_state_subject_and_states_a_world_of_its_own() {
        assert_eq!(OPERATION, OperationId::AnnounceMaturity);
        assert_eq!(SUBJECT, ObjectId::State);

        let census = all_fixtures();
        for case in &census {
            assert_eq!(case.order(), CanonicalOrder::SINGLE_STATE_SUBJECT);
            assert_eq!(case.order().subject(), SUBJECT);
            assert_eq!(case.order().subjects(), 1);
            assert!(case.order().is_trivial());
            assert_eq!(case.predecessor().maturity, Maturity::Unannounced);
        }

        let identities: BTreeSet<SemanticFixtureId> =
            census.iter().map(MaturitySemanticCase::id).collect();
        assert_eq!(identities.len(), census.len());

        // No two rows are one world under two names: a run of either
        // would otherwise be evidence for the property the other is
        // named for.
        let worlds: HashSet<(
            StateMetadata,
            AnnouncementLeadBounds,
            MaturityAnnouncementRequest,
        )> = census
            .iter()
            .map(|case| (case.predecessor(), case.bounds(), case.request()))
            .collect();
        assert_eq!(worlds.len(), census.len());
    }

    /// Each class with a case subject, run over the registry's own
    /// cases. The lead pair carries the distinguishing negative — each
    /// fails on the other's case, which is what makes either a witness
    /// rather than a label. The single-field predicate has no
    /// constructible negative here: a case's only constructor derives
    /// its successor through the realization's own transition, so a case
    /// whose unaffected fields moved cannot be built at all, and that
    /// unbuildability is the property itself rather than a gap in the
    /// exercise. It is therefore run positively over every fixture.
    #[test]
    fn each_in_wave_class_holds_on_its_own_row_and_the_lead_classes_reject_each_other() {
        let census = all_fixtures();
        let keyed = census
            .iter()
            .filter(|case| case.row().witness().is_some())
            .count();
        assert_eq!(keyed, 3);

        for case in &census {
            if let Some(witness) = case.row().witness() {
                assert_eq!(witness.holds(case), ClassVerdict::Holds, "{:?}", case.id());
            }
            assert_eq!(
                MaturityClassWitness::SingleFieldTransition.holds(case),
                ClassVerdict::Holds,
                "{:?}",
                case.id()
            );
        }

        let minimum = named(PositiveRow::MinimumValidLead);
        let maximum = named(PositiveRow::MaximumValidLead);
        assert_eq!(
            MaturityClassWitness::MinimumLead.holds(&maximum),
            ClassVerdict::Fails
        );
        assert_eq!(
            MaturityClassWitness::MaximumLead.holds(&minimum),
            ClassVerdict::Fails
        );
    }

    /// The nonce class's subject is a link's retained search, so the
    /// predicate runs over the cheapest link the closure module
    /// publishes and the record behind its verdict is then read again by
    /// hand: a predicate agreeing with an independent reading of the
    /// same record is what keeps this from passing while claiming
    /// nothing.
    ///
    /// Its three refusals are exercised over records written here rather
    /// than over a link, because a scan that skipped a candidate, one
    /// that refused a candidate for a reason no nonce change repairs,
    /// and one that settled above the first candidate it admitted are
    /// all searches the constructor will not perform. A predicate
    /// nothing can refute states nothing, and these are the three ways
    /// this one can be.
    #[test]
    fn the_nonce_class_reads_the_links_own_retained_search() {
        let bundle = linked_maturity_bundle(
            MaturityDeployment::Demonstration,
            crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
        )
        .expect("the demonstration deployment links through the real curve");
        assert!(nonce_search_was_least_first(&bundle));
        assert_eq!(
            bundle.instances().len(),
            1,
            "the link retains the one application it ran"
        );

        for instance in bundle.instances() {
            let evidence = instance.constructor().evidence();
            for (earlier, (nonce, refusal)) in evidence.rejected.iter().enumerate() {
                assert!(refusal.retryable(), "{refusal:?}");
                assert_eq!(usize::try_from(nonce.get()), Ok(earlier));
            }
            assert_eq!(
                usize::try_from(evidence.selected.get()),
                Ok(evidence.rejected.len())
            );
        }

        let repairable = StateConstructorRefusal::CanonicalBranchSideNotSatisfied;
        assert!(repairable.retryable());
        assert!(search_was_least_first(&StateNonceEvidence {
            rejected: vec![(StateRepresentationNonce::new(0), repairable)],
            selected: StateRepresentationNonce::new(1),
        }));
        assert!(!search_was_least_first(&StateNonceEvidence {
            rejected: vec![(
                StateRepresentationNonce::new(0),
                StateConstructorRefusal::MetadataEncodingRefused
            )],
            selected: StateRepresentationNonce::new(1),
        }));
        assert!(!search_was_least_first(&StateNonceEvidence {
            rejected: vec![(StateRepresentationNonce::new(1), repairable)],
            selected: StateRepresentationNonce::new(2),
        }));
        assert!(!search_was_least_first(&StateNonceEvidence {
            rejected: vec![(StateRepresentationNonce::new(0), repairable)],
            selected: StateRepresentationNonce::new(2),
        }));

        for row in [
            PositiveRow::RepresentationNonceZero,
            PositiveRow::RepresentationNonceNonzero,
        ] {
            assert_eq!(
                row.witness(),
                Some(MaturityClassWitness::NonzeroRepresentationNonce)
            );
            assert_eq!(
                row.carried_reason(),
                Some(UnfixedReason::RepresentationSearchFact)
            );
        }
        assert_eq!(
            MaturityClassWitness::NonzeroRepresentationNonce
                .holds(&named(PositiveRow::MinimumValidLead)),
            ClassVerdict::NoCaseSubject(UnfixedReason::RepresentationSearchFact)
        );
    }

    /// The two classes whose subjects sit outside this layer answer the
    /// sponsored and the recovery rows and nothing else, each with the
    /// reason that puts it elsewhere — and that reason is the row's own,
    /// so a class carried here and a row no fixture answers cannot come
    /// to disagree about why.
    #[test]
    fn the_out_of_wave_classes_answer_exactly_the_sponsored_and_recovery_rows() {
        let case = named(PositiveRow::MinimumValidLead);
        for (row, witness, reason) in [
            (
                PositiveRow::SponsoredWithoutChange,
                MaturityClassWitness::Sponsored,
                UnfixedReason::SponsoredFormHasNoCarrier,
            ),
            (
                PositiveRow::SponsoredWithChange,
                MaturityClassWitness::Sponsored,
                UnfixedReason::SponsoredFormHasNoCarrier,
            ),
            (
                PositiveRow::PublicSuccessorRecovery,
                MaturityClassWitness::PublicRecovery,
                UnfixedReason::RecoveryComparesTargetMaterial,
            ),
        ] {
            assert_eq!(row.witness(), Some(witness));
            assert_eq!(witness.no_case_subject(), Some(reason));
            assert_eq!(row.carried_reason(), Some(reason));
            assert_eq!(witness.holds(&case), ClassVerdict::NoCaseSubject(reason));
        }

        let answering: Vec<PositiveRow> = PositiveRow::ALL
            .iter()
            .copied()
            .filter(|row| {
                matches!(
                    row.witness(),
                    Some(MaturityClassWitness::Sponsored | MaturityClassWitness::PublicRecovery)
                )
            })
            .collect();
        assert_eq!(
            answering,
            vec![
                PositiveRow::SponsoredWithoutChange,
                PositiveRow::SponsoredWithChange,
                PositiveRow::PublicSuccessorRecovery
            ]
        );
    }

    /// The keying, checked rather than argued: the transcription's §16.1
    /// section and this registry's rows are the same fourteen names in
    /// the same order, and each row's position lands on an entry of that
    /// section, so the entry a name is read from is the row it claims to
    /// be. The ordinal a reader checks against the guide is held to the
    /// same position, because two tables of one fact are two statements
    /// and this is where they meet.
    #[test]
    fn the_registry_reads_its_row_names_from_the_safety_matrix() {
        let transcribed: Vec<&'static str> = rows_of(MaturitySafetySection::Positive)
            .map(MaturitySafetyRow::name)
            .collect();
        let registered: Vec<&'static str> = PositiveRow::ALL
            .iter()
            .copied()
            .map(PositiveRow::name)
            .collect();
        assert_eq!(transcribed.len(), 14);
        assert_eq!(registered, transcribed);

        for row in PositiveRow::ALL.iter().copied() {
            let entry = &MATURITY_SAFETY_ROWS[row.position()];
            assert_eq!(entry.section(), MaturitySafetySection::Positive);
            assert_eq!(entry.name(), row.name());
            assert_eq!(usize::try_from(row.ordinal()), Ok(row.position() + 1));
        }
    }

    /// Every class §15.5 names states its own predicate and is reachable
    /// from a row, because a class nothing keys to is a predicate nothing
    /// runs; and the three whose subject is not a case say so, because a
    /// class silently inapplicable to a case would be indistinguishable
    /// from one a case refutes.
    #[test]
    fn every_class_states_its_predicate_and_is_keyed_to_a_row() {
        assert_eq!(MaturityClassWitness::ALL.len(), 6);

        let statements: BTreeSet<&'static str> = MaturityClassWitness::ALL
            .iter()
            .copied()
            .map(MaturityClassWitness::predicate)
            .collect();
        assert_eq!(statements.len(), MaturityClassWitness::ALL.len());

        for witness in MaturityClassWitness::ALL.iter().copied() {
            assert!(witness.predicate().len() > 20, "{witness:?}");
            assert!(
                PositiveRow::ALL
                    .iter()
                    .any(|row| row.witness() == Some(witness)),
                "{witness:?}"
            );
        }

        let carried: Vec<MaturityClassWitness> = MaturityClassWitness::ALL
            .iter()
            .copied()
            .filter(|witness| witness.no_case_subject().is_some())
            .collect();
        assert_eq!(
            carried,
            vec![
                MaturityClassWitness::NonzeroRepresentationNonce,
                MaturityClassWitness::Sponsored,
                MaturityClassWitness::PublicRecovery
            ]
        );
    }
}
