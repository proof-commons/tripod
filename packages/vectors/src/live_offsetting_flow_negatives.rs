//! The offsetting-flow negative ceremony: a balanced three-in three-out
//! candidate whose extra input-and-output pair passes consensus intact
//! and is refused by the coordinator's own cardinality clause.
//!
//! # The row this ceremony drives
//!
//! `second-offsetting-u-flow`. Its fault is an ADDED flow — one more
//! source and one more destination, of equal value in the closed asset —
//! carried alongside the flow the operation declares. Because the pair
//! offsets exactly, the per-asset sum still balances and consensus has
//! nothing to refuse; what refuses is the covenant's live cardinality
//! fragment, which inspects the input and output counts and requires
//! each to equal the number the operation committed to.
//!
//! # Why the separator is the SHAPE
//!
//! The cardinality clause answers with a plain equality failure, the
//! same words any count mismatch would draw, so the verdict is
//! program-generic and cannot separate this row from another count
//! fault. The transaction SHAPE can: a `(3, 3)` candidate is distinct
//! from every shape already driven — `(2, 1)`, `(2, 3)`, `(1, 2)` — and
//! from the two-in two-out control it is offered against.
//!
//! # Why a ceremony of its own
//!
//! The owner-signing negative ceremony funds exactly two receipts and
//! its leaf-arrangement mutants are two positions wide by construction.
//! Funding a third coin there would rebuild its successor and move the
//! control digest it has already recorded, so the third flow needs a
//! ceremony that funds three coins from the start.

use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationSubject, TargetFundingSubject,
    TargetSubmissionSubject,
};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::taproot::Digest32;

use crate::error::VectorError;
use crate::live_owner_observation::{ObservedFundedCoin, asset_of};
use crate::live_owner_signing_negatives::{
    OwnerSigningNegativeRefusal, RECEIPT_AMOUNT, explicit_destination_program,
    explicit_spent_outputs, finalize_explicit, observed_explicit_coins, relink_explicit,
    sign_explicit_candidate,
};
use crate::live_plan::demonstration_live_abi;
use crate::live_report::LiveMutationLocator;

/// How many receipts this ceremony funds.
///
/// Three, one more than the successor it offers as its control consumes.
/// The extra coin is the offsetting flow's own source, and it is funded up
/// front rather than added later because a ceremony that funded two and
/// then wanted a third would have to fund twice and compare candidates
/// built against different chain states.
const FUNDED_RECEIPTS: u8 = 3;

/// How many coins the control consumes, and how many outputs it creates.
const CONTROL_WIDTH: usize = 2;

/// How many coins the mutant consumes, and how many outputs it creates.
///
/// Equal on both sides deliberately: the added flow is a source AND a
/// destination of the same value, so the candidate stays balanced and
/// consensus has nothing to refuse. An added output alone would break the
/// tally and be refused before the covenant ran.
const MUTANT_WIDTH: usize = 3;

/// The §15 row this ceremony drives.
const ROW: &str = "second-offsetting-u-flow";

/// The ceremony's own name for the offsetting-flow submission.
pub const MUTANT_STEP: &str = "offsetting-flow";

/// The ceremony's own name for the control submission.
pub const CONTROL_STEP: &str = "submit-two-in-two-out-control";

/// The issuance step's name.
const ISSUE_STEP: &str = "issue-protocol-asset";

/// The funding step's name.
const FUND_STEP: &str = "fund-explicit-constructor";

/// What this ceremony refuses, before any node is asked.
///
/// Every member is a construction or infrastructure fact and none is a
/// target verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OffsettingFlowRefusal {
    /// The shared explicit build route refused, carrying its own reason.
    ExplicitBuildRefused(OwnerSigningNegativeRefusal),
    /// The node funded fewer coins than the offsetting flow consumes, so
    /// the wider candidate could not be built at all.
    FundingTooNarrow {
        /// How many coins the node reported.
        funded: usize,
        /// How many the mutant needs.
        needed: usize,
    },
}

impl From<OwnerSigningNegativeRefusal> for OffsettingFlowRefusal {
    fn from(refusal: OwnerSigningNegativeRefusal) -> Self {
        Self::ExplicitBuildRefused(refusal)
    }
}

/// The offsetting-flow mutant, as this ceremony built, submitted and
/// observed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OffsettingFlowObservation {
    row: &'static str,
    control_shape: (usize, usize),
    mutant_shape: (usize, usize),
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
}

impl OffsettingFlowObservation {
    /// The §15 row this mutant drives.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The control's transaction shape: its input and output counts.
    #[must_use]
    pub const fn control_shape(&self) -> (usize, usize) {
        self.control_shape
    }

    /// The mutant's transaction shape: its input and output counts.
    ///
    /// This is the separating fact. The clause that refuses this candidate
    /// answers with a plain equality failure, the same words any count
    /// mismatch draws, so the verdict is the fragment's rather than the
    /// row's and cannot separate this row from another count fault. The
    /// shape can, and no other registered row drives a three-in three-out
    /// candidate.
    #[must_use]
    pub const fn mutant_shape(&self) -> (usize, usize) {
        self.mutant_shape
    }

    /// How many bytes this mutant handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The layer the target refused this mutant at, where it was observed.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The node's own words, where it gave any.
    #[must_use]
    pub fn observed_detail(&self) -> Option<&str> {
        self.observed_detail.as_deref()
    }
}

/// The control, as this ceremony submitted and observed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OffsettingFlowControlObservation {
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
    accepted_txid: Option<String>,
}

impl OffsettingFlowControlObservation {
    /// How many bytes the control handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The layer the target answered the control at.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The node's own words, where it gave any.
    #[must_use]
    pub fn observed_detail(&self) -> Option<&str> {
        self.observed_detail.as_deref()
    }

    /// The transaction identity the node accepted the control under.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }
}

/// One run's transcript.
#[derive(Clone, Debug, Default)]
pub struct OffsettingFlowNegativeRecord {
    issued_asset: Option<String>,
    relinked: bool,
    coins: Vec<ObservedFundedCoin>,
    mutant: Option<OffsettingFlowObservation>,
    control: Option<OffsettingFlowControlObservation>,
    refusal: Option<OffsettingFlowRefusal>,
}

impl OffsettingFlowNegativeRecord {
    /// The asset the run issued, as the node printed it.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// Whether the deployment relinked against the issued asset.
    #[must_use]
    pub const fn relinked(&self) -> bool {
        self.relinked
    }

    /// The funded coins, in the order the node reported them.
    #[must_use]
    pub fn coins(&self) -> &[ObservedFundedCoin] {
        &self.coins
    }

    /// The offsetting-flow mutant, where it was built.
    #[must_use]
    pub const fn mutant(&self) -> Option<&OffsettingFlowObservation> {
        self.mutant.as_ref()
    }

    /// The control, where it was built.
    #[must_use]
    pub const fn control(&self) -> Option<&OffsettingFlowControlObservation> {
        self.control.as_ref()
    }

    /// What the ceremony refused, where it refused.
    #[must_use]
    pub const fn refusal(&self) -> Option<&OffsettingFlowRefusal> {
        self.refusal.as_ref()
    }

    /// The capture locator for one step, where that step carries one.
    ///
    /// Only the mutant does. The control declares no mutation because it
    /// is not one, and a locator on an accepted candidate would claim a
    /// fault where the run recorded an acceptance.
    #[must_use]
    pub fn capture_locator(&self, step: &str) -> Option<LiveMutationLocator> {
        if step != MUTANT_STEP {
            return None;
        }
        let mutant = self.mutant.as_ref()?;
        Some(LiveMutationLocator::TransactionShape {
            control_inputs: mutant.control_shape().0,
            mutant_inputs: mutant.mutant_shape().0,
            control_outputs: mutant.control_shape().1,
            mutant_outputs: mutant.mutant_shape().1,
        })
    }

    /// What this ceremony does NOT establish, in its own words.
    #[must_use]
    pub fn non_claims() -> Vec<&'static str> {
        vec![
            "predicts no exact verdict for the offsetting flow beyond the layer: the covenant's \
             cardinality clause answers with a plain equality failure, and the words a target \
             gives for one are the target's",
            "separates its row by SHAPE and not by verdict: the equality failure is the \
             fragment's and would read the same for any count fault, so the three-in three-out \
             shape is what makes this observation this row's",
            "establishes nothing about an unbalanced added flow: the pair this candidate carries \
             offsets exactly, so consensus has nothing to refuse and the refusal observed is the \
             covenant's rather than the tally's",
            "claims nothing about any deployment but the one this run created and destroyed",
        ]
    }
}

/// Where the ceremony is in its order of operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    /// Issue the disposable asset the deployment is linked against.
    Issue,
    /// Fund the three explicit receipts.
    Fund,
    /// Submit the offsetting-flow mutant, before the control so its coins
    /// stay unspent for the acceptance.
    Mutant,
    /// Submit the two-in two-out control, last, which is what consumes
    /// them.
    Control,
    /// Nothing further.
    Done,
}

/// The offsetting-flow negative ceremony.
pub struct OffsettingFlowNegativePlanner {
    stage: Stage,
    abi: CandidateLiveTransferAbi,
    explicit_program: Vec<u8>,
    genesis_block_hash: Digest32,
    record: OffsettingFlowNegativeRecord,
}

impl OffsettingFlowNegativePlanner {
    /// The ceremony bound to one deployment's printed genesis identity.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] where the candidate ABI or
    /// the explicit destination constructor is unavailable.
    pub fn new(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        let abi = demonstration_live_abi()?;
        let explicit_program = explicit_destination_program(&abi)?;
        Ok(Self {
            stage: Stage::Issue,
            abi,
            explicit_program,
            genesis_block_hash: crate::live_owner_observation::printed_order(
                printed_genesis_identity,
            ),
            record: OffsettingFlowNegativeRecord::default(),
        })
    }

    /// The ceremony's own record of the run.
    #[must_use]
    pub const fn record(&self) -> &OffsettingFlowNegativeRecord {
        &self.record
    }

    /// Record one refusal and stop.
    const fn refuse(&mut self, refusal: OffsettingFlowRefusal) -> PlanRefused {
        if self.record.refusal.is_none() {
            self.record.refusal = Some(refusal);
        }
        self.stage = Stage::Done;
        PlanRefused
    }

    /// The issuance or funding step for the explicit constructor's program.
    fn funding_step(&self, name: &str, issue: bool) -> OperationStep {
        OperationStep::new(
            name,
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: issue,
                asset: if issue {
                    None
                } else {
                    self.record.issued_asset.clone()
                },
                output_program: self.explicit_program.clone(),
                outputs: FUNDED_RECEIPTS,
                amount_per_output: RECEIPT_AMOUNT,
            })),
        )
    }

    /// Link the deployment against the asset the target issued.
    fn relink(&mut self, response: &NativeOperationResponse) -> Result<(), OffsettingFlowRefusal> {
        let relinked = relink_explicit(response)?;
        self.explicit_program = relinked.explicit_program;
        self.record.issued_asset = Some(relinked.issued_asset);
        self.record.relinked = true;
        self.abi = relinked.abi;
        Ok(())
    }

    /// Take the funded coins from the node's own report of them.
    fn settle_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), OffsettingFlowRefusal> {
        let expected_asset = self
            .record
            .issued_asset
            .as_deref()
            .and_then(asset_of)
            .ok_or(OffsettingFlowRefusal::ExplicitBuildRefused(
                OwnerSigningNegativeRefusal::IssuanceNamedNoAsset,
            ))?;
        let coins = observed_explicit_coins(
            response,
            expected_asset,
            &self.explicit_program,
            RECEIPT_AMOUNT,
        )?;
        if coins.len() < MUTANT_WIDTH {
            return Err(OffsettingFlowRefusal::FundingTooNarrow {
                funded: coins.len(),
                needed: MUTANT_WIDTH,
            });
        }
        self.record.coins = coins;
        Ok(())
    }

    /// One signed candidate over the first `width` funded coins, paying
    /// `width` destinations.
    fn candidate(&self, width: usize) -> Result<Vec<u8>, OffsettingFlowRefusal> {
        let coins =
            self.record
                .coins
                .get(..width)
                .ok_or(OffsettingFlowRefusal::FundingTooNarrow {
                    funded: self.record.coins.len(),
                    needed: width,
                })?;
        let finalized = finalize_explicit(&self.abi, coins, width)?;
        let spent_outputs = explicit_spent_outputs(&finalized, self.genesis_block_hash)?;
        let (bytes, _message) = sign_explicit_candidate(
            finalized.protected(),
            &finalized,
            &spent_outputs,
            self.genesis_block_hash,
        )?;
        Ok(bytes)
    }

    /// Build the offsetting-flow mutant and stage it for submission.
    fn stage_mutant(&mut self) -> Result<Vec<u8>, OffsettingFlowRefusal> {
        let bytes = self.candidate(MUTANT_WIDTH)?;
        self.record.mutant = Some(OffsettingFlowObservation {
            row: ROW,
            control_shape: (CONTROL_WIDTH, CONTROL_WIDTH),
            mutant_shape: (MUTANT_WIDTH, MUTANT_WIDTH),
            submitted_bytes: bytes.len(),
            observed_layer: None,
            observed_detail: None,
        });
        Ok(bytes)
    }

    /// Record what the target did with the mutant.
    fn settle_mutant(&mut self, response: &NativeOperationResponse) {
        if let Some(mutant) = self.record.mutant.as_mut() {
            mutant.observed_layer = Some(response.observed_layer);
            mutant.observed_detail.clone_from(&response.observed_detail);
        }
    }

    /// Build the control and stage it for submission.
    ///
    /// Its observation is recorded here, before the node has answered, so
    /// a run that dies mid-submission still states what was handed over.
    fn stage_control(&mut self) -> Result<Vec<u8>, OffsettingFlowRefusal> {
        let bytes = self.candidate(CONTROL_WIDTH)?;
        self.record.control = Some(OffsettingFlowControlObservation {
            submitted_bytes: bytes.len(),
            observed_layer: None,
            observed_detail: None,
            accepted_txid: None,
        });
        Ok(bytes)
    }

    /// Record what the target did with the control.
    fn settle_control(&mut self, response: &NativeOperationResponse) {
        if let Some(control) = self.record.control.as_mut() {
            control.observed_layer = Some(response.observed_layer);
            control
                .observed_detail
                .clone_from(&response.observed_detail);
            control.accepted_txid.clone_from(&response.accepted_txid);
        }
    }
}

impl TargetOperationPlanner for OffsettingFlowNegativePlanner {
    fn next_step(
        &mut self,
        previous: Option<(
            &target_elements_conformance::protocol::OperationCaseId,
            &NativeOperationResponse,
        )>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    if let Err(refusal) = self.relink(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Fund;
                }
                Stage::Fund => {
                    if let Err(refusal) = self.settle_funding(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Mutant;
                }
                Stage::Mutant => {
                    self.settle_mutant(response);
                    self.stage = Stage::Control;
                }
                Stage::Control => {
                    self.settle_control(response);
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        match self.stage {
            Stage::Issue => Ok(Some(self.funding_step(ISSUE_STEP, true))),
            Stage::Fund => Ok(Some(self.funding_step(FUND_STEP, false))),
            Stage::Mutant => match self.stage_mutant() {
                Ok(bytes) => Ok(Some(OperationStep::new(
                    MUTANT_STEP,
                    OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                        transaction_bytes: bytes,
                    })),
                ))),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Control => match self.stage_control() {
                Ok(bytes) => Ok(Some(OperationStep::new(
                    CONTROL_STEP,
                    OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                        transaction_bytes: bytes,
                    })),
                ))),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Done => Ok(None),
        }
    }
}

/// One run's transcript, one fact per line.
#[must_use]
pub fn render_offsetting_flow_negatives(record: &OffsettingFlowNegativeRecord) -> String {
    let mut lines = vec!["role offsetting-flow-negative-run".to_owned()];
    lines.push(format!(
        "issued_asset {}",
        record.issued_asset().unwrap_or("none")
    ));
    lines.push(format!("relinked {}", record.relinked()));

    for (index, coin) in record.coins().iter().enumerate() {
        lines.push(format!(
            "coin {index} program_bytes {} node_fields_match_expectation {}",
            coin.program().len(),
            coin.matches_expectation(),
        ));
    }

    if let Some(mutant) = record.mutant() {
        lines.push(format!(
            "offsetting_flow row {} control_shape {}in-{}out mutant_shape {}in-{}out submitted_bytes {} layer {} detail {}",
            mutant.row(),
            mutant.control_shape().0,
            mutant.control_shape().1,
            mutant.mutant_shape().0,
            mutant.mutant_shape().1,
            mutant.submitted_bytes(),
            mutant
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            mutant.observed_detail().unwrap_or("none"),
        ));
    } else {
        lines.push("offsetting_flow none".to_owned());
    }

    if let Some(control) = record.control() {
        lines.push(format!(
            "control submitted_bytes {} layer {} txid {} detail {}",
            control.submitted_bytes(),
            control
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            control.accepted_txid().unwrap_or("none"),
            control.observed_detail().unwrap_or("none"),
        ));
    } else {
        lines.push("control none".to_owned());
    }

    if let Some(refusal) = record.refusal() {
        lines.push(format!("ceremony_refused {refusal:?}"));
    }

    for claim in OffsettingFlowNegativeRecord::non_claims() {
        lines.push(format!("non_claim {claim}"));
    }
    lines.push("each_row_by_its_own_mutant true".to_owned());

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::{CONTROL_WIDTH, FUNDED_RECEIPTS, MUTANT_WIDTH};

    #[test]
    fn the_mutant_is_one_flow_wider_than_the_control_on_both_sides() {
        // The row is an ADDED flow: one more source and one more
        // destination. Widening only one side would break the per-asset
        // sum, and the candidate would be refused at the consensus tally
        // rather than at the covenant's cardinality clause — a refusal
        // that belongs to conservation and not to this row.
        assert_eq!(
            MUTANT_WIDTH,
            CONTROL_WIDTH + 1,
            "the mutant does not carry exactly one added flow",
        );
    }

    #[test]
    fn the_ceremony_funds_every_coin_the_mutant_consumes() {
        // The mutant spends all three funded coins and the control spends
        // two of them. Funding fewer than the mutant consumes would make
        // the wider candidate unbuildable at the last moment, after the
        // chain state it was to be built against already existed.
        assert_eq!(
            usize::from(FUNDED_RECEIPTS),
            MUTANT_WIDTH,
            "the funding does not cover the widest candidate",
        );
        assert!(
            CONTROL_WIDTH < usize::from(FUNDED_RECEIPTS),
            "the control does not leave a coin over for the added flow",
        );
    }
}
