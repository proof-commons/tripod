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
use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationStepKind,
    OperationSubject, TargetSubmissionSubject,
};
use transaction::bytes::{Outpoint, ValueField};
use transaction::live_census::OwnerSigningCensus;
use transaction::live_construct::{
    PrivateDestinationOpening, PrivateInputOpening, PrivateLiveFinalization, PrivateLiveOpenings,
    finalize_private_live_transfer,
};
use transaction::live_materialize::{
    ConfidentialOutputRole, FixtureOpeningReference, FrozenConfidentialFixtureView,
    NonProtocolFundingRegion, SCALAR_BYTES,
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
use crate::live_plan::{FIRST_SCALAR, SECOND_SCALAR, published_owner, reviewed_target};
use crate::live_private_restart::{
    ConsumedReceipt, LinkedDeployment, PrivateRestartRefusal, RestartConfidentialCoin,
    assemble_control, confidential_funding_step, issue_step, link_and_register,
    observe_funded_coins, private_program, verify_readback_signature,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrivateShape {
    /// One receipt in, three outputs: two recipients and one balancing
    /// change output back to the sender.
    Split,
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
}

impl PrivateShape {
    /// All six, in the order the restart runs them.
    pub const ALL: [Self; 6] = [
        Self::Split,
        Self::ManyToMany,
        Self::SeveralDistinctOwners,
        Self::StrictOneToOne,
        Self::OneToOneWithFee,
        Self::PrivateMerge,
    ];

    /// The ceremony's own name for the shape, used as the report
    /// extension and the successor fixture handle's discriminator.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Split => "private-split",
            Self::ManyToMany => "private-many-to-many",
            Self::SeveralDistinctOwners => "private-several-distinct-owners",
            Self::StrictOneToOne => "private-strict-one-to-one",
            Self::OneToOneWithFee => "private-one-to-one-with-fee",
            Self::PrivateMerge => "private-merge",
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
            Self::StrictOneToOne | Self::OneToOneWithFee => None,
            // The merge DOES have a row, and it is the only shape this
            // wave adds that has one.
            Self::PrivateMerge => Some("private-merge"),
        }
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
            | Self::ManyToMany
            | Self::SeveralDistinctOwners
            | Self::StrictOneToOne
            | Self::OneToOneWithFee => PredecessorShape::DualParity,
            Self::PrivateMerge => PredecessorShape::TripleNonCanceling,
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
            | Self::ManyToMany
            | Self::SeveralDistinctOwners
            | Self::StrictOneToOne
            | Self::PrivateMerge => 0,
            Self::OneToOneWithFee => 1,
        }
    }

    /// Which predecessor outputs this shape consumes, in fixed order.
    #[must_use]
    const fn consumed(self) -> &'static [ConsumedReceipt] {
        match self {
            Self::Split | Self::StrictOneToOne | Self::OneToOneWithFee => {
                &[ConsumedReceipt::Primary]
            }
            // The merge consumes the same two INDICES the two-input
            // shapes do. Against the triple predecessor those indices
            // carry the same two roles, and the difference that matters
            // is the third output standing beside them: it is why the
            // pair's blinders do not cancel.
            Self::ManyToMany | Self::SeveralDistinctOwners | Self::PrivateMerge => {
                &[ConsumedReceipt::Primary, ConsumedReceipt::Balancing]
            }
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
        match self {
            // 700_000_000 in, split three ways back to the two owners.
            Self::Split => vec![
                primary(SECOND_SCALAR, 400_000_000),
                primary(FIRST_SCALAR, 200_000_000),
                balancing(FIRST_SCALAR, 100_000_000),
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
        let linked =
            link_and_register(self.shape.predecessor(), ConsumedReceipt::Primary, printed)?;
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
                    output_program: if destination.role == FixtureOutputRole::Fee {
                        Vec::new()
                    } else {
                        private_program(&linked.abi, &destination.scalar)?
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
        self.record.coins = observe_funded_coins(linked, response)?;
        Ok(())
    }

    /// The successor's wire bytes, every owner's authorization in its own
    /// input's witness.
    fn control_bytes(&mut self) -> Result<Vec<u8>, PrivateRestartRefusal> {
        let finalization = self.finalize_shape()?;
        let built = assemble_control(&finalization, self.genesis_block_hash)?;
        self.spent_owner_bytes = built.spent_owner_bytes;
        self.record.receipt_leaves = built.receipt_leaves;
        self.record.output_witness_proof_bytes = built.output_witness_proof_bytes;
        let bytes = built.transaction.encode();
        self.record.submitted_bytes = bytes.len();
        self.census = Some(built.census);
        Ok(bytes)
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
                opening: FixtureOpeningReference::new(
                    linked.predecessor.handle().as_str().to_owned(),
                    linked.predecessor_digest,
                    receipt.index(),
                ),
                explicit_amount: linked.predecessor.amounts()[receipt.index()],
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
        let request = LiveTransferRequest::new(
            receipts,
            live_destinations,
            LiveTransferRepresentationPlan::PrivateCommitted,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            // Present because the private request vocabulary requires it;
            // the materializer takes its blinders from fixtures, so it is
            // not a source of any opening.
            Some(PublicTestRandomness::from_published_bytes([0x7e; 32])),
        )
        .map_err(|_| PrivateRestartRefusal::ControlNotRequestable)?;

        // The openings carry the roles the shape stated, not the roles a
        // position implies. This is the second face of the same
        // retirement: the fixture registry stopped inferring the balance
        // from an output's index, and so does the layer that hands the
        // materializer its openings.
        let destination_openings: Vec<PrivateDestinationOpening> = destinations
            .iter()
            .enumerate()
            .map(|(index, destination)| PrivateDestinationOpening {
                fixture: FixtureOpeningReference::new(
                    self.shape.successor_handle(),
                    successor.digest,
                    index,
                ),
                role: match destination.role {
                    FixtureOutputRole::Primary => ConfidentialOutputRole::Primary,
                    // The fee role is stated rather than swept into the
                    // catch-all. It used to fall through to `Balancing`,
                    // which would have asked the materializer to solve a
                    // blinder for an output that must not carry one — a
                    // blinded fee, and not a fee.
                    FixtureOutputRole::Fee => ConfidentialOutputRole::Fee,
                    // Both solving roles are the view's one solving role:
                    // the sole form is a solve over no others, which is
                    // the same instruction to the materializer.
                    _ => ConfidentialOutputRole::Balancing,
                },
            })
            .collect();

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

        finalize_private_live_transfer(
            &linked.abi,
            &request,
            &view,
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

#[cfg(test)]
mod byte_identity_tests {
    use super::{MultiShapePlanner, PrivateShape, hex, run_of_record as run};

    /// Every shape that ran BEFORE this wave still registers its
    /// successor under the digest that run recorded.
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
    /// The digests below were written down by ceremonies that ran against
    /// a pinned node before any of it. Recomputing them re-derives every
    /// blinder, every nonce input, every range-proof seed and every
    /// commitment prefix of four fixtures whose successors a target
    /// ACCEPTED, so the claim that nothing moved is a running check
    /// rather than a sentence in a commit message.
    ///
    /// The predecessor half of the same claim is checked by the sibling
    /// test in the private-restart module, which holds the dual-parity
    /// predecessor and both one-to-one successors to their own recorded
    /// digests.
    #[test]
    fn the_shapes_that_ran_before_this_wave_register_under_their_recorded_digests() {
        // The genesis identity is not a term of any fixture digest, and
        // this test would fail loudly if it became one.
        let genesis: transaction::taproot::Digest32 = [0x11_u8; 32];
        for (shape, expected) in [
            (PrivateShape::Split, run::SPLIT_SUCCESSOR_DIGEST),
            (PrivateShape::ManyToMany, run::MANY_TO_MANY_SUCCESSOR_DIGEST),
            (
                PrivateShape::SeveralDistinctOwners,
                run::SEVERAL_OWNERS_SUCCESSOR_DIGEST,
            ),
            (
                PrivateShape::StrictOneToOne,
                run::STRICT_ONE_TO_ONE_SUCCESSOR_DIGEST,
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
                expected,
                "{}'s successor fixture drifted from the run of record",
                shape.name(),
            );
        }
    }

    /// The two shapes this wave added record their own digests, and they
    /// are not each other's and not anybody else's.
    ///
    /// A new fixture that happened to derive an existing case's digest
    /// would mean the handle is not a term of the transcript, which is a
    /// defect and not a coincidence.
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

        // And the two new ones are the ones their runs recorded.
        assert_eq!(digests[4], run::FEE_BEARING_SUCCESSOR_DIGEST);
        assert_eq!(digests[5], run::MERGE_SUCCESSOR_DIGEST);
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

    /// The split shape's successor fixture digest.
    pub const SPLIT_SUCCESSOR_DIGEST: &str =
        "75d3cb8d99dff1f7fb5ad39dbf7fa1fe986896d7056d55020911acdf9eeea23f";

    /// The identity the target computed for the accepted split.
    ///
    /// One receipt consumed, THREE outputs created: two recipients and
    /// the balancing change. The row `private-split` moves on THIS
    /// acceptance and cites it.
    pub const SPLIT_ACCEPTED_TXID: &str =
        "56c97ec9748ec730a6a6fab111c15830a4e215c0652ec5df4e4b34e7bd955a0c";

    /// How many bytes the split handed to the node.
    pub const SPLIT_SUBMITTED_BYTES: usize = 13_499;

    /// The range-proof bytes each of the split's three outputs carried.
    pub const SPLIT_OUTPUT_WITNESS_PROOF_BYTES: [usize; 3] = [4_174, 4_174, 4_174];

    /// The split's wall time, in seconds.
    pub const SPLIT_WALL_SECONDS: f64 = 12.9;

    /// The many-to-many shape's successor fixture digest.
    pub const MANY_TO_MANY_SUCCESSOR_DIGEST: &str =
        "4de73ff0bdece00b0a5f5959beba455adc9998ae2aaef65ffdaffec157a3cc6e";

    /// The identity the target computed for the accepted many-to-many.
    ///
    /// TWO receipts consumed and THREE outputs created — the
    /// representative case, whose input and output counts both exceed the
    /// one-to-one control's, so it is not a one-to-many or a many-to-one
    /// under another name.
    pub const MANY_TO_MANY_ACCEPTED_TXID: &str =
        "fc1769b853b3f3abf24a6eb996fa76e5e19c87adfd379034392e9487af85a58d";

    /// How many bytes the many-to-many handed to the node.
    pub const MANY_TO_MANY_SUBMITTED_BYTES: usize = 13_882;

    /// The many-to-many's wall time, in seconds.
    pub const MANY_TO_MANY_WALL_SECONDS: f64 = 13.9;

    /// The several-distinct-owners shape's successor fixture digest.
    pub const SEVERAL_OWNERS_SUCCESSOR_DIGEST: &str =
        "c80fc25c07cf59c969e8d0bd62edd9540b15731ec5fc548ad5d20c06dff39ec9";

    /// The identity the target computed for the accepted
    /// several-distinct-owners transfer.
    ///
    /// TWO receipts consumed under two DISTINCT published owners, each
    /// input carrying the leaf its own position executes, and two outputs
    /// created. Its subject is the input owners rather than the
    /// cardinality.
    pub const SEVERAL_OWNERS_ACCEPTED_TXID: &str =
        "15ff668df082fb22f2222a52f8cc0fc27f5787ec82b3c0408472ae063fc1e207";

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

    /// The strict one-to-one successor fixture's digest.
    pub const STRICT_ONE_TO_ONE_SUCCESSOR_DIGEST: &str =
        "b41dbdee47f2de2f6a9dec1e0c6ffc4c21e6e6a591828d5e793b82b11436f70d";

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
    pub const RECEIPT_LEAVES: [usize; 6] = [1, 2, 2, 1, 1, 2];

    /// How many outputs each shape created, in the same order.
    pub const OUTPUT_COUNTS: [usize; 6] = [3, 3, 2, 1, 2, 1];

    // --- The private merge: the row this wave moves --------------------

    /// The merge's successor fixture digest.
    pub const MERGE_SUCCESSOR_DIGEST: &str =
        "e928537da63bae600cb1d8982dbd671b1c24ad4df71a697ba643492dad3c82fe";

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

    // --- The fee-bearing shape: a refusal, and NOT an acceptance -------

    /// The fee-bearing one-to-one's successor fixture digest.
    ///
    /// The fixture registered, derived and PROJECTED. Recording the digest
    /// is recording that much and no more.
    pub const FEE_BEARING_SUCCESSOR_DIGEST: &str =
        "d08a306819cc5ad713b393ecd956f2b5b38c7eb46de20069e8780620d79ed0cf";

    /// No identity is minted for the fee-bearing shape, and this constant
    /// exists to say so in the module acceptances are cited from.
    ///
    /// The target ACCEPTED NOTHING. Filing a non-acceptance among the
    /// identities would be the one error a run of record exists to
    /// prevent, so the shape's evidence is its own reproducible bytes and
    /// the verdict the target returned, both recorded below.
    pub const FEE_BEARING_ACCEPTED_IDENTITY: Option<&str> = None;

    /// How many bytes the fee-bearing one-to-one handed to the node.
    pub const FEE_BEARING_SUBMITTED_BYTES: usize = 4_870;

    /// The output-witness entries the fee-bearing candidate carried.
    ///
    /// This array is the fee projection's own evidence, and it is the
    /// reason the run is worth recording despite the refusal. The blinded
    /// output carries a range proof of the usual size; the FEE output
    /// carries an empty entry. A fee that had been mapped onto the
    /// balancing role would read `[4_174, 4_174]` here — a blinded fee,
    /// and not a fee at all.
    pub const FEE_BEARING_PROOF_BYTES: [usize; 2] = [4_174, 0];

    /// The verdict the target returned, verbatim and unmapped.
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
    /// shape vocabulary reads a two-destination sponsorless shape as TWO
    /// RECEIPT OUTPUTS, and the receipt covenant therefore requires the
    /// second output to carry the second owner's private receipt
    /// constructor program. The second output is the fee, whose program is
    /// empty, so the comparison fails.
    pub const FEE_BEARING_OBSERVED_DETAIL: &str =
        "mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)";

    /// The fee-bearing run's wall time, in seconds.
    pub const FEE_BEARING_WALL_SECONDS: f64 = 12.4;
}
