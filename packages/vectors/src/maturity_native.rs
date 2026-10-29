//! A three-step operation plan for the sponsorless maturity announcement.
//!
//! Issuance first discovers the singleton identifier, so its bootstrap destination cannot be the predecessor program linked for that still-unknown identifier. The returned asset is linked through the public constructor before a second funding step pays the exact predecessor program; only that response supplies the outpoint the announcement spends.
//!
//! The declaration belongs to the composed schedule: the archived whole-metadata run declares a relay-policy refusal, while the variable-metadata schedule declares acceptance. Funding has no matrix row and therefore no declared layer, although construction requires accepted funding. Transcript evidence replays the planner before comparing each observation with its declaration.
//!
//! Established acceptance binds the accepted identity, mined readback and submitted bytes under the replayed schedule. A scripted response can establish these bindings, so the value authenticates no executor, proves no current-root freshness and promotes no row standing.

use architecture::ARCHITECTURE;
use linker::{
    CandidateDeploymentIdentity, CandidateLinkedMaturityBundle, OperatorDeploymentBinding,
    StateLinkDeploymentParameters, StateLinkSources, StateSingletonAsset, link_state_candidate,
};
use realization::{Cycle, RealizationError, StateSingletonDeclaration};
use tapscript::{StackItem, TapscriptError};
use target_elements::EncodingClass;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    FundedOutput, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse, ObservedOutcomeLayer,
    OperationCaseId, OperationSubject, ResponseShapeDefect, TargetFundingSubject,
    TargetSubmissionSubject,
};
use target_elements_conformance::test_material::TestSigningDefect;
use transaction::bytes::{AssetField, AssetId, Outpoint, TargetTransaction, Txid, ValueField};
use transaction::error::TransactionRefusal;
use transaction::live_request::{RequestedForm, SponsorChangeRequest};
use transaction::operator_right::{BranchContext, OperatorRightRegistry, RightRefusal};
use transaction::operator_signing::{OPERATOR_SIGHASH_TYPE_BYTE, OperatorSigningResponse};
use transaction::state_abi::derive_maturity_announcement_abi;
#[cfg(test)]
use transaction::state_construct::construct_maturity_announcement;
use transaction::state_finalize::{FinalizedMaturityAnnouncement, finalize_maturity_announcement};
use transaction::state_request::MaturityAnnouncementRequest;
use transaction::state_signing::OperatorSigningStarted;
use transaction::state_view::{MaturityViewStatement, PublicMaturityStateView};

use crate::live_capability::OracleLiveCurve;
use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of};
use crate::maturity_closure::{
    MaturityClosureRefusal, MaturityDeployment, MaturityWitnessSelection, OracleStateCurve,
    closure_target, decode_announcement_leaf, linked_announcement_bytes, maturity_sources,
};
use crate::maturity_mutants::MaturityMutantStageRefusal;
use crate::maturity_operator::{OPERATOR_HANDLE, OperatorVerifier};
pub use crate::observed_boundary::{matches_boundary, observed_boundary};

/// The funding exchanges followed by the exact signed announcement.
pub const ANNOUNCEMENT_STEPS: [&str; 3] = [
    "issue-maturity-singleton",
    "fund-maturity-predecessor",
    "sponsorless",
];

/// The matrix boundary declared before the submission is executed.
///
/// Funding is infrastructure rather than a matrix row; settlement still requires acceptance.
#[must_use]
pub fn expected_layer(
    subject: &str,
    schedule: tapscript::StateWitnessSchedule,
) -> Option<ObservedOutcomeLayer> {
    match (subject, schedule) {
        ("sponsorless", tapscript::StateWitnessSchedule::WholeMetadata) => {
            Some(ObservedOutcomeLayer::RelayPolicyRejection)
        }
        ("sponsorless", tapscript::StateWitnessSchedule::VariableMetadata) => {
            Some(ObservedOutcomeLayer::Accepted)
        }
        _ => None,
    }
}

/// A construction refusal retained whole, or an exchange that cannot advance this plan.
///
/// The executor receives only its marker; callers read the operation's reason here. Funding disagreements have no compact-generation vector identity, and a missing response is not a failure to build a live-transfer substrate.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MaturityNativePlanRefusal {
    /// The public closure or linker refused, in its own vocabulary.
    Closure(Box<MaturityClosureRefusal>),
    /// The public transaction construction or authorization refused.
    Transaction(Box<TransactionRefusal>),
    /// The architecture's singleton declaration refused.
    Realization(Box<RealizationError>),
    /// The internal-key stack item did not meet the reviewed encoding.
    Tapscript(Box<TapscriptError>),
    /// The published signing material refused to produce a signature.
    Signing(TestSigningDefect),
    /// An exact mutant could not be staged for a native offer.
    MutantStage(Box<MaturityMutantStageRefusal>),
    /// The construction-right registry refused the frozen candidate.
    ConstructionRight(Box<RightRefusal>),
    /// A response was missing, unsolicited, or named a different pending case.
    UnexpectedResponse,
    /// The response used another protocol revision.
    ResponseSchema {
        /// The revision the response carried.
        offered: u32,
    },
    /// The protocol's own shape validator refused the response.
    ResponseShape(ResponseShapeDefect),
    /// An infrastructure funding step did not establish an accepted coin.
    FundingNotAccepted {
        /// The actual observation, retained without reinterpretation.
        observed: ObservedOutcomeLayer,
    },
    /// Issuance did not return a decodable asset identifier.
    IssuedAssetMissingOrInvalid,
    /// A funding response did not carry exactly one singleton output.
    FundingCardinality {
        /// The number actually reported.
        supplied: usize,
    },
    /// A funded coin disagreed with the requested asset, amount, program or issuance role.
    FundingMismatch,
    /// The reported coin did not name a decodable spendable outpoint.
    FundedOutpointInvalid,
    /// An accepted submission's identity or readback did not describe the submitted bytes.
    SubmissionReadbackMismatch,
    /// Completion was requested before all three exchanges settled.
    IncompleteTranscript,
    /// A recorded request differed from the exact next request reconstructed by replay.
    TranscriptStepMismatch {
        /// The zero-based position of the differing or surplus request.
        position: usize,
    },
    /// The sponsored policy detail was absent, ambiguous or contradicted its provenance.
    SponsoredPolicy,
    /// The observation did not settle a target verdict or its requested route.
    SponsoredUndecided,
    /// A signing response did not bind the exact request and input.
    SponsoredSignature,
    /// The paid fee did not meet the measured-size rule.
    SponsoredFee,
    /// A sponsored positive did not use relay and ordinary block selection.
    SponsoredRoute,
    /// A submitted subject did not reach its declared boundary.
    SponsoredBoundary,
    /// The bounded semantic selection did not produce the required least nonce.
    SponsoredNonceSelection,
    /// Constructor or semantic projection refused the submitted bytes.
    SponsoredProjection(Box<crate::maturity_continuity::MaturityContinuityRefusal>),
    /// Public recovery refused the same submitted bytes.
    SponsoredRecovery(Box<crate::maturity_recovery::MaturityRecoveryRefusal>),
}

type Refusal = MaturityNativePlanRefusal;

/// One settled exchange with its declaration retained beside its observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityNativeObservation {
    subject: &'static str,
    declared_layer: Option<ObservedOutcomeLayer>,
    observed_layer: ObservedOutcomeLayer,
}

impl MaturityNativeObservation {
    /// The planner's subject name, in execution order.
    #[must_use]
    pub const fn subject(&self) -> &'static str {
        self.subject
    }

    /// The layer declared before execution, absent for infrastructure funding.
    #[must_use]
    pub const fn declared_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.declared_layer
    }

    /// The response's observation, preserved even when it contradicts the declaration.
    #[must_use]
    pub const fn observed_layer(&self) -> ObservedOutcomeLayer {
        self.observed_layer
    }
}

/// The sponsorless row's comparison, without an executor-trust or acceptance claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityNativeStanding {
    /// The recorded refusal reached the row's declared relay-policy boundary.
    AnsweredAtDeclaredBoundary,
    /// The recorded layer differs; the observation remains available unchanged.
    ObservedElsewhere,
}

/// A route to the accepted sponsorless positive control still required by §23.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityAcceptanceRoute {
    /// Reopen the witness design to fit the deployed relay policy.
    RelayWitnessRestructure,
    /// Add a block-layer submission subject through a protocol revision.
    BlockLayerSubmissionSubject,
    /// Relay accepted and ordinary mining selected the transaction.
    RelayAndTemplate,
    /// Relay refused; explicit block inclusion accepted the same bytes.
    DirectBlockAfterRelayRefusal,
}

/// A checked accepted submission and mined readback retained by the planner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityAcceptedReadback {
    identity: Txid,
    witness_identity: Txid,
    block_hash: [u8; 32],
    block_height: u32,
    bytes: Vec<u8>,
}

impl MaturityAcceptedReadback {
    /// The identity computed from the submitted transaction without witness.
    #[must_use]
    pub const fn identity(&self) -> Txid {
        self.identity
    }

    /// The identity computed from the complete submitted transaction.
    #[must_use]
    pub const fn witness_identity(&self) -> Txid {
        self.witness_identity
    }

    /// The mined block hash in the target's displayed byte order.
    #[must_use]
    pub const fn block_hash(&self) -> &[u8; 32] {
        &self.block_hash
    }

    /// The height reported by the mined readback.
    #[must_use]
    pub const fn block_height(&self) -> u32 {
        self.block_height
    }

    /// The exact submitted bytes also reported by the mined readback.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
impl MaturityAcceptedReadback {
    /// Assemble a readback for mismatch tests. Production readbacks are constructed only by submission checking, so an established production claim remains traceable to a transcript or admitted archive.
    #[must_use]
    pub(crate) const fn from_parts(
        identity: Txid,
        witness_identity: Txid,
        block_hash: [u8; 32],
        block_height: u32,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            identity,
            witness_identity,
            block_hash,
            block_height,
            bytes,
        }
    }
}

/// The sponsorless acceptance obligation derived from exact transcript replay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityAcceptanceObligation {
    /// Neither a relay refusal nor an off-declaration observation closes this obligation.
    Outstanding {
        /// The two remaining routes, rather than a relaxed-policy acceptance claim.
        routes: [MaturityAcceptanceRoute; 2],
    },
    /// Every accepted identity, readback and submitted-byte binding held under this schedule.
    Established {
        /// The schedule reconstructed by replay.
        schedule: tapscript::StateWitnessSchedule,
        /// The accepted identity computed from the submitted transaction.
        identity: Txid,
        /// The checked mined readback and submitted bytes.
        readback: MaturityAcceptedReadback,
    },
}

/// Settled transcript observations compared with the planner's prior declaration.
///
/// This value establishes transcript consistency under the retained schedule, not provenance or current-root freshness. A scripted response can establish acceptance; a native-run claim additionally needs executor and capture provenance. Standing remains the declared-versus-observed comparison.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityNativeEvidence {
    schedule: tapscript::StateWitnessSchedule,
    identity: CandidateDeploymentIdentity,
    branch: BranchContext,
    observations: Vec<MaturityNativeObservation>,
    standing: MaturityNativeStanding,
    acceptance_obligation: MaturityAcceptanceObligation,
}

impl MaturityNativeEvidence {
    /// Reconstructs each exact request and settles every response before deriving standing.
    ///
    /// The deployment and branch are caller-supplied replay context, not observations inferred from funding. The branch value establishes no current-root freshness.
    ///
    /// # Errors
    /// Preserves the planner's construction and response refusals; returns `TranscriptStepMismatch` for an altered or surplus request and `IncompleteTranscript` for missing exchanges.
    ///
    /// # Panics
    /// Panics only if the fixed architecture omits its singleton asset or a linked bundle retains no constructor, which the published architecture and public linker cannot arrange.
    #[must_use = "transcript replay can refuse the selected schedule or exchanges"]
    pub fn from_transcript(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        selection: MaturityWitnessSelection,
        exchanges: &[(OperationStep, NativeOperationResponse)],
    ) -> Result<Self, MaturityNativePlanRefusal> {
        let mut planner = MaturityAnnouncementPlanner::new(identity.clone(), branch, selection)?;
        let mut next = replay_next(&mut planner, None)?;
        for (position, (step, response)) in exchanges.iter().enumerate() {
            if next.as_ref() != Some(step) {
                return Err(Refusal::TranscriptStepMismatch { position });
            }
            next = replay_next(&mut planner, Some((step.case(), response)))?;
        }
        let settled = planner.completed_transcript()?;
        let observations = settled
            .iter()
            .zip(ANNOUNCEMENT_STEPS)
            .map(|((_, response), subject)| MaturityNativeObservation {
                subject,
                declared_layer: expected_layer(subject, planner.schedule()),
                observed_layer: response.observed_layer,
            })
            .collect();
        let (_, submission) = settled.last().ok_or(Refusal::IncompleteTranscript)?;
        let answered = expected_layer("sponsorless", planner.schedule())
            .and_then(crate::observed_boundary::observed_boundary)
            .is_some_and(|boundary| {
                crate::observed_boundary::matches_boundary(boundary, submission.observed_layer)
            });
        let acceptance_obligation = if planner.schedule()
            == tapscript::StateWitnessSchedule::VariableMetadata
            && submission.observed_layer == ObservedOutcomeLayer::Accepted
        {
            let readback = planner
                .readback()
                .cloned()
                .ok_or(Refusal::SubmissionReadbackMismatch)?;
            MaturityAcceptanceObligation::Established {
                schedule: planner.schedule(),
                identity: readback.identity(),
                readback,
            }
        } else {
            MaturityAcceptanceObligation::Outstanding {
                routes: [
                    MaturityAcceptanceRoute::RelayWitnessRestructure,
                    MaturityAcceptanceRoute::BlockLayerSubmissionSubject,
                ],
            }
        };
        Ok(Self {
            schedule: planner.schedule(),
            identity,
            branch,
            observations,
            standing: if answered {
                MaturityNativeStanding::AnsweredAtDeclaredBoundary
            } else {
                MaturityNativeStanding::ObservedElsewhere
            },
            acceptance_obligation,
        })
    }

    /// The composed schedule used to reconstruct every recorded request.
    #[must_use]
    pub const fn schedule(&self) -> tapscript::StateWitnessSchedule {
        self.schedule
    }

    /// The deployment supplied for exact replay.
    #[must_use]
    pub const fn identity(&self) -> &CandidateDeploymentIdentity {
        &self.identity
    }

    /// The caller-supplied branch and checkpoint, with no freshness claim.
    #[must_use]
    pub const fn branch(&self) -> BranchContext {
        self.branch
    }

    /// Every settled step's declaration and observation, in execution order.
    #[must_use]
    pub fn observations(&self) -> &[MaturityNativeObservation] {
        &self.observations
    }

    /// Whether the sponsorless observation answered its declared boundary.
    #[must_use]
    pub const fn standing(&self) -> MaturityNativeStanding {
        self.standing
    }

    /// The accepted binding or its outstanding routes.
    #[must_use]
    pub const fn acceptance_obligation(&self) -> &MaturityAcceptanceObligation {
        &self.acceptance_obligation
    }
}

fn replay_next(
    planner: &mut MaturityAnnouncementPlanner,
    previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
) -> Result<Option<OperationStep>, Refusal> {
    planner.next_step(previous).map_err(|PlanRefused| {
        planner
            .refusal()
            .cloned()
            .unwrap_or(Refusal::UnexpectedResponse)
    })
}

fn closure(refusal: MaturityClosureRefusal) -> Refusal {
    Refusal::Closure(Box::new(refusal))
}

fn transaction(refusal: TransactionRefusal) -> Refusal {
    Refusal::Transaction(Box::new(refusal))
}

fn link_refusal(refusal: linker::LinkRefusal) -> Refusal {
    closure(MaturityClosureRefusal::LinkRefused(refusal))
}

/// The announcement cycle selected from the predecessor's inclusive lead window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityLead {
    /// The earliest admissible announcement cycle.
    Minimum,
    /// One cycle after the earliest, capped at the latest cycle.
    Interior,
    /// The latest admissible announcement cycle.
    Maximum,
}

impl MaturityLead {
    /// Choose the cycle this lead announces within the inclusive window.
    #[must_use]
    pub fn announced(self, window: (Cycle, Cycle)) -> Cycle {
        let (earliest, latest) = window;
        match self {
            Self::Minimum => earliest,
            Self::Interior => Cycle::new(earliest.get().saturating_add(1)).min(latest),
            Self::Maximum => latest,
        }
    }
}

/// One run retaining the exact linked subject and every settled exchange.
pub struct MaturityAnnouncementPlanner {
    schedule: tapscript::StateWitnessSchedule,
    lead: MaturityLead,
    identity: CandidateDeploymentIdentity,
    branch: BranchContext,
    position: usize,
    pending: Option<OperationStep>,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    bundle: CandidateLinkedMaturityBundle,
    amount: u64,
    asset: Option<String>,
    predecessor: Option<Outpoint>,
    announcement: Option<FinalizedMaturityAnnouncement>,
    submission: Option<Vec<u8>>,
    readback: Option<MaturityAcceptedReadback>,
    refusal: Option<Refusal>,
}

impl MaturityAnnouncementPlanner {
    /// Starts the published-signer ceremony under the caller's deployment and branch.
    ///
    /// The branch and checkpoint are caller-supplied context. Neither this constructor nor the later funding observation establishes current-root freshness; the native-run caller supplies that external context.
    ///
    /// # Errors
    /// Preserves refusals from the public closure, deployment binding and singleton declaration.
    ///
    /// # Panics
    /// Panics only if the fixed architecture omits its singleton asset or a linked bundle retains no constructor, which the published architecture and public linker cannot arrange.
    #[must_use = "planner construction can refuse the selected schedule or deployment"]
    pub fn new(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        selection: MaturityWitnessSelection,
    ) -> Result<Self, MaturityNativePlanRefusal> {
        Self::at_lead(identity, branch, selection, MaturityLead::Interior)
    }

    /// Starts the published-signer ceremony at a selected admissible lead.
    ///
    /// # Errors
    /// Preserves refusals from the public closure, deployment binding and singleton declaration.
    ///
    #[must_use = "planner construction can refuse the selected schedule or deployment"]
    pub fn at_lead(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        selection: MaturityWitnessSelection,
        lead: MaturityLead,
    ) -> Result<Self, MaturityNativePlanRefusal> {
        let parameters = MaturityDeployment::PublishedSignerHeld
            .parameters()
            .map_err(closure)?;
        let declaration = singleton_declaration()?;
        let (bundle, schedule) = link_for(
            &identity,
            AssetId::from_internal(*parameters.singleton()),
            selection,
        )?;
        Ok(Self {
            schedule,
            lead,
            identity,
            branch,
            position: 0,
            pending: None,
            exchanges: Vec::new(),
            bundle,
            amount: declaration.fixed_amount().get(),
            asset: None,
            predecessor: None,
            announcement: None,
            submission: None,
            readback: None,
            refusal: None,
        })
    }

    /// The schedule retained by the composed record used for this run.
    #[must_use]
    pub const fn schedule(&self) -> tapscript::StateWitnessSchedule {
        self.schedule
    }

    /// The bootstrap link, replaced by the runtime-singleton link after issuance.
    #[must_use]
    pub const fn bundle(&self) -> &CandidateLinkedMaturityBundle {
        &self.bundle
    }

    /// The predecessor outpoint supplied by the accepted runtime funding response.
    #[must_use]
    pub const fn predecessor(&self) -> Option<Outpoint> {
        self.predecessor
    }

    /// The finalized announcement once the predecessor funding has settled.
    #[must_use]
    pub const fn announcement(&self) -> Option<&FinalizedMaturityAnnouncement> {
        self.announcement.as_ref()
    }

    /// The authorized candidate serialization, including the STATE witness.
    #[must_use]
    pub fn submission_bytes(&self) -> Option<&[u8]> {
        self.submission.as_deref()
    }

    /// The accepted identity and mined readback after a checked submission.
    #[must_use]
    pub const fn readback(&self) -> Option<&MaturityAcceptedReadback> {
        self.readback.as_ref()
    }

    /// The first reason this single-run planner stopped, unchanged on later calls.
    #[must_use]
    pub const fn refusal(&self) -> Option<&MaturityNativePlanRefusal> {
        self.refusal.as_ref()
    }

    /// The ordered subjects and responses for the subsequent native-run observation row.
    ///
    /// Completion means that all exchanges settled, not that their observed layer matched the declaration. The native-run consumer compares that layer and retains the executor's own provenance; this reader manufactures neither a standing nor executor trust.
    ///
    /// # Errors
    /// Returns the retained refusal after a failed run, or `IncompleteTranscript` before completion.
    pub fn completed_transcript(
        &self,
    ) -> Result<&[(OperationStep, NativeOperationResponse)], MaturityNativePlanRefusal> {
        if let Some(refusal) = &self.refusal {
            return Err(refusal.clone());
        }
        if self.position != ANNOUNCEMENT_STEPS.len() || self.pending.is_some() {
            return Err(Refusal::IncompleteTranscript);
        }
        Ok(&self.exchanges)
    }

    fn make_step(&self) -> Result<Option<OperationStep>, Refusal> {
        let Some(name) = ANNOUNCEMENT_STEPS.get(self.position) else {
            return Ok(None);
        };
        let subject = if self.position < 2 {
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: self.position == 0,
                asset: self.asset.clone(),
                output_program: predecessor_program(&self.bundle),
                outputs: 1,
                amount_per_output: self.amount,
            }))
        } else {
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: self
                    .submission
                    .clone()
                    .ok_or(Refusal::IncompleteTranscript)?,
            }))
        };
        Ok(Some(OperationStep::new(name, subject)))
    }

    fn advance(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, Refusal> {
        match (self.pending.take(), previous) {
            (Some(step), Some((case, response)))
                if step.case() == case && &response.case == case =>
            {
                if response.schema != NATIVE_PROTOCOL_SCHEMA {
                    return Err(Refusal::ResponseSchema {
                        offered: response.schema,
                    });
                }
                response.validate_shape().map_err(Refusal::ResponseShape)?;
                self.exchanges.push((step.clone(), response.clone()));
                match step.subject() {
                    OperationSubject::Funding(subject) => self.settle_funding(subject, response)?,
                    OperationSubject::Submission(_) => {
                        self.readback = self.check_submission(response)?;
                    }
                    _ => return Err(Refusal::UnexpectedResponse),
                }
                self.position += 1;
            }
            (None, None) if self.position == 0 || self.position == ANNOUNCEMENT_STEPS.len() => {}
            _ => return Err(Refusal::UnexpectedResponse),
        }
        let step = self.make_step()?;
        self.pending.clone_from(&step);
        Ok(step)
    }

    fn settle_funding(
        &mut self,
        subject: &TargetFundingSubject,
        response: &NativeOperationResponse,
    ) -> Result<(), Refusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(Refusal::FundingNotAccepted {
                observed: response.observed_layer,
            });
        }
        let asset = if subject.issue_asset {
            response
                .issued_asset
                .as_deref()
                .filter(|value| asset_of(value).is_some())
                .ok_or(Refusal::IssuedAssetMissingOrInvalid)?
        } else {
            if response.issued_asset.is_some() {
                return Err(Refusal::FundingMismatch);
            }
            subject.asset.as_deref().ok_or(Refusal::FundingMismatch)?
        };
        let [coin] = response.funded_outputs.as_slice() else {
            return Err(Refusal::FundingCardinality {
                supplied: response.funded_outputs.len(),
            });
        };
        let outpoint = check_coin(subject, coin, asset)?;
        let decoded_asset = asset_of(asset).ok_or(Refusal::IssuedAssetMissingOrInvalid)?;
        if subject.issue_asset {
            // Admission is applied at construction; relinking reconstructs the
            // schedule that admission retained rather than making a second choice.
            (self.bundle, self.schedule) = link_for(
                &self.identity,
                decoded_asset,
                MaturityWitnessSelection::Retained(self.schedule),
            )?;
            self.asset = Some(asset.to_owned());
        } else {
            let finalized = build_announcement(
                &self.bundle,
                decoded_asset,
                outpoint,
                self.branch,
                self.amount,
                self.lead,
            )?;
            self.submission = Some(sign_announcement(&finalized)?);
            self.announcement = Some(finalized);
            self.predecessor = Some(outpoint);
        }
        Ok(())
    }

    fn check_submission(
        &self,
        response: &NativeOperationResponse,
    ) -> Result<Option<MaturityAcceptedReadback>, Refusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Ok(None);
        }
        let bytes = self
            .submission
            .as_ref()
            .ok_or(Refusal::IncompleteTranscript)?;
        let readback = response
            .mined_readback
            .as_ref()
            .ok_or(Refusal::SubmissionReadbackMismatch)?;
        let candidate = TargetTransaction::decode(bytes).map_err(transaction)?;
        let identity = Txid::from_internal(sha256(&sha256(&candidate.encode_without_witness())));
        let witness_identity = Txid::from_internal(sha256(&sha256(bytes)));
        if response
            .accepted_txid
            .as_deref()
            .and_then(|value| Txid::from_target_display(value).ok())
            != Some(identity)
            || readback.transaction_id != identity.to_target_display()
            || readback.raw_transaction.as_slice() != bytes
            || readback.witness_transaction_id != witness_identity.to_target_display()
        {
            return Err(Refusal::SubmissionReadbackMismatch);
        }
        let block_hash = decode_hex(&readback.block_hash)
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or(Refusal::SubmissionReadbackMismatch)?;
        Ok(Some(MaturityAcceptedReadback {
            identity,
            witness_identity,
            block_hash,
            block_height: readback.block_height,
            bytes: bytes.clone(),
        }))
    }
}

impl TargetOperationPlanner for MaturityAnnouncementPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if self.refusal.is_some() {
            return Err(PlanRefused);
        }
        self.advance(previous).map_err(|refusal| {
            self.refusal = Some(refusal);
            self.pending = None;
            PlanRefused
        })
    }
}

fn singleton_declaration() -> Result<StateSingletonDeclaration, Refusal> {
    let specification = ARCHITECTURE
        .asset(architecture::AssetId::Pid)
        .expect("the fixed architecture declares the STATE singleton");
    StateSingletonDeclaration::from_architecture_asset(specification)
        .map_err(|refusal| Refusal::Realization(Box::new(refusal)))
}

fn predecessor_program(bundle: &CandidateLinkedMaturityBundle) -> Vec<u8> {
    bundle
        .instances()
        .first()
        .expect("the linker retains its predecessor constructor")
        .constructor()
        .output_program()
}

fn link_for(
    identity: &CandidateDeploymentIdentity,
    asset: AssetId,
    selection: MaturityWitnessSelection,
) -> Result<
    (
        CandidateLinkedMaturityBundle,
        tapscript::StateWitnessSchedule,
    ),
    Refusal,
> {
    link_for_metadata(identity, asset, selection, None)
}

fn link_for_metadata(
    identity: &CandidateDeploymentIdentity,
    asset: AssetId,
    selection: MaturityWitnessSelection,
    metadata: Option<realization::StateMetadata>,
) -> Result<
    (
        CandidateLinkedMaturityBundle,
        tapscript::StateWitnessSchedule,
    ),
    Refusal,
> {
    let target = closure_target().map_err(closure)?;
    let sources =
        maturity_sources(MaturityDeployment::PublishedSignerHeld, selection).map_err(closure)?;
    let seed = sources.link(&OracleStateCurve).map_err(closure)?;
    let deployment = seed.deployment();
    let internal = StackItem::encoded(
        &target,
        EncodingClass::XOnlyPublicKey,
        seed.policy().internal_key().key().to_vec(),
    )
    .map_err(|refusal| Refusal::Tapscript(Box::new(refusal)))?;
    let operator = OperatorDeploymentBinding::bind(
        &target,
        deployment.operator().key().clone(),
        deployment.operator().profile().clone(),
        identity.clone(),
        &internal,
    )
    .map_err(link_refusal)?;
    let bridge = StateLinkDeploymentParameters::bind(
        &target,
        seed.plan().clone(),
        deployment.lead_bounds(),
        identity.clone(),
        operator,
        deployment.maximum_control_path_depth(),
        sources.record(),
    )
    .map_err(link_refusal)?;
    let singleton = StateSingletonAsset::new(*asset.internal());
    let declaration = singleton_declaration()?;
    let metadata = metadata.unwrap_or_else(|| {
        seed.instances()
            .first()
            .expect("the linker retains its predecessor constructor")
            .metadata()
            .semantic
    });
    let linked = link_state_candidate(
        &target,
        &StateLinkSources::new(
            sources.record(),
            &bridge,
            sources.constructor(),
            &singleton,
            &declaration,
            &metadata,
            &OracleStateCurve,
        ),
    )
    .map_err(link_refusal)?;
    let bytes = linked_announcement_bytes(&linked, &target).map_err(closure)?;
    decode_announcement_leaf(&target, &bytes).map_err(closure)?;
    Ok((linked, sources.record().schedule()))
}

fn build_announcement(
    bundle: &CandidateLinkedMaturityBundle,
    asset: AssetId,
    outpoint: Outpoint,
    branch: BranchContext,
    amount: u64,
    lead: MaturityLead,
) -> Result<FinalizedMaturityAnnouncement, Refusal> {
    build_with_sponsor(bundle, asset, outpoint, branch, amount, lead, None)
}

fn build_with_sponsor(
    bundle: &CandidateLinkedMaturityBundle,
    asset: AssetId,
    outpoint: Outpoint,
    branch: BranchContext,
    amount: u64,
    lead: MaturityLead,
    sponsor: Option<&transaction::state_construct::MaturitySponsorContext<'_>>,
) -> Result<FinalizedMaturityAnnouncement, Refusal> {
    let target = closure_target().map_err(closure)?;
    let predecessor = bundle
        .instances()
        .first()
        .expect("the linker retains its predecessor constructor");
    let metadata = predecessor.metadata();
    let view = PublicMaturityStateView::new([
        MaturityViewStatement::CurrentStateOutpoint(outpoint),
        MaturityViewStatement::AssetAndAmount(
            AssetField::Explicit(asset),
            ValueField::Explicit(amount),
        ),
        MaturityViewStatement::PredecessorMetadata(metadata.semantic),
        MaturityViewStatement::PredecessorRepresentationNonce(metadata.representation),
        MaturityViewStatement::CurrentRootBinding(branch),
        MaturityViewStatement::PredecessorProgram(predecessor.constructor().output_program()),
        MaturityViewStatement::AcceptedLinkedBundle(Box::new(bundle.clone())),
    ])
    .map_err(transaction)?
    .validate(&target, &OracleStateCurve)
    .map_err(transaction)?;
    let abi = derive_maturity_announcement_abi(&target, &view).map_err(transaction)?;
    let (earliest, latest) = bundle
        .deployment()
        .lead_bounds()
        .bounds()
        .window(metadata.semantic.cycle)
        .map_err(|refusal| {
            transaction(TransactionRefusal::MaturitySuccessorTransitionRefused { refusal })
        })?;
    let announced = lead.announced((earliest, latest));
    let request = MaturityAnnouncementRequest::new(
        announced,
        if sponsor.is_some() {
            RequestedForm::Sponsored
        } else {
            RequestedForm::Sponsorless
        },
        if sponsor.is_some_and(|context| context.capability.offer().change().is_some()) {
            SponsorChangeRequest::Requested
        } else {
            SponsorChangeRequest::NotRequested
        },
    )
    .map_err(transaction)?;
    let construction = transaction::state_construct::construct_maturity_announcement_with_sponsor(
        &target,
        &abi,
        &view,
        &request,
        &OracleStateCurve,
        sponsor,
    )
    .map_err(transaction)?;
    Ok(finalize_maturity_announcement(construction))
}

fn sign_announcement(finalized: &FinalizedMaturityAnnouncement) -> Result<Vec<u8>, Refusal> {
    sign_with_sponsor(finalized, None)
}

fn sign_with_sponsor(
    finalized: &FinalizedMaturityAnnouncement,
    sponsor: Option<&dyn transaction::sponsor::SponsorCapability>,
) -> Result<Vec<u8>, Refusal> {
    let target = closure_target().map_err(closure)?;
    let state =
        OperatorSigningStarted::open(finalized, &target, &OracleLiveCurve::new(target.clone()))
            .map_err(transaction)?;
    let signature = OPERATOR_HANDLE
        .material()
        .map_err(Refusal::Signing)?
        .sign(state.request().message().with_vector_grown(), &[0; 32])
        .map_err(Refusal::Signing)?;
    let binding = state.request().binding();
    let answer = OperatorSigningResponse::new(
        state.request().input_index(),
        signature.to_vec(),
        OPERATOR_SIGHASH_TYPE_BYTE,
        state.protected_bytes().to_vec(),
        binding.key().clone(),
        binding.deployment().clone(),
        binding.capability_revision(),
    );
    let mut registry = OperatorRightRegistry::default();
    let right = registry
        .issue(state.construction_right_scope(), state.protected_bytes())
        .map_err(|refusal| Refusal::ConstructionRight(Box::new(refusal)))?;
    let authorized = state
        .authorize(&mut registry, right, [answer], &OperatorVerifier)
        .map_err(|failure| transaction(failure.refusal))?;
    let ready = authorized
        .bind_for_submission_with_sponsor(&target, sponsor)
        .map_err(transaction)?;
    Ok(ready.bytes().to_vec())
}

fn check_coin(
    subject: &TargetFundingSubject,
    coin: &FundedOutput,
    asset: &str,
) -> Result<Outpoint, Refusal> {
    if coin.asset != asset
        || coin.amount_satoshis != subject.amount_per_output
        || decode_hex(&coin.script).as_deref() != Some(subject.output_program.as_slice())
    {
        return Err(Refusal::FundingMismatch);
    }
    outpoint_of(&coin.outpoint).ok_or(Refusal::FundedOutpointInvalid)
}

/// Funded default-policy observations, with replay independent of capture provenance.
pub mod sponsored {
    use super::{
        AssetField, AssetId, BranchContext, CandidateDeploymentIdentity,
        CandidateLinkedMaturityBundle, Cycle, FinalizedMaturityAnnouncement, FundedOutput,
        MaturityAcceptanceRoute, MaturityLead, MaturityWitnessSelection, NATIVE_PROTOCOL_SCHEMA,
        NativeOperationResponse, OPERATOR_HANDLE, ObservedOutcomeLayer, OperationCaseId,
        OperationStep, OperationSubject, OracleLiveCurve, Outpoint, PlanRefused, Refusal,
        TargetFundingSubject, TargetOperationPlanner, TargetSubmissionSubject, TargetTransaction,
        TransactionRefusal, Txid, ValueField, asset_of, build_announcement, build_with_sponsor,
        check_coin, closure, closure_target, decode_hex, link_for, link_for_metadata, outpoint_of,
        predecessor_program, sha256, sign_announcement, sign_with_sponsor, singleton_declaration,
        transaction,
    };
    use std::collections::VecDeque;
    use target_elements_conformance::protocol::{
        TargetSponsorFundingSubject, TargetSponsorSigningSubject, WireSighashProfile,
    };
    use transaction::bytes::{InputWitness, TargetInput, TargetOutput};
    use transaction::sponsor::{
        SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest,
    };
    use transaction::view::{PublicConstructionView, PublicOutputView};

    /// The positive families, followed by the separately reauthorized allocation.
    pub const CASES: [&str; 13] = [
        "minimum-valid-lead",
        "maximum-valid-lead",
        "representative-interior-lead",
        "smallest-current-cycle",
        "current-cycle-near-checked-upper-domain",
        "nontrivial-unaffected-fields",
        "representation-nonce-zero",
        "representation-nonce-nonzero",
        "accepted-target-transaction-with-semantic-constructor-root-and-recovery-projections-all-matching",
        "sponsored-without-change",
        "sponsored-with-change-where-supported",
        "reauthorized-fee-change",
        "zero-valued-sponsor-member-under-exact-role-structure",
    ];
    /// Exact distinct submitted negatives; allocation also supplies the fee signing wait.
    pub const NEGATIVES: [&str; 5] = [
        "missing-sponsor-authorization",
        "sponsor-change-at-state-output-0",
        "fee-change-substitution",
        "balanced-state-corruption-compensated-by-sponsor-change",
        "sponsor-input-added-after-signing",
    ];

    /// Read a Safety key from the exact submission grammar regenerated by replay.
    ///
    /// Funding, signing, positive-floor and allocation controls and operator
    /// faults supply no key for this sponsored-row fold. Membership-only claims
    /// have no submission.
    #[must_use]
    pub fn submission_row(
        step: &OperationStep,
    ) -> Option<(crate::maturity_safety::MaturitySafetySection, &'static str)> {
        use crate::maturity_safety::MaturitySafetySection;
        if !matches!(step.subject(), OperationSubject::Submission(_)) {
            return None;
        }
        let mut parts = step.case().step.split(':');
        let case = parts.next()?;
        let label = parts.next()?;
        let ordinal = parts.next()?;
        if parts.next().is_some() || ordinal.parse::<usize>().ok()?.to_string() != ordinal {
            return None;
        }
        if label == "control" {
            // Only the two sponsored positives are keyed. The nine positive-floor
            // controls of `CASES[..9]` were accepted by direct block after relay
            // refusal, which establishes neither relay acceptance nor ordinary
            // template selection, so their rows keep awaiting relay admissibility.
            return CASES[9..11]
                .iter()
                .find(|name| **name == case)
                .map(|name| (MaturitySafetySection::Positive, *name));
        }
        if case != CASES[10] {
            return None;
        }
        NEGATIVES[..4]
            .iter()
            .find(|name| **name == label)
            .map(|name| (MaturitySafetySection::SponsorFault, *name))
    }

    const RESERVE: u64 = 1_000_000;
    pub(crate) const NEGATIVE_REASONS: [&str; 5] = [
        "mandatory-script-verify-flag-failed (Witness program hash mismatch)",
        "mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)",
        "mandatory-script-verify-flag-failed (Invalid Schnorr signature)",
        "bad-txns-in-ne-out",
        "mandatory-script-verify-flag-failed (Invalid Schnorr signature)",
    ];

    pub(crate) fn check_reason(
        response: &NativeOperationResponse,
        expected: &str,
        refused: bool,
    ) -> Result<(), Refusal> {
        let value: serde_json::Value = serde_json::from_str(
            response
                .observed_detail
                .as_deref()
                .ok_or(Refusal::SponsoredBoundary)?,
        )
        .map_err(|_| Refusal::SponsoredBoundary)?;
        let (relay, verdict) = if value["route"] == "setup" {
            (
                &value["detail"]["relay_detail"],
                &value["detail"]["outcome"]["verdict_reason"],
            )
        } else {
            (&value["relay_detail"], &value["detail"]["verdict_reason"])
        };
        if relay != expected || (refused && verdict != expected) {
            return Err(Refusal::SponsoredBoundary);
        }
        Ok(())
    }

    // RIPEMD-160 over the fixed 32-byte SHA-256 digest; one padded block.
    pub(crate) fn hash160(bytes: &[u8]) -> Vec<u8> {
        const LEFT: [usize; 80] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 7, 4, 13, 1, 10, 6, 15, 3, 12, 0,
            9, 5, 2, 14, 11, 8, 3, 10, 14, 4, 9, 15, 8, 1, 2, 7, 0, 6, 13, 11, 5, 12, 1, 9, 11, 10,
            0, 8, 12, 4, 13, 3, 7, 15, 14, 5, 6, 2, 4, 0, 5, 9, 7, 12, 2, 10, 14, 1, 3, 8, 11, 6,
            15, 13,
        ];
        const RIGHT: [usize; 80] = [
            5, 14, 7, 0, 9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 6, 11, 3, 7, 0, 13, 5, 10, 14,
            15, 8, 12, 4, 9, 1, 2, 15, 5, 1, 3, 7, 14, 6, 9, 11, 8, 12, 2, 10, 0, 4, 13, 8, 6, 4,
            1, 3, 11, 15, 0, 5, 12, 2, 13, 9, 7, 10, 14, 12, 15, 10, 4, 1, 5, 8, 7, 6, 2, 13, 14,
            0, 3, 9, 11,
        ];
        const SL: [u32; 80] = [
            11, 14, 15, 12, 5, 8, 7, 9, 11, 13, 14, 15, 6, 7, 9, 8, 7, 6, 8, 13, 11, 9, 7, 15, 7,
            12, 15, 9, 11, 7, 13, 12, 11, 13, 6, 7, 14, 9, 13, 15, 14, 8, 13, 6, 5, 12, 7, 5, 11,
            12, 14, 15, 14, 15, 9, 8, 9, 14, 5, 6, 8, 6, 5, 12, 9, 15, 5, 11, 6, 8, 13, 12, 5, 12,
            13, 14, 11, 8, 5, 6,
        ];
        const SR: [u32; 80] = [
            8, 9, 9, 11, 13, 15, 15, 5, 7, 7, 8, 11, 14, 14, 12, 6, 9, 13, 15, 7, 12, 8, 9, 11, 7,
            7, 12, 7, 6, 15, 13, 11, 9, 7, 15, 11, 8, 6, 6, 14, 12, 13, 5, 14, 13, 13, 7, 5, 15, 5,
            8, 11, 14, 14, 6, 14, 6, 9, 12, 9, 12, 5, 15, 8, 8, 5, 12, 9, 12, 5, 14, 6, 8, 13, 6,
            5, 15, 13, 11, 11,
        ];
        const KL: [u32; 5] = [0, 0x5a82_7999, 0x6ed9_eba1, 0x8f1b_bcdc, 0xa953_fd4e];
        const KR: [u32; 5] = [0x50a2_8be6, 0x5c4d_d124, 0x6d70_3ef3, 0x7a6d_76e9, 0];
        let mut words = [0_u32; 16];
        for (word, chunk) in words.iter_mut().zip(sha256(bytes).as_chunks::<4>().0) {
            *word = u32::from_le_bytes(*chunk);
        }
        words[8] = 0x80;
        words[14] = 256;
        let initial = [
            0x6745_2301_u32,
            0xefcd_ab89,
            0x98ba_dcfe,
            0x1032_5476,
            0xc3d2_e1f0,
        ];
        let mut left = initial;
        let mut right = initial;
        let mix = |round, x, y, z| match round {
            0 => x ^ y ^ z,
            1 => (x & y) | (!x & z),
            2 => (x | !y) ^ z,
            3 => (x & z) | (y & !z),
            _ => x ^ (y | !z),
        };
        for (i, (&li, &ri)) in LEFT.iter().zip(&RIGHT).enumerate() {
            for (state, index, shift, round, constant) in [
                (&mut left, li, SL[i], i / 16, KL[i / 16]),
                (&mut right, ri, SR[i], 4 - i / 16, KR[i / 16]),
            ] {
                let [first, second, third, fourth, fifth] = *state;
                let sum = first
                    .wrapping_add(mix(round, second, third, fourth))
                    .wrapping_add(words[index])
                    .wrapping_add(constant)
                    .rotate_left(shift)
                    .wrapping_add(fifth);
                *state = [fifth, sum, second, third.rotate_left(10), fourth];
            }
        }
        [
            initial[1].wrapping_add(left[2]).wrapping_add(right[3]),
            initial[2].wrapping_add(left[3]).wrapping_add(right[4]),
            initial[3].wrapping_add(left[4]).wrapping_add(right[0]),
            initial[4].wrapping_add(left[0]).wrapping_add(right[1]),
            initial[0].wrapping_add(left[1]).wrapping_add(right[2]),
        ]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect()
    }

    /// Policy detail in the existing response field; no protocol member is added.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Policy(pub serde_json::Value);

    fn keys(value: &serde_json::Value, expected: &[&str]) -> bool {
        value.as_object().is_some_and(|object| {
            object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key))
        })
    }

    fn amount(value: &serde_json::Value) -> Result<u64, Refusal> {
        let text = value.as_str().ok_or(Refusal::SponsoredPolicy)?;
        let (whole, fraction) = text.split_once('.').ok_or(Refusal::SponsoredPolicy)?;
        if fraction.len() != 8
            || !whole
                .bytes()
                .chain(fraction.bytes())
                .all(|byte| byte.is_ascii_digit())
        {
            return Err(Refusal::SponsoredPolicy);
        }
        whole
            .parse::<u64>()
            .ok()
            .and_then(|whole| whole.checked_mul(100_000_000))
            .and_then(|whole| {
                fraction
                    .parse::<u64>()
                    .ok()
                    .and_then(|fraction| whole.checked_add(fraction))
            })
            .filter(|amount| *amount > 0)
            .ok_or(Refusal::SponsoredPolicy)
    }

    impl Policy {
        /// Validate the two different provenance claims and the retained startup arguments.
        /// # Errors
        /// Returns `SponsoredPolicy` for a missing, surplus or contradictory field.
        pub fn parse(value: serde_json::Value) -> Result<Self, Refusal> {
            if !keys(
                &value,
                &[
                    "relay",
                    "mempool",
                    "block",
                    "unit",
                    "block_identifier",
                    "relay_provenance",
                    "block_provenance",
                    "start_arguments",
                ],
            ) || value["unit"] != "BTC/kvB"
                || value["block"] != "0.00000100"
                || value["block_identifier"] != "DEFAULT_BLOCK_MIN_TX_FEE"
                || value["relay_provenance"] != "reported by the node"
                || value["block_provenance"] != "declared by the pinned source, no override"
            {
                return Err(Refusal::SponsoredPolicy);
            }
            for key in ["relay", "mempool", "block"] {
                amount(&value[key])?;
            }
            let args = value["start_arguments"]
                .as_array()
                .ok_or(Refusal::SponsoredPolicy)?;
            if args.is_empty()
                || args.iter().any(|arg| {
                    arg.as_str().is_none_or(|arg| {
                        arg.starts_with("-minrelaytxfee") || arg.starts_with("-blockmintxfee")
                    })
                })
                || !args.iter().any(|arg| arg == "-daemon=0")
            {
                return Err(Refusal::SponsoredPolicy);
            }
            Ok(Self(value))
        }

        /// Twice the larger effective floor over measured virtual size, rounded upward.
        /// # Errors
        /// Returns `SponsoredFee` on arithmetic overflow or an absent measurement.
        pub fn fee(&self, weight: u64) -> Result<u64, Refusal> {
            let rate = ["relay", "mempool", "block"]
                .into_iter()
                .map(|key| amount(&self.0[key]))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .max()
                .ok_or(Refusal::SponsoredFee)?;
            if weight == 0 {
                return Err(Refusal::SponsoredFee);
            }
            weight
                .checked_add(3)
                .map(|n| n / 4)
                .and_then(|n| n.checked_mul(rate))
                .and_then(|n| n.checked_mul(2))
                .and_then(|n| n.checked_add(999))
                .map(|n| n / 1000)
                .filter(|fee| *fee > 1 && *fee < RESERVE)
                .ok_or(Refusal::SponsoredFee)
        }
    }

    /// Decode an observation detail without conflating its relay and block outcomes.
    /// # Errors
    /// Returns `SponsoredPolicy` or `SponsoredRoute` for inconsistent evidence.
    pub fn detail(
        response: &NativeOperationResponse,
    ) -> Result<(Policy, Option<MaturityAcceptanceRoute>), Refusal> {
        if response.observed_layer == ObservedOutcomeLayer::ExecutorInfrastructureFailure {
            return Err(Refusal::SponsoredUndecided);
        }
        let value: serde_json::Value = serde_json::from_str(
            response
                .observed_detail
                .as_deref()
                .ok_or(Refusal::SponsoredPolicy)?,
        )
        .map_err(|_| Refusal::SponsoredPolicy)?;
        if !keys(
            &value,
            &[
                "schema",
                "policy",
                "route",
                "relay_allowed",
                "relay_detail",
                "detail",
            ],
        ) || value["schema"] != 1
        {
            return Err(Refusal::SponsoredPolicy);
        }
        let policy = Policy::parse(value["policy"].clone())?;
        if value["route"] == "undecided" {
            return Err(Refusal::SponsoredUndecided);
        }
        if matches!(
            value["route"].as_str(),
            Some("relay-and-template" | "direct-block")
        ) {
            let facts = &value["detail"];
            let readback = response
                .mined_readback
                .as_ref()
                .ok_or(Refusal::SponsoredRoute)?;
            if facts["txid"] != readback.transaction_id
                || facts["included_block"] != readback.block_hash
                || facts["inclusion_in_produced"].as_bool()
                    != Some(facts["produced_block"] == facts["included_block"])
                || !facts["load_five_minutes"].is_number()
                || !facts["mempool_size"].is_u64()
                || facts["mempool_minimum"] != policy.0["mempool"]
            {
                return Err(Refusal::SponsoredRoute);
            }
            if value["route"] == "relay-and-template" {
                let template = &facts["template"];
                if facts["inclusion_in_produced"] != true
                    || template["member"] != true
                    || !template["weight"].is_u64()
                    || !template["txids"]
                        .as_array()
                        .is_some_and(|ids| ids.contains(&facts["txid"]))
                    || !(template["source"] == "assembly-read"
                        || (template["source"] == "unavailable-isolated-production"
                            && facts["inclusion_in_produced"] == true
                            && ["-listen=0", "-dnsseed=0", "-connect=0"].iter().all(|arg| {
                                policy.0["start_arguments"]
                                    .as_array()
                                    .is_some_and(|args| args.iter().any(|value| value == arg))
                            })))
                {
                    return Err(Refusal::SponsoredRoute);
                }
            } else if facts["inclusion_in_produced"] != true {
                return Err(Refusal::SponsoredRoute);
            }
        }
        let route = match value["route"].as_str() {
            Some("setup") if value["relay_allowed"].is_null() => None,
            Some("refused")
                if response.observed_layer != ObservedOutcomeLayer::Accepted
                    && value["relay_allowed"] == false
                    && value["relay_detail"].is_string() =>
            {
                None
            }
            Some("relay-and-template")
                if response.observed_layer == ObservedOutcomeLayer::Accepted
                    && value["relay_allowed"] == true
                    && value["relay_detail"].is_null() =>
            {
                Some(MaturityAcceptanceRoute::RelayAndTemplate)
            }
            Some("direct-block")
                if response.observed_layer == ObservedOutcomeLayer::Accepted
                    && value["relay_allowed"] == false
                    && value["relay_detail"].is_string() =>
            {
                Some(MaturityAcceptanceRoute::DirectBlockAfterRelayRefusal)
            }
            _ => return Err(Refusal::SponsoredRoute),
        };
        Ok((policy, route))
    }

    #[cfg_attr(test, derive(Clone))]
    struct Sponsor {
        coin: FundedOutput,
        fee: u64,
        change: bool,
        answer: Option<SponsorSignature>,
    }
    impl SponsorCapability for Sponsor {
        fn offer(&self) -> SponsorOffer {
            SponsorOffer::new(
                [outpoint_of(&self.coin.outpoint).expect("validated coin")],
                self.fee,
                self.change
                    .then_some(ValueField::Explicit(self.coin.amount_satoshis - self.fee)),
            )
            .expect("one distinct coin")
        }
        fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
            let program = decode_hex(&self.coin.script)?;
            Some((0, program.get(2..)?.to_vec()))
        }
        fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
            self.answer
                .clone()
                .filter(|answer| answer.bound_to() == request.transaction() && request.input() == 1)
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Action {
        Issue,
        Predecessor,
        TemplateCoin,
        TemplateSign,
        Measure,
        FinalCoin,
        FinalSign,
        MutantSign(usize, usize),
        MutantSubmit(usize),
        Control,
        Zero,
        ZeroSubmit,
    }

    /// A row still undecided after its bounded fresh attempt.
    #[cfg_attr(test, derive(Clone))]
    #[derive(Debug, PartialEq, Eq)]
    pub struct WaitingRow {
        /// The operation identifying the unresolved row.
        pub row: String,
        /// The retained undecided observation, without a target verdict.
        pub reason: String,
    }

    /// One serialized schedule; controls consume their coins after their negative subjects.
    #[cfg_attr(test, derive(Clone))]
    pub struct Planner {
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        schedule: Option<tapscript::StateWitnessSchedule>,
        case: usize,
        queue: VecDeque<(OperationStep, Action)>,
        pending: Option<(OperationStep, Action)>,
        exchanges: Vec<(OperationStep, NativeOperationResponse)>,
        bundle: Option<CandidateLinkedMaturityBundle>,
        asset: Option<String>,
        predecessor: Option<Outpoint>,
        finalized: Option<FinalizedMaturityAnnouncement>,
        template: Option<FundedOutput>,
        sponsor: Option<Sponsor>,
        signed: Vec<u8>,
        mutant: Option<TargetTransaction>,
        policy: Option<Policy>,
        fee: u64,
        measured: u64,
        refusal: Option<Refusal>,
        retry: Option<Action>,
        retried: Vec<(usize, Action)>,
        waiting: Vec<WaitingRow>,
        run_start: usize,
        credited: Vec<usize>,
        skipped_negatives: Vec<usize>,
    }

    impl Planner {
        /// Start the fixed ceremony under the stated deployment and branch.
        /// # Errors
        /// Preserves public linking and construction refusals.
        /// # Panics
        /// Panics only if the fixed architecture omits its singleton, which its published census cannot arrange.
        pub fn new(
            identity: CandidateDeploymentIdentity,
            branch: BranchContext,
        ) -> Result<Self, Refusal> {
            let mut plan = Self {
                identity,
                branch,
                schedule: None,
                case: 0,
                queue: VecDeque::new(),
                pending: None,
                exchanges: Vec::new(),
                bundle: None,
                asset: None,
                predecessor: None,
                finalized: None,
                template: None,
                sponsor: None,
                signed: Vec::new(),
                mutant: None,
                policy: None,
                fee: 0,
                measured: 0,
                refusal: None,
                retry: None,
                retried: Vec::new(),
                waiting: Vec::new(),
                run_start: 0,
                credited: Vec::new(),
                skipped_negatives: Vec::new(),
            };
            plan.issue()?;
            Ok(plan)
        }

        /// The exact settled exchanges, including a failed terminal attempt.
        #[must_use]
        pub fn exchanges(&self) -> &[(OperationStep, NativeOperationResponse)] {
            &self.exchanges
        }
        /// The schedule recorded by the link that issued each ceremony case.
        ///
        /// Read from the linked bundle's own record rather than restated, so a
        /// reader comparing it against an expected schedule compares the link.
        #[must_use]
        pub const fn schedule(&self) -> Option<tapscript::StateWitnessSchedule> {
            self.schedule
        }
        /// The schedule the linker recorded for the bundle the last case submitted under.
        #[cfg(test)]
        pub(crate) fn linked_schedule(&self) -> Option<tapscript::StateWitnessSchedule> {
            self.bundle
                .as_ref()
                .map(|bundle| bundle.record().schedule())
        }
        /// Rows still undecided after their one fresh attempt, with the retained reason.
        #[must_use]
        pub fn waiting(&self) -> &[WaitingRow] {
            &self.waiting
        }
        /// Waiting rows and retained reasons for capture and report binding.
        #[must_use]
        pub fn waiting_json(&self) -> Vec<serde_json::Value> {
            self.waiting
                .iter()
                .map(|entry| serde_json::json!({"row":entry.row, "reason":entry.reason}))
                .collect()
        }
        /// Submitted observations paired with an accepted control from their own run.
        #[must_use]
        pub fn credited(&self) -> Vec<&(OperationStep, NativeOperationResponse)> {
            self.credited
                .iter()
                .map(|index| &self.exchanges[*index])
                .collect()
        }
        /// Exact per-operation observations, including per-probe policy and concurrent state.
        #[must_use]
        pub fn probes(&self) -> Vec<&str> {
            self.exchanges
                .iter()
                .filter_map(|(_, response)| response.observed_detail.as_deref())
                .collect()
        }
        /// The first refusal, retained without reinterpretation.
        #[must_use]
        pub const fn refusal(&self) -> Option<&Refusal> {
            self.refusal.as_ref()
        }
        /// The policy read from the ceremony's responses.
        #[must_use]
        pub const fn policy(&self) -> Option<&Policy> {
            self.policy.as_ref()
        }
        /// All operations, fundings, signings and submissions; credited acceptances and negatives.
        #[must_use]
        pub fn census(&self) -> [usize; 6] {
            let mut counts = [self.exchanges.len(), 0, 0, 0, 0, 0];
            for (index, (step, response)) in self.exchanges.iter().enumerate() {
                match step.subject() {
                    OperationSubject::Funding(_) | OperationSubject::SponsorFunding(_) => {
                        counts[1] += 1;
                    }
                    OperationSubject::SponsorSigning(_) => counts[2] += 1,
                    OperationSubject::Submission(_) => {
                        counts[3] += 1;
                        if !self.credited.contains(&index) {
                            continue;
                        }
                        if response.observed_layer == ObservedOutcomeLayer::Accepted {
                            counts[4] += 1;
                        } else if matches!(
                            response.observed_layer,
                            ObservedOutcomeLayer::ConsensusRejectionBeforeScript
                                | ObservedOutcomeLayer::RelayPolicyRejection
                                | ObservedOutcomeLayer::ScriptPathRejection
                                | ObservedOutcomeLayer::KeyPathRejection
                        ) {
                            counts[5] += 1;
                        }
                    }
                    _ => {}
                }
            }
            counts
        }
        /// Require complete, policy-bound observations without promoting Safety standings.
        /// # Errors
        /// Preserves the first refusal or reports an incomplete schedule.
        pub fn complete(&self) -> Result<(), Refusal> {
            if let Some(refusal) = &self.refusal {
                return Err(refusal.clone());
            }
            if self.case != CASES.len() || self.pending.is_some() || !self.queue.is_empty() {
                return Err(Refusal::IncompleteTranscript);
            }
            Ok(())
        }
        fn push(&mut self, label: &str, subject: OperationSubject, action: Action) {
            let name = format!(
                "{}:{label}:{}",
                CASES
                    .get(self.case)
                    .unwrap_or(&"zero-valued-sponsor-member-under-exact-role-structure"),
                self.exchanges.len() + self.queue.len()
            );
            self.queue
                .push_back((OperationStep::new(&name, subject), action));
        }
        fn issue(&mut self) -> Result<(), Refusal> {
            self.run_start = self.exchanges.len();
            let (bundle, _) = link_for(
                &self.identity,
                AssetId::from_internal([0x11; 32]),
                MaturityWitnessSelection::Retained(
                    tapscript::StateWitnessSchedule::VariableMetadata,
                ),
            )?;
            self.schedule = Some(bundle.record().schedule());
            self.push(
                "issue",
                OperationSubject::Funding(Box::new(TargetFundingSubject {
                    issue_asset: true,
                    asset: None,
                    output_program: predecessor_program(&bundle),
                    outputs: 1,
                    amount_per_output: singleton_declaration()?.fixed_amount().get(),
                })),
                Action::Issue,
            );
            Ok(())
        }
        const fn lead(&self) -> MaturityLead {
            match self.case {
                0 => MaturityLead::Minimum,
                1 | 4 => MaturityLead::Maximum,
                _ => MaturityLead::Interior,
            }
        }
        fn build(&self) -> Result<FinalizedMaturityAnnouncement, Refusal> {
            let bundle = self.bundle.as_ref().ok_or(Refusal::IncompleteTranscript)?;
            let asset = asset_of(self.asset.as_deref().ok_or(Refusal::IncompleteTranscript)?)
                .ok_or(Refusal::FundingMismatch)?;
            let coin = self.predecessor.ok_or(Refusal::IncompleteTranscript)?;
            let amount = singleton_declaration()?.fixed_amount().get();
            if let Some(sponsor) = &self.sponsor {
                let view = PublicConstructionView::new([public_coin(&sponsor.coin)?])
                    .map_err(transaction)?;
                let context = transaction::state_construct::MaturitySponsorContext {
                    reserve_asset: asset_of(&sponsor.coin.asset).ok_or(Refusal::FundingMismatch)?,
                    view: &view,
                    capability: sponsor,
                };
                build_with_sponsor(
                    bundle,
                    asset,
                    coin,
                    self.branch,
                    amount,
                    self.lead(),
                    Some(&context),
                )
            } else {
                build_announcement(bundle, asset, coin, self.branch, amount, self.lead())
            }
        }
        fn fund_sponsor(&mut self, amount: u64, action: Action) {
            self.push(
                "fund-sponsor",
                OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                    sponsor_outputs: 1,
                    amount_per_sponsor_output: amount,
                })),
                action,
            );
        }
        fn sign(&mut self, raw: Vec<u8>, coin: &FundedOutput, index: u16, action: Action) {
            self.push(
                "sign-sponsor",
                OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
                    finalized_transaction: raw,
                    sponsor_input_index: index,
                    sponsor_outpoint: coin.outpoint.clone(),
                    sighash_profile: WireSighashProfile::AllInputsAllOutputs,
                })),
                action,
            );
        }
        fn submit(&mut self, label: &str, bytes: Vec<u8>, action: Action) {
            self.push(
                label,
                OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                    transaction_bytes: bytes,
                })),
                action,
            );
        }
        fn select_bundle(&self, asset: AssetId) -> Result<CandidateLinkedMaturityBundle, Refusal> {
            let selection = MaturityWitnessSelection::Retained(
                self.schedule.ok_or(Refusal::IncompleteTranscript)?,
            );
            let (seed, _) = link_for(&self.identity, asset, selection)?;
            let mut metadata = seed.instances()[0].metadata().semantic;
            if self.case == 3 {
                metadata.cycle = Cycle::ZERO;
            }
            if self.case == 4 {
                metadata.cycle =
                    Cycle::new(u64::MAX - seed.deployment().lead_bounds().bounds().maximum().get());
            }
            if self.case == 5 {
                metadata.omega = realization::ProtocolAmount::new(91)
                    .map_err(|e| Refusal::Realization(Box::new(e)))?;
                metadata.y_l = realization::ProtocolAmount::new(17)
                    .map_err(|e| Refusal::Realization(Box::new(e)))?;
                metadata.y_t = realization::ProtocolAmount::new(13)
                    .map_err(|e| Refusal::Realization(Box::new(e)))?;
                metadata.q = realization::ProtocolAmount::new(7)
                    .map_err(|e| Refusal::Realization(Box::new(e)))?;
            }
            for attempt in 0..256_u64 {
                let (bundle, _) =
                    link_for_metadata(&self.identity, asset, selection, Some(metadata))?;
                if !matches!(self.case, 6 | 7) {
                    return Ok(bundle);
                }
                let trial = build_announcement(
                    &bundle,
                    asset,
                    Outpoint::new(Txid::from_internal([1; 32]), 0).map_err(transaction)?,
                    self.branch,
                    singleton_declaration()?.fixed_amount().get(),
                    self.lead(),
                )?;
                let nonce = trial.construction().successor_constructor().nonce().get();
                if (nonce == 0) == (self.case == 6) {
                    return Ok(bundle);
                }
                metadata.omega = realization::ProtocolAmount::new(1000 + attempt)
                    .map_err(|e| Refusal::Realization(Box::new(e)))?;
            }
            Err(Refusal::SponsoredNonceSelection)
        }
        #[expect(
            clippy::too_many_lines,
            reason = "Each arm settles one explicitly scheduled operation and records its next obligation."
        )]
        fn settle(
            &mut self,
            step: &OperationStep,
            action: Action,
            response: &NativeOperationResponse,
        ) -> Result<(), Refusal> {
            let (policy, route) = match detail(response) {
                Err(Refusal::SponsoredUndecided) => return self.reprobe(step, action, response),
                other => other?,
            };
            self.policy = Some(policy);
            match action {
                Action::Issue => {
                    let OperationSubject::Funding(subject) = step.subject() else {
                        return Err(Refusal::UnexpectedResponse);
                    };
                    let asset = response
                        .issued_asset
                        .as_deref()
                        .ok_or(Refusal::IssuedAssetMissingOrInvalid)?;
                    let coin = only_coin(response)?;
                    check_coin(subject, coin, asset)?;
                    self.bundle =
                        Some(self.select_bundle(asset_of(asset).ok_or(Refusal::FundingMismatch)?)?);
                    self.asset = Some(asset.to_owned());
                    self.sponsor = None;
                    self.push(
                        "predecessor",
                        OperationSubject::Funding(Box::new(TargetFundingSubject {
                            issue_asset: false,
                            asset: self.asset.clone(),
                            output_program: predecessor_program(
                                self.bundle.as_ref().ok_or(Refusal::IncompleteTranscript)?,
                            ),
                            outputs: 1,
                            amount_per_output: singleton_declaration()?.fixed_amount().get(),
                        })),
                        Action::Predecessor,
                    );
                }
                Action::Predecessor => {
                    let OperationSubject::Funding(subject) = step.subject() else {
                        return Err(Refusal::UnexpectedResponse);
                    };
                    self.predecessor = Some(check_coin(
                        subject,
                        only_coin(response)?,
                        self.asset.as_deref().ok_or(Refusal::FundingMismatch)?,
                    )?);
                    if self.case < 9 {
                        self.finalized = Some(self.build()?);
                        self.signed = sign_announcement(
                            self.finalized
                                .as_ref()
                                .ok_or(Refusal::IncompleteTranscript)?,
                        )?;
                        self.submit("control", self.signed.clone(), Action::Control);
                    } else {
                        self.fund_sponsor(RESERVE, Action::TemplateCoin);
                    }
                }
                Action::TemplateCoin | Action::FinalCoin => {
                    let coin = only_coin(response)?.clone();
                    let OperationSubject::SponsorFunding(subject) = step.subject() else {
                        return Err(Refusal::UnexpectedResponse);
                    };
                    if coin.amount_satoshis != subject.amount_per_sponsor_output
                        || decode_hex(&coin.script)
                            .is_none_or(|p| p.len() != 22 || p[..2] != [0, 20])
                    {
                        return Err(Refusal::FundingMismatch);
                    }
                    public_coin(&coin)?;
                    let template = matches!(action, Action::TemplateCoin);
                    if template {
                        self.template = Some(coin.clone());
                    }
                    let fee = if template {
                        if self.case == 9 { RESERVE } else { RESERVE / 2 }
                    } else {
                        self.fee
                    };
                    self.sponsor = Some(Sponsor {
                        coin: coin.clone(),
                        fee,
                        change: self.case != 9,
                        answer: None,
                    });
                    let finalized = self.build()?;
                    let raw = finalized.protected_bytes().to_vec();
                    self.finalized = Some(finalized);
                    self.sign(
                        raw,
                        &coin,
                        1,
                        if template {
                            Action::TemplateSign
                        } else {
                            Action::FinalSign
                        },
                    );
                }
                Action::TemplateSign | Action::FinalSign => {
                    let signature = self.checked_signature(step, response)?;
                    let sponsor = self.sponsor.as_mut().ok_or(Refusal::IncompleteTranscript)?;
                    sponsor.answer = Some(signature);
                    self.signed = sign_with_sponsor(
                        self.finalized
                            .as_ref()
                            .ok_or(Refusal::IncompleteTranscript)?,
                        Some(sponsor),
                    )?;
                    if matches!(action, Action::TemplateSign) {
                        let coin = sponsor.coin.clone();
                        self.sign(self.signed.clone(), &coin, 1, Action::Measure);
                    } else if let Some(target) = self.retry.take() {
                        if self.case == 10 {
                            self.next_mutant(0)?;
                        } else {
                            match target {
                                Action::Zero | Action::ZeroSubmit => {
                                    self.fund_sponsor(0, Action::Zero);
                                }
                                _ => self.submit("control", self.signed.clone(), Action::Control),
                            }
                        }
                    } else if self.case == 12 {
                        self.fund_sponsor(0, Action::Zero);
                    } else if self.case == 10 {
                        self.next_mutant(0)?;
                    } else {
                        self.submit("control", self.signed.clone(), Action::Control);
                    }
                }
                Action::Measure => {
                    self.checked_signature(step, response)?;
                    self.measured = response
                        .resources
                        .transaction_weight
                        .ok_or(Refusal::SponsoredFee)?;
                    // Eight extra witness weight units cover the maximal DER-size difference.
                    self.fee = self
                        .policy
                        .as_ref()
                        .ok_or(Refusal::SponsoredPolicy)?
                        .fee(self.measured.checked_add(8).ok_or(Refusal::SponsoredFee)?)?;
                    if self.case == 11 {
                        self.fee += 1;
                    }
                    self.fund_sponsor(
                        if self.case == 9 { self.fee } else { RESERVE },
                        Action::FinalCoin,
                    );
                }
                Action::MutantSign(which, index) => {
                    let signature = self.checked_signature(step, response)?;
                    let candidate = self.mutant.as_ref().ok_or(Refusal::IncompleteTranscript)?;
                    let mut witnesses = candidate.witnesses().to_vec();
                    witnesses[index] = InputWitness::new(signature.stack().to_vec());
                    let candidate = replace(
                        candidate,
                        candidate.inputs().to_vec(),
                        candidate.outputs().to_vec(),
                        witnesses,
                    )?;
                    self.mutant = Some(candidate.clone());
                    if index + 1 < candidate.inputs().len() {
                        let coin = self.template.clone().ok_or(Refusal::IncompleteTranscript)?;
                        self.sign(candidate.encode(), &coin, 2, Action::MutantSign(which, 2));
                    } else if which == NEGATIVES.len() {
                        self.submit("zero-member", candidate.encode(), Action::ZeroSubmit);
                    } else {
                        self.submit(
                            NEGATIVES[which],
                            candidate.encode(),
                            Action::MutantSubmit(which),
                        );
                    }
                }
                Action::MutantSubmit(which) => {
                    let expected = if which == 3 {
                        ObservedOutcomeLayer::ConsensusRejectionBeforeScript
                    } else {
                        ObservedOutcomeLayer::ScriptPathRejection
                    };
                    if response.observed_layer != expected {
                        return Err(Refusal::SponsoredBoundary);
                    }
                    check_reason(response, NEGATIVE_REASONS[which], true)?;
                    self.next_mutant(which + 1)?;
                }
                Action::Control => {
                    if response.observed_layer != ObservedOutcomeLayer::Accepted {
                        return Err(Refusal::SponsoredBoundary);
                    }
                    if self.case >= 9 && route != Some(MaturityAcceptanceRoute::RelayAndTemplate) {
                        return Err(Refusal::SponsoredRoute);
                    }
                    if self.case < 9
                        && route != Some(MaturityAcceptanceRoute::DirectBlockAfterRelayRefusal)
                    {
                        return Err(Refusal::SponsoredRoute);
                    }
                    if self.case < 9 {
                        check_reason(response, "min relay fee not met", false)?;
                    }
                    check_readback(&self.signed, response)?;
                    if self.case < 9 {
                        self.project()?;
                    }
                    if self.case >= 9 {
                        let weight = response
                            .resources
                            .transaction_weight
                            .ok_or(Refusal::SponsoredFee)?;
                        if self.fee
                            < self
                                .policy
                                .as_ref()
                                .ok_or(Refusal::SponsoredPolicy)?
                                .fee(weight)?
                        {
                            return Err(Refusal::SponsoredFee);
                        }
                    }
                    self.credited
                        .extend((self.run_start..self.exchanges.len()).filter(|index| {
                            matches!(
                                self.exchanges[*index].0.subject(),
                                OperationSubject::Submission(_)
                            )
                        }));
                    self.case += 1;
                    self.retry = None;
                    if self.case < CASES.len() {
                        self.issue()?;
                    }
                }
                Action::Zero => {
                    if !matches!(
                        response.observed_layer,
                        ObservedOutcomeLayer::Accepted
                            | ObservedOutcomeLayer::ConsensusRejectionBeforeScript
                            | ObservedOutcomeLayer::RelayPolicyRejection
                    ) {
                        return Err(Refusal::SponsoredBoundary);
                    }
                    if response.observed_layer != ObservedOutcomeLayer::Accepted {
                        check_reason(response, "bad-txns-in-ne-out", true)?;
                    }
                    if response.observed_layer == ObservedOutcomeLayer::Accepted {
                        self.zero_member(only_coin(response)?.clone())?;
                    } else {
                        self.case = CASES.len();
                    }
                }
                Action::ZeroSubmit => match response.observed_layer {
                    ObservedOutcomeLayer::Accepted => {
                        self.case = CASES.len();
                        check_readback(
                            &self
                                .mutant
                                .as_ref()
                                .ok_or(Refusal::IncompleteTranscript)?
                                .encode(),
                            response,
                        )?;
                        self.credited.push(self.exchanges.len() - 1);
                    }
                    ObservedOutcomeLayer::ScriptPathRejection
                    | ObservedOutcomeLayer::ConsensusRejectionBeforeScript
                    | ObservedOutcomeLayer::RelayPolicyRejection => {
                        self.case = CASES.len();
                        self.credited.push(self.exchanges.len() - 1);
                    }
                    _ => return Err(Refusal::SponsoredBoundary),
                },
            }
            Ok(())
        }
        fn reprobe(
            &mut self,
            step: &OperationStep,
            action: Action,
            response: &NativeOperationResponse,
        ) -> Result<(), Refusal> {
            let target = self.retry.unwrap_or(match action {
                Action::MutantSubmit(_) | Action::Control | Action::Zero | Action::ZeroSubmit => {
                    action
                }
                Action::MutantSign(which, _) => Action::MutantSubmit(which),
                _ => Action::Control,
            });
            self.queue.clear();
            if self.retried.contains(&(self.case, target)) {
                self.waiting.push(WaitingRow {
                    row: step.case().step.clone(),
                    reason: response.observed_detail.clone().unwrap_or_default(),
                });
                self.retry = None;
                if let Action::MutantSubmit(which) = target {
                    self.skipped_negatives.push(which);
                    self.retry = Some(Action::Control);
                    return self.issue();
                }
                self.case += 1;
            } else {
                self.retried.push((self.case, target));
                self.retry = Some(target);
            }
            if self.case < CASES.len() {
                self.issue()?;
            }
            Ok(())
        }
        fn checked_signature(
            &self,
            step: &OperationStep,
            response: &NativeOperationResponse,
        ) -> Result<SponsorSignature, Refusal> {
            let OperationSubject::SponsorSigning(subject) = step.subject() else {
                return Err(Refusal::UnexpectedResponse);
            };
            let coin = self
                .sponsor
                .as_ref()
                .map(|sponsor| &sponsor.coin)
                .into_iter()
                .chain(self.template.as_ref())
                .find(|coin| coin.outpoint == subject.sponsor_outpoint)
                .ok_or(Refusal::SponsoredSignature)?;
            checked_signature(step, response, coin)
        }
        fn next_mutant(&mut self, which: usize) -> Result<(), Refusal> {
            if self.skipped_negatives.contains(&which) {
                return self.next_mutant(which + 1);
            }
            if which == NEGATIVES.len() {
                self.submit("control", self.signed.clone(), Action::Control);
                return Ok(());
            }
            let original = TargetTransaction::decode(&self.signed).map_err(transaction)?;
            let mut inputs = original.inputs().to_vec();
            let mut outputs = original.outputs().to_vec();
            let mut witnesses = original.witnesses().to_vec();
            match which {
                0 => witnesses[1] = InputWitness::new(Vec::new()),
                1 => outputs.swap(0, 1),
                2 => {
                    outputs[1] = move_value(&outputs[1], -1)?;
                    outputs[2] = move_value(&outputs[2], 1)?;
                }
                3 => {
                    outputs[0] = move_value(&outputs[0], 1)?;
                    outputs[1] = move_value(&outputs[1], -1)?;
                }
                4 => {
                    let coin = self
                        .template
                        .as_ref()
                        .ok_or(Refusal::IncompleteTranscript)?;
                    inputs.push(TargetInput::new(
                        outpoint_of(&coin.outpoint).ok_or(Refusal::FundingMismatch)?,
                        u32::MAX,
                    ));
                    witnesses.push(InputWitness::new(Vec::new()));
                    outputs[1] = move_value(
                        &outputs[1],
                        i64::try_from(coin.amount_satoshis)
                            .map_err(|_| Refusal::FundingMismatch)?,
                    )?;
                }
                _ => return Err(Refusal::UnexpectedResponse),
            }
            let mut candidate = replace(&original, inputs, outputs, witnesses)?;
            if matches!(which, 1 | 3) {
                candidate = resign(
                    &candidate,
                    self.finalized
                        .as_ref()
                        .ok_or(Refusal::IncompleteTranscript)?,
                    None,
                )?;
            }
            self.mutant = Some(candidate.clone());
            if which == 0 {
                self.submit(
                    NEGATIVES[which],
                    candidate.encode(),
                    Action::MutantSubmit(which),
                );
            } else {
                let coin = self
                    .sponsor
                    .as_ref()
                    .ok_or(Refusal::IncompleteTranscript)?
                    .coin
                    .clone();
                self.sign(candidate.encode(), &coin, 1, Action::MutantSign(which, 1));
            }
            Ok(())
        }
        fn zero_member(&mut self, coin: FundedOutput) -> Result<(), Refusal> {
            if coin.amount_satoshis != 0 {
                return Err(Refusal::FundingMismatch);
            }
            let original = TargetTransaction::decode(&self.signed).map_err(transaction)?;
            let mut inputs = original.inputs().to_vec();
            inputs.push(TargetInput::new(
                outpoint_of(&coin.outpoint).ok_or(Refusal::FundingMismatch)?,
                u32::MAX,
            ));
            let mut witnesses = original.witnesses().to_vec();
            witnesses.push(InputWitness::new(Vec::new()));
            let candidate = replace(&original, inputs, original.outputs().to_vec(), witnesses)?;
            let candidate = resign(
                &candidate,
                self.finalized
                    .as_ref()
                    .ok_or(Refusal::IncompleteTranscript)?,
                Some(&coin),
            )?;
            self.template = Some(coin);
            self.mutant = Some(candidate.clone());
            let sponsor = self
                .sponsor
                .as_ref()
                .ok_or(Refusal::IncompleteTranscript)?
                .coin
                .clone();
            self.sign(
                candidate.encode(),
                &sponsor,
                1,
                Action::MutantSign(NEGATIVES.len(), 1),
            );
            Ok(())
        }
        fn project(&self) -> Result<(), Refusal> {
            use crate::maturity_continuity::{
                MaturityByteSource, MaturityFundedPredecessor, MaturityProjectionInput,
                project_maturity_continuity,
            };
            use crate::maturity_history::StateRootEdge;
            use crate::maturity_recovery::{PublicAnnouncementHandoff, PublicAnnouncementLocator};
            let bundle = self.bundle.as_ref().ok_or(Refusal::IncompleteTranscript)?;
            let funded = MaturityFundedPredecessor {
                outpoint: self.predecessor.ok_or(Refusal::IncompleteTranscript)?,
                asset: asset_of(self.asset.as_deref().ok_or(Refusal::IncompleteTranscript)?)
                    .ok_or(Refusal::FundingMismatch)?,
                amount: singleton_declaration()?.fixed_amount().get(),
                program: predecessor_program(bundle),
            };
            let continuity = project_maturity_continuity(MaturityProjectionInput {
                source: MaturityByteSource::NodeFreeSubmitReady,
                submitted_bytes: &self.signed,
                funded: &funded,
                branch: self.branch,
                bundle,
                identity: &self.identity,
            })
            .map_err(|e| Refusal::SponsoredProjection(Box::new(e)))?;
            let edge = StateRootEdge::from_continuity(&continuity);
            let recovered = StateRootEdge::from_public_handoff(PublicAnnouncementHandoff::new(
                PublicAnnouncementLocator::new(
                    MaturityByteSource::NodeFreeSubmitReady,
                    edge.successor().txid(),
                ),
                self.signed.clone(),
                0,
                bundle.record().schedule(),
                bundle.deployment().lead_bounds().bounds(),
                bundle.static_subtree().clone(),
                bundle.policy().internal_key(),
                bundle.contract(),
            ))
            .map_err(|e| Refusal::SponsoredRecovery(Box::new(e)))?;
            if recovered != edge {
                return Err(Refusal::SponsoredBoundary);
            }
            Ok(())
        }
        fn advance(
            &mut self,
            previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
        ) -> Result<Option<OperationStep>, Refusal> {
            match (self.pending.take(), previous) {
                (Some((step, action)), Some((case, response)))
                    if step.case() == case && &response.case == case =>
                {
                    if response.schema != NATIVE_PROTOCOL_SCHEMA {
                        return Err(Refusal::ResponseSchema {
                            offered: response.schema,
                        });
                    }
                    response.validate_shape().map_err(Refusal::ResponseShape)?;
                    self.exchanges.push((step.clone(), response.clone()));
                    self.settle(&step, action, response)?;
                }
                (None, None) if self.exchanges.is_empty() => {}
                _ => return Err(Refusal::UnexpectedResponse),
            }
            self.pending = self.queue.pop_front();
            Ok(self.pending.as_ref().map(|(step, _)| step.clone()))
        }
    }
    impl TargetOperationPlanner for Planner {
        fn next_step(
            &mut self,
            previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
        ) -> Result<Option<OperationStep>, PlanRefused> {
            if self.refusal.is_some() {
                return Err(PlanRefused);
            }
            self.advance(previous).map_err(|refusal| {
                self.refusal = Some(refusal);
                PlanRefused
            })
        }
    }

    fn only_coin(response: &NativeOperationResponse) -> Result<&FundedOutput, Refusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(Refusal::FundingNotAccepted {
                observed: response.observed_layer,
            });
        }
        let [coin] = response.funded_outputs.as_slice() else {
            return Err(Refusal::FundingCardinality {
                supplied: response.funded_outputs.len(),
            });
        };
        Ok(coin)
    }
    fn public_coin(coin: &FundedOutput) -> Result<PublicOutputView, Refusal> {
        Ok(PublicOutputView::new(
            outpoint_of(&coin.outpoint).ok_or(Refusal::FundingMismatch)?,
            AssetField::Explicit(asset_of(&coin.asset).ok_or(Refusal::FundingMismatch)?),
            ValueField::Explicit(coin.amount_satoshis),
            decode_hex(&coin.script).ok_or(Refusal::FundingMismatch)?,
        ))
    }
    pub(crate) fn checked_signature(
        step: &OperationStep,
        response: &NativeOperationResponse,
        coin: &FundedOutput,
    ) -> Result<SponsorSignature, Refusal> {
        let OperationSubject::SponsorSigning(subject) = step.subject() else {
            return Err(Refusal::UnexpectedResponse);
        };
        if response.observed_layer != ObservedOutcomeLayer::Accepted
            || response.signature_bound_to.as_deref()
                != Some(subject.finalized_transaction.as_slice())
            || response.sponsor_witness.len() != 2
            || response.sponsor_witness.iter().any(Vec::is_empty)
        {
            return Err(Refusal::SponsoredSignature);
        }
        let hash_type = match subject.sighash_profile {
            WireSighashProfile::AllInputsAllOutputs => 1,
            _ => return Err(Refusal::SponsoredSignature),
        };
        let mut program = vec![0, 20];
        program.extend(hash160(&response.sponsor_witness[1]));
        if decode_hex(&coin.script).as_ref() != Some(&program)
            || response.sponsor_witness[0].last() != Some(&hash_type)
        {
            return Err(Refusal::SponsoredSignature);
        }
        Ok(SponsorSignature::new(
            subject.finalized_transaction.clone(),
            response.sponsor_witness.clone(),
        ))
    }
    fn replace(
        original: &TargetTransaction,
        inputs: Vec<TargetInput>,
        outputs: Vec<TargetOutput>,
        witnesses: Vec<InputWitness>,
    ) -> Result<TargetTransaction, Refusal> {
        TargetTransaction::new(
            original.version(),
            inputs,
            outputs,
            original.lock_time(),
            witnesses,
        )
        .map_err(transaction)
    }
    fn move_value(output: &TargetOutput, delta: i64) -> Result<TargetOutput, Refusal> {
        let ValueField::Explicit(amount) = output.value() else {
            return Err(Refusal::FundingMismatch);
        };
        let amount = amount
            .checked_add_signed(delta)
            .ok_or(Refusal::FundingMismatch)?;
        Ok(TargetOutput::new(
            output.asset(),
            ValueField::Explicit(amount),
            output.nonce(),
            output.program().to_vec(),
        ))
    }
    fn resign(
        candidate: &TargetTransaction,
        finalized: &FinalizedMaturityAnnouncement,
        extra: Option<&FundedOutput>,
    ) -> Result<TargetTransaction, Refusal> {
        let target = closure_target().map_err(closure)?;
        let binding = finalized
            .construction()
            .validated_view()
            .view()
            .accepted_linked_bundle()
            .deployment()
            .operator();
        let mut genesis = *binding.deployment().genesis_id();
        genesis.reverse();
        let leaf = finalized.executing_leaf(&target);
        let mut spent = finalized.spent_outputs();
        if let Some(coin) = extra {
            spent.push(
                transaction::script_path_signing::SpentOutputCensusEntry::new(
                    AssetField::Explicit(asset_of(&coin.asset).ok_or(Refusal::FundingMismatch)?),
                    ValueField::Explicit(coin.amount_satoshis),
                    decode_hex(&coin.script).ok_or(Refusal::FundingMismatch)?,
                ),
            );
        }
        let request = transaction::operator_signing::OperatorSigningRequest::freeze(
            &target,
            binding,
            candidate.clone(),
            spent,
            transaction::script_path_signing::LiveDeployment::new(genesis),
            transaction::operator_signing::OperatorSigningInput::new(
                0,
                *leaf.tapleaf_hash(),
                leaf.leaf_version(),
                leaf.leaf_script().to_vec(),
                leaf.control_block().to_vec(),
            ),
            &OracleLiveCurve::new(target.clone()),
        )
        .map_err(|refusal| {
            transaction(
                TransactionRefusal::MaturityAnnouncementSigningRequestRefused {
                    refusal: Box::new(refusal),
                },
            )
        })?;
        let signature = OPERATOR_HANDLE
            .material()
            .map_err(Refusal::Signing)?
            .sign(request.message().with_vector_grown(), &[0; 32])
            .map_err(Refusal::Signing)?;
        let mut witnesses = candidate.witnesses().to_vec();
        let mut stack = witnesses[0].stack().to_vec();
        stack[6] = signature.to_vec();
        witnesses[0] = InputWitness::new(stack);
        replace(
            candidate,
            candidate.inputs().to_vec(),
            candidate.outputs().to_vec(),
            witnesses,
        )
    }
    /// Bind accepted identity, witness identity and exact mined bytes to a submission.
    /// # Errors
    /// Returns `SubmissionReadbackMismatch` for any identity or byte disagreement.
    pub fn check_readback(bytes: &[u8], response: &NativeOperationResponse) -> Result<(), Refusal> {
        let candidate = TargetTransaction::decode(bytes).map_err(transaction)?;
        let txid = Txid::from_internal(sha256(&sha256(&candidate.encode_without_witness())))
            .to_target_display();
        let wtxid = Txid::from_internal(sha256(&sha256(bytes))).to_target_display();
        let readback = response
            .mined_readback
            .as_ref()
            .ok_or(Refusal::SubmissionReadbackMismatch)?;
        if response.accepted_txid.as_deref() != Some(txid.as_str())
            || readback.transaction_id != txid
            || readback.witness_transaction_id != wtxid
            || readback.raw_transaction != bytes
            || readback.block_height == 0
            || decode_hex(&readback.block_hash).is_none_or(|hash| hash.len() != 32)
        {
            return Err(Refusal::SubmissionReadbackMismatch);
        }
        Ok(())
    }
    /// Replay every request before deriving the schedule's census and bindings.
    /// # Errors
    /// Returns the first exact-request, policy, signing, boundary or readback refusal.
    /// # Panics
    /// Panics only if the fixed architecture omits its singleton, which its census cannot arrange.
    pub fn replay(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        exchanges: &[(OperationStep, NativeOperationResponse)],
    ) -> Result<Planner, Refusal> {
        replay_from(Planner::new(identity, branch)?, exchanges, |_| {})
    }

    fn replay_from(
        mut planner: Planner,
        exchanges: &[(OperationStep, NativeOperationResponse)],
        mut checkpoint: impl FnMut(&Planner),
    ) -> Result<Planner, Refusal> {
        let start = planner.exchanges.len();
        let mut next = if start == 0 {
            planner.advance(None)?
        } else {
            planner.pending.as_ref().map(|(step, _)| step.clone())
        };
        for (position, (step, response)) in exchanges.iter().enumerate().skip(start) {
            if next.as_ref() != Some(step) {
                return Err(Refusal::TranscriptStepMismatch { position });
            }
            next = planner.advance(Some((step.case(), response)))?;
            checkpoint(&planner);
        }
        planner.complete()?;
        Ok(planner)
    }

    #[cfg(test)]
    pub(crate) fn replay_through_checkpoints(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        exchanges: &[(OperationStep, NativeOperationResponse)],
    ) -> Result<Planner, Refusal> {
        let seed = cached_prefix(
            &REPLAY_CHECKPOINTS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            &identity,
            branch,
            exchanges,
        );
        let planner = seed.map_or_else(|| Planner::new(identity, branch), Ok)?;
        replay_from(planner, exchanges, |planner| {
            let checkpoint = planner.clone();
            REPLAY_CHECKPOINTS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(checkpoint);
        })
    }

    #[cfg(test)]
    static REPLAY_CHECKPOINTS: std::sync::Mutex<Vec<Planner>> = std::sync::Mutex::new(Vec::new());

    #[cfg(test)]
    fn cached_prefix(
        cache: &[Planner],
        identity: &CandidateDeploymentIdentity,
        branch: BranchContext,
        exchanges: &[(OperationStep, NativeOperationResponse)],
    ) -> Option<Planner> {
        cache
            .iter()
            .filter(|seed| {
                &seed.identity == identity
                    && seed.branch == branch
                    && exchanges.starts_with(&seed.exchanges)
            })
            .max_by_key(|seed| seed.exchanges.len())
            .cloned()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::maturity_closure::{MaturityDeploymentParameters, maturity_sources_with};
    use crate::maturity_negative_half::{MaturityNegativeHalfGap, STILL_REQUIRED};
    use crate::maturity_safety::MaturitySafetySection;
    use std::fmt::Write as _;
    use std::io::Write as _;
    use std::sync::OnceLock;
    use target_elements_conformance::protocol::WireOutpoint;
    use target_elements_conformance::protocol::{MinedFundingReadback, NativeResourceObservation};
    use transaction::operator_signing::ScriptPathSignatureVerifier;

    fn copy_plan(plan: &MaturityAnnouncementPlanner) -> MaturityAnnouncementPlanner {
        MaturityAnnouncementPlanner {
            schedule: plan.schedule,
            lead: plan.lead,
            identity: plan.identity.clone(),
            branch: plan.branch,
            position: plan.position,
            pending: plan.pending.clone(),
            exchanges: plan.exchanges.clone(),
            bundle: plan.bundle.clone(),
            amount: plan.amount,
            asset: plan.asset.clone(),
            predecessor: plan.predecessor,
            announcement: plan.announcement.clone(),
            submission: plan.submission.clone(),
            readback: plan.readback.clone(),
            refusal: plan.refusal.clone(),
        }
    }

    fn planner() -> MaturityAnnouncementPlanner {
        static SEED: OnceLock<MaturityAnnouncementPlanner> = OnceLock::new();
        copy_plan(SEED.get_or_init(|| {
            let mut genesis = [0x22; 32];
            genesis[0] = 0x01;
            genesis[31] = 0xfe;
            MaturityAnnouncementPlanner::new(
                CandidateDeploymentIdentity::new([0x17; 32], genesis).expect("identity"),
                BranchContext::new([0x41; 32], 7).expect("branch"),
                crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
            )
            .expect("public planner")
        }))
    }

    fn hex(bytes: &[u8]) -> String {
        let mut result = String::new();
        for byte in bytes {
            write!(result, "{byte:02x}").expect("String write");
        }
        result
    }

    fn blank(step: &OperationStep) -> NativeOperationResponse {
        NativeOperationResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: step.case().clone(),
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: None,
            issued_asset: None,
            funded_outputs: Vec::new(),
            confidential_funded_outputs: Vec::new(),
            mined_readback: None,
            accepted_txid: None,
            sponsor_witness: Vec::new(),
            script_path_witness: Vec::new(),
            signer_public_key: None,
            signed_profile: None,
            signing_genesis: None,
            signature_bound_to: None,
            resources: NativeResourceObservation::default(),
        }
    }

    fn sponsored_policy() -> serde_json::Value {
        serde_json::json!({"relay":"0.00000100", "mempool":"0.00000100", "block":"0.00000100", "unit":"BTC/kvB",
            "block_identifier":"DEFAULT_BLOCK_MIN_TX_FEE", "relay_provenance":"reported by the node",
            "block_provenance":"declared by the pinned source, no override", "start_arguments":["elementsd", "-daemon=0"]})
    }

    fn sponsored_script() -> sponsored::Planner {
        let seed =
            crate::maturity_corpus::maturity_variable_run_of_record().expect("historical bindings");
        sponsored_script_for(seed.evidence().identity().clone(), seed.evidence().branch())
    }

    pub fn sponsored_script_for(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
    ) -> sponsored::Planner {
        sponsored_script_with_zero(identity, branch, false)
    }

    fn sponsored_script_with_zero(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        accept_zero: bool,
    ) -> sponsored::Planner {
        sponsored_script_with_retries(identity, branch, accept_zero, false)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "The synthetic schedule supplies each operation response and bounded undecided scenarios."
    )]
    pub fn sponsored_script_with_retries(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        accept_zero: bool,
        retries: bool,
    ) -> sponsored::Planner {
        type Key = (CandidateDeploymentIdentity, BranchContext, bool, bool);
        static CACHE: std::sync::Mutex<Vec<(Key, sponsored::Planner)>> =
            std::sync::Mutex::new(Vec::new());
        let key = (identity.clone(), branch, accept_zero, retries);
        let cached = CACHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .find(|(stored, _)| stored == &key)
            .map(|(_, plan)| plan.clone());
        if let Some(plan) = cached {
            return plan;
        }
        let mut plan = sponsored::Planner::new(identity, branch).expect("sponsored plan");
        let mut next = plan.next_step(None).expect("start");
        let mut serial = 0_u64;
        let mut attempts = std::collections::HashMap::new();
        while let Some(step) = next {
            serial += 1;
            assert!(serial < 250);
            let mut response = blank(&step);
            let mut route = "setup";
            let mut relay = serde_json::Value::Null;
            let mut reason = serde_json::Value::Null;
            match step.subject() {
                OperationSubject::Funding(subject) => {
                    response = funding_response(&step);
                    response.funded_outputs[0].outpoint.txid = format!("{serial:064x}");
                    if let Some(asset) = &subject.asset {
                        response.funded_outputs[0].asset = asset.clone();
                    }
                }
                OperationSubject::SponsorFunding(subject) => {
                    if subject.amount_per_sponsor_output == 0 && !accept_zero {
                        response.observed_layer =
                            ObservedOutcomeLayer::ConsensusRejectionBeforeScript;
                        reason = serde_json::json!("bad-txns-in-ne-out");
                    } else {
                        response.funded_outputs.push(FundedOutput {
                            outpoint: WireOutpoint {
                                txid: format!("{serial:064x}"),
                                vout: 0,
                            },
                            asset: "77".repeat(32),
                            amount_satoshis: subject.amount_per_sponsor_output,
                            script: format!("0014{}", hex(&sponsored::hash160(&[2; 33]))),
                        });
                    }
                }
                OperationSubject::SponsorSigning(subject) => {
                    response.sponsor_witness = vec![vec![0x30, 1, 1], vec![2; 33]];
                    response.signature_bound_to = Some(subject.finalized_transaction.clone());
                    response.resources.transaction_weight = Some(40_000);
                }
                OperationSubject::Submission(_) => {
                    let control = step.case().step.contains(":control:")
                        || step.case().step.contains(":zero-member:");
                    let layer = if control {
                        ObservedOutcomeLayer::Accepted
                    } else if step.case().step.contains("balanced-state-corruption") {
                        ObservedOutcomeLayer::ConsensusRejectionBeforeScript
                    } else {
                        ObservedOutcomeLayer::ScriptPathRejection
                    };
                    response = submission_response(&step, layer);
                    response.resources.transaction_weight = Some(39_992);
                    let template = step.case().step.starts_with("sponsored-")
                        || step.case().step.starts_with("reauthorized-");
                    relay = serde_json::json!(control && template);
                    if relay == false {
                        reason = serde_json::json!(
                            sponsored::NEGATIVES
                                .iter()
                                .position(|name| step.case().step.contains(name))
                                .map_or("min relay fee not met", |index| {
                                    sponsored::NEGATIVE_REASONS[index]
                                })
                        );
                    }
                    route = if !control {
                        "refused"
                    } else if template {
                        "relay-and-template"
                    } else {
                        "direct-block"
                    };
                }
                _ => panic!("unexpected subject"),
            }
            let facts = if route == "setup" && !reason.is_null() {
                let reason = reason.take();
                serde_json::json!({"relay_detail":reason, "outcome":{"verdict_reason":reason}})
            } else if route == "refused" {
                serde_json::json!({"verdict_reason":reason})
            } else {
                response
                    .mined_readback
                    .as_ref()
                    .map_or(serde_json::Value::Null, |readback| {
                        serde_json::json!({"txid":readback.transaction_id, "load_five_minutes":1000,
                    "mempool_size":999, "mempool_minimum":"0.00000100",
                    "produced_block":readback.block_hash, "included_block":readback.block_hash,
                    "inclusion_in_produced":true, "template":{"source":"assembly-read",
                    "weight":4_000_000, "member":true, "txids":[readback.transaction_id]}})
                    })
            };
            response.observed_detail = Some(
                serde_json::json!({"schema":1, "policy":sponsored_policy(), "route":route,
                "relay_allowed":relay, "relay_detail":reason, "detail":facts})
                .to_string(),
            );
            if retries && step.case().step.contains(sponsored::NEGATIVES[2]) {
                let attempt = attempts.entry("negative-retry".to_owned()).or_insert(0);
                if *attempt == 0 {
                    response = blank(&step);
                    response.observed_layer = ObservedOutcomeLayer::ExecutorInfrastructureFailure;
                    response.observed_detail = Some(serde_json::json!({"outcome":"undecided", "reason":"call-budget-expired", "wall_seconds":240}).to_string());
                }
                *attempt += 1;
            }
            if retries && step.case().step.contains(":control:") {
                let row = step.case().step.split(':').next().expect("row");
                let attempt = attempts.entry(row.to_owned()).or_insert(0);
                let index = sponsored::CASES
                    .iter()
                    .position(|name| *name == row)
                    .expect("case");
                let kind = [0, 1, 2, 9, 10, 11]
                    .iter()
                    .position(|case| *case == index)
                    .unwrap_or(6);
                if kind < 6 && *attempt < if kind == 5 { 2 } else { 1 } {
                    response = blank(&step);
                    response.observed_layer = ObservedOutcomeLayer::ExecutorInfrastructureFailure;
                    response.observed_detail = Some(serde_json::json!({"outcome":"undecided", "wall_seconds":240,
                        "reason":(["call-budget-expired", "included-before-template", "included-before-direct-block", "included-during-direct-block", "undecided-by-displacement", "included-outside-direct-block"][kind])}).to_string());
                }
                *attempt += 1;
            }
            next = plan
                .next_step(Some((step.case(), &response)))
                .unwrap_or_else(|_| panic!("{}: {:?}", step.case().step, plan.refusal()));
        }
        plan.complete().expect("complete");
        let entry = (key, plan.clone());
        CACHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(entry);
        plan
    }

    #[test]
    fn sponsored_schedule_replays_requests_routes_and_readbacks() {
        let started = std::time::Instant::now();
        assert_eq!(
            hex(&sponsored::hash160(b"")),
            "b472a266d0bd89c13706a4132ccfb16f7c3b9fcb"
        );
        let plan = sponsored_script();
        let (step, response) = plan
            .exchanges()
            .iter()
            .find(|(step, _)| matches!(step.subject(), OperationSubject::SponsorSigning(_)))
            .expect("signing");
        let OperationSubject::SponsorSigning(subject) = step.subject() else {
            unreachable!()
        };
        let coin = plan
            .exchanges()
            .iter()
            .flat_map(|(_, response)| &response.funded_outputs)
            .find(|coin| coin.outpoint == subject.sponsor_outpoint)
            .expect("funded coin");
        for item in [0, 1] {
            let mut changed = response.clone();
            let byte = changed.sponsor_witness[item].last_mut().expect("witness");
            *byte ^= 1;
            assert_eq!(
                sponsored::checked_signature(step, &changed, coin).err(),
                Some(Refusal::SponsoredSignature)
            );
        }
        let mut cases = std::collections::HashSet::new();
        for (step, _) in plan.exchanges() {
            assert!(
                cases.insert(step.case().clone()),
                "unique operation identity"
            );
        }
        let seed =
            crate::maturity_corpus::maturity_variable_run_of_record().expect("historical bindings");
        let replay = sponsored::replay_through_checkpoints(
            seed.evidence().identity().clone(),
            seed.evidence().branch(),
            plan.exchanges(),
        )
        .expect("replay");
        assert_eq!(replay.census(), plan.census());
        assert_eq!(plan.census()[3..], [17, 12, 5]);
        let zero = sponsored_script_with_zero(
            seed.evidence().identity().clone(),
            seed.evidence().branch(),
            true,
        );
        assert_eq!(zero.census()[3..], [18, 13, 5]);
        writeln!(
            std::io::stderr().lock(),
            "\nRUN-REPORT sponsored_schedule_replays_requests_routes_and_readbacks wall={:.3}s",
            started.elapsed().as_secs_f64()
        )
        .expect("timing output");
    }

    #[test]
    fn sponsored_schedule_is_the_one_its_replayed_exchanges_were_linked_under() {
        let corpus =
            crate::maturity_corpus::maturity_sponsored_run_of_record().expect("admitted archive");
        let stored = corpus.evidence.schedule();
        assert_eq!(stored, corpus.evidence.linked_schedule());
        assert_eq!(
            stored,
            Some(tapscript::StateWitnessSchedule::VariableMetadata)
        );
        let plan = sponsored_script();
        assert_eq!(plan.schedule(), plan.linked_schedule());
    }

    #[test]
    fn sponsored_replay_refuses_changed_policy_route_and_readback() {
        let started = std::time::Instant::now();
        let plan = sponsored_script();
        for mutation in 0..5 {
            let seed = crate::maturity_corpus::maturity_variable_run_of_record()
                .expect("historical bindings");
            let mut exchanges = plan.exchanges().to_vec();
            let (_, response) = exchanges
                .iter_mut()
                .find(|(step, _)| {
                    step.case()
                        .step
                        .starts_with("sponsored-without-change:control:")
                })
                .expect("control");
            let mut detail: serde_json::Value =
                serde_json::from_str(response.observed_detail.as_deref().expect("detail"))
                    .expect("JSON");
            let expected = match mutation {
                0 => {
                    detail["policy"]["block_provenance"] =
                        serde_json::json!("reported by the node");
                    Refusal::SponsoredPolicy
                }
                1 => {
                    detail["route"] = serde_json::json!("direct-block");
                    detail["relay_allowed"] = serde_json::json!(false);
                    detail["relay_detail"] = serde_json::json!("refused");
                    Refusal::SponsoredRoute
                }
                2 => {
                    response
                        .mined_readback
                        .as_mut()
                        .expect("readback")
                        .raw_transaction[0] ^= 1;
                    Refusal::SubmissionReadbackMismatch
                }
                3 => {
                    detail["detail"]["inclusion_in_produced"] = serde_json::json!(false);
                    detail["detail"]["produced_block"] = serde_json::json!("foreign");
                    Refusal::SponsoredRoute
                }
                _ => {
                    detail["detail"]["template"]["member"] = serde_json::json!(false);
                    Refusal::SponsoredRoute
                }
            };
            response.observed_detail = Some(detail.to_string());
            assert_eq!(
                sponsored::replay_through_checkpoints(
                    seed.evidence().identity().clone(),
                    seed.evidence().branch(),
                    &exchanges
                )
                .err(),
                Some(expected)
            );
        }
        writeln!(
            std::io::stderr().lock(),
            "\nRUN-REPORT sponsored_replay_refuses_changed_policy_route_and_readback wall={:.3}s",
            started.elapsed().as_secs_f64()
        )
        .expect("timing output");
    }

    #[test]
    fn sponsored_fee_uses_measured_size_and_the_larger_floor() {
        let mut value = sponsored_policy();
        value["mempool"] = serde_json::json!("0.00002000");
        let policy = sponsored::Policy::parse(value.clone()).expect("policy");
        assert_eq!(policy.fee(4001), Ok(4004));
        assert_eq!(policy.fee(0), Err(Refusal::SponsoredFee));
        value["start_arguments"]
            .as_array_mut()
            .expect("args")
            .push(serde_json::json!("-blockmintxfee=0"));
        assert_eq!(
            sponsored::Policy::parse(value),
            Err(Refusal::SponsoredPolicy)
        );
    }

    fn funding_response(step: &OperationStep) -> NativeOperationResponse {
        let OperationSubject::Funding(subject) = step.subject() else {
            panic!("funding")
        };
        let asset = format!("01{}fe", "55".repeat(30));
        let mut response = blank(step);
        if subject.issue_asset {
            response.issued_asset = Some(asset.clone());
        }
        response.funded_outputs.push(FundedOutput {
            outpoint: WireOutpoint {
                txid: format!("02{}fd", "66".repeat(30)),
                vout: if subject.issue_asset { 3 } else { 7 },
            },
            asset,
            amount_satoshis: subject.amount_per_output,
            script: hex(&subject.output_program),
        });
        response.validate_shape().expect("funding shape");
        response
    }

    fn issued() -> (MaturityAnnouncementPlanner, OperationStep) {
        static ISSUED: OnceLock<(MaturityAnnouncementPlanner, OperationStep)> = OnceLock::new();
        let (plan, step) = ISSUED.get_or_init(|| {
            let mut plan = planner();
            let issue = plan.next_step(None).expect("issue").expect("step");
            let response = funding_response(&issue);
            let funding = plan
                .next_step(Some((issue.case(), &response)))
                .expect("funding")
                .expect("step");
            (plan, funding)
        });
        (copy_plan(plan), step.clone())
    }

    fn funded() -> (MaturityAnnouncementPlanner, OperationStep) {
        static FUNDED: OnceLock<(MaturityAnnouncementPlanner, OperationStep)> = OnceLock::new();
        let (plan, step) = FUNDED.get_or_init(|| {
            let (mut plan, funding) = issued();
            let response = funding_response(&funding);
            let submission = plan
                .next_step(Some((funding.case(), &response)))
                .expect("submission")
                .expect("step");
            (plan, submission)
        });
        (copy_plan(plan), step.clone())
    }

    fn variable_planner() -> MaturityAnnouncementPlanner {
        static SEED: OnceLock<MaturityAnnouncementPlanner> = OnceLock::new();
        copy_plan(SEED.get_or_init(|| {
            let historical = planner();
            MaturityAnnouncementPlanner::new(
                historical.identity,
                historical.branch,
                MaturityWitnessSelection::Retained(
                    tapscript::StateWitnessSchedule::VariableMetadata,
                ),
            )
            .expect("variable planner")
        }))
    }

    fn variable_funded() -> (MaturityAnnouncementPlanner, OperationStep) {
        static FUNDED: OnceLock<(MaturityAnnouncementPlanner, OperationStep)> = OnceLock::new();
        let (plan, step) = FUNDED.get_or_init(|| {
            let mut plan = variable_planner();
            let issue = plan.next_step(None).expect("issue").expect("step");
            let issue_response = funding_response(&issue);
            let funding = plan
                .next_step(Some((issue.case(), &issue_response)))
                .expect("funding")
                .expect("step");
            let funding_response = funding_response(&funding);
            let submission = plan
                .next_step(Some((funding.case(), &funding_response)))
                .expect("submission")
                .expect("step");
            (plan, submission)
        });
        (copy_plan(plan), step.clone())
    }

    fn variable_settled(layer: ObservedOutcomeLayer) -> MaturityAnnouncementPlanner {
        let (mut plan, step) = variable_funded();
        let response = submission_response(&step, layer);
        assert_eq!(plan.next_step(Some((step.case(), &response))), Ok(None));
        plan
    }

    fn variable_evidence(plan: &MaturityAnnouncementPlanner) -> MaturityNativeEvidence {
        MaturityNativeEvidence::from_transcript(
            plan.identity.clone(),
            plan.branch,
            MaturityWitnessSelection::Retained(tapscript::StateWitnessSchedule::VariableMetadata),
            plan.completed_transcript().expect("settled exchanges"),
        )
        .expect("variable replay")
    }

    fn submission_response(
        step: &OperationStep,
        layer: ObservedOutcomeLayer,
    ) -> NativeOperationResponse {
        let mut response = blank(step);
        response.observed_layer = layer;
        let OperationSubject::Submission(subject) = step.subject() else {
            panic!("submission")
        };
        if layer == ObservedOutcomeLayer::Accepted {
            let candidate = TargetTransaction::decode(&subject.transaction_bytes).expect("decode");
            let txid = Txid::from_internal(sha256(&sha256(&candidate.encode_without_witness())))
                .to_target_display();
            response.accepted_txid = Some(txid.clone());
            response.mined_readback = Some(MinedFundingReadback {
                transaction_id: txid,
                witness_transaction_id: Txid::from_internal(sha256(&sha256(
                    &subject.transaction_bytes,
                )))
                .to_target_display(),
                block_hash: "77".repeat(32),
                block_height: 11,
                raw_transaction: subject.transaction_bytes.clone(),
            });
        } else {
            response.observed_detail = Some("scripted observation, no node was invoked".to_owned());
        }
        response.validate_shape().expect("submission shape");
        response
    }

    fn assert_terminal(
        plan: &mut MaturityAnnouncementPlanner,
        step: &OperationStep,
        response: &NativeOperationResponse,
        expected: Refusal,
    ) {
        assert_eq!(
            plan.next_step(Some((step.case(), response))),
            Err(PlanRefused)
        );
        assert_eq!(plan.refusal(), Some(&expected));
        assert_eq!(plan.pending, None);
        assert_eq!(plan.next_step(None), Err(PlanRefused));
        assert_eq!(
            plan.next_step(Some((step.case(), response))),
            Err(PlanRefused)
        );
        assert_eq!(plan.completed_transcript().err(), Some(expected));
    }

    #[test]
    fn scripted_accepted_responses_follow_the_exact_sequence() {
        let mut plan = planner();
        let mut current = plan.next_step(None).expect("start");
        let mut names = Vec::new();
        while let Some(step) = current {
            names.push(step.case().step.clone());
            let response = match step.subject() {
                OperationSubject::Funding(_) => funding_response(&step),
                OperationSubject::Submission(_) => {
                    submission_response(&step, ObservedOutcomeLayer::Accepted)
                }
                _ => panic!("only funding and submission"),
            };
            current = plan
                .next_step(Some((step.case(), &response)))
                .expect("advance");
        }
        assert_eq!(names, ANNOUNCEMENT_STEPS);
        let exchanges = plan.completed_transcript().expect("complete");
        assert_eq!(exchanges.len(), ANNOUNCEMENT_STEPS.len());
        for ((step, response), name) in exchanges.iter().zip(ANNOUNCEMENT_STEPS) {
            assert_eq!(step.case().step, name);
            assert_eq!(response.case, *step.case());
            assert_eq!(response.observed_layer, ObservedOutcomeLayer::Accepted);
        }
        assert_eq!(plan.next_step(None).expect("remains complete"), None);
    }

    #[test]
    fn funding_subjects_use_the_bootstrap_then_the_runtime_link() {
        let mut bootstrap = planner();
        let issue = bootstrap.next_step(None).expect("issue").expect("step");
        let OperationSubject::Funding(subject) = issue.subject() else {
            panic!("funding")
        };
        assert!(subject.issue_asset);
        assert_eq!(subject.asset, None);
        assert_eq!(subject.outputs, 1);
        assert_eq!(
            subject.amount_per_output,
            singleton_declaration()
                .expect("singleton")
                .fixed_amount()
                .get()
        );
        assert_eq!(
            subject.output_program,
            predecessor_program(bootstrap.bundle())
        );
        let (runtime, funding) = issued();
        let OperationSubject::Funding(subject) = funding.subject() else {
            panic!("funding")
        };
        let issued = funding_response(&issue).issued_asset.expect("issued asset");
        let asset = asset_of(&issued).expect("asset");
        let parameters = MaturityDeployment::PublishedSignerHeld
            .parameters()
            .expect("parameters");
        let independently_linked = maturity_sources_with(
            MaturityDeploymentParameters::new(
                *asset.internal(),
                *parameters.operator_key(),
                parameters.lead(),
            ),
            crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
        )
        .expect("sources")
        .link(&OracleStateCurve)
        .expect("link");
        assert!(!subject.issue_asset);
        assert_eq!(subject.asset.as_deref(), Some(issued.as_str()));
        assert_eq!(subject.outputs, 1);
        assert_eq!(subject.amount_per_output, 1);
        assert_eq!(
            subject.output_program,
            predecessor_program(&independently_linked)
        );
        assert_ne!(
            subject.output_program,
            predecessor_program(bootstrap.bundle())
        );
        assert_eq!(runtime.bundle().deployment().identity(), &runtime.identity);
        assert_eq!(
            runtime.bundle().deployment().operator().deployment(),
            &runtime.identity
        );
        let target = closure_target().expect("target");
        assert_eq!(
            linked_announcement_bytes(runtime.bundle(), &target).expect("runtime leaf"),
            linked_announcement_bytes(&independently_linked, &target).expect("independent leaf")
        );
    }

    fn independent_announcement(
        plan: &MaturityAnnouncementPlanner,
        coin: &FundedOutput,
    ) -> FinalizedMaturityAnnouncement {
        let target = closure_target().expect("target");
        let bundle = plan.bundle();
        let predecessor = bundle.instances().first().expect("predecessor");
        let metadata = predecessor.metadata();
        let view = PublicMaturityStateView::new([
            MaturityViewStatement::CurrentStateOutpoint(
                outpoint_of(&coin.outpoint).expect("outpoint"),
            ),
            MaturityViewStatement::AssetAndAmount(
                AssetField::Explicit(asset_of(&coin.asset).expect("asset")),
                ValueField::Explicit(coin.amount_satoshis),
            ),
            MaturityViewStatement::PredecessorMetadata(metadata.semantic),
            MaturityViewStatement::PredecessorRepresentationNonce(metadata.representation),
            MaturityViewStatement::CurrentRootBinding(plan.branch),
            MaturityViewStatement::PredecessorProgram(decode_hex(&coin.script).expect("program")),
            MaturityViewStatement::AcceptedLinkedBundle(Box::new(bundle.clone())),
        ])
        .expect("view")
        .validate(&target, &OracleStateCurve)
        .expect("validated view");
        let abi = derive_maturity_announcement_abi(&target, &view).expect("ABI");
        let (earliest, latest) = bundle
            .deployment()
            .lead_bounds()
            .bounds()
            .window(metadata.semantic.cycle)
            .expect("window");
        let announced = Cycle::new(earliest.get().checked_add(1).expect("next cycle"));
        assert!(announced <= latest);
        let request = MaturityAnnouncementRequest::new(
            announced,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
        )
        .expect("request");
        finalize_maturity_announcement(
            construct_maturity_announcement(&target, &abi, &view, &request, &OracleStateCurve)
                .expect("construction"),
        )
    }

    fn independent_submission(finalized: &FinalizedMaturityAnnouncement) -> Vec<u8> {
        let protected = finalized.protected_bytes().to_vec();
        let target = closure_target().expect("target");
        let started =
            OperatorSigningStarted::open(finalized, &target, &OracleLiveCurve::new(target.clone()))
                .expect("freeze");
        let request = started.request();
        let signature = OPERATOR_HANDLE
            .material()
            .expect("material")
            .sign(request.message().with_vector_grown(), &[0; 32])
            .expect("sign");
        OperatorVerifier
            .verify(
                request.binding().key().bytes(),
                request.message().with_vector_grown(),
                &signature,
            )
            .expect("verify");
        let answer = OperatorSigningResponse::new(
            request.input_index(),
            signature.to_vec(),
            OPERATOR_SIGHASH_TYPE_BYTE,
            request.frozen_bytes().to_vec(),
            request.binding().key().clone(),
            request.binding().deployment().clone(),
            request.binding().capability_revision(),
        );
        let mut registry = OperatorRightRegistry::default();
        let right = registry
            .issue(
                started.construction_right_scope(),
                started.protected_bytes(),
            )
            .expect("right");
        let authorized = started
            .authorize(&mut registry, right, [answer], &OperatorVerifier)
            .expect("authorize");
        assert_eq!(authorized.protected_bytes(), protected);
        let ready = authorized
            .bind_for_submission(&target)
            .expect("submit ready");
        assert_eq!(ready.protected_bytes(), protected);
        assert_eq!(finalized.protected_bytes(), protected);
        assert_ne!(ready.bytes(), protected);
        ready.bytes().to_vec()
    }

    #[test]
    fn submission_is_independently_rebuilt_over_the_reported_outpoint() {
        let (plan, step) = funded();
        let (_, funding) = issued();
        let response = funding_response(&funding);
        let [coin] = response.funded_outputs.as_slice() else {
            panic!("one coin")
        };
        let independent = independent_announcement(&plan, coin);
        let ready_bytes = independent_submission(&independent);
        let finalized = plan.announcement().expect("finalized");
        assert_eq!(finalized.protected_bytes(), independent.protected_bytes());
        assert_eq!(
            finalized.protected().inputs()[0].outpoint(),
            outpoint_of(&coin.outpoint).expect("outpoint")
        );
        assert_eq!(
            plan.predecessor(),
            Some(finalized.protected().inputs()[0].outpoint())
        );
        assert_eq!(
            finalized
                .construction()
                .validated_view()
                .view()
                .current_root_binding(),
            plan.branch
        );
        let OperationSubject::Submission(subject) = step.subject() else {
            panic!("submission")
        };
        let TargetSubmissionSubject { transaction_bytes } = subject.as_ref();
        assert_eq!(transaction_bytes, &ready_bytes);
        assert_eq!(plan.submission_bytes(), Some(ready_bytes.as_slice()));
        let mut changed = coin.clone();
        changed.outpoint.vout += 1;
        let other = independent_announcement(&plan, &changed);
        assert_ne!(other.protected_bytes(), finalized.protected_bytes());
        assert_ne!(independent_submission(&other), ready_bytes);
    }

    #[test]
    fn declared_layers_match_the_matrix_and_operator_infrastructure() {
        let rows = crate::maturity_safety::rows();
        for name in ANNOUNCEMENT_STEPS {
            if let Some(layer) =
                expected_layer(name, tapscript::StateWitnessSchedule::WholeMetadata)
            {
                let matching: Vec<_> = rows
                    .iter()
                    .filter(|row| {
                        row.section() == MaturitySafetySection::Positive && row.name() == name
                    })
                    .collect();
                assert_eq!(matching.len(), 1);
                assert!(crate::observed_boundary::matches_boundary(
                    matching[0].refusing_layer().expect("declared boundary"),
                    layer
                ));
            }
        }
        assert_eq!(
            expected_layer(
                "sponsorless",
                tapscript::StateWitnessSchedule::WholeMetadata
            ),
            Some(ObservedOutcomeLayer::RelayPolicyRejection)
        );
        for (name, operator) in ANNOUNCEMENT_STEPS[..2]
            .iter()
            .zip(&crate::maturity_operator::NATIVE_STEPS[..2])
        {
            assert_eq!(
                expected_layer(name, tapscript::StateWitnessSchedule::WholeMetadata),
                None
            );
            assert_eq!(
                expected_layer(name, tapscript::StateWitnessSchedule::WholeMetadata),
                crate::maturity_operator::expected_layer(operator)
            );
        }
        assert_eq!(
            expected_layer("unknown", tapscript::StateWitnessSchedule::WholeMetadata),
            None
        );
    }

    const NON_ACCEPTED: [ObservedOutcomeLayer; 6] = [
        ObservedOutcomeLayer::FixtureConstructionFailure,
        ObservedOutcomeLayer::ExecutorInfrastructureFailure,
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        ObservedOutcomeLayer::ScriptPathRejection,
        ObservedOutcomeLayer::KeyPathRejection,
        ObservedOutcomeLayer::RelayPolicyRejection,
    ];

    #[test]
    fn each_funding_step_refuses_every_nonaccepted_layer() {
        for layer in NON_ACCEPTED {
            let mut initial = planner();
            let step = initial.next_step(None).expect("start").expect("step");
            for (mut plan, step) in vec![(initial, step), issued()].into_boxed_slice() {
                let mut response = blank(&step);
                response.observed_layer = layer;
                response.validate_shape().expect("refusal shape");
                assert_terminal(
                    &mut plan,
                    &step,
                    &response,
                    Refusal::FundingNotAccepted { observed: layer },
                );
            }
        }
    }

    #[test]
    fn submission_observations_are_retained_without_rewriting_the_layer() {
        for layer in NON_ACCEPTED
            .into_iter()
            .chain([ObservedOutcomeLayer::Accepted])
        {
            let (mut plan, step) = funded();
            let response = submission_response(&step, layer);
            assert_eq!(
                plan.next_step(Some((step.case(), &response)))
                    .expect("settle"),
                None
            );
            let exchanges = plan.completed_transcript().expect("complete");
            assert_eq!(exchanges.last(), Some(&(step, response)));
            assert_eq!(
                expected_layer("sponsorless", plan.schedule()),
                Some(ObservedOutcomeLayer::RelayPolicyRejection)
            );
        }
    }

    #[test]
    fn incomplete_runs_have_no_completed_transcript() {
        for plan in vec![planner(), issued().0, funded().0].into_boxed_slice() {
            assert_eq!(
                plan.completed_transcript().err(),
                Some(Refusal::IncompleteTranscript)
            );
        }
    }

    #[test]
    fn missing_unsolicited_and_mismatched_responses_are_terminal() {
        let (mut plan, step) = issued();
        assert_eq!(plan.next_step(None), Err(PlanRefused));
        assert_eq!(plan.refusal(), Some(&Refusal::UnexpectedResponse));
        assert_eq!(plan.next_step(None), Err(PlanRefused));
        let response = funding_response(&step);
        assert_terminal(
            &mut planner(),
            &step,
            &response,
            Refusal::UnexpectedResponse,
        );
        let (mut plan, step) = issued();
        let mut response = funding_response(&step);
        response.case.step = "other".to_owned();
        assert_terminal(&mut plan, &step, &response, Refusal::UnexpectedResponse);
        let (mut plan, step) = issued();
        let response = funding_response(&step);
        let other = OperationStep::new("other", step.subject().clone());
        assert_terminal(&mut plan, &other, &response, Refusal::UnexpectedResponse);
    }

    #[test]
    fn another_schema_and_malformed_shapes_are_terminal() {
        let (mut plan, step) = issued();
        let mut response = funding_response(&step);
        response.schema = NATIVE_PROTOCOL_SCHEMA + 1;
        assert_terminal(
            &mut plan,
            &step,
            &response,
            Refusal::ResponseSchema {
                offered: response.schema,
            },
        );
        let (mut plan, step) = issued();
        let response = blank(&step);
        assert_terminal(
            &mut plan,
            &step,
            &response,
            Refusal::ResponseShape(ResponseShapeDefect::AcceptedOperationOmitsObservation),
        );
    }

    #[test]
    fn issuance_requires_a_decodable_asset() {
        for asset in [None, Some("not-an-asset".to_owned())] {
            let mut plan = planner();
            let step = plan.next_step(None).expect("start").expect("step");
            let mut response = funding_response(&step);
            response.issued_asset = asset;
            assert_terminal(
                &mut plan,
                &step,
                &response,
                Refusal::IssuedAssetMissingOrInvalid,
            );
        }
    }

    #[test]
    fn surplus_funding_and_invalid_outpoints_are_terminal() {
        let (mut plan, step) = issued();
        let mut response = funding_response(&step);
        response
            .funded_outputs
            .push(response.funded_outputs[0].clone());
        assert_terminal(
            &mut plan,
            &step,
            &response,
            Refusal::FundingCardinality { supplied: 2 },
        );
        let (mut plan, step) = issued();
        let mut response = funding_response(&step);
        response.funded_outputs[0].outpoint.txid = "not-a-txid".to_owned();
        assert_terminal(&mut plan, &step, &response, Refusal::FundedOutpointInvalid);
    }

    #[test]
    fn funding_must_match_every_requested_coin_term() {
        let changes: [fn(&mut NativeOperationResponse); 4] = [
            |response| response.funded_outputs[0].asset = "99".repeat(32),
            |response| response.funded_outputs[0].amount_satoshis += 1,
            |response| response.funded_outputs[0].script = "51".to_owned(),
            |response| response.issued_asset = Some("99".repeat(32)),
        ];
        for change in changes {
            let (mut plan, step) = issued();
            let mut response = funding_response(&step);
            change(&mut response);
            assert_terminal(&mut plan, &step, &response, Refusal::FundingMismatch);
        }
    }

    #[test]
    fn accepted_submissions_owe_identity_and_readback_but_relay_refusals_forbid_them() {
        let (plan, step) = funded();
        let accepted = submission_response(&step, ObservedOutcomeLayer::Accepted);
        let mut missing_identity = accepted.clone();
        missing_identity.accepted_txid = None;
        let mut missing_readback = accepted.clone();
        missing_readback.mined_readback = None;
        for response in [missing_identity, missing_readback] {
            assert_terminal(
                &mut copy_plan(&plan),
                &step,
                &response,
                Refusal::ResponseShape(ResponseShapeDefect::AcceptedOperationOmitsObservation),
            );
        }
        let mut relay = submission_response(&step, ObservedOutcomeLayer::RelayPolicyRejection);
        assert_eq!(relay.accepted_txid, None);
        assert_eq!(relay.mined_readback, None);
        relay.accepted_txid = accepted.accepted_txid;
        assert_terminal(
            &mut copy_plan(&plan),
            &step,
            &relay,
            Refusal::ResponseShape(ResponseShapeDefect::RefusedOperationCarriesObservation),
        );
        relay.accepted_txid = None;
        relay.mined_readback = accepted.mined_readback;
        assert_terminal(
            &mut copy_plan(&plan),
            &step,
            &relay,
            Refusal::ResponseShape(ResponseShapeDefect::RefusedOperationCarriesObservation),
        );
    }

    #[test]
    fn accepted_readback_must_bind_the_exact_transaction_and_witness() {
        let changes: [fn(&mut NativeOperationResponse); 4] = [
            |response| response.accepted_txid = Some("99".repeat(32)),
            |response| {
                response
                    .mined_readback
                    .as_mut()
                    .expect("readback")
                    .transaction_id = "99".repeat(32);
            },
            |response| {
                response
                    .mined_readback
                    .as_mut()
                    .expect("readback")
                    .witness_transaction_id = "99".repeat(32);
            },
            |response| {
                response
                    .mined_readback
                    .as_mut()
                    .expect("readback")
                    .raw_transaction[0] ^= 1;
            },
        ];
        for change in changes {
            let (mut plan, step) = funded();
            let mut response = submission_response(&step, ObservedOutcomeLayer::Accepted);
            change(&mut response);
            response
                .validate_shape()
                .expect("shape alone admits changed readback");
            assert_terminal(
                &mut plan,
                &step,
                &response,
                Refusal::SubmissionReadbackMismatch,
            );
        }
    }

    #[test]
    fn the_register_no_longer_requires_the_sponsorless_target_run() {
        use crate::maturity_corpus::MATURITY_VARIABLE_RUN_ADDRESS;
        use crate::maturity_evidence::{
            MaturityExecutorProvenanceExpectation, MaturityRowStanding,
            derive_maturity_evidence_plan_with,
        };

        for entry in STILL_REQUIRED {
            assert_ne!(entry.row(), "sponsorless");
            assert_ne!(entry.gap(), MaturityNegativeHalfGap::TargetRunNotYetPlanned);
        }
        let plan = derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
            crate::maturity_evidence::MaturityConstructorMaterial::Absent(
                crate::maturity_evidence::MaturityConstructorMaterialAbsence::NotSuppliedToDerivation,
            ),
        )
        .expect("evidence plan");
        let row = plan
            .rows()
            .iter()
            .find(|row| row.row().name() == "sponsorless")
            .expect("sponsorless row");
        assert_eq!(row.row().section(), MaturitySafetySection::Positive);
        assert!(row.standing().is_answered());
        assert!(matches!(
            row.standing(),
            MaturityRowStanding::NativeAcceptanceObserved {
                run_address,
                schedule: tapscript::StateWitnessSchedule::VariableMetadata,
                ..
            } if *run_address == MATURITY_VARIABLE_RUN_ADDRESS
        ));
        assert_eq!(
            expected_layer(
                "sponsorless",
                tapscript::StateWitnessSchedule::WholeMetadata
            ),
            Some(ObservedOutcomeLayer::RelayPolicyRejection)
        );
        assert_eq!(
            expected_layer(
                "sponsorless",
                tapscript::StateWitnessSchedule::VariableMetadata
            ),
            Some(ObservedOutcomeLayer::Accepted)
        );
    }

    fn settled(layer: ObservedOutcomeLayer) -> MaturityAnnouncementPlanner {
        let (mut plan, step) = funded();
        let response = submission_response(&step, layer);
        assert_eq!(plan.next_step(Some((step.case(), &response))), Ok(None));
        plan
    }

    fn evidence(plan: &MaturityAnnouncementPlanner) -> MaturityNativeEvidence {
        MaturityNativeEvidence::from_transcript(
            plan.identity.clone(),
            plan.branch,
            crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
            plan.completed_transcript().expect("settled exchanges"),
        )
        .expect("exact replay")
    }

    #[test]
    fn planner_refuses_emission_of_the_replay_only_schedule() {
        let plan = planner();
        let schedule = tapscript::StateWitnessSchedule::WholeMetadata;
        assert_eq!(
            MaturityAnnouncementPlanner::new(
                plan.identity.clone(),
                plan.branch,
                MaturityWitnessSelection::Emission(schedule),
            )
            .err(),
            Some(Refusal::Closure(Box::new(
                MaturityClosureRefusal::ReplayOnlySchedule { schedule }
            )))
        );
    }

    #[test]
    fn planner_retains_the_variable_metadata_schedule() {
        let plan = planner();
        let schedule = tapscript::StateWitnessSchedule::VariableMetadata;
        let variable = MaturityAnnouncementPlanner::new(
            plan.identity.clone(),
            plan.branch,
            MaturityWitnessSelection::Retained(schedule),
        )
        .expect("the variable schedule has a linked constructor");
        assert_eq!(variable.schedule(), schedule);
        assert_eq!(variable.bundle().record().schedule(), schedule);
    }

    #[test]
    fn planner_and_replayed_evidence_retain_the_composed_schedule() {
        let schedule = tapscript::StateWitnessSchedule::WholeMetadata;
        let bootstrap = planner();
        assert_eq!(bootstrap.schedule(), schedule);
        assert_eq!(bootstrap.bundle().record().schedule(), schedule);
        let plan = settled(ObservedOutcomeLayer::RelayPolicyRejection);
        assert_eq!(plan.schedule(), schedule);
        assert_eq!(plan.bundle().record().schedule(), schedule);
        assert_eq!(evidence(&plan).schedule(), schedule);
    }

    fn assert_outstanding(evidence: &MaturityNativeEvidence) {
        assert_eq!(
            evidence.acceptance_obligation(),
            &MaturityAcceptanceObligation::Outstanding {
                routes: [
                    MaturityAcceptanceRoute::RelayWitnessRestructure,
                    MaturityAcceptanceRoute::BlockLayerSubmissionSubject,
                ],
            }
        );
    }

    #[test]
    fn evidence_retains_the_declared_refusal_and_outstanding_acceptance() {
        let plan = settled(ObservedOutcomeLayer::RelayPolicyRejection);
        let evidence = evidence(&plan);
        assert_eq!(evidence.identity(), &plan.identity);
        assert_eq!(evidence.branch(), plan.branch);
        assert_eq!(evidence.observations().len(), ANNOUNCEMENT_STEPS.len());
        for (observation, subject) in evidence.observations().iter().zip(ANNOUNCEMENT_STEPS) {
            assert_eq!(observation.subject(), subject);
            assert_eq!(
                observation.declared_layer(),
                expected_layer(subject, evidence.schedule())
            );
            let expected = if subject == "sponsorless" {
                ObservedOutcomeLayer::RelayPolicyRejection
            } else {
                ObservedOutcomeLayer::Accepted
            };
            assert_eq!(observation.observed_layer(), expected);
        }
        assert_eq!(
            evidence.standing(),
            MaturityNativeStanding::AnsweredAtDeclaredBoundary
        );
        assert_outstanding(&evidence);
    }

    #[test]
    fn accepted_observation_is_off_declaration_and_acceptance_stays_outstanding() {
        let plan = settled(ObservedOutcomeLayer::Accepted);
        let evidence = evidence(&plan);
        let submission = evidence.observations().last().expect("submission");
        assert_eq!(submission.subject(), "sponsorless");
        assert_eq!(submission.observed_layer(), ObservedOutcomeLayer::Accepted);
        assert_eq!(
            submission.declared_layer(),
            Some(ObservedOutcomeLayer::RelayPolicyRejection)
        );
        assert_eq!(
            evidence.standing(),
            MaturityNativeStanding::ObservedElsewhere
        );
        assert_outstanding(&evidence);
    }

    #[test]
    fn evidence_refuses_every_incomplete_prefix() {
        let plan = settled(ObservedOutcomeLayer::RelayPolicyRejection);
        let exchanges = plan.completed_transcript().expect("complete");
        for (length, _) in exchanges.iter().enumerate() {
            assert_eq!(
                MaturityNativeEvidence::from_transcript(
                    plan.identity.clone(),
                    plan.branch,
                    crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
                    &exchanges[..length]
                ),
                Err(Refusal::IncompleteTranscript)
            );
        }
    }

    #[test]
    fn standing_agrees_with_matches_boundary_for_every_observed_layer() {
        let boundary = crate::observed_boundary::observed_boundary(
            expected_layer(
                "sponsorless",
                tapscript::StateWitnessSchedule::WholeMetadata,
            )
            .expect("declaration"),
        )
        .expect("target boundary");
        for observed in NON_ACCEPTED
            .into_iter()
            .chain([ObservedOutcomeLayer::Accepted])
        {
            let evidence = evidence(&settled(observed));
            assert_eq!(
                evidence.standing() == MaturityNativeStanding::AnsweredAtDeclaredBoundary,
                crate::observed_boundary::matches_boundary(boundary, observed)
            );
            assert_eq!(
                evidence
                    .observations()
                    .last()
                    .expect("submission")
                    .observed_layer(),
                observed
            );
            assert_outstanding(&evidence);
        }
    }

    #[test]
    fn evidence_replay_refuses_altered_reordered_and_surplus_steps() {
        let plan = settled(ObservedOutcomeLayer::RelayPolicyRejection);
        let exchanges = plan.completed_transcript().expect("complete");
        for (position, (step, _)) in exchanges.iter().enumerate() {
            let mut altered = exchanges.to_vec();
            let mut subject = step.subject().clone();
            match &mut subject {
                OperationSubject::Funding(funding) => funding.output_program.push(0),
                OperationSubject::Submission(submission) => submission.transaction_bytes.push(0),
                _ => panic!("only funding and submission"),
            }
            altered[position].0 = OperationStep::new(&step.case().step, subject);
            assert_eq!(
                MaturityNativeEvidence::from_transcript(
                    plan.identity.clone(),
                    plan.branch,
                    crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
                    &altered
                ),
                Err(Refusal::TranscriptStepMismatch { position })
            );
        }
        let mut reordered = exchanges.to_vec();
        reordered.swap(0, 1);
        assert_eq!(
            MaturityNativeEvidence::from_transcript(
                plan.identity.clone(),
                plan.branch,
                crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
                &reordered
            ),
            Err(Refusal::TranscriptStepMismatch { position: 0 })
        );
        let mut surplus = exchanges.to_vec();
        surplus.push(exchanges[0].clone());
        assert_eq!(
            MaturityNativeEvidence::from_transcript(
                plan.identity.clone(),
                plan.branch,
                crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
                &surplus
            ),
            Err(Refusal::TranscriptStepMismatch { position: 3 })
        );
    }

    #[test]
    fn evidence_replay_preserves_the_planners_response_refusal() {
        let plan = settled(ObservedOutcomeLayer::RelayPolicyRejection);
        let mut exchanges = plan.completed_transcript().expect("complete").to_vec();
        exchanges[0].1.funded_outputs[0].amount_satoshis += 1;
        assert_eq!(
            MaturityNativeEvidence::from_transcript(
                plan.identity.clone(),
                plan.branch,
                crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
                &exchanges
            ),
            Err(Refusal::FundingMismatch)
        );
    }

    #[test]
    fn variable_schedule_declares_acceptance_and_funding_remains_undeclared() {
        for schedule in tapscript::StateWitnessSchedule::ALL {
            for subject in &ANNOUNCEMENT_STEPS[..2] {
                assert_eq!(expected_layer(subject, schedule), None);
            }
        }
        assert_eq!(
            expected_layer(
                "sponsorless",
                tapscript::StateWitnessSchedule::WholeMetadata
            ),
            Some(ObservedOutcomeLayer::RelayPolicyRejection)
        );
        assert_eq!(
            expected_layer(
                "sponsorless",
                tapscript::StateWitnessSchedule::VariableMetadata
            ),
            Some(ObservedOutcomeLayer::Accepted)
        );
    }

    #[test]
    fn variable_scripted_acceptance_establishes_every_checked_readback_field() {
        let plan = variable_settled(ObservedOutcomeLayer::Accepted);
        let evidence = variable_evidence(&plan);
        assert_eq!(
            evidence.standing(),
            MaturityNativeStanding::AnsweredAtDeclaredBoundary
        );
        let MaturityAcceptanceObligation::Established {
            schedule,
            identity,
            readback,
        } = evidence.acceptance_obligation()
        else {
            panic!("accepted variable transcript establishes readback")
        };
        assert_eq!(*schedule, tapscript::StateWitnessSchedule::VariableMetadata);
        let response = &plan.completed_transcript().expect("complete")[2].1;
        let mined = response.mined_readback.as_ref().expect("mined readback");
        assert_eq!(*identity, readback.identity());
        assert_eq!(
            identity.to_target_display(),
            response
                .accepted_txid
                .as_deref()
                .expect("accepted identity")
        );
        assert_eq!(
            readback.witness_identity().to_target_display(),
            mined.witness_transaction_id
        );
        assert_eq!(readback.block_hash(), &[0x77; 32]);
        assert_eq!(readback.block_height(), mined.block_height);
        assert_eq!(readback.bytes(), mined.raw_transaction.as_slice());
        assert_eq!(
            readback.bytes(),
            plan.submission_bytes().expect("submitted bytes")
        );
        assert_eq!(plan.readback(), Some(readback));
    }

    #[test]
    fn variable_relay_refusal_is_observed_elsewhere_and_outstanding() {
        let plan = variable_settled(ObservedOutcomeLayer::RelayPolicyRejection);
        let evidence = variable_evidence(&plan);
        assert_eq!(
            evidence.standing(),
            MaturityNativeStanding::ObservedElsewhere
        );
        assert_eq!(
            evidence.observations()[2].declared_layer(),
            Some(ObservedOutcomeLayer::Accepted)
        );
        assert_outstanding(&evidence);
        assert_eq!(plan.readback(), None);
    }

    #[test]
    fn variable_readback_mutations_refuse_submission_readback_mismatch() {
        let changes: [fn(&mut NativeOperationResponse); 4] = [
            |response| response.accepted_txid = Some("99".repeat(32)),
            |response| {
                response
                    .mined_readback
                    .as_mut()
                    .expect("readback")
                    .transaction_id = "99".repeat(32);
            },
            |response| {
                response
                    .mined_readback
                    .as_mut()
                    .expect("readback")
                    .witness_transaction_id = "99".repeat(32);
            },
            |response| {
                response
                    .mined_readback
                    .as_mut()
                    .expect("readback")
                    .raw_transaction[0] ^= 1;
            },
        ];
        for change in changes {
            let (mut plan, step) = variable_funded();
            let mut response = submission_response(&step, ObservedOutcomeLayer::Accepted);
            change(&mut response);
            assert_terminal(
                &mut plan,
                &step,
                &response,
                Refusal::SubmissionReadbackMismatch,
            );
            assert_eq!(plan.readback(), None);
        }
    }

    #[test]
    fn malformed_block_hash_refuses_submission_readback_mismatch() {
        for block_hash in ["77".repeat(31), "77".repeat(33), "gg".repeat(32)] {
            let (mut plan, step) = variable_funded();
            let mut response = submission_response(&step, ObservedOutcomeLayer::Accepted);
            response
                .mined_readback
                .as_mut()
                .expect("readback")
                .block_hash = block_hash;
            assert_terminal(
                &mut plan,
                &step,
                &response,
                Refusal::SubmissionReadbackMismatch,
            );
            assert_eq!(plan.readback(), None);
        }
    }

    #[test]
    fn variable_refusal_carrying_accepted_fields_refuses_response_shape() {
        let (plan, step) = variable_funded();
        let accepted = submission_response(&step, ObservedOutcomeLayer::Accepted);
        let mut refused = submission_response(&step, ObservedOutcomeLayer::RelayPolicyRejection);
        refused.accepted_txid = accepted.accepted_txid;
        assert_terminal(
            &mut copy_plan(&plan),
            &step,
            &refused,
            Refusal::ResponseShape(ResponseShapeDefect::RefusedOperationCarriesObservation),
        );
        refused.accepted_txid = None;
        refused.mined_readback = accepted.mined_readback;
        assert_terminal(
            &mut copy_plan(&plan),
            &step,
            &refused,
            Refusal::ResponseShape(ResponseShapeDefect::RefusedOperationCarriesObservation),
        );
    }

    #[test]
    fn variable_incomplete_prefixes_refuse_incomplete_transcript() {
        let plan = variable_settled(ObservedOutcomeLayer::Accepted);
        let exchanges = plan.completed_transcript().expect("complete");
        for length in 0..exchanges.len() {
            assert_eq!(
                MaturityNativeEvidence::from_transcript(
                    plan.identity.clone(),
                    plan.branch,
                    MaturityWitnessSelection::Retained(
                        tapscript::StateWitnessSchedule::VariableMetadata
                    ),
                    &exchanges[..length],
                ),
                Err(Refusal::IncompleteTranscript)
            );
        }
    }

    #[test]
    fn repeated_variable_replay_yields_equal_established_values() {
        let plan = variable_settled(ObservedOutcomeLayer::Accepted);
        let first = variable_evidence(&plan);
        let second = variable_evidence(&plan);
        assert!(matches!(
            first.acceptance_obligation(),
            MaturityAcceptanceObligation::Established { .. }
        ));
        assert_eq!(
            first.acceptance_obligation(),
            second.acceptance_obligation()
        );
    }
}
