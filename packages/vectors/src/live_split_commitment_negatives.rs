//! The split-commitment negative ceremony.
//!
//! A confidential one-in three-out successor whose mutant copies one
//! output's value commitment onto another, so the duplicated value
//! breaks the per-asset sum at a locator no other row occupies.
//!
//! # The row this ceremony drives
//!
//! `copied-commitment`. Its fault is a value commitment that appears
//! twice: the output side then commits one amount two times while the
//! input side carries it once, so the closed asset's in-equals-out sum
//! breaks by exactly the copied value and the target refuses the
//! candidate in its amount check before any script runs.
//!
//! # Why a ceremony of its own, and why a THREE-output successor
//!
//! The verdict this row draws is the words `private-ct-imbalance`
//! already drew, so the verdict cannot be what separates the two rows —
//! the locator has to be. `private-ct-imbalance` mutates the value
//! commitment at output one of a one-in two-out successor. A successor
//! with a THIRD confidential output gives this row an output-index-two
//! value-commitment field on a `(1, 3)` shape: both members of the
//! `(range, shape)` pair differ, which is the same three-part separator
//! the §15.5 consensus rows rest on.
//!
//! That third output is why this is a new ceremony rather than a mutant
//! added to `conservation-negatives`, which builds a one-in two-out
//! successor. Widening that ceremony's successor would move the control
//! digest it has already recorded, and a recorded control that moves is
//! the one thing the corpus's stability property forbids.

use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetSubmissionSubject,
};
use transaction::bytes::{
    SerializedFieldLocator, SerializedOutputField, TargetOutput, TargetTransaction, ValueField,
};
use transaction::taproot::Digest32;

use crate::error::VectorError;
use crate::live_multi_shapes::{MultiShapePlanner, PrivateShape};
use crate::live_report::LiveMutationLocator;

/// The §15 row this ceremony drives.
const ROW: &str = "copied-commitment";

/// The ceremony's own name for the copied-commitment submission.
pub const MUTANT_STEP: &str = "copied-commitment";

/// The ceremony's own name for the control submission.
pub const CONTROL_STEP: &str = "submit-split-control";

/// The output whose value commitment is copied.
const SOURCE_OUTPUT: usize = 0;

/// The output the commitment is copied ONTO.
///
/// The third output, which is the one the narrower confidential successor
/// does not have. `private-ct-imbalance` rewrites the commitment at output
/// one of a one-in two-out successor; placing this row's copy at output
/// TWO of a one-in three-out successor makes both members of the declared
/// `(range, shape)` pair differ, and the pair is what separates two rows
/// that draw the same words from the target.
const COPIED_OUTPUT: usize = 2;

/// The shape the successor must have for this row's locator to mean what
/// it says: one input, three outputs.
const DECLARED_SHAPE: (usize, usize) = (1, 3);

/// What this ceremony refuses, before any node is asked.
///
/// Every member is a construction fact and none is a target verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SplitCommitmentRefusal {
    /// The shape route refused to reach a control.
    ShapeRouteRefused,
    /// The shape route offered something other than a submission, so no
    /// control bytes could be taken from it.
    ControlNotOffered,
    /// The control did not decode.
    ControlNotDecodable,
    /// The control is not the shape this row's locator describes.
    ///
    /// A guard on the coupling to the split shape rather than a defensive
    /// check: the row declares an output-index-two commitment on a one-in
    /// three-out successor, and a successor of another shape would leave
    /// that declaration naming a field the candidate does not have.
    ShapeNotAsDeclared {
        /// How many inputs the control carried.
        inputs: usize,
        /// How many outputs it carried.
        outputs: usize,
    },
    /// The control has no output at one of the two indices this ceremony
    /// reads.
    OutputMissing {
        /// The index that was absent.
        index: usize,
    },
    /// The source output carries no value commitment, so there is nothing
    /// confidential to copy.
    SourceNotConfidential,
    /// The two outputs already carry the SAME commitment, so copying one
    /// onto the other would produce the control again and the run would
    /// compare a candidate with itself.
    MutantNotDistinct,
    /// The mutated candidate did not reassemble.
    MutantNotConstructible,
}

/// The copied-commitment mutant, as this ceremony built, submitted and
/// observed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopiedCommitmentObservation {
    row: &'static str,
    source_output: usize,
    copied_output: usize,
    shape: (usize, usize),
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
}

impl CopiedCommitmentObservation {
    /// The §15 row this mutant drives.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The output whose commitment was copied, and the output it was
    /// copied onto.
    #[must_use]
    pub const fn outputs(&self) -> (usize, usize) {
        (self.source_output, self.copied_output)
    }

    /// The successor's shape: its input and output counts.
    #[must_use]
    pub const fn shape(&self) -> (usize, usize) {
        self.shape
    }

    /// The separating fact this mutant declares: the copied field's
    /// locator together with the shape it sits on.
    ///
    /// Neither member separates this row alone. The verdict is the words
    /// `private-ct-imbalance` already drew, and a value-commitment field
    /// is what that row mutates too; what differs is WHICH output and on
    /// what shape, so the pair is the separator and either half of it
    /// taken by itself is not.
    #[must_use]
    pub const fn separator(&self) -> (usize, (usize, usize)) {
        (self.copied_output, self.shape)
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
pub struct SplitControlObservation {
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
    accepted_txid: Option<String>,
}

impl SplitControlObservation {
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
pub struct SplitCommitmentNegativeRecord {
    mutant: Option<CopiedCommitmentObservation>,
    control: Option<SplitControlObservation>,
    refusal: Option<SplitCommitmentRefusal>,
}

impl SplitCommitmentNegativeRecord {
    /// The copied-commitment mutant, where it was built.
    #[must_use]
    pub const fn mutant(&self) -> Option<&CopiedCommitmentObservation> {
        self.mutant.as_ref()
    }

    /// The control, where it was built.
    #[must_use]
    pub const fn control(&self) -> Option<&SplitControlObservation> {
        self.control.as_ref()
    }

    /// What the ceremony refused, where it refused.
    #[must_use]
    pub const fn refusal(&self) -> Option<&SplitCommitmentRefusal> {
        self.refusal.as_ref()
    }

    /// The capture locator for one step, where that step carries one.
    ///
    /// Only the mutant does. The control is an acceptance and a locator on
    /// it would file a mutation against a run that recorded none.
    #[must_use]
    pub fn capture_locator(&self, step: &str) -> Option<LiveMutationLocator> {
        if step != MUTANT_STEP {
            return None;
        }
        let mutant = self.mutant.as_ref()?;
        Some(LiveMutationLocator::SerializedOutputField(
            SerializedFieldLocator::new(mutant.outputs().1, SerializedOutputField::ValueCommitment),
        ))
    }

    /// What this ceremony does NOT establish, in its own words.
    #[must_use]
    pub fn non_claims() -> Vec<&'static str> {
        vec![
            "separates its row by LOCATOR and not by verdict: a duplicated value commitment \
             breaks the per-asset sum by the copied amount and draws the words \
             private-ct-imbalance already drew, so the output index and the shape are what make \
             this observation this row's",
            "establishes nothing about the range proof at the copied output, which still proves \
             the value that output used to carry: the amount check is queued before the \
             range-proof loop, so the tally is what refuses and the stale proof is never reached",
            "predicts no words beyond the layer, the refusal being the target's own to phrase",
            "claims nothing about any deployment but the one this run created and destroyed",
        ]
    }
}

/// Copy the source output's value commitment onto the copied output.
///
/// The duplication is the whole fault: the output side then commits one
/// amount twice while the input side carries it once, so the closed
/// asset's in-equals-out sum breaks by exactly the copied value.
///
/// Everything else is the control's, the stale range proof at the copied
/// output included. That proof still proves the value the output used to
/// carry and no longer matches the commitment above it, which does not
/// matter here: the target queues its amount check before the range-proof
/// loop, so the tally refuses the candidate before the proof is examined.
///
/// # Errors
///
/// Each member of [`SplitCommitmentRefusal`] this can reach names the
/// construction fact that stopped it: the control not decoding, the
/// successor not being the declared shape, an absent output, a source
/// output with no commitment to copy, a copy that would reproduce the
/// control, or a mutant that does not reassemble.
fn copy_value_commitment(control_bytes: &[u8]) -> Result<Vec<u8>, SplitCommitmentRefusal> {
    let control = TargetTransaction::decode(control_bytes)
        .map_err(|_| SplitCommitmentRefusal::ControlNotDecodable)?;
    let shape = (control.inputs().len(), control.outputs().len());
    if shape != DECLARED_SHAPE {
        return Err(SplitCommitmentRefusal::ShapeNotAsDeclared {
            inputs: shape.0,
            outputs: shape.1,
        });
    }

    let mut outputs = control.outputs().to_vec();
    let source_value = outputs
        .get(SOURCE_OUTPUT)
        .ok_or(SplitCommitmentRefusal::OutputMissing {
            index: SOURCE_OUTPUT,
        })?
        .value();
    let ValueField::Commitment(commitment) = source_value else {
        return Err(SplitCommitmentRefusal::SourceNotConfidential);
    };
    let copied_value = outputs
        .get(COPIED_OUTPUT)
        .ok_or(SplitCommitmentRefusal::OutputMissing {
            index: COPIED_OUTPUT,
        })?
        .value();
    // A copy that changed nothing would hand the node the control twice
    // and the comparison would be vacuous, so a collision fails loudly
    // rather than passing quietly.
    if copied_value == source_value {
        return Err(SplitCommitmentRefusal::MutantNotDistinct);
    }

    let target = outputs
        .get(COPIED_OUTPUT)
        .ok_or(SplitCommitmentRefusal::OutputMissing {
            index: COPIED_OUTPUT,
        })?;
    let replaced = TargetOutput::new(
        target.asset(),
        ValueField::Commitment(commitment),
        target.nonce(),
        target.program().to_vec(),
    );
    *outputs
        .get_mut(COPIED_OUTPUT)
        .ok_or(SplitCommitmentRefusal::OutputMissing {
            index: COPIED_OUTPUT,
        })? = replaced;

    let mutant = TargetTransaction::with_output_witnesses(
        control.version(),
        control.inputs().to_vec(),
        outputs,
        control.lock_time(),
        control.witnesses().to_vec(),
        control.output_witnesses().to_vec(),
    )
    .map_err(|_| SplitCommitmentRefusal::MutantNotConstructible)?;
    Ok(mutant.encode())
}

/// Where the ceremony is in its order of operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    /// Issue the disposable asset, through the shape route.
    Issue,
    /// Fund the confidential predecessor, through the shape route.
    Fund,
    /// Submit the copied-commitment mutant, before the control so the
    /// predecessor stays unspent for the acceptance.
    Mutant,
    /// Submit the split control, last, which is what consumes it.
    Control,
    /// The control has been offered and its answer is still to be read.
    Settled,
    /// Nothing further.
    Done,
}

/// The split-commitment negative ceremony.
///
/// It COMPOSES the shape route rather than restating it: the one-in
/// three-out confidential successor, its destinations and its blinder
/// plan are the split shape's own, and this ceremony drives that route to
/// a control and then mutates the bytes it produced. A second statement
/// of the shape here could drift from the one every other run builds, and
/// a successor built from a drifted plan would still balance and still
/// submit — the capture would simply record a shape nobody meant.
pub struct SplitCommitmentNegativePlanner {
    inner: MultiShapePlanner,
    stage: Stage,
    control_bytes: Option<Vec<u8>>,
    record: SplitCommitmentNegativeRecord,
}

impl SplitCommitmentNegativePlanner {
    /// The ceremony bound to one deployment's printed genesis identity.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] where the reviewed target
    /// does not build.
    pub fn new(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        Ok(Self {
            inner: MultiShapePlanner::for_shape(PrivateShape::Split, printed_genesis_identity)?,
            stage: Stage::Issue,
            control_bytes: None,
            record: SplitCommitmentNegativeRecord::default(),
        })
    }

    /// The ceremony's own record of the run.
    #[must_use]
    pub const fn record(&self) -> &SplitCommitmentNegativeRecord {
        &self.record
    }

    /// Record one refusal and stop.
    const fn refuse(&mut self, refusal: SplitCommitmentRefusal) -> PlanRefused {
        if self.record.refusal.is_none() {
            self.record.refusal = Some(refusal);
        }
        self.stage = Stage::Done;
        PlanRefused
    }

    /// The submission bytes a step from the shape route carries.
    fn submission_bytes(step: Option<&OperationStep>) -> Option<Vec<u8>> {
        match step?.subject() {
            OperationSubject::Submission(subject) => Some(subject.transaction_bytes.clone()),
            _ => None,
        }
    }

    /// Record what the target did with the mutant.
    fn settle_mutant(&mut self, response: &NativeOperationResponse) {
        if let Some(mutant) = self.record.mutant.as_mut() {
            mutant.observed_layer = Some(response.observed_layer);
            mutant.observed_detail.clone_from(&response.observed_detail);
        }
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

impl SplitCommitmentNegativePlanner {
    /// Advance the shape route one step, translating its refusal into this
    /// ceremony's own.
    fn drive_inner(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        let stepped = self.inner.next_step(previous);
        match stepped {
            Ok(step) => Ok(step),
            Err(_) => Err(self.refuse(SplitCommitmentRefusal::ShapeRouteRefused)),
        }
    }
}

impl TargetOperationPlanner for SplitCommitmentNegativePlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        match self.stage {
            Stage::Issue => {
                let step = self.drive_inner(previous)?;
                self.stage = Stage::Fund;
                Ok(step)
            }
            Stage::Fund => {
                let step = self.drive_inner(previous)?;
                self.stage = Stage::Mutant;
                Ok(step)
            }
            Stage::Mutant => {
                // The shape route settles its funding here and offers the
                // successor it built. Those bytes are this ceremony's
                // CONTROL: the mutant is cut from them and goes first, and
                // the control itself is held back for the acceptance.
                let step = self.drive_inner(previous)?;
                let Some(control_bytes) = Self::submission_bytes(step.as_ref()) else {
                    return Err(self.refuse(SplitCommitmentRefusal::ControlNotOffered));
                };
                let mutant_bytes = match copy_value_commitment(&control_bytes) {
                    Ok(bytes) => bytes,
                    Err(refusal) => return Err(self.refuse(refusal)),
                };
                self.record.mutant = Some(CopiedCommitmentObservation {
                    row: ROW,
                    source_output: SOURCE_OUTPUT,
                    copied_output: COPIED_OUTPUT,
                    shape: DECLARED_SHAPE,
                    submitted_bytes: mutant_bytes.len(),
                    observed_layer: None,
                    observed_detail: None,
                });
                self.record.control = Some(SplitControlObservation {
                    submitted_bytes: control_bytes.len(),
                    observed_layer: None,
                    observed_detail: None,
                    accepted_txid: None,
                });
                self.control_bytes = Some(control_bytes);
                self.stage = Stage::Control;
                Ok(Some(OperationStep::new(
                    MUTANT_STEP,
                    OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                        transaction_bytes: mutant_bytes,
                    })),
                )))
            }
            Stage::Control => {
                if let Some((_case, response)) = previous {
                    self.settle_mutant(response);
                }
                let Some(bytes) = self.control_bytes.clone() else {
                    return Err(self.refuse(SplitCommitmentRefusal::ControlNotOffered));
                };
                self.stage = Stage::Settled;
                Ok(Some(OperationStep::new(
                    CONTROL_STEP,
                    OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                        transaction_bytes: bytes,
                    })),
                )))
            }
            Stage::Settled => {
                if let Some((_case, response)) = previous {
                    self.settle_control(response);
                }
                self.stage = Stage::Done;
                Ok(None)
            }
            Stage::Done => Ok(None),
        }
    }
}

/// One run's transcript, one fact per line.
#[must_use]
pub fn render_split_commitment_negatives(record: &SplitCommitmentNegativeRecord) -> String {
    let mut lines = vec!["role split-commitment-negative-run".to_owned()];

    if let Some(mutant) = record.mutant() {
        let (source, _) = mutant.outputs();
        let (copied, shape) = mutant.separator();
        lines.push(format!(
            "copied_commitment row {} source_output {source} separator output-{copied}-on-{}in-{}out submitted_bytes {} layer {} detail {}",
            mutant.row(),
            shape.0,
            shape.1,
            mutant.submitted_bytes(),
            mutant
                .observed_layer()
                .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}")),
            mutant.observed_detail().unwrap_or("none"),
        ));
    } else {
        lines.push("copied_commitment none".to_owned());
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

    for claim in SplitCommitmentNegativeRecord::non_claims() {
        lines.push(format!("non_claim {claim}"));
    }
    lines.push("each_row_by_its_own_mutant true".to_owned());

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::{COPIED_OUTPUT, DECLARED_SHAPE, SOURCE_OUTPUT};

    /// The output index `private-ct-imbalance` mutates, on its own narrower
    /// successor. Restated here as the thing this row must NOT collide
    /// with rather than left implicit in prose.
    const IMBALANCE_OUTPUT: usize = 1;

    /// The shape `private-ct-imbalance` drives.
    const IMBALANCE_SHAPE: (usize, usize) = (1, 2);

    #[test]
    fn the_copy_runs_between_two_distinct_outputs_the_successor_actually_has() {
        assert_ne!(
            SOURCE_OUTPUT, COPIED_OUTPUT,
            "the copy would write an output's commitment back onto itself",
        );
        assert!(
            SOURCE_OUTPUT < DECLARED_SHAPE.1 && COPIED_OUTPUT < DECLARED_SHAPE.1,
            "the copy names an output the declared shape does not have",
        );
    }

    #[test]
    fn the_declared_separator_differs_from_the_imbalance_row_in_both_members() {
        // The two rows draw the SAME words from the target, so neither the
        // verdict nor the field kind can separate them and the (range,
        // shape) pair has to. Both members differ here, which is what the
        // row's class narrowing rests on: sharing a class is what the
        // matrix expects of two faults with one arithmetic, and sharing an
        // observation is what it forbids.
        assert_ne!(
            COPIED_OUTPUT, IMBALANCE_OUTPUT,
            "the two value-commitment rows would declare the same output index",
        );
        assert_ne!(
            DECLARED_SHAPE, IMBALANCE_SHAPE,
            "the two value-commitment rows would declare the same shape",
        );
    }
}
