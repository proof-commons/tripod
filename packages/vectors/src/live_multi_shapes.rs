//! The restart order's fifth step: the remaining positive private shapes,
//! each as its own multi-output or multi-input fixture, against a real
//! node.
//!
//! # What this clears
//!
//! Step five of the mandatory restart order (task:guide-ctf-exec:restart-order)
//! is "the remaining positive private shapes, only where each one
//! accepts". Wave six stopped the order there and typed the stop
//! [`crate::live_evidence::LiveInfrastructureBlocker::MultiOutputShapeConstructorAbsent`]:
//! the one-to-one control's fixture and ceremony are fixed at two outputs
//! and one input, and a split needs a three-output manifest while a
//! many-to-many or a several-owner transfer needs more inputs and more
//! outputs than that construction supplied.
//!
//! This module builds those fixtures. The multi-output constructor is
//! `live_proof_bearing_observation::register_multi`, the
//! arity-general sibling of the one-to-one registration, and the ceremony
//! reuses the shared construction spine — the linked predecessor, the
//! funding step, the funded-coin observation, and the census-sign-assemble
//! tail — differing only in how many inputs it consumes and how many
//! outputs it creates.
//!
//! # What it does not do
//!
//! It moves no row by running. A row of the §15.2 matrix moves on an
//! observed acceptance, and the observation is the artifact the native
//! test produces rather than the existence of the ceremony. The
//! `run_of_record` identities are the ones one execution against a real
//! node produced, recorded so a later reader can ask the chain the same
//! question.
//!
//! # The shape it does not build, and why the reason changed
//!
//! A private-merge is one output, and the registry used to refuse any
//! manifest of fewer than two. On that ground this module recorded the
//! merge unconstructible, and recorded the guide's own merge predicate —
//! inputs at least two and outputs exactly one — as mutually
//! unsatisfiable with the registry's floor.
//!
//! THAT GROUND IS GONE. The floor was a first-party construction-model
//! convention rather than a consensus rule, the shape census typed it as
//! one, and it has been structurally removed: a manifest whose single
//! output declares the fully-solved balancing form registers, and the
//! strict one-to-one built on that form has been accepted by a real node.
//! The merge predicate is satisfiable as the guide states it.
//!
//! The merge still does not run HERE, and the reason is now a different
//! one that this module should not let a reader confuse with the old one.
//! This ceremony funds ONE predecessor from an explicit input, so that
//! predecessor's two output blinders are ordered additive inverses. A
//! merge consuming both halves of an inverse pair presents a ZERO input
//! blinder sum, and the lone output's blinder is forced to that sum — a
//! commitment of exactly the value times the value generator, hiding
//! nothing while the tally still balances. The registry refuses it, and
//! refusing it is correct.
//!
//! So the wall moved from the output count to the blinders, which is to
//! say from an accident to the property that actually matters. What the
//! merge needs is a precursor whose outputs do not cancel, and the census
//! files that as its removal path.

use std::collections::BTreeMap;

use linker::OwnerParameter;
use linker::live_backend::{LiveTransferComposition, LiveTransferRepresentationPlan};
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationStepKind,
    OperationSubject, TargetSubmissionSubject,
};
use transaction::bytes::{Outpoint, ValueField};
use transaction::live_census::OwnerSigningCensus;
use transaction::live_construct::{
    PrivateDestinationOpening, PrivateInputOpening, PrivateLiveFinalization, PrivateLiveOpenings,
    finalize_private_live_transfer_composing,
};
use transaction::live_materialize::{
    ConfidentialInputRegion, ConfidentialOutputRole, FixtureOpeningReference,
    FrozenConfidentialFixtureView, NonProtocolFundingRegion, SCALAR_BYTES,
};
use transaction::live_message::{WitnessVectorTreatment, candidate_owner_message};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SponsorChangeRequest,
};
use transaction::taproot::Digest32;
use transaction::view::{PublicConstructionView, PublicOutputView};

use target_elements_conformance::confidential_fixture::{
    ConfidentialFixtureOutput, FixtureOutputRole,
};

use crate::confidential_materializer::{
    FirstPartyCommitmentCheck, ReferenceConfidentialMaterializer,
};
use crate::confidential_predecessor::{PredecessorShape, TRIPLE_PREDECESSOR_AMOUNTS};
use crate::error::VectorError;
use crate::live_owner_observation::printed_order;
use crate::live_plan::{
    FIRST_SCALAR, LiveShapeVocabulary, RESERVE_ASSET, SECOND_SCALAR, published_owner,
    reviewed_target,
};
use crate::live_private_restart::{
    ConsumedReceipt, LinkedDeployment, PrivateRestartRefusal, RestartConfidentialCoin,
    assemble_control, confidential_funding_step, explicit_funding_step, issue_step,
    link_and_register_composing, observe_explicit_coins, observe_funded_coins, owner_program,
    verify_readback_signature,
};
use crate::live_proof_bearing_observation::{materialization_profiles, register_multi};

/// One destination of a shape: which published owner receives it and how
/// much, in the order the outputs are created.
#[derive(Clone, Copy, Debug)]
struct Destination {
    /// The receiving owner's published scalar.
    scalar: [u8; SCALAR_BYTES],
    /// The semantic amount the output carries.
    amount: u64,
    /// What this output does in the balance.
    ///
    /// Stated per destination rather than derived from its position. A
    /// shape whose single output is fully solved cannot be described by a
    /// position at all, because the fully-solved form is something a
    /// manifest DECLARES.
    role: FixtureOutputRole,
}

/// The remaining positive private shapes of §15.2 this wave builds.
///
/// Each is a concrete representative case, not a family: the matrix row it
/// moves is moved on an acceptance of THIS shape, and a representative is
/// named as one rather than presented as a proof over every shape of its
/// class.
/// What the fee-bearing shape pays as its fee, in the protocol asset.
///
/// # Why this figure
///
/// It is the fee the sponsor arc's accepted control carried, so the two
/// fee-bearing acceptances this workspace has recorded carry the same
/// figure and a reader comparing them is comparing one number. That is
/// the whole of the reason: nothing about the target requires this
/// amount, and the asset is NOT the same one — the sponsor arc paid in
/// the reserve asset and this shape pays in the disposable protocol asset
/// its own consumed coin carries, because that is the only asset whose
/// tally the fee can close.
///
/// It is nonzero, which is not a preference. A zero-value explicit output
/// is admitted by the target only where its scriptPubKey is unspendable,
/// and an empty script is not unspendable, so a zero fee output is
/// refused outright — the fee-only row of the shape register records the
/// same clause.
const FEE_AMOUNT: u64 = 250;

/// What the entry crossing's single explicit receipt carries.
///
/// Small, because nothing about the shape depends on the figure and a
/// large one would only make the explicit amount it publishes look
/// meaningful. What matters is that it is nonzero and that the two
/// destinations sum to it.
const ENTRY_CROSSING_RECEIPT: u64 = 5_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrivateShape {
    /// One receipt in, three outputs: two recipients and one balancing
    /// change output back to the sender.
    Split,
    /// ONE receipt in and TWO blinded receipts out, with no change and no
    /// fee: the PURE split.
    ///
    /// The shape §16.1's split pair states for its private member, which
    /// is not the shape [`Self::Split`] runs. That one creates three
    /// outputs because it keeps a balancing change back for the sender;
    /// this one creates exactly the two the pair's fixture names, and the
    /// second of them is a RECEIPT rather than the fee role the only
    /// other recorded one-in-two-out private run carries. A pair member
    /// is not answered by a run of a different cardinality nor by one
    /// whose second output is a different role, so neither recorded run
    /// reaches it and this shape exists to be run.
    ///
    /// It is constructible for the same arithmetic that makes the entry
    /// crossing constructible at two outputs: the first output's blinder
    /// is DERIVED and the second is solved from the consumed sum less
    /// that one. What differs is the input side -- a single confidential
    /// coin's blinder rather than an explicit input's zero -- so the
    /// solved blinder is a nonzero sum less a derived value, and the
    /// registry refuses a zero one by name if that ever comes out wrong.
    ///
    /// It pays NO fee, which is the pair fixture's own statement: its two
    /// destinations consume the whole source. Every identity the
    /// confidential lane has recorded was built sponsorless and carries
    /// no fee output, so this is the lane's ordinary case rather than a
    /// concession made to reach the pair.
    PureSplit,
    /// Two receipts in, three outputs: the representative many-to-many
    /// case, chosen as the smallest transfer whose input and output
    /// cardinalities both exceed the one-to-one control's.
    ManyToMany,
    /// Two receipts in, two outputs, the two inputs owned by two distinct
    /// published owners, each input carrying the leaf its position
    /// executes.
    SeveralDistinctOwners,
    /// ONE receipt in and ONE output out: the strict one-to-one.
    ///
    /// The shape the registry's two-output floor used to refuse. Its lone
    /// output declares the fully-solved balancing form and takes the
    /// consumed coin's own value blinder, so there is no second output to
    /// absorb anything and none is needed.
    ///
    /// It is a one-INPUT shape, which is what keeps it constructible: the
    /// input blinder sum is a single consumed coin's blinder and cannot
    /// cancel against anything. A two-input merge of this ceremony's
    /// inverse-pair predecessor would force a ZERO blinder, and that the
    /// registry refuses.
    StrictOneToOne,
    /// ONE receipt in, one blinded output and one FEE output out.
    ///
    /// The first shape this lane builds that pays a fee at all. Every
    /// identity the confidential lane has recorded was built sponsorless
    /// and carries no fee output, so this one is the shape where the
    /// target's fee rules are exercised rather than avoided.
    ///
    /// The blinded output is the BALANCING one and not the sole form: a
    /// manifest of two outputs may not declare the sole form, and it does
    /// not need to. The fee is held out of the solve at a zero blinder, so
    /// the balancing output's blinder is solved over no others and comes
    /// out as the consumed coin's own blinder -- nonzero, because a single
    /// consumed coin's blinder is.
    ///
    /// The fee is in the PROTOCOL asset, which is the only asset that
    /// makes the tally close. A fee in some other asset would leave the
    /// protocol sum short by the fee.
    OneToOneWithFee,
    /// TWO receipts in and ONE output out: the private merge.
    ///
    /// The shape that met two walls. The first was the registry's
    /// two-output floor, removed when the sole-balancing form landed; the
    /// second was the zero blinder the only available predecessor forced,
    /// its two output blinders being ordered additive inverses.
    ///
    /// This shape spends the THREE-output non-canceling predecessor
    /// instead, and that is the whole of what makes it constructible.
    /// Three blinders summing to zero cancel in no pair: consuming
    /// outputs zero and one leaves a sum equal to the negation of output
    /// two's blinder, which is a DERIVED blinder and therefore never
    /// zero. The lone output's blinder is forced to that sum, so it is
    /// nonzero for a reason that can be stated rather than hoped for --
    /// and the registry would refuse a zero one by name if it were wrong.
    PrivateMerge,
    /// Blinded receipts spent into EXPLICIT destinations beside the
    /// blinded absorber a nonzero consumed blinder sum requires.
    ///
    /// The exit direction of representation crossing, and the one shape
    /// here whose consumed and created sides are read under different
    /// plans. It spends the triple predecessor's non-canceling pair for
    /// the merge's reason said the other way round: the merge needs a
    /// nonzero forced blinder so its lone output hides something, and
    /// this shape needs one so its absorber has something to absorb.
    ExitCrossing,
    /// EXPLICIT receipts spent into two blinded destinations.
    ///
    /// The entry direction of representation crossing, and the only
    /// shape here whose predecessor is not a confidential fixture at
    /// all: it spends an ordinary explicit coin sitting at a receipt
    /// constructor's program, which is what makes it a covenant-governed
    /// TRANSFER rather than the funding step this workspace has always
    /// performed.
    ///
    /// TWO blinded destinations and never one. A single one would have
    /// to declare the sole-balancing form, the solve would return the
    /// consumed blinder sum unchanged, and that sum is ZERO because
    /// explicit coins carry zero blinders -- a commitment that hides
    /// nothing. Both this registry and Elements' own wallet refuse that,
    /// for the same reason, and neither is a protocol rule.
    EntryCrossing,
    /// The PRIVATE member of the pairs arc's §16.1 one-to-one pair.
    ///
    /// One confidential receipt consumed and one created, and in that it
    /// is the strict one-to-one's shape. What makes it a different member
    /// of this enum is not its width: it is that every parameter it has
    /// is READ OFF the arc's one semantic fixture
    /// ([`crate::live_pair_arc::pair_arc_fixture`]) rather than written
    /// here, and the explicit lane's paired member reads the same
    /// fixture. §16.1's load-bearing requirement is that a pair begins
    /// from ONE fixture materialized twice, and two shapes stated
    /// independently by two authors do not satisfy it however alike they
    /// come out — which is precisely the substitution the pair registry's
    /// `NotSubmittedShapeAcceptedElsewhere` verdict exists to deny.
    ///
    /// It consumes the dual-parity predecessor's PRIMARY coin, and the
    /// fixture's source amount is that coin's amount for the reason the
    /// explicit member's funding amount follows the fixture: the
    /// confidential side spends a coin a registered predecessor already
    /// carries, and the explicit side's funding step can be asked for any
    /// amount.
    PairedOneToOne,
}

impl PrivateShape {
    /// All nine, in the order the restart runs them.
    ///
    /// The pure split is LAST rather than beside the split it is a
    /// sibling of, and the position is load-bearing. This array's order
    /// is read positionally by the byte-identity tests, which pin each
    /// recorded successor digest at its index; inserting a shape in the
    /// middle would renumber every digest after it and make a test that
    /// exists to catch drift report drift that did not happen. A shape
    /// added after a run is appended, so the indices a run wrote down
    /// keep meaning what they meant.
    pub const ALL: [Self; 10] = [
        Self::Split,
        Self::ManyToMany,
        Self::SeveralDistinctOwners,
        Self::StrictOneToOne,
        Self::OneToOneWithFee,
        Self::PrivateMerge,
        Self::ExitCrossing,
        Self::EntryCrossing,
        Self::PureSplit,
        Self::PairedOneToOne,
    ];

    /// The ceremony's own name for the shape, used as the report
    /// extension and the successor fixture handle's discriminator.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Split => "private-split",
            Self::PureSplit => "private-pure-split",
            Self::ManyToMany => "private-many-to-many",
            Self::SeveralDistinctOwners => "private-several-distinct-owners",
            Self::StrictOneToOne => "private-strict-one-to-one",
            Self::OneToOneWithFee => "private-one-to-one-with-fee",
            Self::PrivateMerge => "private-merge",
            Self::ExitCrossing => "private-exit-crossing",
            Self::EntryCrossing => "private-entry-crossing",
            Self::PairedOneToOne => "private-paired-one-to-one",
        }
    }

    /// The §15.2 safety-matrix row this shape's acceptance moves, where
    /// it moves one.
    ///
    /// `None` is a real answer and not a missing entry. The strict
    /// one-to-one and the fee-bearing one-to-one are shapes of the
    /// CONSENSUS census, which enumerates what the target's balance rule
    /// admits; the §15.2 positive private table enumerates the guide's own
    /// classes and has no member for either. An acceptance of one
    /// therefore moves a census entry and no row, and naming a row here
    /// that the table does not carry would be inventing one to have
    /// something to move.
    ///
    /// The fee-bearing shape is emphatically NOT `private-sponsor-values`.
    /// That row is about a sponsor paying another party's fee, whose
    /// signer dependency this workspace does not close; this shape pays
    /// its own fee out of its own consumed coin, and mapping it onto that
    /// row would answer a question nobody asked it.
    #[must_use]
    pub const fn row_name(self) -> Option<&'static str> {
        match self {
            Self::Split => Some("private-split"),
            Self::ManyToMany => Some("private-many-to-many-representative"),
            Self::SeveralDistinctOwners => Some("private-several-distinct-owners"),
            // Four shapes name no row, and for ONE reason rather than
            // four: §15.2's positive private table enumerates the
            // guide's own classes, and it has no member for a strict
            // one-to-one, for a transfer that pays its own fee, or for
            // one whose two sides are read under different plans. Each
            // moves a census entry instead, and naming a row here that
            // the table does not carry would be inventing one to have
            // something to move.
            //
            // The pure split shares the arm for a DIFFERENT reason, and
            // the reason is written here because a shared `None` hides
            // it. §15.2 DOES carry a `private-split` row, and
            // [`Self::Split`] already moved it on an acceptance of its
            // own three-output shape. This shape exists for §16.1's
            // split PAIR, whose private member states two created
            // outputs, and a pair member is not a matrix row. Pointing
            // it at `private-split` would move a row that has already
            // moved, and would claim the three-output run and this one
            // are the same shape -- which is the very fact the pair's
            // failing conjunct records.
            //
            // The arc's paired member shares the arm for a THIRD reason.
            // §15.2 carries `projection-equality-with-paired-explicit`
            // and this shape is half of what answers it — but a row is
            // moved by a RELATION over both members' acceptances and not
            // by either acceptance alone, so the member cannot name the
            // row. The arc's ledger moves it, and naming it here would
            // let one half of a pair move a row about the pair.
            Self::StrictOneToOne
            | Self::OneToOneWithFee
            | Self::ExitCrossing
            | Self::EntryCrossing
            | Self::PureSplit
            | Self::PairedOneToOne => None,
            // The merge DOES have a row, and it is the only shape of
            // these that has one.
            Self::PrivateMerge => Some("private-merge"),
        }
    }

    /// How many receipts this shape consumes.
    ///
    /// Read off `Self::consumed` rather than stated, so a shape whose
    /// consumed set changed cannot go on reporting the old width.
    #[must_use]
    pub const fn input_count(self) -> usize {
        self.consumed().len()
    }

    /// How many outputs this shape creates.
    #[must_use]
    pub fn output_count(self) -> usize {
        self.destinations().len()
    }

    /// The successor fixture's handle, its own per shape so a digest drift
    /// between two shapes is detectable.
    #[must_use]
    fn successor_handle(self) -> String {
        format!("ctf-v1/wave-seven-{}-successor", self.name())
    }

    /// Which predecessor this shape's ceremony funds.
    ///
    /// Every shape but the merge spends the dual-parity predecessor,
    /// whose two coins are all any of them needs. The merge spends the
    /// three-output non-canceling one, because the dual-parity coins are
    /// an inverse pair and merging both halves of an inverse pair forces
    /// a zero blinder -- a commitment that hides nothing while the tally
    /// still balances, which the registry refuses and should.
    #[must_use]
    pub const fn predecessor(self) -> PredecessorShape {
        match self {
            Self::Split
            | Self::PureSplit
            | Self::ManyToMany
            | Self::SeveralDistinctOwners
            | Self::StrictOneToOne
            | Self::PairedOneToOne
            | Self::OneToOneWithFee => PredecessorShape::DualParity,
            // The entry crossing names one too, and never funds it. Its
            // consumed coins are explicit and belong to no confidential
            // fixture; the shape is carried so every other accessor that
            // asks stays total, and `funds_explicitly` is what decides
            // which funding step actually runs.
            Self::PrivateMerge | Self::ExitCrossing | Self::EntryCrossing => {
                PredecessorShape::TripleNonCanceling
            }
        }
    }

    /// Which shape vocabulary this shape's deployment emits programs
    /// for.
    ///
    /// Only the fee-bearing shape asks for the fee-bearing candidate, and
    /// the narrowness is the point rather than caution. A candidate's
    /// shape set becomes one coordinator leaf per shape, the leaves tweak
    /// the taproot output key, and that key is the destination program
    /// each of these shapes' recorded successor digests was taken over --
    /// so linking a shape against the wider vocabulary would move a
    /// digest that a run against a pinned node already wrote down. Five
    /// of the six keep the demonstration candidate and keep their
    /// digests, byte for byte.
    #[must_use]
    pub const fn vocabulary(self) -> LiveShapeVocabulary {
        match self {
            Self::Split
            | Self::PureSplit
            | Self::ManyToMany
            | Self::SeveralDistinctOwners
            | Self::StrictOneToOne
            | Self::PairedOneToOne
            | Self::PrivateMerge
            | Self::ExitCrossing
            | Self::EntryCrossing => LiveShapeVocabulary::Demonstration,
            Self::OneToOneWithFee => LiveShapeVocabulary::FeeBearing,
        }
    }

    /// How this shape pairs a representation plan to each of its sides.
    ///
    /// Six of the seven are wholly private and say so. The exit crossing
    /// is the one that is not, and stating the composition per shape is
    /// what lets one ceremony serve both -- the crossing is a private-lane
    /// shape, because every composition but the wholly explicit one has a
    /// blinded field somebody must build.
    ///
    /// A crossing composition also picks a different DEPLOYMENT, because
    /// it seats its crossing constructor at the key its consumed side is
    /// recognized under. That deployment's taptree is its own, exactly as
    /// the fee-bearing vocabulary's is, so no digest any of the other six
    /// recorded can move to buy it.
    #[must_use]
    pub const fn composition(self) -> LiveTransferComposition {
        match self {
            Self::Split
            | Self::PureSplit
            | Self::ManyToMany
            | Self::SeveralDistinctOwners
            | Self::StrictOneToOne
            | Self::PairedOneToOne
            | Self::OneToOneWithFee
            | Self::PrivateMerge => LiveTransferComposition::HomogeneousPrivate,
            Self::ExitCrossing => LiveTransferComposition::ExitUnblinding,
            Self::EntryCrossing => LiveTransferComposition::EntryBlinding,
        }
    }

    /// Whether this shape's predecessor coins are funded EXPLICITLY.
    ///
    /// True for the entry crossing alone, and it is the one fact that
    /// decides which funding step runs. A confidential predecessor is a
    /// registered fixture whose openings the ceremony re-derives; an
    /// explicit one is an ordinary coin at a receipt program, with no
    /// opening to derive and a zero blinder to contribute.
    #[must_use]
    pub const fn funds_explicitly(self) -> bool {
        matches!(self, Self::EntryCrossing)
    }

    /// How many explicit receipt coins the entry crossing is funded
    /// with, and what each carries.
    #[must_use]
    pub const fn explicit_funding(self) -> Option<(u8, u64)> {
        match self {
            Self::EntryCrossing => Some((1, ENTRY_CROSSING_RECEIPT)),
            _ => None,
        }
    }

    /// How many of this shape's outputs are EXPLICIT receipt
    /// destinations.
    ///
    /// Read by the same caller that reads [`Self::fee_output_count`] and
    /// for the same reason: an explicit value carries no range proof, so
    /// a lane asserting a proof on every output-witness entry would fail
    /// on this shape. The two counts are separate because the outputs are
    /// different things -- a fee has no program and these have real ones
    /// -- and folding them into one number would let a ceremony that
    /// built a fee where a destination belonged still pass.
    #[must_use]
    pub const fn explicit_destination_count(self) -> usize {
        match self {
            Self::Split
            | Self::PureSplit
            | Self::ManyToMany
            | Self::SeveralDistinctOwners
            | Self::StrictOneToOne
            | Self::PairedOneToOne
            | Self::OneToOneWithFee
            | Self::PrivateMerge
            | Self::EntryCrossing => 0,
            Self::ExitCrossing => 2,
        }
    }

    /// How many of this shape's outputs are fee outputs.
    ///
    /// Read by a caller that needs to know which output-witness entries
    /// are EXPECTED to be empty. A fee output carries an explicit value
    /// and therefore no range proof, so a lane asserting a proof on every
    /// entry would fail on the one shape that pays a fee — and loosening
    /// that assertion to "some entries carry proofs" would stop it
    /// catching a blinded output that lost its proof, which is the thing
    /// it exists to catch.
    #[must_use]
    pub const fn fee_output_count(self) -> usize {
        match self {
            Self::Split
            | Self::PureSplit
            | Self::ManyToMany
            | Self::SeveralDistinctOwners
            | Self::StrictOneToOne
            | Self::PairedOneToOne
            | Self::PrivateMerge
            | Self::ExitCrossing
            | Self::EntryCrossing => 0,
            Self::OneToOneWithFee => 1,
        }
    }

    /// Which predecessor outputs this shape consumes, in fixed order.
    #[must_use]
    const fn consumed(self) -> &'static [ConsumedReceipt] {
        match self {
            Self::Split
            | Self::PureSplit
            | Self::StrictOneToOne
            | Self::PairedOneToOne
            | Self::OneToOneWithFee
            | Self::EntryCrossing => &[ConsumedReceipt::Primary],
            // The merge consumes the same two INDICES the two-input
            // shapes do. Against the triple predecessor those indices
            // carry the same two roles, and the difference that matters
            // is the third output standing beside them: it is why the
            // pair's blinders do not cancel.
            Self::ManyToMany
            | Self::SeveralDistinctOwners
            | Self::PrivateMerge
            | Self::ExitCrossing => &[ConsumedReceipt::Primary, ConsumedReceipt::Balancing],
        }
    }

    /// The shape's destinations, in output order, the last of which is the
    /// balancing output.
    ///
    /// The sums are the consumed receipts' own semantic amounts, which is
    /// what conservation means on this lane; the target checks it by
    /// commitment balance and a set that did not add up would be refused
    /// there rather than here.
    #[must_use]
    fn destinations(self) -> Vec<Destination> {
        // A primary output's blinder is derived; the balancing one's is
        // solved from the others. Every shape below states which is which
        // rather than leaving it to be read off the output order.
        let primary = |scalar, amount| Destination {
            scalar,
            amount,
            role: FixtureOutputRole::Primary,
        };
        let balancing = |scalar, amount| Destination {
            scalar,
            amount,
            role: FixtureOutputRole::Balancing,
        };
        // An EXPLICIT receipt destination: a real program and a public
        // amount, carrying no opening at all. It brings no blinder to the
        // solve and joins the sum at the all-zero one every explicit
        // value is committed with.
        let explicit = |scalar, amount| Destination {
            scalar,
            amount,
            role: FixtureOutputRole::ExplicitDestination,
        };
        match self {
            // 700_000_000 in, split three ways back to the two owners.
            Self::Split => vec![
                primary(SECOND_SCALAR, 400_000_000),
                primary(FIRST_SCALAR, 200_000_000),
                balancing(FIRST_SCALAR, 100_000_000),
            ],
            // 700_000_000 in, split TWO ways between the two owners and
            // nothing held back. The whole consumed amount travels, so
            // there is no change output and no fee, which is exactly the
            // difference between this shape and the three-output split
            // above it.
            Self::PureSplit => vec![
                primary(SECOND_SCALAR, 400_000_000),
                balancing(FIRST_SCALAR, 300_000_000),
            ],
            // 1_000_000_000 in across two receipts, three ways out.
            Self::ManyToMany => vec![
                primary(SECOND_SCALAR, 500_000_000),
                primary(FIRST_SCALAR, 300_000_000),
                balancing(SECOND_SCALAR, 200_000_000),
            ],
            // 1_000_000_000 in across two distinctly owned receipts, two
            // ways out.
            Self::SeveralDistinctOwners => {
                vec![
                    primary(SECOND_SCALAR, 600_000_000),
                    balancing(FIRST_SCALAR, 400_000_000),
                ]
            }
            // ONE receipt in and ONE output out: the strict one-to-one,
            // whose lone output declares the fully-solved form and takes
            // the consumed coin's own blinder. Nothing is split and
            // nothing changes hands twice, so the whole consumed amount
            // travels to the second owner.
            Self::StrictOneToOne => vec![Destination {
                scalar: SECOND_SCALAR,
                amount: 700_000_000,
                role: FixtureOutputRole::SoleBalancing,
            }],
            // ONE receipt in, one blinded output and one fee. The fee's
            // scalar is never read -- a fee output carries no program at
            // all, which is most of what makes it a fee -- and it is
            // written as the second owner's only so the destination
            // literal has the shape its neighbours have.
            Self::OneToOneWithFee => vec![
                balancing(SECOND_SCALAR, 700_000_000 - FEE_AMOUNT),
                Destination {
                    scalar: SECOND_SCALAR,
                    amount: FEE_AMOUNT,
                    role: FixtureOutputRole::Fee,
                },
            ],
            // 900_000_000 in across the triple predecessor's first two
            // coins, ONE way out. The lone output declares the
            // fully-solved form, exactly as the strict one-to-one's does;
            // what is new is that the sum it is forced to comes from two
            // coins rather than one, and does not cancel.
            Self::PrivateMerge => vec![Destination {
                scalar: SECOND_SCALAR,
                amount: TRIPLE_PREDECESSOR_AMOUNTS[0] + TRIPLE_PREDECESSOR_AMOUNTS[1],
                role: FixtureOutputRole::SoleBalancing,
            }],
            // 900_000_000 in across the triple predecessor's non-canceling
            // pair, out as two EXPLICIT destinations and one blinded
            // absorber. The absorber is LAST, and that is not a layout
            // choice: the covenant's positional leaf declares the last
            // destination as the absorber and checks that position and no
            // other, so a set that put it anywhere else would build a
            // candidate the covenant refuses.
            //
            // The absorber's blinder is not stated here and is not
            // chosen. With no output deriving one there is nothing to
            // subtract, so the registry's solve returns the consumed
            // blinder sum ITSELF -- which is why the pair must not cancel
            // and why this shape spends the same predecessor the merge
            // does.
            // Read off the arc's ONE fixture, endpoint for endpoint. The
            // form is SOLE-BALANCING for the reason the strict
            // one-to-one's is: a single output has nothing beside it to
            // absorb anything, so its blinder is forced to the consumed
            // coin's own. A fixture stating a second destination would
            // not materialize here, and the arc's own test is what holds
            // the fixture to the shape this arm can build rather than
            // leaving the agreement to be assumed.
            Self::PairedOneToOne => crate::live_pair_arc::pair_arc_fixture()
                .destinations()
                .iter()
                .map(|endpoint| Destination {
                    scalar: crate::live_pair_arc::published_scalar(endpoint.owner()),
                    amount: endpoint.amount(),
                    role: FixtureOutputRole::SoleBalancing,
                })
                .collect(),
            // ONE explicit receipt in, TWO blinded destinations out.
            // The floor of two is the registry's own arithmetic and not
            // a preference: an explicit input contributes a zero
            // blinder, so a single blinded output would be forced to a
            // zero blinder and hide nothing.
            Self::EntryCrossing => vec![
                primary(SECOND_SCALAR, ENTRY_CROSSING_RECEIPT - 1_000),
                balancing(FIRST_SCALAR, 1_000),
            ],
            Self::ExitCrossing => vec![
                explicit(SECOND_SCALAR, 500_000_000),
                explicit(FIRST_SCALAR, 300_000_000),
                balancing(
                    SECOND_SCALAR,
                    TRIPLE_PREDECESSOR_AMOUNTS[0] + TRIPLE_PREDECESSOR_AMOUNTS[1]
                        - 500_000_000
                        - 300_000_000,
                ),
            ],
        }
    }
}

/// What a sole-balancing output's FORCED blinder came out to be.
///
/// Four facts, none of them the blinder itself. A record that carried the
/// scalar would be publishing an opening; what a reader needs is whether
/// it is zero, and these say so alongside the arithmetic that makes the
/// answer checkable.
///
/// `forced_blinder_is_the_consumed_sum` is the one that makes the other
/// three mean anything. Without it a reader has the registry's word that
/// the forced blinder is the consumed sum; with it the ceremony has
/// summed the coins it is actually spending, independently of the
/// registry's own solve, and compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForcedBlinderCensus {
    consumed_coins: usize,
    consumed_sum_is_zero: bool,
    forced_blinder_is_zero: bool,
    forced_blinder_is_the_consumed_sum: bool,
}

impl ForcedBlinderCensus {
    /// How many coins the sum was taken over.
    #[must_use]
    pub const fn consumed_coins(self) -> usize {
        self.consumed_coins
    }

    /// Whether the consumed coins' blinders cancel.
    ///
    /// True for a merge of an inverse pair, which is exactly the case the
    /// registry refuses.
    #[must_use]
    pub const fn consumed_sum_is_zero(self) -> bool {
        self.consumed_sum_is_zero
    }

    /// Whether the forced blinder is the zero scalar.
    ///
    /// The question the whole census exists to answer. A blinded output
    /// whose blinder is zero hides nothing, and nothing anywhere in this
    /// workspace may claim hiding for one.
    #[must_use]
    pub const fn forced_blinder_is_zero(self) -> bool {
        self.forced_blinder_is_zero
    }

    /// Whether the forced blinder is the sum the ceremony computed.
    #[must_use]
    pub const fn forced_blinder_is_the_consumed_sum(self) -> bool {
        self.forced_blinder_is_the_consumed_sum
    }
}

/// One funded confidential coin the node reported, as the ceremony carries
/// it into the successor.
///
/// The linked deployment and the shared spine produce
/// [`RestartConfidentialCoin`]s; this ceremony consumes one or two of
/// them.
type Coin = RestartConfidentialCoin;

/// What the second origin checked, where an acceptance was observed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiShapeReverification {
    accepted_txid: String,
    readback_matches_submission: bool,
    verified: bool,
}

impl MultiShapeReverification {
    /// The identity the target computed.
    #[must_use]
    pub fn accepted_txid(&self) -> &str {
        &self.accepted_txid
    }

    /// Whether the bytes the node reported are the bytes it was handed.
    #[must_use]
    pub const fn readback_matches_submission(&self) -> bool {
        self.readback_matches_submission
    }

    /// Whether the accepted witness verifies against the independently
    /// recomputed message.
    #[must_use]
    pub const fn verified(&self) -> bool {
        self.verified
    }
}

/// The transcript one run of a shape produces.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MultiShapeRecord {
    shape: Option<&'static str>,
    input_count: usize,
    output_count: usize,
    issued_asset: Option<String>,
    predecessor_digest: Option<[u8; 32]>,
    successor_digest: Option<[u8; 32]>,
    coins: Vec<Coin>,
    receipt_leaves: usize,
    output_witness_proof_bytes: Vec<usize>,
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
    accepted_txid: Option<String>,
    reverification: Option<MultiShapeReverification>,
    /// The weight the TARGET reported for the submitted candidate.
    ///
    /// Read off the node's own decode rather than computed here, which
    /// is the only figure §20.5's comparison can use: a weight this
    /// workspace calculated would be comparing its arithmetic with
    /// itself. `None` where no submission reached the node.
    observed_weight: Option<u64>,
    refusal: Option<PrivateRestartRefusal>,
    forced_blinder: Option<ForcedBlinderCensus>,
}

impl MultiShapeRecord {
    /// The shape's own name.
    #[must_use]
    pub const fn shape(&self) -> Option<&'static str> {
        self.shape
    }

    /// How many receipts the successor consumed.
    #[must_use]
    pub const fn input_count(&self) -> usize {
        self.input_count
    }

    /// How many outputs the successor created.
    #[must_use]
    pub const fn output_count(&self) -> usize {
        self.output_count
    }

    /// What the sole output's forced blinder came out to be, for a shape
    /// that has one.
    #[must_use]
    pub const fn forced_blinder(&self) -> Option<ForcedBlinderCensus> {
        self.forced_blinder
    }

    /// The disposable asset the run issued.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// The successor fixture's digest.
    #[must_use]
    pub const fn successor_digest(&self) -> Option<[u8; 32]> {
        self.successor_digest
    }

    /// The coins the node reported for the confidential funding step.
    #[must_use]
    pub fn coins(&self) -> &[Coin] {
        &self.coins
    }

    /// How many receipt leaves the control consumed.
    #[must_use]
    pub const fn receipt_leaves(&self) -> usize {
        self.receipt_leaves
    }

    /// The range-proof byte counts of the candidate's own outputs.
    #[must_use]
    pub fn output_witness_proof_bytes(&self) -> &[usize] {
        &self.output_witness_proof_bytes
    }

    /// How many bytes were handed to the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The layer the node's verdict arrived at.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The node's own words.
    #[must_use]
    pub fn observed_detail(&self) -> Option<&str> {
        self.observed_detail.as_deref()
    }

    /// The identity the target computed, where it accepted.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// The weight the target reported, where a candidate reached it.
    #[must_use]
    pub const fn observed_weight(&self) -> Option<u64> {
        self.observed_weight
    }

    /// The second origin's answer, where there was an acceptance to check.
    #[must_use]
    pub const fn reverification(&self) -> Option<&MultiShapeReverification> {
        self.reverification.as_ref()
    }

    /// The construction refusal, where the ceremony stopped before the
    /// node.
    #[must_use]
    pub const fn refusal(&self) -> Option<&PrivateRestartRefusal> {
        self.refusal.as_ref()
    }

    /// Whether this run produced an accepted control whose witness the
    /// second origin verified.
    #[must_use]
    pub fn produced_an_accepted_control(&self) -> bool {
        self.observed_layer == Some(ObservedOutcomeLayer::Accepted)
            && self
                .reverification
                .as_ref()
                .is_some_and(|check| check.verified() && check.readback_matches_submission())
    }
}

/// What the ceremony is doing next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    /// Issue the disposable asset the deployment is linked against.
    Issue,
    /// Fund the confidential predecessor at the receipt constructors.
    Fund,
    /// Submit the one shape.
    Submit,
    /// Nothing further.
    Done,
}

/// The successor this shape registers, held once the asset exists.
#[derive(Clone, Debug)]
struct Successor {
    digest: [u8; 32],
    view: transaction::live_materialize::ConfidentialFixtureView,
}

/// The Wave-7 multi-output / multi-input shape ceremony.
pub struct MultiShapePlanner {
    shape: PrivateShape,
    stage: Stage,
    genesis_block_hash: Digest32,
    linked: Option<LinkedDeployment>,
    successor: Option<Successor>,
    submitted: Option<Vec<u8>>,
    census: Option<OwnerSigningCensus>,
    spent_owner_bytes: Option<Vec<u8>>,
    record: MultiShapeRecord,
}

impl MultiShapePlanner {
    /// The ceremony for one shape, bound to a deployment's printed genesis
    /// identity.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] where the reviewed target
    /// does not build.
    pub fn for_shape(
        shape: PrivateShape,
        printed_genesis_identity: Digest32,
    ) -> Result<Self, VectorError> {
        reviewed_target()?;
        let record = MultiShapeRecord {
            shape: Some(shape.name()),
            input_count: shape.consumed().len(),
            output_count: shape.destinations().len(),
            ..MultiShapeRecord::default()
        };
        Ok(Self {
            shape,
            stage: Stage::Issue,
            genesis_block_hash: printed_order(printed_genesis_identity),
            linked: None,
            successor: None,
            submitted: None,
            census: None,
            spent_owner_bytes: None,
            record,
        })
    }

    /// The ceremony for one shape against an asset ALREADY issued.
    ///
    /// # Why a second entry point rather than a flag
    ///
    /// Because it is a different ceremony. [`Self::for_shape`] runs a
    /// whole chain's worth of steps starting with an issuance, and a run
    /// that has to share an asset with something else cannot issue one:
    /// two issuances are two assets, and §6.6 requires a pair's two
    /// members to carry the SAME exact explicit `U`. The pairs arc is the
    /// caller, and its explicit member issues.
    ///
    /// The stage machine is otherwise untouched — the issuance stage is
    /// SETTLED here rather than skipped, so everything downstream of it
    /// sees exactly the state an issuing run leaves.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] where the reviewed
    /// target does not build or the deployment does not link against the
    /// named asset.
    pub fn for_shape_against_issued_asset(
        shape: PrivateShape,
        printed_genesis_identity: Digest32,
        printed_asset: &str,
    ) -> Result<Self, VectorError> {
        let mut planner = Self::for_shape(shape, printed_genesis_identity)?;
        planner
            .settle_asset(printed_asset)
            .map_err(|_| VectorError::LiveSubstrateUnavailable)?;
        planner.stage = Stage::Fund;
        Ok(planner)
    }

    /// The transcript.
    #[must_use]
    pub const fn record(&self) -> &MultiShapeRecord {
        &self.record
    }

    /// The case identity of the confidential funding step.
    #[must_use]
    pub fn funding_case() -> OperationCaseId {
        OperationCaseId {
            operation: OperationStepKind::FundConfidential,
            step: crate::confidential_predecessor::FUND_STEP.to_owned(),
        }
    }

    /// Record a refusal and stop.
    fn refuse(&mut self, refusal: PrivateRestartRefusal) -> PlanRefused {
        if self.record.refusal.is_none() {
            self.record.refusal = Some(refusal);
        }
        self.stage = Stage::Done;
        PlanRefused
    }

    /// Link the deployment against the issued asset and register this
    /// shape's successor.
    fn settle_asset(&mut self, printed: &str) -> Result<(), PrivateRestartRefusal> {
        // The shared spine links the predecessor and both owners' private
        // receipt programs. Its own one-to-one successor is registered and
        // unused; this ceremony registers its own multi-output successor
        // against the same linked deployment.
        let linked = link_and_register_composing(
            self.shape.predecessor(),
            ConsumedReceipt::Primary,
            printed,
            self.shape.vocabulary(),
            self.shape.composition(),
            RESERVE_ASSET,
        )?;
        self.record.issued_asset = Some(printed.to_owned());
        self.record.predecessor_digest = Some(linked.predecessor_digest);

        let successor = self.register_successor(&linked)?;
        self.record.successor_digest = Some(successor.digest);
        self.record.forced_blinder = self.census_the_forced_blinder(&linked, &successor)?;
        self.successor = Some(successor);
        self.linked = Some(linked);
        Ok(())
    }

    /// What the sole output's FORCED blinder is, as computed facts.
    ///
    /// # Why this is computed and not argued
    ///
    /// A sole-balancing output's blinder is not chosen; it is forced to
    /// the sum of the consumed coins' blinders. When that sum is zero the
    /// output commitment is exactly the value times the value generator
    /// -- a point anybody recomputes from a guessed amount, carrying a
    /// blinded output's form and none of its hiding -- so whether the sum
    /// is zero is the difference between a merge worth building and one
    /// that hides nothing.
    ///
    /// The argument that it is nonzero here is a good one: three blinders
    /// summing to zero leave any two summing to the negation of the
    /// third, and no blinder is ever zero. But an argument is not an
    /// observation, and the whole discipline of this lane is that the
    /// difference is stated rather than blurred. So the ceremony ASKS,
    /// against the fixtures it actually registered, and writes the answer
    /// into its own transcript where a reader can see it.
    ///
    /// `None` for a shape whose outputs are not a single solved one:
    /// there is no forced blinder to census, which is a different fact
    /// from a forced blinder that came out zero.
    ///
    /// # Errors
    ///
    /// [`PrivateRestartRefusal::PredecessorBlindersDoNotClose`] where a
    /// consumed coin carries no opening or a blinder is not a readable
    /// scalar.
    fn census_the_forced_blinder(
        &self,
        linked: &LinkedDeployment,
        successor: &Successor,
    ) -> Result<Option<ForcedBlinderCensus>, PrivateRestartRefusal> {
        let [sole] = successor.view.outputs() else {
            return Ok(None);
        };
        let forced = sole
            .value_blinder()
            .ok_or(PrivateRestartRefusal::PredecessorBlindersDoNotClose)?;

        let mut consumed = Vec::with_capacity(self.shape.consumed().len());
        for receipt in self.shape.consumed() {
            let blinder = linked
                .predecessor_view
                .outputs()
                .get(receipt.index())
                .ok_or(PrivateRestartRefusal::PredecessorBlindersDoNotClose)?
                .value_blinder()
                .ok_or(PrivateRestartRefusal::PredecessorBlindersDoNotClose)?;
            consumed.push(*blinder);
        }
        let sum = target_elements_conformance::confidential_fixture::sum_blinders(&consumed)
            .ok_or(PrivateRestartRefusal::PredecessorBlindersDoNotClose)?;

        Ok(Some(ForcedBlinderCensus {
            consumed_coins: consumed.len(),
            consumed_sum_is_zero: sum == [0_u8; 32],
            forced_blinder_is_zero: *forced == [0_u8; 32],
            forced_blinder_is_the_consumed_sum: *forced == sum,
        }))
    }

    /// Register this shape's successor: one output per destination, the
    /// last balancing, its input blinder sum the sum of the consumed
    /// coins' own blinders.
    fn register_successor(
        &self,
        linked: &LinkedDeployment,
    ) -> Result<Successor, PrivateRestartRefusal> {
        let destinations = self.shape.destinations();
        let outputs: Vec<ConfidentialFixtureOutput> = destinations
            .iter()
            .map(|destination| {
                Ok(ConfidentialFixtureOutput {
                    role: destination.role,
                    semantic_amount: destination.amount,
                    // A fee output's program is empty, and the registry
                    // REQUIRES it empty of that role rather than merely
                    // tolerating it. Deriving a receipt constructor here
                    // and handing it over would be refused there, which is
                    // the clause working.
                    // Under the CREATED side's plan. A destination is a
                    // coin this transfer mints, and the constructor it
                    // must be paid to is the one that will RECOGNIZE it
                    // when somebody spends it next -- which for a
                    // crossing is not the side this transfer's own
                    // receipts were read under. Resolving these under the
                    // consumed plan would register a fixture whose
                    // outputs the construction does not pay to.
                    output_program: if destination.role == FixtureOutputRole::Fee {
                        Vec::new()
                    } else {
                        owner_program(
                            &linked.abi,
                            &destination.scalar,
                            self.shape.composition().created(),
                        )?
                    },
                })
            })
            .collect::<Result<_, PrivateRestartRefusal>>()?;

        let handle = self.shape.successor_handle();
        let (digest, view) = register_multi(
            &handle,
            *linked.asset.internal(),
            self.input_blinder_sum(linked)?,
            outputs,
        )
        .map_err(|_| PrivateRestartRefusal::FixtureNotRegistrable {
            handle: handle.clone(),
        })?;
        Ok(Successor {
            digest: *digest.bytes(),
            view,
        })
    }

    /// The successor's input blinder sum: the sum of the consumed
    /// predecessor coins' value blinders.
    ///
    /// # Summed over the coins, not stated from the fixture
    ///
    /// This used to answer a two-input shape with a literal zero, on the
    /// ground that this ceremony's one predecessor is funded from an
    /// explicit input and its two output blinders therefore come out
    /// ordered additive inverses. That ground was sound and the answer was
    /// right, for that predecessor. It was a statement about one manifest's
    /// STRUCTURE standing in for a sum over the coins actually being spent,
    /// and it was true of no other predecessor — a ceremony consuming two
    /// coins of a wider one has a nonzero sum, and the stated zero would
    /// have built a candidate whose value balance does not close, which is
    /// a thing one learns from a chain.
    ///
    /// So the sum is computed, over exactly the coins the shape names as
    /// consumed, through the conformance crate's own scalar arithmetic.
    /// The inverse pair still sums to zero and the registry still refuses
    /// that by name; what changed is that the zero is now an OBSERVATION
    /// about two blinders rather than an assumption about one manifest.
    ///
    /// # Errors
    ///
    /// [`PrivateRestartRefusal::PredecessorBlindersDoNotClose`] where a
    /// consumed index names no predecessor output, where that output
    /// carries no opening, or where a blinder is not a readable scalar.
    fn input_blinder_sum(
        &self,
        linked: &LinkedDeployment,
    ) -> Result<[u8; 32], PrivateRestartRefusal> {
        // An explicitly funded predecessor contributes NOTHING to the
        // sum, and states that rather than deriving it: an explicit
        // value is committed with the all-zero blinder, so the sum over
        // any number of them is zero. It is returned here rather than
        // summed from openings the coins do not have.
        if self.shape.funds_explicitly() {
            return Ok([0_u8; 32]);
        }
        let mut blinders: Vec<[u8; 32]> = Vec::with_capacity(self.shape.consumed().len());
        for consumed in self.shape.consumed() {
            let output = linked
                .predecessor_view
                .outputs()
                .get(consumed.index())
                .ok_or(PrivateRestartRefusal::PredecessorBlindersDoNotClose)?;
            let blinder = output
                .value_blinder()
                .ok_or(PrivateRestartRefusal::PredecessorBlindersDoNotClose)?;
            blinders.push(*blinder);
        }
        target_elements_conformance::confidential_fixture::sum_blinders(&blinders)
            .ok_or(PrivateRestartRefusal::PredecessorBlindersDoNotClose)
    }

    /// The confidential funding step, against the registered predecessor.
    fn funding_step(&self) -> Result<OperationStep, PrivateRestartRefusal> {
        let linked = self
            .linked
            .as_ref()
            .ok_or(PrivateRestartRefusal::IssuanceNamedNoAsset)?;
        let printed = self
            .record
            .issued_asset
            .clone()
            .ok_or(PrivateRestartRefusal::IssuanceNamedNoAsset)?;
        // Two funding steps, chosen by the shape rather than by a
        // parameter. An entry crossing's consumed coins belong to no
        // confidential fixture, so there is no predecessor manifest to
        // bind the funding to and nothing for the adapter to derive.
        if let Some((outputs, amount)) = self.shape.explicit_funding() {
            let program = owner_program(
                &linked.abi,
                &FIRST_SCALAR,
                self.shape.composition().consumed(),
            )?;
            return Ok(explicit_funding_step(program, printed, outputs, amount));
        }
        Ok(confidential_funding_step(linked, printed))
    }

    /// Take the funded coins from the node's own report of them.
    fn settle_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), PrivateRestartRefusal> {
        let linked = self
            .linked
            .as_ref()
            .ok_or(PrivateRestartRefusal::MalformedConfidentialOutput)?;
        if let Some((outputs, amount)) = self.shape.explicit_funding() {
            let program = owner_program(
                &linked.abi,
                &FIRST_SCALAR,
                self.shape.composition().consumed(),
            )?;
            self.record.coins =
                observe_explicit_coins(response, linked.asset, &program, outputs as usize, amount)?;
            return Ok(());
        }
        self.record.coins = observe_funded_coins(linked, response)?;
        Ok(())
    }

    /// The successor's wire bytes, every owner's authorization in its own
    /// input's witness.
    fn control_bytes(&mut self) -> Result<Vec<u8>, PrivateRestartRefusal> {
        let finalization = self.finalize_shape()?;
        let built = assemble_control(&finalization, self.genesis_block_hash, None)?;
        self.spent_owner_bytes = built.spent_owner_bytes;
        self.record.receipt_leaves = built.receipt_leaves;
        self.record.output_witness_proof_bytes = built.output_witness_proof_bytes;
        let bytes = built.transaction.encode();
        self.record.submitted_bytes = bytes.len();
        self.census = Some(built.census);
        Ok(bytes)
    }

    /// One opening per destination, each carrying the role the SHAPE
    /// stated.
    ///
    /// Never the role a position implies. This is the second face of the
    /// same retirement: the fixture registry stopped inferring the
    /// balance from an output's index, and so does the layer that hands
    /// the materializer its openings.
    fn destination_openings(
        &self,
        destinations: &[Destination],
        digest: [u8; 32],
    ) -> Vec<PrivateDestinationOpening> {
        destinations
            .iter()
            .enumerate()
            .map(|(index, destination)| PrivateDestinationOpening {
                fixture: FixtureOpeningReference::new(self.shape.successor_handle(), digest, index),
                role: match destination.role {
                    FixtureOutputRole::Primary => ConfidentialOutputRole::Primary,
                    // The fee role is stated rather than swept into the
                    // catch-all. It used to fall through to `Balancing`,
                    // which would have asked the materializer to solve a
                    // blinder for an output that must not carry one — a
                    // blinded fee, and not a fee.
                    FixtureOutputRole::Fee => ConfidentialOutputRole::Fee,
                    // The sponsor change is stated for exactly the reason
                    // the fee is, and the catch-all below is why it has
                    // to be: falling through to `Balancing` would ask the
                    // materializer to SOLVE a blinder for the sponsor's
                    // remainder, and a solved blinder absorbs the input
                    // sum. The remainder would stop being the sponsor's
                    // and the protocol receipts would stop balancing —
                    // one substitution producing two wrong outputs.
                    FixtureOutputRole::SponsorChange { .. } => {
                        ConfidentialOutputRole::SponsorChange
                    }
                    // Stated for the third time for the same reason:
                    // swept into the catch-all it would ask for a SOLVED
                    // blinder on an output that carries none, declaring
                    // two solving outputs where the registry admits one.
                    FixtureOutputRole::ExplicitDestination => {
                        ConfidentialOutputRole::ExplicitDestination
                    }
                    // Both solving roles are the view's one solving role:
                    // the sole form is a solve over no others, which is
                    // the same instruction to the materializer.
                    _ => ConfidentialOutputRole::Balancing,
                },
            })
            .collect()
    }

    /// Finalize this shape's successor through the private lane's own entry
    /// point, against the coins the node reported.
    fn finalize_shape(&self) -> Result<PrivateLiveFinalization, PrivateRestartRefusal> {
        let linked = self
            .linked
            .as_ref()
            .ok_or(PrivateRestartRefusal::FundingCreatedNoPredecessor)?;
        let successor = self
            .successor
            .as_ref()
            .ok_or(PrivateRestartRefusal::FundingCreatedNoPredecessor)?;

        let consumed = self.shape.consumed();
        let mut spent_views = Vec::with_capacity(consumed.len());
        let mut input_openings = Vec::with_capacity(consumed.len());
        for receipt in consumed {
            let coin = self
                .record
                .coins
                .get(receipt.index())
                .ok_or(PrivateRestartRefusal::FundingCreatedNoPredecessor)?;
            spent_views.push(PublicOutputView::new(
                coin.outpoint(),
                coin.asset(),
                coin.value(),
                coin.program().to_vec(),
            ));
            input_openings.push(PrivateInputOpening {
                region: ConfidentialInputRegion::Receipt,
                // ABSENT for an explicit receipt, which has no opening
                // to name: its amount is public and its blinder is the
                // all-zero one. A reference here would name a fixture
                // output that does not exist.
                opening: (!self.shape.funds_explicitly()).then(|| {
                    FixtureOpeningReference::new(
                        linked.predecessor.handle().as_str().to_owned(),
                        linked.predecessor_digest,
                        receipt.index(),
                    )
                }),
                // The amount the coin really carries. For an explicit
                // receipt that is what was funded, not what a
                // predecessor manifest states -- there is no manifest.
                explicit_amount: match self.shape.explicit_funding() {
                    Some((_, amount)) => amount,
                    None => linked.predecessor.amounts()[receipt.index()],
                },
                zero_asset_blinder: [0_u8; SCALAR_BYTES],
            });
        }
        let view = PublicConstructionView::new(spent_views)
            .map_err(|_| PrivateRestartRefusal::ControlNotRequestable)?;

        let destinations = self.shape.destinations();
        let receipts: Vec<Outpoint> = consumed
            .iter()
            .map(|receipt| self.record.coins[receipt.index()].outpoint())
            .collect();
        let live_destinations: Vec<LiveReceiptDestination> = destinations
            .iter()
            .map(Self::receipt_destination)
            .collect::<Result<_, _>>()?;
        let request = LiveTransferRequest::new_composing(
            receipts,
            live_destinations,
            self.shape.composition(),
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            // Present because the CREATED side is confidential and
            // blinding is what consumes randomness; the materializer
            // takes its blinders from fixtures, so it is not a source of
            // any opening. An exit crossing creates explicit values and
            // offers none, which is the same rule read the other way.
            //
            // Withheld where the created side is explicit, because the
            // request refuses randomness it has no blinding to spend it
            // on -- and that refusal is the reason the rule had to move
            // from the consumed side to the created one.
            (self.shape.composition().created()
                == LiveTransferRepresentationPlan::PrivateCommitted)
                .then(|| PublicTestRandomness::from_published_bytes([0x7e; 32])),
        )
        .map_err(|_| PrivateRestartRefusal::ControlNotRequestable)?;

        let destination_openings = self.destination_openings(&destinations, successor.digest);

        let openings = PrivateLiveOpenings::new(
            input_openings,
            destination_openings,
            NonProtocolFundingRegion::default(),
            materialization_profiles(),
        );

        let fixtures = FrozenConfidentialFixtureView::new(BTreeMap::from([
            (
                linked.predecessor.handle().as_str().to_owned(),
                linked.predecessor_view.clone(),
            ),
            (self.shape.successor_handle(), successor.view.clone()),
        ]));

        finalize_private_live_transfer_composing(
            &reviewed_target().map_err(|_| PrivateRestartRefusal::SubstrateUnavailable)?,
            &linked.abi,
            &request,
            self.shape.composition(),
            &view,
            None,
            &openings,
            &fixtures,
            &ReferenceConfidentialMaterializer::new(),
            &FirstPartyCommitmentCheck::new(),
        )
        .map_err(|refusal| PrivateRestartRefusal::FinalizationRefused(format!("{refusal:?}")))
    }

    /// One receipt destination, an owner and a protocol value.
    fn receipt_destination(
        destination: &Destination,
    ) -> Result<LiveReceiptDestination, PrivateRestartRefusal> {
        let owner = published_owner(&destination.scalar)
            .map_err(|_| PrivateRestartRefusal::SubstrateUnavailable)?;
        let value = ProtocolValue::new(destination.amount)
            .map_err(|_| PrivateRestartRefusal::ControlNotRequestable)?;
        Ok(LiveReceiptDestination::new(
            OwnerParameter::new(owner),
            value,
        ))
    }

    /// Record what the target did with the successor.
    fn settle_control(&mut self, response: &NativeOperationResponse) {
        self.record.observed_layer = Some(response.observed_layer);
        self.record
            .observed_detail
            .clone_from(&response.observed_detail);
        self.record
            .accepted_txid
            .clone_from(&response.accepted_txid);

        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return;
        }
        let (Some(readback), Some(submitted), Some(census)) = (
            response.mined_readback.as_ref(),
            self.submitted.as_ref(),
            self.census.as_ref(),
        ) else {
            return;
        };

        let readback_matches_submission = readback.raw_transaction == *submitted;
        let verified = census
            .signing_inputs()
            .first()
            .map(|input| candidate_owner_message(census, input, WitnessVectorTreatment::BothGrown))
            .zip(self.spent_owner_bytes.as_ref())
            .is_some_and(|(message, owner)| {
                verify_readback_signature(&readback.raw_transaction, &message, owner)
            });

        self.record.observed_weight = response.resources.transaction_weight;
        self.record.reverification = Some(MultiShapeReverification {
            accepted_txid: readback.transaction_id.clone(),
            readback_matches_submission,
            verified,
        });
    }
}

impl TargetOperationPlanner for MultiShapePlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    let Some(printed) = response.issued_asset.clone() else {
                        return Err(self.refuse(PrivateRestartRefusal::IssuanceNamedNoAsset));
                    };
                    if let Err(refusal) = self.settle_asset(&printed) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Fund;
                }
                Stage::Fund => {
                    if let Err(refusal) = self.settle_funding(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Submit;
                }
                Stage::Submit => {
                    self.settle_control(response);
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        match self.stage {
            Stage::Issue => Ok(Some(issue_step())),
            Stage::Fund => match self.funding_step() {
                Ok(step) => Ok(Some(step)),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Submit => match self.control_bytes() {
                Ok(bytes) => {
                    self.submitted = Some(bytes.clone());
                    Ok(Some(OperationStep::new(
                        self.shape.name(),
                        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                            transaction_bytes: bytes,
                        })),
                    )))
                }
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Done => Ok(None),
        }
    }
}

/// The commitment prefix a reported coin carried, where the node reported
/// a commitment.
#[must_use]
const fn commitment_prefix(coin: &Coin) -> Option<u8> {
    match coin.value() {
        ValueField::Commitment(commitment) => commitment.first().copied(),
        _ => None,
    }
}

/// The transcript, one fact per line.
#[must_use]
pub fn render_multi_shape(record: &MultiShapeRecord) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    let _ = writeln!(out, "run {}", record.shape().unwrap_or("absent"));
    let _ = writeln!(out, "input_count {}", record.input_count());
    let _ = writeln!(out, "output_count {}", record.output_count());
    let _ = writeln!(
        out,
        "issued_asset {}",
        record.issued_asset().unwrap_or("absent"),
    );
    let _ = writeln!(
        out,
        "successor_digest {}",
        record
            .successor_digest()
            .map_or_else(|| "absent".to_owned(), hex),
    );
    for (index, coin) in record.coins().iter().enumerate() {
        let _ = writeln!(
            out,
            "coin {index} rangeproof_bytes {} commitment_prefix {} matches_expectation {}",
            coin.rangeproof_bytes(),
            commitment_prefix(coin)
                .map_or_else(|| "none".to_owned(), |prefix| format!("{prefix:#04x}")),
            coin.matches_expectation(),
        );
    }
    if let Some(census) = record.forced_blinder() {
        // The forced blinder, as facts and never as bytes. A transcript
        // carrying the scalar would be publishing an opening; what a
        // reader needs is whether it is zero, and whether the ceremony
        // checked that independently of the registry's own solve.
        let _ = writeln!(
            out,
            "forced_blinder consumed_coins {} consumed_sum_is_zero {} \
             forced_blinder_is_zero {} forced_blinder_is_the_consumed_sum {}",
            census.consumed_coins(),
            census.consumed_sum_is_zero(),
            census.forced_blinder_is_zero(),
            census.forced_blinder_is_the_consumed_sum(),
        );
    }
    let _ = writeln!(out, "receipt_leaves {}", record.receipt_leaves());
    let _ = writeln!(
        out,
        "output_witness_proof_bytes {:?}",
        record.output_witness_proof_bytes(),
    );
    let _ = writeln!(out, "submitted_bytes {}", record.submitted_bytes());
    // The TARGET's figure, not one computed here. A weight this
    // workspace calculated would be comparing its arithmetic with
    // itself, which is not a comparison.
    match record.observed_weight() {
        Some(weight) => {
            let _ = writeln!(out, "target_reported_weight {weight}");
        }
        None => {
            let _ = writeln!(out, "target_reported_weight none");
        }
    }
    let _ = writeln!(
        out,
        "observed_layer {}",
        record
            .observed_layer()
            .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
    );
    let _ = writeln!(
        out,
        "observed_detail {}",
        record.observed_detail().unwrap_or("none"),
    );
    let _ = writeln!(
        out,
        "accepted_txid {}",
        record.accepted_txid().unwrap_or("none"),
    );
    if let Some(check) = record.reverification() {
        let _ = writeln!(
            out,
            "reverification readback_matches_submission {} verified {}",
            check.readback_matches_submission(),
            check.verified(),
        );
    }
    let _ = writeln!(
        out,
        "construction_refusal {}",
        record
            .refusal()
            .map_or_else(|| "none".to_owned(), |refusal| format!("{refusal:?}")),
    );
    let _ = writeln!(
        out,
        "produced_an_accepted_control {}",
        record.produced_an_accepted_control(),
    );

    // What this run does NOT establish, in its own bytes.
    out.push_str("evidences_no_negative_case true\n");
    out.push_str("evidences_no_minimality_relation true\n");
    out.push_str("moves_the_sponsor_row false\n");
    out
}

/// One digest, printed.
fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

/// Forward expectations under the sole live fixture-digest v2 algorithm.
///
/// These are fixture identities, not run observations. Each awaits a node-accepted v2 run before
/// it can pair with an accepted identity as historical evidence.
#[cfg(test)]
mod forward_fixture_digest_v2 {
    /// The private-split successor fixture's forward digest under fixture-digest v2.
    ///
    /// This is the same fixture whose recorded v1 digest and accepted identity are immutable halves
    /// of one historical observation in [`super::run_of_record`]. Algorithm v2 binds amounts
    /// unconditionally; forward runs bind here. No node acceptance under v2 is claimed.
    pub const SPLIT_SUCCESSOR_DIGEST: &str =
        "52f9d832e9ff93070ea9849cd8d5f817746fdb8bc544e6d94207810e594014ca";

    /// The many-to-many successor fixture's forward digest under fixture-digest v2.
    ///
    /// This is the same fixture whose recorded v1 digest and accepted identity are immutable halves
    /// of one historical observation in [`super::run_of_record`]. Algorithm v2 binds amounts
    /// unconditionally; forward runs bind here. No node acceptance under v2 is claimed.
    pub const MANY_TO_MANY_SUCCESSOR_DIGEST: &str =
        "67ec9e516460e453fcc0bf6cfacfdc58141afefa0b4e4c2d3ff3c2d7a2495b98";

    /// The several-distinct-owners successor fixture's forward digest under fixture-digest v2.
    ///
    /// This is the same fixture whose recorded v1 digest and accepted identity are immutable halves
    /// of one historical observation in [`super::run_of_record`]. Algorithm v2 binds amounts
    /// unconditionally; forward runs bind here. No node acceptance under v2 is claimed.
    pub const SEVERAL_OWNERS_SUCCESSOR_DIGEST: &str =
        "6d86b884565d6d4bc1f81fb6915dc91987d97a62e5a3ca896403ef6d4963e30c";

    /// The strict one-to-one successor fixture's forward digest under fixture-digest v2.
    ///
    /// This is the same fixture whose recorded v1 digest and accepted identity are immutable halves
    /// of one historical observation in [`super::run_of_record`]. Algorithm v2 binds amounts
    /// unconditionally; forward runs bind here. No node acceptance under v2 is claimed.
    pub const STRICT_ONE_TO_ONE_SUCCESSOR_DIGEST: &str =
        "7ec97a6e6cf6bf7e6c4308c007e55cf6ac799113abb3b93e3fb3d559c510ec14";

    /// The fee-bearing successor fixture's forward digest under fixture-digest v2.
    ///
    /// This is the same fixture whose recorded v1 digest and accepted identity are immutable halves
    /// of one historical observation in [`super::run_of_record`]. Algorithm v2 binds amounts
    /// unconditionally; forward runs bind here. No node acceptance under v2 is claimed.
    pub const FEE_BEARING_SUCCESSOR_DIGEST: &str =
        "bba4ea6e919b8619d75d035b5f6ee9e8aac3a87c00336c8a86c5973a84936bad";

    /// The private-merge successor fixture's forward digest under fixture-digest v2.
    ///
    /// This is the same fixture whose recorded v1 digest and accepted identity are immutable halves
    /// of one historical observation in [`super::run_of_record`]. Algorithm v2 binds amounts
    /// unconditionally; forward runs bind here. No node acceptance under v2 is claimed.
    pub const MERGE_SUCCESSOR_DIGEST: &str =
        "d4a4a2e4371a81d4d74ad2fbceef64303091496c374e11e2208c6d3bde1ea0d9";

    /// The exit-crossing successor fixture's forward digest under fixture-digest v2.
    ///
    /// This is the same fixture whose recorded v1 digest and accepted identity are immutable halves
    /// of one historical observation in [`super::run_of_record`]. Algorithm v2 binds amounts
    /// unconditionally; forward runs bind here. No node acceptance under v2 is claimed.
    pub const EXIT_CROSSING_SUCCESSOR_DIGEST: &str =
        "013df551e2862b4018f2e9b2c2cc83174571234f1ad200605de66232514ea001";

    /// The entry-crossing successor fixture's forward digest under fixture-digest v2.
    ///
    /// This is the same fixture whose recorded v1 digest and accepted identity are immutable halves
    /// of one historical observation in [`super::run_of_record`]. Algorithm v2 binds amounts
    /// unconditionally; forward runs bind here. No node acceptance under v2 is claimed.
    pub const ENTRY_CROSSING_SUCCESSOR_DIGEST: &str =
        "7c3bda6049cb80635b0580148002c185eeee8ea48e67bfd1fd698e28a65b17c0";

    /// The pure-split successor fixture's forward digest under fixture-digest v2.
    ///
    /// This is the same fixture whose recorded v1 digest and accepted identity are immutable halves
    /// of one historical observation in [`super::run_of_record`]. Algorithm v2 binds amounts
    /// unconditionally; forward runs bind here. No node acceptance under v2 is claimed.
    pub const PURE_SPLIT_SUCCESSOR_DIGEST: &str =
        "2c8318413c8a726c6c65587d4c48d60af1ba82f1622152d216266c5858658671";

    /// The paired one-to-one successor fixture's forward digest under fixture-digest v2.
    ///
    /// This fixture's v1 run recorded an accepted identity but no fixture digest, so there is no
    /// historical v1 sibling to rewrite or revalidate. Algorithm v2 binds amounts unconditionally;
    /// forward runs bind here. No node acceptance under v2 is claimed.
    pub const PAIRED_ONE_TO_ONE_SUCCESSOR_DIGEST: &str =
        "e8a824533192cd400e97343d3e5ca69d71fe1138d740723b24b87cc7bf2fd796";
}

#[cfg(test)]
mod byte_identity_tests {
    use super::{
        MultiShapePlanner, PrivateShape, forward_fixture_digest_v2 as forward_v2, hex,
        run_of_record as run,
    };

    /// Every pre-wave shape registers its successor under the forward digest for the live v2
    /// algorithm.
    ///
    /// # What this is a check on
    ///
    /// This wave added two members to the shape vocabulary, a second
    /// predecessor to the ceremony, a fee member to the materializer's
    /// output roles, an optional opening to the projected view, and an
    /// arity-general prefix rule on both sides of the executor wire. Any
    /// one of those could have perturbed the derivation of a case that
    /// already existed -- a transcript member emitted unconditionally, a
    /// role code reassigned, a parity counter that settles one step
    /// later -- and a perturbed derivation is a different fixture wearing
    /// the same handle.
    ///
    /// Recomputing the live v2 identities re-derives every blinder, every nonce input, every
    /// range-proof seed, and every commitment prefix. A derivation drift still lands here before it
    /// lands on a chain; the algorithm move itself is represented by separate forward pins rather
    /// than by rewriting or live-revalidating the recorded v1 digests.
    ///
    /// Four accepted identities remain paired with their immutable recorded v1 digests. The paired
    /// arc recorded an accepted identity but no v1 fixture digest, so its forward pin has no
    /// historical digest sibling. The predecessor half is checked by the sibling test in the
    /// private-restart module.
    #[test]
    fn the_pre_wave_shapes_register_under_their_forward_v2_digests() {
        // The genesis identity is not a term of any fixture digest, and
        // this test would fail loudly if it became one.
        let genesis: transaction::taproot::Digest32 = [0x11_u8; 32];
        for (shape, expected_v2) in [
            (PrivateShape::Split, forward_v2::SPLIT_SUCCESSOR_DIGEST),
            (
                PrivateShape::ManyToMany,
                forward_v2::MANY_TO_MANY_SUCCESSOR_DIGEST,
            ),
            (
                PrivateShape::SeveralDistinctOwners,
                forward_v2::SEVERAL_OWNERS_SUCCESSOR_DIGEST,
            ),
            (
                PrivateShape::StrictOneToOne,
                forward_v2::STRICT_ONE_TO_ONE_SUCCESSOR_DIGEST,
            ),
            (
                PrivateShape::PairedOneToOne,
                forward_v2::PAIRED_ONE_TO_ONE_SUCCESSOR_DIGEST,
            ),
        ] {
            let mut planner =
                MultiShapePlanner::for_shape(shape, genesis).expect("the ceremony builds");
            planner
                .settle_asset(run::ISSUED_ASSET)
                .expect("the run of record's own fixtures register");
            assert_eq!(
                planner
                    .record()
                    .successor_digest()
                    .map(hex)
                    .expect("the successor registered"),
                expected_v2,
                "{}'s successor fixture drifted from its forward v2 pin",
                shape.name(),
            );
        }

        // Owner ruling Q19: v2 binds amounts unconditionally, so every fixture with a recorded v1
        // digest must differ from that historical sibling. Per Q21 these compare static pins; they
        // do not revalidate v1 with a legacy algorithm.
        for (shape, digest_v2, recorded_v1) in [
            (
                "private-split",
                forward_v2::SPLIT_SUCCESSOR_DIGEST,
                run::SPLIT_SUCCESSOR_DIGEST,
            ),
            (
                "many-to-many",
                forward_v2::MANY_TO_MANY_SUCCESSOR_DIGEST,
                run::MANY_TO_MANY_SUCCESSOR_DIGEST,
            ),
            (
                "several-distinct-owners",
                forward_v2::SEVERAL_OWNERS_SUCCESSOR_DIGEST,
                run::SEVERAL_OWNERS_SUCCESSOR_DIGEST,
            ),
            (
                "strict-one-to-one",
                forward_v2::STRICT_ONE_TO_ONE_SUCCESSOR_DIGEST,
                run::STRICT_ONE_TO_ONE_SUCCESSOR_DIGEST,
            ),
        ] {
            assert_ne!(
                digest_v2, recorded_v1,
                "owner ruling Q19 requires {shape}'s amount-bearing fixture digest to move in v2",
            );
        }
    }

    /// The five amount-bearing shapes with forward pins carry their own live v2 digests, and no
    /// shape collides with another.
    ///
    /// A new fixture that happened to derive an existing case's digest
    /// would mean the handle is not a term of the transcript, which is a
    /// defect and not a coincidence.
    ///
    /// Each accepted identity and recorded v1 digest remains the immutable pair from its historical
    /// run. The v2 pin names the same fixture under the live algorithm and awaits its own accepted
    /// run before it can become half of a new observation pair.
    #[test]
    fn the_shapes_this_wave_added_carry_digests_of_their_own() {
        let genesis: transaction::taproot::Digest32 = [0x11_u8; 32];
        let mut digests = Vec::new();
        for shape in PrivateShape::ALL {
            let mut planner =
                MultiShapePlanner::for_shape(shape, genesis).expect("the ceremony builds");
            planner
                .settle_asset(run::ISSUED_ASSET)
                .expect("every shape's fixtures register");
            digests.push(
                planner
                    .record()
                    .successor_digest()
                    .map(hex)
                    .expect("the successor registered"),
            );
        }
        assert_eq!(digests.len(), PrivateShape::ALL.len());

        let mut distinct = digests.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(
            distinct.len(),
            digests.len(),
            "two shapes registered the same successor fixture: {digests:?}",
        );

        assert_eq!(digests[4], forward_v2::FEE_BEARING_SUCCESSOR_DIGEST);
        assert_eq!(digests[5], forward_v2::MERGE_SUCCESSOR_DIGEST);
        assert_eq!(digests[6], forward_v2::EXIT_CROSSING_SUCCESSOR_DIGEST);
        assert_eq!(digests[7], forward_v2::ENTRY_CROSSING_SUCCESSOR_DIGEST);
        assert_eq!(digests[8], forward_v2::PURE_SPLIT_SUCCESSOR_DIGEST);

        // Owner ruling Q19: v2 binds amounts unconditionally, so every amount-bearing fixture's
        // digest must differ from its recorded v1 sibling. Per Q21 these compare static pins; they
        // do not revalidate v1 with a legacy algorithm.
        for (shape, digest_v2, recorded_v1) in [
            (
                "fee-bearing",
                forward_v2::FEE_BEARING_SUCCESSOR_DIGEST,
                run::FEE_BEARING_SUCCESSOR_DIGEST,
            ),
            (
                "merge",
                forward_v2::MERGE_SUCCESSOR_DIGEST,
                run::MERGE_SUCCESSOR_DIGEST,
            ),
            (
                "exit-crossing",
                forward_v2::EXIT_CROSSING_SUCCESSOR_DIGEST,
                run::EXIT_CROSSING_SUCCESSOR_DIGEST,
            ),
            (
                "entry-crossing",
                forward_v2::ENTRY_CROSSING_SUCCESSOR_DIGEST,
                run::ENTRY_CROSSING_SUCCESSOR_DIGEST,
            ),
            (
                "pure-split",
                forward_v2::PURE_SPLIT_SUCCESSOR_DIGEST,
                run::PURE_SPLIT_SUCCESSOR_DIGEST,
            ),
        ] {
            assert_ne!(
                digest_v2, recorded_v1,
                "owner ruling Q19 requires {shape}'s amount-bearing fixture digest to move in v2",
            );
        }
    }

    /// The fee-bearing recorded v1 digest and accepted identity remain one historical observation,
    /// while the live v2 fixture identity is checked separately.
    ///
    /// The historical pair does not move and is not live-revalidated. Fixture-digest v2 binds
    /// amounts unconditionally, so the same fixture now has a distinct forward digest. No node is
    /// claimed to have accepted a candidate under v2; that would require a new run and observation
    /// pair.
    #[test]
    fn the_fee_bearing_digest_and_its_acceptance_belong_to_one_run() {
        let genesis: transaction::taproot::Digest32 = [0x11_u8; 32];
        let mut planner = MultiShapePlanner::for_shape(PrivateShape::OneToOneWithFee, genesis)
            .expect("the ceremony builds");
        planner
            .settle_asset(run::ISSUED_ASSET)
            .expect("the fee-bearing successor registers");
        let digest_v2 = planner
            .record()
            .successor_digest()
            .map(hex)
            .expect("the ceremony recorded a successor digest");

        assert_eq!(
            digest_v2,
            forward_v2::FEE_BEARING_SUCCESSOR_DIGEST,
            "the fee-bearing fixture registers under its forward v2 digest",
        );

        // The acceptance is present, is the identity the register cites,
        // and is a target-computed one rather than a placeholder.
        assert_eq!(
            run::FEE_BEARING_ACCEPTED_IDENTITY,
            Some(run::FEE_BEARING_SUCCESSOR_IDENTITY),
            "the optional acceptance and the cited identity are the same run",
        );
        assert_eq!(
            run::FEE_BEARING_SUCCESSOR_IDENTITY.len(),
            64,
            "an accepted identity is a target-computed transaction identity",
        );
        assert_ne!(
            run::FEE_BEARING_SUCCESSOR_IDENTITY,
            run::FEE_BEARING_SUCCESSOR_DIGEST,
            "the identity a node computed is not the digest a registry computed",
        );
    }
}

#[cfg(test)]
mod tests {
    use target_elements_conformance::confidential_fixture::{
        ConfidentialFixtureOutput, FixtureOutputRole, RegistrationRefusal,
    };

    use crate::live_proof_bearing_observation::{register_multi, registry_refusal_for};

    /// One output, stated whole.
    ///
    /// The builders below name each output's role rather than relying on
    /// its position, which is the retirement these tests are the first
    /// readers of.
    fn output(role: FixtureOutputRole, amount: u64, program: Vec<u8>) -> ConfidentialFixtureOutput {
        ConfidentialFixtureOutput {
            role,
            semantic_amount: amount,
            output_program: program,
        }
    }

    /// A disposable asset for the offline registration checks. Public
    /// test material under ADR-015; it names no chain.
    const ASSET: [u8; 32] = [0x3b; 32];

    /// The zero input blinder sum a two-input transfer's consumed coins
    /// sum to, and a valid scalar the registry admits.
    ///
    /// It is zero for a real reason and not for convenience: this
    /// ceremony's predecessor is funded from an explicit input, so its
    /// own input blinder sum is zero and its two output blinders come out
    /// ordered additive inverses. A transfer consuming both of them
    /// therefore presents exactly this sum.
    const ZERO_SUM: [u8; 32] = [0_u8; 32];

    /// An input blinder sum that does not cancel.
    ///
    /// What a ONE-input transfer presents: a single consumed coin's own
    /// blinder, which has nothing to cancel against. Public disposable
    /// test material under ADR-015.
    const NON_CANCELING_SUM: [u8; 32] = {
        let mut bytes = [0_u8; 32];
        bytes[31] = 0x2a;
        bytes
    };

    /// The multi-output constructor the step-five stop said was absent
    /// now builds a three-output successor: the split shape's fixture,
    /// with two primary recipients and one balancing change output.
    #[test]
    fn the_multi_output_constructor_builds_a_three_output_successor() {
        let (_digest, view) = register_multi(
            "ctf-v1/test-three-output",
            ASSET,
            ZERO_SUM,
            vec![
                output(FixtureOutputRole::Primary, 400_000_000, vec![0x51]),
                output(FixtureOutputRole::Primary, 200_000_000, vec![0x52]),
                output(FixtureOutputRole::Balancing, 100_000_000, vec![0x53]),
            ],
        )
        .expect("the three-output successor registers");
        assert_eq!(
            view.outputs().len(),
            3,
            "the constructor built a fixture of the stated output arity",
        );
    }

    /// And a two-output successor, for the several-distinct-owners shape.
    #[test]
    fn the_multi_output_constructor_builds_a_two_output_successor() {
        let (_digest, view) = register_multi(
            "ctf-v1/test-two-output",
            ASSET,
            ZERO_SUM,
            vec![
                output(FixtureOutputRole::Primary, 600_000_000, vec![0x51]),
                output(FixtureOutputRole::Balancing, 400_000_000, vec![0x52]),
            ],
        )
        .expect("the two-output successor registers");
        assert_eq!(view.outputs().len(), 2);
    }

    /// The one-output floor, as it stands after the single-output form
    /// was admitted.
    ///
    /// # What this test used to say, and why it is worth reading twice
    ///
    /// It used to record that a one-output shape — private-merge, the
    /// strict one-to-one, and the fee-only case alike — hit a single
    /// cardinality wall: the registry counted outputs before inspecting
    /// any of them, so one output was refused `OutputSetTooSmall`
    /// whatever that output was. The shape census then typed that wall as
    /// a first-party convention rather than a consensus rule, and the
    /// convention has since been structurally removed.
    ///
    /// The wall did not disappear. It NARROWED, to exactly the case it
    /// was always about: a lone output that asks to be solved from others
    /// that are not there. A lone output that DECLARES the fully-solved
    /// form registers. Both halves are asserted here, at the site that
    /// recorded the wall, so a reader meets the removal where they would
    /// have met the refusal.
    #[test]
    fn the_one_output_floor_narrowed_to_the_undeclared_case() {
        let refusal = registry_refusal_for(
            "ctf-v1/test-one-output",
            ASSET,
            ZERO_SUM,
            vec![output(
                FixtureOutputRole::Balancing,
                700_000_000,
                vec![0x51],
            )],
        )
        .expect("an undeclared one-output manifest is still refused");
        assert_eq!(
            refusal,
            RegistrationRefusal::OutputSetTooSmall { found: 1 },
            "a lone output not declaring the form meets the floor it always met",
        );

        // And the removal itself. The input blinder sum is NONZERO here
        // because that is the whole condition the form carries: a sole
        // output's blinder is forced to this sum, and a sum that cancelled
        // would force a zero blinder that hides nothing.
        let (_digest, view) = register_multi(
            "ctf-v1/test-one-output-declared",
            ASSET,
            NON_CANCELING_SUM,
            vec![output(
                FixtureOutputRole::SoleBalancing,
                700_000_000,
                vec![0x51],
            )],
        )
        .expect("the declared single-output form registers");
        assert_eq!(view.outputs().len(), 1);
        assert_eq!(
            view.outputs()[0].value_blinder(),
            Some(&NON_CANCELING_SUM),
            "the lone output's blinder is the input blinder sum itself",
        );
    }

    /// A merge of this ceremony's inverse-pair predecessor is refused
    /// rather than built.
    ///
    /// The degeneracy the single-output form carries, at the one place a
    /// reader of this ceremony would look for it. The predecessor's two
    /// output blinders are ordered additive inverses, so a two-input merge
    /// consuming both of them presents a ZERO input blinder sum; the lone
    /// output's blinder is forced to that sum, and a zero blinder hides
    /// nothing at all. The registry refuses it, so this ceremony cannot
    /// build a merge from its own predecessor even now that the
    /// cardinality floor has moved.
    #[test]
    fn a_merge_of_the_inverse_pair_is_refused_as_a_zero_blinder() {
        let refusal = registry_refusal_for(
            "ctf-v1/test-merge-canceling",
            ASSET,
            ZERO_SUM,
            vec![output(
                FixtureOutputRole::SoleBalancing,
                1_000_000_000,
                vec![0x51],
            )],
        )
        .expect("a canceling merge is refused");
        assert!(
            matches!(refusal, RegistrationRefusal::Derivation { .. }),
            "the wall is now the zero blinder and no longer the output count",
        );
    }

    /// The deterministic-public-fixture-openings observation the §15.2 row
    /// asks for, per the byte-identity contract (§6.7): recomputing a
    /// fixture from its manifest yields the same digest and the same
    /// per-output openings byte for byte. This is a first-party
    /// determinism fact over fixture openings, not a target submission, so
    /// it carries no target-computed identity and does not move the matrix
    /// row through the acceptance-only delta.
    #[test]
    fn a_fixture_recomputes_byte_identically_from_its_manifest() {
        let register = || {
            register_multi(
                "ctf-v1/test-determinism",
                ASSET,
                ZERO_SUM,
                vec![
                    output(FixtureOutputRole::Primary, 400_000_000, vec![0x51]),
                    output(FixtureOutputRole::Primary, 200_000_000, vec![0x52]),
                    output(FixtureOutputRole::Balancing, 100_000_000, vec![0x53]),
                ],
            )
            .expect("the successor registers")
        };
        let (first_digest, first_view) = register();
        let (second_digest, second_view) = register();
        assert_eq!(
            first_digest.bytes(),
            second_digest.bytes(),
            "the fixture digest recomputes identically from the manifest",
        );
        assert_eq!(first_view.outputs().len(), second_view.outputs().len());
        for (first, second) in first_view.outputs().iter().zip(second_view.outputs()) {
            assert_eq!(
                first.value_blinder(),
                second.value_blinder(),
                "an output's value blinder recomputes identically",
            );
        }
    }

    /// A fee output is expressible now, and the empty-program clause
    /// still refuses an output that is not a fee.
    ///
    /// # What this test used to say
    ///
    /// It recorded the fee-only case as unconstructible a second, deeper
    /// way than the cardinality floor: a fee output carries an empty
    /// scriptPubKey, the fixture vocabulary had no fee role, and every
    /// output was required to carry a nonempty program. Even with the
    /// floor relaxed, a fee output was inexpressible.
    ///
    /// The vocabulary now HAS the role, and the clause reads on the role
    /// rather than on every output alike. So the same empty program is
    /// admitted where the output declares itself a fee and refused where
    /// it does not — which is the difference between a rule and an
    /// exception, and the reason the role was added rather than the clause
    /// loosened.
    #[test]
    fn the_empty_program_clause_now_reads_on_the_role() {
        // Not a fee, and still refused. Nothing was loosened.
        let refusal = registry_refusal_for(
            "ctf-v1/test-empty-program",
            ASSET,
            ZERO_SUM,
            vec![
                output(FixtureOutputRole::Primary, 500_000_000, vec![0x51]),
                output(FixtureOutputRole::Balancing, 500_000_000, Vec::new()),
            ],
        )
        .expect("a non-fee output with no program is refused");
        assert_eq!(
            refusal,
            RegistrationRefusal::OutputProgramEmpty { output: 1 },
            "an output that does not declare itself a fee still needs a program",
        );

        // A fee, and admitted. The blinded output balances; the fee is
        // held out of the solve at a zero blinder.
        let (_, view) = register_multi(
            "ctf-v1/test-fee-role",
            ASSET,
            NON_CANCELING_SUM,
            vec![
                output(FixtureOutputRole::Balancing, 900_000_000, vec![0x51]),
                output(FixtureOutputRole::Fee, 100_000_000, Vec::new()),
            ],
        )
        .expect("the fee-bearing manifest registers AND projects");

        // This assertion used to read the other way. The projection
        // refused `FeeRoleNotProjectable`, and the refusal was correct
        // while the materializer had no fee stage: mapping a fee onto the
        // balancing role to get past that line would have produced a
        // BLINDED fee output, which is not a fee at the target. The stage
        // exists now, so the refusal does not — the variant is gone from
        // the vocabulary rather than left standing as a stopper nothing
        // can return — and the test is updated to the new fact instead of
        // being loosened to accept either.
        assert_eq!(view.outputs().len(), 2);
        let fee = &view.outputs()[1];
        assert_eq!(
            fee.role(),
            transaction::live_materialize::ConfidentialOutputRole::Fee
        );
        assert_eq!(fee.semantic_amount(), 100_000_000);
        assert!(
            fee.output_program().is_empty(),
            "a fee output's program is empty, which is most of what makes it one",
        );

        // The opening is ABSENT and not zero-filled. Three zeroed scalars
        // would read like an opening, and an explicit output has none.
        assert!(fee.value_blinder().is_none());
        assert!(fee.nonce_input().is_none());
        assert!(fee.rangeproof_seed().is_none());

        // And the blinded output beside it still solves. The fee is held
        // out of the solve at a zero blinder, so the sole balancing output
        // takes the input blinder sum unchanged.
        assert_eq!(
            view.outputs()[0].value_blinder(),
            Some(&NON_CANCELING_SUM),
            "the fee contributes nothing to the solve",
        );
    }
}

/// The run of record: what one execution of the fifth step observed.
///
/// # Why the observation is a constant and not a stored file
///
/// The evidence a run produces is the observation, and an observation
/// nobody can name is not evidence. These constants are the identities
/// and figures ONE run against a real node produced, written down so that
/// a later reader can ask the chain the same question, and so that a
/// claim made anywhere in this workspace about the fifth step can be
/// traced to a transaction identity rather than to a test having been
/// written.
///
/// It re-runs nothing and proves nothing by existing. What it does is
/// make the run's own answer quotable.
///
/// Fixture digests recorded under algorithm v1 remain immutable run data. Each accepted identity
/// and recorded v1 digest are the two halves of one historical observation; forward v2 fixture
/// identities live separately and await their own node-accepted runs.
///
/// The target: Elements Core v28.99.0-b7fc5d080a7e, at the pinned tip the
/// lane binds itself to, on a disposable development chain the run
/// created and destroyed. All three shapes ran serialized against one
/// node, because the shared instance does not sustain the parallel lane's
/// concurrent nodes.
pub mod run_of_record {
    /// The disposable asset all three runs issued.
    ///
    /// The same identity the earlier steps issued, because the issuing
    /// step is theirs and the chain is created fresh per run.
    pub const ISSUED_ASSET: &str =
        "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

    /// The split shape's recorded successor fixture digest under fixture-digest v1.
    ///
    /// This value and `SPLIT_ACCEPTED_TXID` are the immutable halves of one historical observation.
    /// The same fixture's forward v2 digest is separate and awaits its own node-accepted run.
    pub const SPLIT_SUCCESSOR_DIGEST: &str =
        "43e15876204c04feecc0dc49387479288923392c9b6c3a0cc7be14e35edd4d99";

    /// The identity the target computed for the accepted split.
    ///
    /// One receipt consumed, THREE outputs created: two recipients and
    /// the balancing change. The row `private-split` moves on THIS
    /// acceptance and cites it.
    pub const SPLIT_ACCEPTED_TXID: &str =
        "56c97ec9748ec730a6a6fab111c15830a4e215c0652ec5df4e4b34e7bd955a0c";

    crate::recorded_acceptance::mint_recorded_acceptance!(split_accepted, SPLIT_ACCEPTED_TXID);

    /// How many bytes the split handed to the node.
    pub const SPLIT_SUBMITTED_BYTES: usize = 13_499;

    /// The range-proof bytes each of the split's three outputs carried.
    pub const SPLIT_OUTPUT_WITNESS_PROOF_BYTES: [usize; 3] = [4_174, 4_174, 4_174];

    /// The split's wall time, in seconds.
    pub const SPLIT_WALL_SECONDS: f64 = 12.9;

    /// The many-to-many shape's recorded successor fixture digest under fixture-digest v1.
    ///
    /// This value and `MANY_TO_MANY_ACCEPTED_TXID` are the immutable halves of one historical
    /// observation. The same fixture's forward v2 digest is separate and awaits its own
    /// node-accepted run.
    pub const MANY_TO_MANY_SUCCESSOR_DIGEST: &str =
        "31162852b1f393b74be3bfa9ef2f44bacd17d0126947911937d5ae708792f421";

    /// The identity the target computed for the accepted many-to-many.
    ///
    /// TWO receipts consumed and THREE outputs created — the
    /// representative case, whose input and output counts both exceed the
    /// one-to-one control's, so it is not a one-to-many or a many-to-one
    /// under another name.
    pub const MANY_TO_MANY_ACCEPTED_TXID: &str =
        "fc1769b853b3f3abf24a6eb996fa76e5e19c87adfd379034392e9487af85a58d";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        many_to_many_accepted,
        MANY_TO_MANY_ACCEPTED_TXID
    );

    /// How many bytes the many-to-many handed to the node.
    pub const MANY_TO_MANY_SUBMITTED_BYTES: usize = 13_882;

    /// The many-to-many's wall time, in seconds.
    pub const MANY_TO_MANY_WALL_SECONDS: f64 = 13.9;

    /// The several-distinct-owners shape's recorded successor fixture digest under fixture-digest
    /// v1.
    ///
    /// This value and `SEVERAL_OWNERS_ACCEPTED_TXID` are the immutable halves of one historical
    /// observation. The same fixture's forward v2 digest is separate and awaits its own
    /// node-accepted run.
    pub const SEVERAL_OWNERS_SUCCESSOR_DIGEST: &str =
        "cfecf21f58fcc4d0571cccb701915f09025a7c8066415fd4d82f62839aba7dcc";

    /// The identity the target computed for the accepted
    /// several-distinct-owners transfer.
    ///
    /// TWO receipts consumed under two DISTINCT published owners, each
    /// input carrying the leaf its own position executes, and two outputs
    /// created. Its subject is the input owners rather than the
    /// cardinality.
    pub const SEVERAL_OWNERS_ACCEPTED_TXID: &str =
        "15ff668df082fb22f2222a52f8cc0fc27f5787ec82b3c0408472ae063fc1e207";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        several_owners_accepted,
        SEVERAL_OWNERS_ACCEPTED_TXID
    );

    /// How many bytes the several-owners transfer handed to the node.
    pub const SEVERAL_OWNERS_SUBMITTED_BYTES: usize = 9_519;

    /// The several-owners transfer's wall time, in seconds.
    pub const SEVERAL_OWNERS_WALL_SECONDS: f64 = 15.6;

    /// The identity the target computed for the accepted STRICT
    /// ONE-TO-ONE.
    ///
    /// # What this identity is evidence of
    ///
    /// ONE receipt consumed and ONE output created — a shape this
    /// workspace's own fixture registry refused to express until the
    /// two-output floor was structurally removed. The consensus shape
    /// census recorded it source-derived-possible and refused, with the
    /// floor named as a first-party convention rather than a protocol
    /// rule; this is the acceptance that moves that entry off the
    /// derivation and onto a chain.
    ///
    /// It moves NO matrix row. The guide's §15.2 positive private table
    /// has no member for the strict one-to-one, and the census entry is
    /// what an acceptance of it moves.
    ///
    /// The lone output's value blinder is FORCED to the input blinder
    /// sum, which for one consumed receipt is that coin's own blinder.
    /// Nothing here is claimed about a merge: a merge consumes two coins
    /// and this consumed one.
    pub const STRICT_ONE_TO_ONE_ACCEPTED_TXID: &str =
        "139b9d4475d93e242fd0c5c8efb986b523945ae905edea427b66ca2050df6db8";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        strict_one_to_one_accepted,
        STRICT_ONE_TO_ONE_ACCEPTED_TXID
    );

    /// The strict one-to-one's recorded successor fixture digest under fixture-digest v1.
    ///
    /// This value and `STRICT_ONE_TO_ONE_ACCEPTED_TXID` are the immutable halves of one historical
    /// observation. The same fixture's forward v2 digest is separate and awaits its own
    /// node-accepted run.
    pub const STRICT_ONE_TO_ONE_SUCCESSOR_DIGEST: &str =
        "00d0179914058b9a1f59ec71de77b3dfd4f48928313a9d52f41f12f003d735b2";

    /// How many bytes the strict one-to-one handed to the node.
    ///
    /// The smallest submission of any shape this lane has run, and for a
    /// structural reason rather than by chance: one output means one range
    /// proof, and the range proof is most of a confidential transaction.
    pub const STRICT_ONE_TO_ONE_SUBMITTED_BYTES: usize = 4_773;

    /// The range-proof bytes its one output witness carried.
    pub const STRICT_ONE_TO_ONE_PROOF_BYTES: [usize; 1] = [4_174];

    /// The strict one-to-one's wall time, in seconds.
    pub const STRICT_ONE_TO_ONE_WALL_SECONDS: f64 = 11.2;

    /// The path the accepted strict one-to-one actually took, recorded
    /// because a shape's acceptance is only as strong as the door it came
    /// through.
    ///
    /// It crossed BOTH. The adapter offers a submission to
    /// `testmempoolaccept` first and reports an acceptance only where that
    /// answered allowed, then confirms it with `generateblock` so the
    /// acceptance is one by block validation too. So this shape was
    /// admitted by mempool policy and then included in a block, rather
    /// than reaching a block as a package child or by consensus retry
    /// after a policy refusal — the retry path the adapter keeps for a
    /// transaction standardness turns away.
    ///
    /// What that does NOT establish is anything about relay on a network
    /// this workspace does not run. The chain is a disposable development
    /// one the run created and destroyed, carrying its own policy, and the
    /// transaction pays no fee because it carries no fee output at all.
    /// The claim is that this node's own mempool admitted it, which is
    /// what was observed and the whole of what is recorded.
    pub const STRICT_ONE_TO_ONE_CROSSED_RELAY_AND_BLOCK: bool = true;

    /// How many receipt inputs each shape consumed, in the order the
    /// restart runs them.
    ///
    /// Recorded rather than assumed, so a shape whose cardinality drifted
    /// is readable here rather than inferred from a name.
    ///
    /// The order is these arrays' OWN and is not
    /// [`super::PrivateShape::ALL`]'s: split, many-to-many,
    /// several-distinct-owners, strict one-to-one, one-to-one-with-fee,
    /// merge, pure split. The two crossings are absent because a
    /// crossing's sides are read under different representation plans
    /// and a single receipt count would state one side as though it were
    /// both. The pure split is LAST for the reason it is last in `ALL`:
    /// it was appended after the runs before it were recorded.
    pub const RECEIPT_LEAVES: [usize; 7] = [1, 2, 2, 1, 1, 2, 1];

    /// How many outputs each shape created, in the same order.
    pub const OUTPUT_COUNTS: [usize; 7] = [3, 3, 2, 1, 2, 1, 2];

    // --- The private merge: the row this wave moves --------------------

    /// The merge's recorded successor fixture digest under fixture-digest v1.
    ///
    /// This value and `MERGE_ACCEPTED_TXID` are the immutable halves of one historical observation.
    /// The same fixture's forward v2 digest is separate and awaits its own node-accepted run.
    pub const MERGE_SUCCESSOR_DIGEST: &str =
        "f1bf90b46d16a814c0a1069bb8fa2ab9ea9708141fb7454d7e6c7720484588aa";

    /// The identity the target computed for the accepted private merge.
    ///
    /// TWO receipts consumed and ONE output created. The row
    /// `private-merge` moves on THIS acceptance and cites it.
    ///
    /// It is the first acceptance of a shape that had met two walls: the
    /// registry's two-output floor, and then the zero blinder its only
    /// available predecessor forced. The floor was removed by the
    /// sole-balancing form and the zero blinder by a predecessor whose
    /// coins do not cancel, and neither removal was worth anything until
    /// this identity existed.
    pub const MERGE_ACCEPTED_TXID: &str =
        "fe48b8c0feb8adeecc78fc78f91a2d24f43871672f3c083f06984383a2ff4a4d";

    crate::recorded_acceptance::mint_recorded_acceptance!(merge_accepted, MERGE_ACCEPTED_TXID);

    /// How many bytes the merge handed to the node.
    pub const MERGE_SUBMITTED_BYTES: usize = 5_156;

    /// The range-proof bytes its one output witness carried.
    ///
    /// One proof for one output, and two inputs' worth of witness beside
    /// it -- which is why the merge is larger than the strict one-to-one
    /// despite having the same output count.
    pub const MERGE_PROOF_BYTES: [usize; 1] = [4_174];

    /// The commitment prefixes the merge's three funded coins carried.
    ///
    /// NOT the admitted pair in fixed order, and that is the arity rule
    /// working rather than a defect. A three-output fixture is held to
    /// membership -- each commitment carries one of the two admitted
    /// prefixes -- because the reviewed target contract states no
    /// fixed-order convention for a third output. Two of these three are
    /// the same prefix, which a fixed-order rule would have rejected and
    /// which the contract does not.
    pub const MERGE_PREDECESSOR_PREFIXES: [u8; 3] = [0x09, 0x08, 0x09];

    /// Whether the merge's forced blinder came out ZERO.
    ///
    /// False, and observed rather than argued. The ceremony summed the
    /// two coins it actually consumed, compared that sum with the blinder
    /// the registry solved for the sole output, and wrote both answers
    /// into its transcript. Nothing in this workspace claims hiding for a
    /// zero-blinder commitment, so a merge that could not say this is
    /// false would not be a merge worth recording.
    pub const MERGE_FORCED_BLINDER_IS_ZERO: bool = false;

    /// Whether the two consumed coins' blinders cancel.
    ///
    /// False. That is the whole difference between this merge and the one
    /// the registry refuses: merging both halves of the dual-parity
    /// predecessor's inverse pair gives a consumed sum of zero, and
    /// merging two coins of a three-output predecessor does not, because
    /// three blinders summing to zero cancel in no pair.
    pub const MERGE_CONSUMED_PAIR_CANCELS: bool = false;

    /// The merge's wall time, in seconds.
    pub const MERGE_WALL_SECONDS: f64 = 10.9;

    // --- The fee-bearing shape: ACCEPTED, after three refusals ---------

    /// The fee-bearing one-to-one's recorded successor fixture digest under fixture-digest v1.
    ///
    /// This value and `FEE_BEARING_SUCCESSOR_IDENTITY` are the immutable halves of one historical
    /// observation. The same fixture's forward v2 digest is separate and awaits its own
    /// node-accepted run.
    pub const FEE_BEARING_SUCCESSOR_DIGEST: &str =
        "32d0a5a76d75c91fab564731e19f29518b4dda492a830f3d4b9ee578ea25f3f8";

    /// The identity a real node computed for the fee-bearing transfer.
    ///
    /// One receipt consumed, one blinded destination created, and the
    /// transaction's own fee paid out of the value it consumed, with no
    /// sponsor anywhere in it. The node accepted it and mined it, and the
    /// bytes it handed back are equal to the bytes submitted.
    ///
    /// # Three refusals stood between the vocabulary and this figure
    ///
    /// Each was a layer the one before it uncovered, and each was a
    /// first-party defect rather than a property of the target. The
    /// registry had no fee output role. The materializer had no fee
    /// projection, so a fee would have been blinded. Then the shape
    /// vocabulary had no sponsorless fee-bearing member, so a
    /// two-destination request selected a two-receipt-output shape and
    /// the receipt covenant demanded a receipt program where the fee's
    /// empty one sat.
    ///
    /// Giving the vocabulary the member uncovered a fourth, which is the
    /// pattern holding rather than breaking: the deployment was welded to
    /// a fee-program digest of 0xb5 bytes that no program hashes to, kept
    /// deliberately so the demonstration's taptree would not move, and
    /// nothing had ever executed the clause that reads it.
    pub const FEE_BEARING_ACCEPTED_IDENTITY: Option<&str> =
        Some("65a4b10b802292f583a15dd0ac9c40e57832981a4a072ee8574f00c5ef692b9f");

    /// The same identity, as the register cites an acceptance.
    ///
    /// The `Option` above says whether an acceptance exists; a consensus
    /// verdict needs the identity itself. Written once and read from
    /// there, so the two can never disagree about what was accepted.
    pub const FEE_BEARING_SUCCESSOR_IDENTITY: &str =
        "65a4b10b802292f583a15dd0ac9c40e57832981a4a072ee8574f00c5ef692b9f";

    /// How many bytes the fee-bearing one-to-one handed to the node.
    pub const FEE_BEARING_SUBMITTED_BYTES: usize = 4_927;

    /// The output-witness entries the fee-bearing candidate carried.
    ///
    /// This array is the fee projection's own evidence, and it is the
    /// reason the run is worth recording despite the refusal. The blinded
    /// output carries a range proof of the usual size; the FEE output
    /// carries an empty entry. A fee that had been mapped onto the
    /// balancing role would read `[4_174, 4_174]` here — a blinded fee,
    /// and not a fee at all.
    pub const FEE_BEARING_PROOF_BYTES: [usize; 2] = [4_174, 0];

    /// The verdict the target returned when the shape vocabulary had no
    /// member for this form.
    ///
    /// KEPT, and kept deliberately, though the shape is now accepted. It
    /// is the diagnosis that located the third layer, and the register's
    /// discipline is that a wall's history survives its removal -- a
    /// removal whose refusal has been deleted cannot be checked against
    /// what it claims to have removed.
    ///
    /// A script-path rejection, at the workspace's OWN receipt covenant
    /// rather than at any confidential rule. The candidate's value balance
    /// was never reached and nothing here is a statement about the target's
    /// fee rules: Elements admits a fee output in a non-policy asset at
    /// consensus and at policy alike, and this node runs with a zero
    /// minimum relay feerate, so the transaction did not fail for carrying
    /// a fee.
    ///
    /// What it failed is the covenant the shape selection built for it.
    /// The request states two destinations, the reviewed live-transfer
    /// shape vocabulary read a two-destination sponsorless shape as TWO
    /// RECEIPT OUTPUTS, and the receipt covenant therefore required the
    /// second output to carry the second owner's private receipt
    /// constructor program. The second output is the fee, whose program is
    /// empty, so the comparison failed.
    pub const FEE_BEARING_OBSERVED_DETAIL: &str =
        "mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)";

    /// The fee-bearing run's wall time, in seconds.
    pub const FEE_BEARING_WALL_SECONDS: f64 = 12.4;

    // --- The exit crossing: representation crossed at a real node -------

    /// The exit crossing's recorded successor fixture digest under fixture-digest v1.
    ///
    /// This value and `EXIT_CROSSING_ACCEPTED_TXID` are the immutable halves of one historical
    /// observation. The same fixture's forward v2 digest is separate and awaits its own
    /// node-accepted run.
    pub const EXIT_CROSSING_SUCCESSOR_DIGEST: &str =
        "9c2b302c4becdff2ed90545006f0d371e69e0a7eabce71c33a2a8cbe60f44fbe";

    /// The identity the target computed for the accepted exit crossing.
    ///
    /// TWO blinded receipts consumed, TWO EXPLICIT destinations created,
    /// and ONE blinded absorber beside them. It is the first transaction
    /// this workspace has built whose consumed and created sides are read
    /// under DIFFERENT representation plans, and the first evidence that
    /// the target admits one -- which it always did, upstream having
    /// tested the shape directly; what did not exist was a vocabulary in
    /// which the candidate could be stated.
    pub const EXIT_CROSSING_ACCEPTED_TXID: &str =
        "b382a7c3a6057e49d9b2c11cead5d1864c53e238176480c9709b21ebd1b7d656";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        exit_crossing_accepted,
        EXIT_CROSSING_ACCEPTED_TXID
    );

    /// How many bytes the exit crossing handed to the node.
    pub const EXIT_CROSSING_SUBMITTED_BYTES: usize = 5_412;

    /// The output-witness proof bytes the exit crossing carried.
    ///
    /// THE CENSUS THAT MAKES THE CROSSING VISIBLE IN THE BYTES, and it
    /// is read off the candidate rather than predicted: two entries
    /// EMPTY and one carrying a range proof. An explicit value admits no
    /// range proof and an explicit asset no surjection proof, so the two
    /// explicit destinations carry neither, and the single blinded
    /// absorber carries the one proof the transaction has.
    ///
    /// A wholly private shape of this arity would carry three proofs and
    /// a wholly explicit one none, so this vector is a shape no
    /// homogeneous transfer can produce.
    pub const EXIT_CROSSING_PROOF_BYTES: [usize; 3] = [0, 0, 4_174];

    /// Whether the exit crossing's consumed pair cancels.
    ///
    /// FALSE, and it is load-bearing rather than incidental. A canceling
    /// pair presents a zero blinder sum, the absorber's solved blinder
    /// would be zero, and an absorber that hides nothing is not an
    /// absorber -- the registry refuses exactly that as a degenerate
    /// balancing scalar. This shape spends the merge's own non-canceling
    /// predecessor for that reason.
    pub const EXIT_CROSSING_CONSUMED_PAIR_CANCELS: bool = false;

    /// The weight the target reported for the exit crossing.
    pub const EXIT_CROSSING_TARGET_WEIGHT: u64 = 6_561;

    /// The exit crossing's wall time, in seconds.
    pub const EXIT_CROSSING_WALL_SECONDS: f64 = 11.5;

    // --- The entry crossing: the other direction, at a real node --------

    /// The entry crossing's recorded successor fixture digest under fixture-digest v1.
    ///
    /// This value and `ENTRY_CROSSING_ACCEPTED_TXID` are the immutable halves of one historical
    /// observation. The same fixture's forward v2 digest is separate and awaits its own
    /// node-accepted run.
    pub const ENTRY_CROSSING_SUCCESSOR_DIGEST: &str =
        "2a2d580164d9ce541b9892d0ab6692e4c0770f8458b0534a2dc2f8e3af08da3f";

    /// The identity the target computed for the accepted entry crossing.
    ///
    /// ONE EXPLICIT receipt consumed and TWO blinded destinations
    /// created. This workspace has performed the SHAPE every ceremony,
    /// as the funding step that mints a confidential predecessor; what
    /// this identity records is the first time the coin it spent sat at
    /// a receipt constructor's program, so the transfer was governed by
    /// the covenant rather than by the adapter.
    pub const ENTRY_CROSSING_ACCEPTED_TXID: &str =
        "7a1771fd3d04cc7ee2c48a0d7daa9287122b6ca07706c943f48a23aa8192793f";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        entry_crossing_accepted,
        ENTRY_CROSSING_ACCEPTED_TXID
    );

    /// How many bytes the entry crossing handed to the node.
    pub const ENTRY_CROSSING_SUBMITTED_BYTES: usize = 9_133;

    /// The output-witness proof bytes the entry crossing carried.
    ///
    /// TWO proofs for two blinded destinations, and the count is the
    /// claim: a single blinded output would have been forced to a ZERO
    /// blinder, because an explicit input contributes one, and its
    /// commitment would have hidden nothing. Two is the floor, and it is
    /// the registry's own arithmetic rather than a preference.
    pub const ENTRY_CROSSING_PROOF_BYTES: [usize; 2] = [4_174, 4_174];

    /// Whether the entry crossing's consumed coin carried a blinder.
    ///
    /// FALSE. Its value was explicit, so the blinder it contributed to
    /// the transaction-wide sum was the all-zero one every explicit
    /// value is committed with -- which is why the balancing output's
    /// blinder is the negation of a searched non-zero primary rather
    /// than a consumed sum.
    pub const ENTRY_CROSSING_CONSUMED_A_BLINDER: bool = false;

    /// The weight the target reported for the entry crossing.
    ///
    /// Larger than the exit crossing's, and the reason is the crossing
    /// itself rather than the arity: two blinded outputs carry two range
    /// proofs where the exit crossing's one blinded absorber carries
    /// one, and a range proof is most of what a confidential output
    /// weighs. The exit direction is CHEAPER on the wire, which is the
    /// same fact its shorter form obligation states in the covenant.
    pub const ENTRY_CROSSING_TARGET_WEIGHT: u64 = 10_093;

    /// The entry crossing's wall time, in seconds.
    pub const ENTRY_CROSSING_WALL_SECONDS: f64 = 9.9;

    /// The identity the target computed for the accepted PURE split.
    ///
    /// One receipt consumed, TWO created, both of them receipts, no
    /// change and no fee. It moves no §15.2 row: `private-split` moved
    /// on the three-output split and a row does not move twice. What it
    /// answers is §16.2's second conjunct for the SPLIT pair, whose
    /// private member is this shape and not that one.
    pub const PURE_SPLIT_ACCEPTED_TXID: &str =
        "544afb18a0016db35e007f1da9a49d60ba163b8cc565adf16fe25a04a198eea0";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        pure_split_accepted,
        PURE_SPLIT_ACCEPTED_TXID
    );

    /// How many bytes the pure split handed to the node.
    pub const PURE_SPLIT_SUBMITTED_BYTES: usize = 9_136;

    /// The output-witness proof bytes the pure split carried.
    ///
    /// TWO proofs for two created outputs, and the count is the whole
    /// claim this run exists to make. A range proof is what a BLINDED
    /// output carries and a fee output carries none, so two proofs over
    /// two outputs is the measured form of "both created outputs are
    /// receipts" -- the conjunct the fee-bearing one-in-two-out run
    /// could not satisfy, its second output having been a fee.
    pub const PURE_SPLIT_PROOF_BYTES: [usize; 2] = [4_174, 4_174];

    /// The weight the target reported for the pure split.
    pub const PURE_SPLIT_TARGET_WEIGHT: u64 = 10_096;

    /// The pure split's recorded successor fixture digest under fixture-digest v1.
    ///
    /// This value and `PURE_SPLIT_ACCEPTED_TXID` are the immutable halves of one historical
    /// observation. The same fixture's forward v2 digest is separate and awaits its own
    /// node-accepted run.
    pub const PURE_SPLIT_SUCCESSOR_DIGEST: &str =
        "e9a68f1d3d93350fdf02f879f392e4b86f57ad3295cda42735c301564bd9c3a3";

    /// The pure split's wall time, in seconds.
    pub const PURE_SPLIT_WALL_SECONDS: f64 = 14.1;
}
