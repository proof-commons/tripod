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
//! [`crate::live_proof_bearing_observation::register_multi`], the
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
//! # The shapes it does not build, and why
//!
//! A private-merge is one output, and the registry refuses a manifest with
//! fewer than two. That is not an unbuilt fixture but an unconstructible
//! one on this lane, and the guide's own merge predicate — inputs at least
//! two and outputs exactly one — is mutually unsatisfiable with the
//! registry's two-output floor. The merge row stays unmoved and typed, and
//! the conflict is filed as a guide erratum rather than resolved by
//! relaxing either rule.

use std::collections::BTreeMap;

use linker::OwnerParameter;
use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements_conformance::confidential_fixture::predecessor_handle;
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

use crate::confidential_materializer::{
    FirstPartyCommitmentCheck, ReferenceConfidentialMaterializer,
};
use crate::confidential_predecessor::PREDECESSOR_AMOUNTS;
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
}

/// The remaining positive private shapes of §15.2 this wave builds.
///
/// Each is a concrete representative case, not a family: the matrix row it
/// moves is moved on an acceptance of THIS shape, and a representative is
/// named as one rather than presented as a proof over every shape of its
/// class.
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
}

impl PrivateShape {
    /// All three, in the order the restart runs them.
    pub const ALL: [Self; 3] = [Self::Split, Self::ManyToMany, Self::SeveralDistinctOwners];

    /// The ceremony's own name for the shape, used as the report
    /// extension and the successor fixture handle's discriminator.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Split => "private-split",
            Self::ManyToMany => "private-many-to-many",
            Self::SeveralDistinctOwners => "private-several-distinct-owners",
        }
    }

    /// The §15.2 safety-matrix row this shape's acceptance moves.
    #[must_use]
    pub const fn row_name(self) -> &'static str {
        match self {
            Self::Split => "private-split",
            Self::ManyToMany => "private-many-to-many-representative",
            Self::SeveralDistinctOwners => "private-several-distinct-owners",
        }
    }

    /// The successor fixture's handle, its own per shape so a digest drift
    /// between two shapes is detectable.
    #[must_use]
    fn successor_handle(self) -> String {
        format!("ctf-v1/wave-seven-{}-successor", self.name())
    }

    /// Which predecessor outputs this shape consumes, in fixed order.
    #[must_use]
    const fn consumed(self) -> &'static [ConsumedReceipt] {
        match self {
            Self::Split => &[ConsumedReceipt::Primary],
            Self::ManyToMany | Self::SeveralDistinctOwners => {
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
        let d = |scalar, amount| Destination { scalar, amount };
        match self {
            // 700_000_000 in, split three ways back to the two owners.
            Self::Split => vec![
                d(SECOND_SCALAR, 400_000_000),
                d(FIRST_SCALAR, 200_000_000),
                d(FIRST_SCALAR, 100_000_000),
            ],
            // 1_000_000_000 in across two receipts, three ways out.
            Self::ManyToMany => vec![
                d(SECOND_SCALAR, 500_000_000),
                d(FIRST_SCALAR, 300_000_000),
                d(SECOND_SCALAR, 200_000_000),
            ],
            // 1_000_000_000 in across two distinctly owned receipts, two
            // ways out.
            Self::SeveralDistinctOwners => {
                vec![d(SECOND_SCALAR, 600_000_000), d(FIRST_SCALAR, 400_000_000)]
            }
        }
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
        let linked = link_and_register(ConsumedReceipt::Primary, printed)?;
        self.record.issued_asset = Some(printed.to_owned());
        self.record.predecessor_digest = Some(linked.predecessor_digest);

        let successor = self.register_successor(&linked)?;
        self.record.successor_digest = Some(successor.digest);
        self.successor = Some(successor);
        self.linked = Some(linked);
        Ok(())
    }

    /// Register this shape's successor: one output per destination, the
    /// last balancing, its input blinder sum the sum of the consumed
    /// coins' own blinders.
    fn register_successor(
        &self,
        linked: &LinkedDeployment,
    ) -> Result<Successor, PrivateRestartRefusal> {
        let destinations = self.shape.destinations();
        let programs: Vec<Vec<u8>> = destinations
            .iter()
            .map(|destination| private_program(&linked.abi, &destination.scalar))
            .collect::<Result<_, _>>()?;
        let amounts: Vec<u64> = destinations
            .iter()
            .map(|destination| destination.amount)
            .collect();

        let handle = self.shape.successor_handle();
        let (digest, view) = register_multi(
            &handle,
            *linked.asset.internal(),
            self.input_blinder_sum(linked),
            &amounts,
            &programs,
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
    /// A one-input shape balances against that one coin's blinder. A
    /// two-input shape consumes both predecessor outputs, whose blinders
    /// are ordered additive inverses because the predecessor was
    /// registered with a zero input blinder sum, so their sum is the zero
    /// scalar. This is the term a two-input successor does not get for
    /// free and a one-input successor does; it is stated from the
    /// predecessor's structure rather than recomputed with a bignum this
    /// crate does not carry.
    #[must_use]
    fn input_blinder_sum(&self, linked: &LinkedDeployment) -> [u8; 32] {
        match self.shape.consumed() {
            [single] => linked
                .predecessor_view
                .outputs()
                .get(single.index())
                .map_or([0_u8; 32], |output| *output.value_blinder()),
            _ => [0_u8; 32],
        }
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
                    predecessor_handle().as_str().to_owned(),
                    linked.predecessor_digest,
                    receipt.index(),
                ),
                explicit_amount: PREDECESSOR_AMOUNTS[receipt.index()],
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

        let last = destinations.len().saturating_sub(1);
        let destination_openings: Vec<PrivateDestinationOpening> = (0..destinations.len())
            .map(|index| PrivateDestinationOpening {
                fixture: FixtureOpeningReference::new(
                    self.shape.successor_handle(),
                    successor.digest,
                    index,
                ),
                role: if index == last {
                    ConfidentialOutputRole::Balancing
                } else {
                    ConfidentialOutputRole::Primary
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
                predecessor_handle().as_str().to_owned(),
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
mod tests {
    use target_elements_conformance::confidential_fixture::RegistrationRefusal;

    use crate::live_proof_bearing_observation::{register_multi, registry_refusal_for};

    /// A disposable asset for the offline registration checks. Public
    /// test material under ADR-015; it names no chain.
    const ASSET: [u8; 32] = [0x3b; 32];

    /// The zero input blinder sum a two-input transfer's consumed coins
    /// sum to, and a valid scalar the registry admits.
    const ZERO_SUM: [u8; 32] = [0_u8; 32];

    /// The multi-output constructor the step-five stop said was absent
    /// now builds a three-output successor: the split shape's fixture,
    /// with two primary recipients and one balancing change output.
    #[test]
    fn the_multi_output_constructor_builds_a_three_output_successor() {
        let (_digest, view) = register_multi(
            "ctf-v1/test-three-output",
            ASSET,
            ZERO_SUM,
            &[400_000_000, 200_000_000, 100_000_000],
            &[vec![0x51], vec![0x52], vec![0x53]],
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
            &[600_000_000, 400_000_000],
            &[vec![0x51], vec![0x52]],
        )
        .expect("the two-output successor registers");
        assert_eq!(view.outputs().len(), 2);
    }

    /// A one-output shape — private-merge, the strict one-to-one, and the
    /// fee-only case alike — hits the registry's two-output floor. This is
    /// the same wall for all three: the registry counts outputs before it
    /// inspects any of them, so a single output is refused
    /// `OutputSetTooSmall` whatever that output is.
    #[test]
    fn a_one_output_shape_hits_the_registry_floor() {
        let refusal = registry_refusal_for(
            "ctf-v1/test-one-output",
            ASSET,
            ZERO_SUM,
            &[700_000_000],
            &[vec![0x51]],
        )
        .expect("a one-output manifest is refused");
        assert_eq!(
            refusal,
            RegistrationRefusal::OutputSetTooSmall { found: 1 },
            "the one-output floor is the merge wall, and it is a cardinality wall",
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
                &[400_000_000, 200_000_000, 100_000_000],
                &[vec![0x51], vec![0x52], vec![0x53]],
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

    /// The fee-only case is unconstructible a second, deeper way than the
    /// cardinality floor: a fee output carries an empty scriptPubKey, and
    /// the confidential fixture vocabulary has no fee role and refuses an
    /// empty output program. Even were the two-output floor relaxed, a fee
    /// output is inexpressible here — shown by a two-output probe whose
    /// balancing output has an empty program, which the registry refuses
    /// `OutputProgramEmpty` rather than admitting.
    #[test]
    fn a_fee_output_is_inexpressible_beyond_the_cardinality_floor() {
        let refusal = registry_refusal_for(
            "ctf-v1/test-fee-output",
            ASSET,
            ZERO_SUM,
            &[500_000_000, 500_000_000],
            &[vec![0x51], vec![]],
        )
        .expect("a fee-shaped empty-program output is refused");
        assert_eq!(
            refusal,
            RegistrationRefusal::OutputProgramEmpty { output: 1 },
            "the fee output's empty program is refused independently of cardinality",
        );
    }
}
