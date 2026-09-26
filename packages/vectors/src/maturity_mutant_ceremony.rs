//! Two replayable native ceremonies for exact maturity mutant offers.
//!
//! Each table funds its honest predecessor, offers its mutants, then submits
//! the signed sponsorless control. Scripted transcripts establish consistency,
//! while a target capture supplies the observation and its provenance.

use linker::CandidateDeploymentIdentity;
use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId,
    OperationSubject, TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::operator_right::BranchContext;

use crate::live_owner_observation::{asset_of, outpoint_of};
use crate::matrix::EvidenceBoundary;
use crate::maturity_closure::{MaturityWitnessSelection, closure_target};
use crate::maturity_continuity::MaturityFundedPredecessor;
use crate::maturity_evidence::{
    MaturityAcceptedControl, MaturityCarrierExecution, MaturityCarrierOutcome,
    MaturityMutationSite, MaturityNativeRefusal, MaturityRowBinding, MaturitySubmittedSubject,
    native_refusal_binds_to_row,
};
use crate::maturity_mutants::{
    MaturityMutantContext, MaturityMutantRow, MaturityMutantTable, foreign_key_predecessor_program,
    stage_maturity_mutant,
};
use crate::maturity_native::{
    MaturityAnnouncementPlanner, MaturityNativePlanRefusal, matches_boundary,
};
use crate::maturity_safety::MaturityCanonicalControl;

/// Predecessor constructor run: funding, four offers, then control.
pub const PREDECESSOR_MUTANT_STEPS: [&str; 8] = [
    "issue-maturity-singleton",
    "fund-maturity-predecessor",
    "fund-foreign-key-predecessor",
    "mutant-predecessor-wrong-internal-key",
    "mutant-predecessor-wrong-control-block",
    "mutant-metadata-leaf-selected-for-execution",
    "mutant-key-path-spend-attempt",
    "sponsorless",
];

/// Successor constructor run: funding, three offers, then control.
pub const SUCCESSOR_MUTANT_STEPS: [&str; 6] = [
    "issue-maturity-singleton",
    "fund-maturity-predecessor",
    "mutant-successor-under-another-static-subtree",
    "mutant-successor-wrong-internal-key",
    "mutant-successor-wrong-parity",
    "sponsorless",
];

/// The ordered operation names for one constructor table.
#[must_use]
pub const fn roster(table: MaturityMutantTable) -> &'static [&'static str] {
    match table {
        MaturityMutantTable::PredecessorConstructor => &PREDECESSOR_MUTANT_STEPS,
        MaturityMutantTable::SuccessorConstructor => &SUCCESSOR_MUTANT_STEPS,
    }
}

fn row_for_step(step: &str) -> Option<MaturityMutantRow> {
    MaturityMutantRow::ALL
        .iter()
        .copied()
        .find(|row| row.step() == step)
}

/// The layer declared before an operation is offered.
#[must_use]
pub fn declared_layer(step: &str) -> Option<ObservedOutcomeLayer> {
    if let Some(row) = row_for_step(step) {
        return Some(match row {
            MaturityMutantRow::KeyPathSpendAttempt => ObservedOutcomeLayer::KeyPathRejection,
            _ => ObservedOutcomeLayer::ScriptPathRejection,
        });
    }
    (step == PREDECESSOR_MUTANT_STEPS[7]).then_some(ObservedOutcomeLayer::Accepted)
}

const fn expected_detail(row: MaturityMutantRow) -> &'static str {
    match row {
        MaturityMutantRow::MetadataLeafSelectedForExecution => {
            "Script failed an OP_VERIFY operation"
        }
        MaturityMutantRow::KeyPathSpendAttempt => "Invalid Schnorr signature",
        _ => "EC scalar mult verify fail",
    }
}

/// Read carrier execution only from the declared layer and the closed detail table.
#[must_use]
pub fn carrier_execution(
    row: MaturityMutantRow,
    observed: ObservedOutcomeLayer,
    detail: Option<&str>,
) -> MaturityCarrierExecution {
    if matches_boundary(row.declared_layer(), observed)
        && detail.is_some_and(|words| words.contains(expected_detail(row)))
    {
        MaturityCarrierExecution::Executed
    } else {
        MaturityCarrierExecution::NotReached
    }
}

type Refusal = MaturityNativePlanRefusal;

fn inner_next(
    inner: &mut MaturityAnnouncementPlanner,
    previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
) -> Result<Option<OperationStep>, Refusal> {
    inner.next_step(previous).map_err(|PlanRefused| {
        inner
            .refusal()
            .cloned()
            .unwrap_or(Refusal::UnexpectedResponse)
    })
}

fn funded_coin(
    subject: &TargetFundingSubject,
    response: &NativeOperationResponse,
) -> Result<MaturityFundedPredecessor, Refusal> {
    if response.observed_layer != ObservedOutcomeLayer::Accepted {
        return Err(Refusal::FundingNotAccepted {
            observed: response.observed_layer,
        });
    }
    if subject.issue_asset || response.issued_asset.is_some() {
        return Err(Refusal::FundingMismatch);
    }
    let asset_text = subject.asset.as_deref().ok_or(Refusal::FundingMismatch)?;
    let asset = asset_of(asset_text).ok_or(Refusal::IssuedAssetMissingOrInvalid)?;
    let [coin] = response.funded_outputs.as_slice() else {
        return Err(Refusal::FundingCardinality {
            supplied: response.funded_outputs.len(),
        });
    };
    if coin.asset != asset_text
        || coin.amount_satoshis != subject.amount_per_output
        || coin.script != crate::maturity_mutant_ceremony::hex_bytes(&subject.output_program)
    {
        return Err(Refusal::FundingMismatch);
    }
    let outpoint = outpoint_of(&coin.outpoint).ok_or(Refusal::FundedOutpointInvalid)?;
    Ok(MaturityFundedPredecessor {
        outpoint,
        asset,
        amount: subject.amount_per_output,
        program: subject.output_program.clone(),
    })
}

fn hex_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// A composed planner that offers each table's mutants before its honest control.
pub struct MaturityMutantCeremonyPlanner {
    /// The honest funding, construction, signing, and control readback planner.
    inner: MaturityAnnouncementPlanner,
    /// Deployment used by the exact stage.
    identity: CandidateDeploymentIdentity,
    /// Caller-supplied branch context.
    branch: BranchContext,
    /// The constructor table selected for this run.
    table: MaturityMutantTable,
    /// The next roster position after a settled operation.
    position: usize,
    /// The one operation awaiting its response.
    pending: Option<OperationStep>,
    /// Settled requests and responses in roster order.
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    /// Signed honest step held until every offer settles.
    control: Option<OperationStep>,
    /// The honest predecessor funding request.
    honest_subject: Option<TargetFundingSubject>,
    /// The accepted honest predecessor coin.
    honest_coin: Option<MaturityFundedPredecessor>,
    /// The separately accepted foreign-key predecessor coin.
    foreign_coin: Option<MaturityFundedPredecessor>,
    /// The reviewed target required by exact staging.
    target: ReviewedElementsTapscriptDefinition,
    /// Whether any mutant was accepted.
    escaped: bool,
    /// The first construction or response refusal.
    refusal: Option<Refusal>,
}

impl MaturityMutantCeremonyPlanner {
    /// Start the variable-metadata run over the honest public planner.
    ///
    /// # Errors
    /// Retains closure and honest planner construction refusals.
    ///
    /// # Panics
    /// Panics only if the fixed architecture omits its singleton asset or a
    /// linked bundle retains no constructor, which the published sources cannot arrange.
    pub fn new(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        table: MaturityMutantTable,
    ) -> Result<Self, Refusal> {
        let inner = MaturityAnnouncementPlanner::new(
            identity.clone(),
            branch,
            MaturityWitnessSelection::Retained(tapscript::StateWitnessSchedule::VariableMetadata),
        )?;
        let target = closure_target().map_err(|error| Refusal::Closure(Box::new(error)))?;
        Ok(Self {
            inner,
            identity,
            branch,
            table,
            position: 0,
            pending: None,
            exchanges: Vec::new(),
            control: None,
            honest_subject: None,
            honest_coin: None,
            foreign_coin: None,
            target,
            escaped: false,
            refusal: None,
        })
    }

    /// The selected table.
    #[must_use]
    pub const fn table(&self) -> MaturityMutantTable {
        self.table
    }

    /// The honest planner, including its checked control readback.
    #[must_use]
    pub const fn inner(&self) -> &MaturityAnnouncementPlanner {
        &self.inner
    }

    /// The additional funded coin for the predecessor internal-key offer.
    #[must_use]
    pub const fn foreign_coin(&self) -> Option<&MaturityFundedPredecessor> {
        self.foreign_coin.as_ref()
    }

    /// Whether any mutant offer was accepted.
    #[must_use]
    pub const fn escaped(&self) -> bool {
        self.escaped
    }

    /// The retained reason for a refused plan.
    #[must_use]
    pub const fn refusal(&self) -> Option<&Refusal> {
        self.refusal.as_ref()
    }

    /// All settled exchanges, whether their observations matched declarations or not.
    ///
    /// # Errors
    /// Returns the retained refusal or reports an incomplete roster.
    pub fn completed_transcript(
        &self,
    ) -> Result<&[(OperationStep, NativeOperationResponse)], Refusal> {
        if let Some(refusal) = &self.refusal {
            return Err(refusal.clone());
        }
        if self.position != roster(self.table).len() || self.pending.is_some() {
            return Err(Refusal::IncompleteTranscript);
        }
        self.inner.completed_transcript()?;
        Ok(&self.exchanges)
    }

    fn stage_offer(&self, row: MaturityMutantRow) -> Result<OperationStep, Refusal> {
        let funded = self
            .honest_coin
            .as_ref()
            .ok_or(Refusal::IncompleteTranscript)?;
        let finalized = self
            .inner
            .announcement()
            .ok_or(Refusal::IncompleteTranscript)?;
        let submitted_bytes = self
            .inner
            .submission_bytes()
            .ok_or(Refusal::IncompleteTranscript)?;
        let context = MaturityMutantContext {
            target: &self.target,
            bundle: self.inner.bundle(),
            finalized,
            submitted_bytes,
            funded,
            identity: &self.identity,
            branch: self.branch,
            foreign_funded: self.foreign_coin.as_ref(),
        };
        let staged = stage_maturity_mutant(row, &context)
            .map_err(|error| Refusal::MutantStage(Box::new(error)))?;
        Ok(OperationStep::new(
            row.step(),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: staged.bytes().to_vec(),
            })),
        ))
    }

    fn make_step(&self) -> Result<Option<OperationStep>, Refusal> {
        let Some(name) = roster(self.table).get(self.position) else {
            return Ok(None);
        };
        if let Some(row) = row_for_step(name) {
            return self.stage_offer(row).map(Some);
        }
        if self.table == MaturityMutantTable::PredecessorConstructor && self.position == 2 {
            let honest = self
                .honest_subject
                .as_ref()
                .ok_or(Refusal::IncompleteTranscript)?;
            return Ok(Some(OperationStep::new(
                name,
                OperationSubject::Funding(Box::new(TargetFundingSubject {
                    issue_asset: false,
                    asset: honest.asset.clone(),
                    output_program: foreign_key_predecessor_program(self.inner.bundle())
                        .map_err(|error| Refusal::MutantStage(Box::new(error)))?,
                    outputs: 1,
                    amount_per_output: honest.amount_per_output,
                })),
            )));
        }
        if self.position + 1 == roster(self.table).len() {
            return self
                .control
                .clone()
                .map(Some)
                .ok_or(Refusal::IncompleteTranscript);
        }
        Err(Refusal::UnexpectedResponse)
    }

    fn settle(
        &mut self,
        step: &OperationStep,
        response: &NativeOperationResponse,
    ) -> Result<(), Refusal> {
        match self.position {
            0 => {
                self.pending = inner_next(&mut self.inner, Some((step.case(), response)))?;
            }
            1 => {
                let OperationSubject::Funding(subject) = step.subject() else {
                    return Err(Refusal::UnexpectedResponse);
                };
                self.control = inner_next(&mut self.inner, Some((step.case(), response)))?;
                self.honest_coin = Some(funded_coin(subject, response)?);
                self.honest_subject = Some((**subject).clone());
            }
            _ if self.table == MaturityMutantTable::PredecessorConstructor
                && self.position == 2 =>
            {
                let OperationSubject::Funding(subject) = step.subject() else {
                    return Err(Refusal::UnexpectedResponse);
                };
                self.foreign_coin = Some(funded_coin(subject, response)?);
            }
            _ if row_for_step(&step.case().step).is_some() => {
                self.escaped |= response.observed_layer == ObservedOutcomeLayer::Accepted;
            }
            _ => {
                let next = inner_next(&mut self.inner, Some((step.case(), response)))?;
                if next.is_some() {
                    return Err(Refusal::UnexpectedResponse);
                }
            }
        }
        Ok(())
    }

    fn advance(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, Refusal> {
        if self.position == 0 && self.pending.is_none() && previous.is_none() {
            let first = inner_next(&mut self.inner, None)?;
            self.pending.clone_from(&first);
            return Ok(first);
        }
        if self.position == roster(self.table).len() && self.pending.is_none() && previous.is_none()
        {
            return Ok(None);
        }
        let (Some(step), Some((case, response))) = (self.pending.take(), previous) else {
            return Err(Refusal::UnexpectedResponse);
        };
        if step.case() != case || &response.case != case {
            return Err(Refusal::UnexpectedResponse);
        }
        if response.schema != NATIVE_PROTOCOL_SCHEMA {
            return Err(Refusal::ResponseSchema {
                offered: response.schema,
            });
        }
        response.validate_shape().map_err(Refusal::ResponseShape)?;
        self.settle(&step, response)?;
        self.exchanges.push((step, response.clone()));
        self.position += 1;
        let next = if self.position == 1 {
            self.pending.take()
        } else {
            self.make_step()?
        };
        self.pending.clone_from(&next);
        Ok(next)
    }
}

impl TargetOperationPlanner for MaturityMutantCeremonyPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if self.refusal.is_some() {
            return Err(PlanRefused);
        }
        self.advance(previous).map_err(|reason| {
            self.refusal = Some(reason);
            self.pending = None;
            PlanRefused
        })
    }
}

/// One exact offer and its predeclared and observed evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityMutantOffer {
    /// Matrix row offered.
    row: MaturityMutantRow,
    /// Operation name in the selected roster.
    step: String,
    /// Matrix boundary declared before execution.
    declared_boundary: EvidenceBoundary,
    /// Target layer declared before execution.
    declared_layer: ObservedOutcomeLayer,
    /// Target layer actually observed.
    observed_layer: ObservedOutcomeLayer,
    /// Target detail, retained verbatim when present.
    observed_detail: String,
    /// Exact submission bytes reconstructed by replay.
    submitted_bytes: Vec<u8>,
    /// Intended carrier and closed-table execution reading.
    carrier: MaturityCarrierOutcome,
    /// Whether an earlier accepted offer spent a coin in this run.
    after_escape: bool,
    /// Refusal record when a control exists and no earlier offer escaped.
    refusal: Option<MaturityNativeRefusal>,
}

impl MaturityMutantOffer {
    /// Matrix row of this offer.
    #[must_use]
    pub const fn row(&self) -> MaturityMutantRow {
        self.row
    }
    /// Operation name of this offer.
    #[must_use]
    pub fn step(&self) -> &str {
        &self.step
    }
    /// Predeclared matrix boundary.
    #[must_use]
    pub const fn declared_boundary(&self) -> EvidenceBoundary {
        self.declared_boundary
    }
    /// Predeclared target layer.
    #[must_use]
    pub const fn declared_layer(&self) -> ObservedOutcomeLayer {
        self.declared_layer
    }
    /// Observed target layer.
    #[must_use]
    pub const fn observed_layer(&self) -> ObservedOutcomeLayer {
        self.observed_layer
    }
    /// Verbatim observation detail, or an empty string.
    #[must_use]
    pub fn observed_detail(&self) -> &str {
        &self.observed_detail
    }
    /// Exact offered transaction bytes.
    #[must_use]
    pub fn submitted_bytes(&self) -> &[u8] {
        &self.submitted_bytes
    }
    /// Intended carrier and execution reading.
    #[must_use]
    pub const fn carrier(&self) -> MaturityCarrierOutcome {
        self.carrier
    }
    /// Whether this offer followed an accepted mutant.
    #[must_use]
    pub const fn after_escape(&self) -> bool {
        self.after_escape
    }
    /// Refusal record, when its prerequisites hold.
    #[must_use]
    pub const fn refusal(&self) -> Option<&MaturityNativeRefusal> {
        self.refusal.as_ref()
    }
}

/// Replayed evidence for one separately controlled constructor table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaturityMutantEvidence {
    /// The selected constructor table.
    table: MaturityMutantTable,
    /// Checked accepted sponsorless control, when one exists.
    control: Option<MaturityAcceptedControl>,
    /// Offers in roster order, including escapes and later observations.
    offers: Vec<MaturityMutantOffer>,
}

impl MaturityMutantEvidence {
    /// Replay every request and response through a fresh composed planner.
    ///
    /// # Errors
    /// Retains construction and response refusals, an altered request position,
    /// or an incomplete roster.
    ///
    /// # Panics
    /// Panics only if the fixed architecture omits its singleton asset or a
    /// linked bundle retains no constructor, which the published sources cannot arrange.
    pub fn from_transcript(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
        table: MaturityMutantTable,
        exchanges: &[(OperationStep, NativeOperationResponse)],
    ) -> Result<Self, Refusal> {
        let mut planner = MaturityMutantCeremonyPlanner::new(identity, branch, table)?;
        let mut next = replay_next(&mut planner, None)?;
        for (position, (step, response)) in exchanges.iter().enumerate() {
            if next.as_ref() != Some(step) {
                return Err(Refusal::TranscriptStepMismatch { position });
            }
            next = replay_next(&mut planner, Some((step.case(), response)))?;
        }
        planner.completed_transcript()?;
        let control = planner.inner().readback().map(|readback| {
            MaturityAcceptedControl::new(
                MaturityCanonicalControl::SponsorlessAnnouncement,
                readback.identity(),
            )
        });
        let offers = collect_offers(table, exchanges, control)?;
        Ok(Self {
            table,
            control,
            offers,
        })
    }

    /// The selected constructor table.
    #[must_use]
    pub const fn table(&self) -> MaturityMutantTable {
        self.table
    }
    /// Accepted control shape and identity, if the honest submission was accepted.
    #[must_use]
    pub const fn control(&self) -> Option<MaturityAcceptedControl> {
        self.control
    }
    /// Every mutant offer in roster order.
    #[must_use]
    pub fn offers(&self) -> &[MaturityMutantOffer] {
        &self.offers
    }
    /// Accepted mutant offers, each retained as an escape.
    pub fn escapes(&self) -> impl Iterator<Item = &MaturityMutantOffer> {
        self.offers
            .iter()
            .filter(|offer| offer.observed_layer == ObservedOutcomeLayer::Accepted)
    }
    /// Compare a recorded refusal with its row's matrix facts.
    #[must_use]
    pub fn binding(&self, offer: &MaturityMutantOffer) -> Option<MaturityRowBinding> {
        offer
            .refusal()
            .map(|refusal| native_refusal_binds_to_row(offer.row().matrix_row(), refusal))
    }
}

fn replay_next(
    planner: &mut MaturityMutantCeremonyPlanner,
    previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
) -> Result<Option<OperationStep>, Refusal> {
    planner.next_step(previous).map_err(|PlanRefused| {
        planner
            .refusal()
            .cloned()
            .unwrap_or(Refusal::UnexpectedResponse)
    })
}

fn collect_offers(
    table: MaturityMutantTable,
    exchanges: &[(OperationStep, NativeOperationResponse)],
    control: Option<MaturityAcceptedControl>,
) -> Result<Vec<MaturityMutantOffer>, Refusal> {
    let mut offers = Vec::with_capacity(table.rows().len());
    let mut escaped = false;
    for (step, response) in exchanges {
        let Some(row) = row_for_step(&step.case().step) else {
            continue;
        };
        let OperationSubject::Submission(subject) = step.subject() else {
            return Err(Refusal::UnexpectedResponse);
        };
        let boundary = row.declared_layer();
        let layer = declared_layer(&step.case().step).ok_or(Refusal::UnexpectedResponse)?;
        let detail = response.observed_detail.clone().unwrap_or_default();
        let bytes = subject.transaction_bytes.clone();
        let carrier = MaturityCarrierOutcome::new(
            row.matrix_row().carrier(),
            carrier_execution(
                row,
                response.observed_layer,
                response.observed_detail.as_deref(),
            ),
        );
        let refusal = if response.observed_layer == ObservedOutcomeLayer::Accepted || escaped {
            None
        } else {
            control.map(|accepted| {
                MaturityNativeRefusal::record(
                    boundary,
                    response.observed_layer,
                    accepted,
                    detail.clone(),
                    MaturitySubmittedSubject::ExactBytes(bytes.clone()),
                    MaturityMutationSite::Locator(row.site()),
                    carrier,
                )
            })
        };
        offers.push(MaturityMutantOffer {
            row,
            step: step.case().step.clone(),
            declared_boundary: boundary,
            declared_layer: layer,
            observed_layer: response.observed_layer,
            observed_detail: detail,
            submitted_bytes: bytes,
            carrier,
            after_escape: escaped,
            refusal,
        });
        escaped |= response.observed_layer == ObservedOutcomeLayer::Accepted;
    }
    Ok(offers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use target_elements_conformance::constructor::tagged::sha256;
    use target_elements_conformance::protocol::{
        FundedOutput, MinedFundingReadback, NativeResourceObservation, WireOutpoint,
    };
    use transaction::bytes::{TargetTransaction, Txid};

    fn identity() -> CandidateDeploymentIdentity {
        CandidateDeploymentIdentity::new([0x11; 32], [0x22; 32]).expect("scripted identity")
    }

    fn branch() -> BranchContext {
        BranchContext::new([0x41; 32], 7).expect("scripted branch")
    }

    fn blank_response(step: &OperationStep) -> NativeOperationResponse {
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

    fn display_hash(bytes: &[u8]) -> String {
        Txid::from_internal(sha256(&sha256(bytes))).to_target_display()
    }

    fn answer(
        step: &OperationStep,
        offered: Option<ObservedOutcomeLayer>,
    ) -> NativeOperationResponse {
        let mut response = blank_response(step);
        match step.subject() {
            OperationSubject::Funding(subject) => {
                let asset = format!("01{}fe", "55".repeat(30));
                if subject.issue_asset {
                    response.issued_asset = Some(asset.clone());
                }
                response.funded_outputs.push(FundedOutput {
                    outpoint: WireOutpoint {
                        txid: format!("02{}fd", "66".repeat(30)),
                        vout: if subject.issue_asset {
                            3
                        } else if step.case().step == PREDECESSOR_MUTANT_STEPS[2] {
                            9
                        } else {
                            7
                        },
                    },
                    asset,
                    amount_satoshis: subject.amount_per_output,
                    script: hex_bytes(&subject.output_program),
                });
            }
            OperationSubject::Submission(subject) => {
                response.observed_layer = offered.unwrap_or_else(|| {
                    declared_layer(&step.case().step).expect("submission declaration")
                });
                if response.observed_layer == ObservedOutcomeLayer::Accepted {
                    let candidate = TargetTransaction::decode(&subject.transaction_bytes)
                        .expect("candidate decoding");
                    let txid = display_hash(&candidate.encode_without_witness());
                    response.accepted_txid = Some(txid.clone());
                    response.mined_readback = Some(MinedFundingReadback {
                        transaction_id: txid,
                        witness_transaction_id: display_hash(&subject.transaction_bytes),
                        block_hash: "77".repeat(32),
                        block_height: 11,
                        raw_transaction: subject.transaction_bytes.clone(),
                    });
                } else {
                    let row = row_for_step(&step.case().step).expect("mutant row");
                    response.observed_detail = Some(expected_detail(row).to_owned());
                }
            }
            _ => unreachable!("only funding and submission"),
        }
        response.validate_shape().expect("scripted shape");
        response
    }

    fn scripted_run(
        table: MaturityMutantTable,
        replacement: Option<(MaturityMutantRow, ObservedOutcomeLayer)>,
    ) -> MaturityMutantCeremonyPlanner {
        let mut planner =
            MaturityMutantCeremonyPlanner::new(identity(), branch(), table).expect("planner");
        let mut next = planner.next_step(None).expect("first step");
        while let Some(step) = next {
            let layer = replacement
                .filter(|(row, _)| row.step() == step.case().step)
                .map(|(_, layer)| layer);
            let response = answer(&step, layer);
            next = planner
                .next_step(Some((step.case(), &response)))
                .expect("settled step");
        }
        planner
    }

    fn evidence(planner: &MaturityMutantCeremonyPlanner) -> MaturityMutantEvidence {
        MaturityMutantEvidence::from_transcript(
            identity(),
            branch(),
            planner.table(),
            planner.completed_transcript().expect("complete exchanges"),
        )
        .expect("replayed evidence")
    }

    #[test]
    fn every_step_declares_the_layer_its_row_declares() {
        for table in [
            MaturityMutantTable::PredecessorConstructor,
            MaturityMutantTable::SuccessorConstructor,
        ] {
            for step in roster(table) {
                match row_for_step(step) {
                    Some(row) => assert_eq!(
                        declared_layer(step).and_then(crate::maturity_native::observed_boundary),
                        Some(row.declared_layer())
                    ),
                    None if *step == PREDECESSOR_MUTANT_STEPS[7] => {
                        assert_eq!(declared_layer(step), Some(ObservedOutcomeLayer::Accepted));
                    }
                    None => assert_eq!(declared_layer(step), None),
                }
            }
        }
        assert_eq!(declared_layer("unknown"), None);
    }

    #[test]
    fn the_carrier_table_is_closed_and_fails_closed() {
        for row in MaturityMutantRow::ALL {
            let layer = declared_layer(row.step()).expect("row layer");
            assert_eq!(
                carrier_execution(row, layer, Some(expected_detail(row))),
                MaturityCarrierExecution::Executed
            );
            for detail in ["Witness program hash mismatch", "unrecognized refusal"] {
                assert_eq!(
                    carrier_execution(row, layer, Some(detail)),
                    MaturityCarrierExecution::NotReached
                );
            }
            assert_eq!(
                carrier_execution(
                    row,
                    ObservedOutcomeLayer::RelayPolicyRejection,
                    Some(expected_detail(row))
                ),
                MaturityCarrierExecution::NotReached
            );
        }
    }

    fn assert_offer_facts(
        offer: &MaturityMutantOffer,
        control: MaturityAcceptedControl,
        evidence: &MaturityMutantEvidence,
    ) {
        assert_eq!(evidence.binding(offer), Some(MaturityRowBinding::Bound));
        let refusal = offer.refusal().expect("bound refusal");
        assert_eq!(refusal.declared_boundary(), offer.declared_boundary());
        assert_eq!(refusal.observed_layer(), offer.observed_layer());
        assert_eq!(refusal.accepted_control(), control);
        assert_eq!(refusal.refusal_detail(), offer.observed_detail());
        assert_eq!(
            refusal.submitted(),
            &MaturitySubmittedSubject::ExactBytes(offer.submitted_bytes().to_vec())
        );
        assert_eq!(
            refusal.site(),
            &MaturityMutationSite::Locator(offer.row().site())
        );
        assert_eq!(
            refusal.intended_carrier(),
            offer.row().matrix_row().carrier()
        );
        assert_eq!(
            refusal.carrier_execution(),
            MaturityCarrierExecution::Executed
        );
    }

    #[test]
    fn a_bound_record_carries_every_fact_the_row_needs() {
        for table in [
            MaturityMutantTable::PredecessorConstructor,
            MaturityMutantTable::SuccessorConstructor,
        ] {
            let planner = scripted_run(table, None);
            let proof = evidence(&planner);
            let control = proof.control().expect("accepted control");
            assert_eq!(
                control.identity(),
                planner.inner().readback().expect("readback").identity()
            );
            for offer in proof.offers() {
                assert_offer_facts(offer, control, &proof);
            }
        }
    }

    #[test]
    fn a_refusal_elsewhere_is_recorded_and_does_not_bind() {
        let wrong = MaturityMutantRow::PredecessorWrongControlBlock;
        let planner = scripted_run(
            MaturityMutantTable::PredecessorConstructor,
            Some((wrong, ObservedOutcomeLayer::RelayPolicyRejection)),
        );
        let proof = evidence(&planner);
        for offer in proof.offers() {
            if offer.row() == wrong {
                assert_eq!(
                    offer.observed_layer(),
                    ObservedOutcomeLayer::RelayPolicyRejection
                );
                assert_eq!(
                    offer.refusal().expect("retained refusal").observed_layer(),
                    offer.observed_layer()
                );
                assert_eq!(
                    proof.binding(offer),
                    Some(MaturityRowBinding::ObservedElsewhere)
                );
            } else {
                assert_eq!(proof.binding(offer), Some(MaturityRowBinding::Bound));
            }
        }
    }

    #[test]
    fn an_accepted_mutant_is_recorded_as_an_escape() {
        let escaped = MaturityMutantRow::PredecessorWrongControlBlock;
        let planner = scripted_run(
            MaturityMutantTable::PredecessorConstructor,
            Some((escaped, ObservedOutcomeLayer::Accepted)),
        );
        assert!(planner.escaped());
        assert_eq!(
            planner.completed_transcript().expect("complete").len(),
            PREDECESSOR_MUTANT_STEPS.len()
        );
        let proof = evidence(&planner);
        let escaped_offer = proof.escapes().next().expect("accepted offer");
        assert_eq!(escaped_offer.row(), escaped);
        assert_eq!(
            escaped_offer.observed_layer(),
            ObservedOutcomeLayer::Accepted
        );
        assert!(escaped_offer.refusal().is_none());
        for offer in proof.offers().iter().skip(2) {
            assert!(offer.after_escape());
            assert!(offer.refusal().is_none());
        }
    }

    #[test]
    fn the_funding_steps_admit_only_accepted_coins() {
        for refused_position in [1, 2] {
            let mut planner = MaturityMutantCeremonyPlanner::new(
                identity(),
                branch(),
                MaturityMutantTable::PredecessorConstructor,
            )
            .expect("planner");
            let mut next = planner.next_step(None).expect("first");
            for position in 0..=refused_position {
                let step = next.expect("funding step");
                let mut response = answer(&step, None);
                if position == refused_position {
                    response.observed_layer = ObservedOutcomeLayer::RelayPolicyRejection;
                    response.funded_outputs.clear();
                    response.observed_detail = Some("scripted funding refusal".to_owned());
                    response.validate_shape().expect("refused funding shape");
                    assert_eq!(
                        planner.next_step(Some((step.case(), &response))),
                        Err(PlanRefused)
                    );
                    assert_eq!(
                        planner.refusal(),
                        Some(&Refusal::FundingNotAccepted {
                            observed: ObservedOutcomeLayer::RelayPolicyRejection
                        })
                    );
                    break;
                }
                next = planner
                    .next_step(Some((step.case(), &response)))
                    .expect("accepted funding");
            }
        }
    }

    #[test]
    fn replay_rebuilds_every_mutant_request() {
        let planner = scripted_run(MaturityMutantTable::PredecessorConstructor, None);
        let exchanges = planner.completed_transcript().expect("complete");
        assert_eq!(evidence(&planner).offers().len(), 4);
        let mut altered = exchanges.to_vec();
        let position = 3;
        altered[position].0 = OperationStep::new(
            MaturityMutantRow::PredecessorWrongControlBlock.step(),
            altered[position].0.subject().clone(),
        );
        assert_eq!(
            MaturityMutantEvidence::from_transcript(
                identity(),
                branch(),
                MaturityMutantTable::PredecessorConstructor,
                &altered,
            ),
            Err(Refusal::TranscriptStepMismatch { position })
        );
    }

    #[test]
    fn the_two_rosters_partition_the_seven_rows() {
        let predecessor = roster(MaturityMutantTable::PredecessorConstructor)
            .iter()
            .filter_map(|step| row_for_step(step))
            .collect::<Vec<_>>();
        let successor = roster(MaturityMutantTable::SuccessorConstructor)
            .iter()
            .filter_map(|step| row_for_step(step))
            .collect::<Vec<_>>();
        assert_eq!(predecessor.len(), 4);
        assert_eq!(successor.len(), 3);
        assert!(predecessor.iter().all(|row| !successor.contains(row)));
        assert_eq!(
            predecessor.into_iter().chain(successor).collect::<Vec<_>>(),
            MaturityMutantRow::ALL
        );
        assert_eq!(roster(MaturityMutantTable::PredecessorConstructor).len(), 8);
        assert_eq!(roster(MaturityMutantTable::SuccessorConstructor).len(), 6);
    }
}
