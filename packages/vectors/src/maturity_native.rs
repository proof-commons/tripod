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
    let metadata = seed
        .instances()
        .first()
        .expect("the linker retains its predecessor constructor")
        .metadata()
        .semantic;
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
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    )
    .map_err(transaction)?;
    let construction =
        construct_maturity_announcement(&target, &abi, &view, &request, &OracleStateCurve)
            .map_err(transaction)?;
    Ok(finalize_maturity_announcement(construction))
}

fn sign_announcement(finalized: &FinalizedMaturityAnnouncement) -> Result<Vec<u8>, Refusal> {
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
        .bind_for_submission(&target)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maturity_closure::{MaturityDeploymentParameters, maturity_sources_with};
    use crate::maturity_negative_half::{MaturityNegativeHalfGap, STILL_REQUIRED};
    use crate::maturity_safety::MaturitySafetySection;
    use std::fmt::Write as _;
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
