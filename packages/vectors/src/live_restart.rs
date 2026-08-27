//! The mandatory restart order, and what a proof-negative is allowed to
//! be evidence about.
//!
//! # Why the order is a type and not a comment
//!
//! The restart order's steps are not a convenience ordering. Each step's
//! entry condition is the previous step's *observed result*
//! (task:guide-ctf-exec:restart-order), and the failure the order exists
//! to prevent is specific: a negative case run before any positive
//! control was accepted looks like evidence about its own mutation and
//! is not. It is evidence that the lane rejects everything, and the lane
//! rejecting everything is compatible with the mutation being irrelevant.
//!
//! That failure is already named. [`LiveInfrastructureBlocker::NoAcceptingControlExists`]
//! is the standing a row gets when nothing on its lane has ever been
//! accepted, and it exists precisely because a run in that state
//! produces no attributable observation.
//!
//! So [`RestartLedger`] admits step results in order and refuses them
//! out of order, and the closeout's disposition is read off the ledger
//! rather than written beside it.
//!
//! # The second lock, which is the one that matters
//!
//! Order enforcement alone would still let a caller record an
//! out-of-order run and simply not tell the ledger. The lock that cannot
//! be routed around is [`BalanceValidControl`]: a proof-negative
//! attribution needs one, and the only constructor takes an *observed
//! acceptance*. A ceremony that never had an accepted control cannot
//! build the value, so it cannot attribute anything, so there is nothing
//! for it to forget to record (rule:guide-ctf-exec:proof-negative).
//!
//! # Why acceptance is what establishes balance validity
//!
//! Not an assertion of this module's and not a recomputation. The
//! target's own layer vocabulary places value conservation — amounts,
//! commitments, range proofs, surjection proofs — at
//! [`ObservedOutcomeLayer::ConsensusRejectionBeforeScript`], before any
//! script runs. A transaction the target *accepted* therefore passed
//! that check, by the target's own account, which is exactly the
//! property "balance-valid in every other respect" needs and is a
//! stronger source for it than anything this workspace could compute.
//!
//! # What "exactly one field" means here, and how it is checked
//!
//! The rule says a proof-negative mutates exactly one field of the
//! control. That is checked against the bytes rather than trusted: the
//! case derives a typed serialized-field locator, the canonical encoder
//! locates that field independently in the control and mutant, and the
//! attribution refuses unless the field differs while the exact prefixes
//! and suffixes outside both located ranges match. A removed range proof
//! therefore has different control and mutant ranges without turning the
//! shifted suffix into a second mutation.
//!
//! # Nothing here submits, mutates, or observes
//!
//! This module records an order and attributes an outcome. The runs are
//! the ceremony's and the layers are the target's.

use std::ops::Range;

use crate::live_evidence::LiveInfrastructureBlocker;
use target_elements_conformance::protocol::ObservedOutcomeLayer;
use transaction::TransactionRefusal;
use transaction::bytes::{
    SerializedFieldLocationRefusal, SerializedFieldLocator, SerializedOutputField,
    TargetTransaction,
};

/// The seven steps, in the order the guide fixes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RestartStep {
    /// One accepted sponsorless private one-to-one control, before any
    /// negative case.
    AcceptedSponsorlessControl,
    /// Both predecessor commitment parities exercised in complete
    /// accepted successors.
    BothParitySuccessors,
    /// Target CT conservation recorded against a balance-valid control.
    TargetCtConservation,
    /// Wrong-blinder, missing-rangeproof, and malformed-rangeproof, each
    /// attributed to its own layer.
    ProofNegatives,
    /// The remaining positive private shapes, only where each accepts.
    RemainingPositiveShapes,
    /// Sponsor cases, only after their independent signer dependency
    /// closes by observation.
    SponsorCases,
    /// Disclosure-minimality pairs, only after both sides of each pair
    /// accept.
    MinimalityPairs,
}

impl RestartStep {
    /// Every step, in order. The array order *is* the mandatory order.
    pub const ALL: [Self; 7] = [
        Self::AcceptedSponsorlessControl,
        Self::BothParitySuccessors,
        Self::TargetCtConservation,
        Self::ProofNegatives,
        Self::RemainingPositiveShapes,
        Self::SponsorCases,
        Self::MinimalityPairs,
    ];

    /// The step's one-based number, as the guide numbers them.
    ///
    /// # Panics
    ///
    /// Never in practice: the position lookup is over [`Self::ALL`],
    /// which is exhaustive by construction, and seven fits a byte. Both
    /// expectations are spelled rather than silently unwrapped so that a
    /// later member added outside the census fails loudly here instead
    /// of being numbered zero.
    #[must_use]
    pub fn number(self) -> u8 {
        let position = Self::ALL
            .iter()
            .position(|step| *step == self)
            .expect("every step is in the census");
        u8::try_from(position + 1).expect("seven steps fit a byte")
    }

    /// The step's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AcceptedSponsorlessControl => "accepted-sponsorless-control",
            Self::BothParitySuccessors => "both-parity-successors",
            Self::TargetCtConservation => "target-ct-conservation",
            Self::ProofNegatives => "proof-negatives",
            Self::RemainingPositiveShapes => "remaining-positive-shapes",
            Self::SponsorCases => "sponsor-cases",
            Self::MinimalityPairs => "minimality-pairs",
        }
    }
}

/// A step result that may be stored in a restart ledger.
///
/// A rendered placeholder cannot cross this boundary:
///
/// ```compile_fail
/// use vectors::live_restart::{RenderedStepResult, RestartLedger, RestartStep};
///
/// let mut ledger = RestartLedger::new();
/// ledger
///     .record(
///         RestartStep::AcceptedSponsorlessControl,
///         RenderedStepResult::NotReached,
///     )
///     .unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordedStepResult {
    /// The step ran and its own acceptance condition was observed.
    Accepted {
        /// The target-computed identities the step's acceptance rests
        /// on, in the order the step submitted them.
        accepted_identities: Vec<String>,
        /// What the step established, in the step's own words.
        established: String,
    },
    /// The step did not accept, and the reason is a carried blocker
    /// rather than a target verdict about the step's subject.
    ///
    /// This is the honest stop. Every later step renders as
    /// [`RenderedStepResult::NotReached`].
    StoppedTyped {
        /// The blocker.
        blocker: LiveInfrastructureBlocker,
        /// Why this blocker stops this step, stated at the stop rather
        /// than left for a reader to infer.
        because: String,
    },
}

impl RecordedStepResult {
    /// Whether the order may continue past a step with this result.
    #[must_use]
    pub const fn continues(&self) -> bool {
        matches!(self, Self::Accepted { .. })
    }
}

/// A step result as a report may display it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderedStepResult {
    /// The step ran and its own acceptance condition was observed.
    Accepted {
        /// The target-computed identities the step's acceptance rests
        /// on, in the order the step submitted them.
        accepted_identities: Vec<String>,
        /// What the step established, in the step's own words.
        established: String,
    },
    /// The step did not accept, and the reason is a carried blocker.
    StoppedTyped {
        /// The blocker.
        blocker: LiveInfrastructureBlocker,
        /// Why this blocker stops this step.
        because: String,
    },
    /// The order stopped before this step.
    NotReached,
}

impl RenderedStepResult {
    /// Whether the order continued past a displayed step.
    #[must_use]
    pub const fn continues(&self) -> bool {
        matches!(self, Self::Accepted { .. })
    }
}

impl From<RecordedStepResult> for RenderedStepResult {
    fn from(recorded: RecordedStepResult) -> Self {
        match recorded {
            RecordedStepResult::Accepted {
                accepted_identities,
                established,
            } => Self::Accepted {
                accepted_identities,
                established,
            },
            RecordedStepResult::StoppedTyped { blocker, because } => {
                Self::StoppedTyped { blocker, because }
            }
        }
    }
}

/// The lifecycle state derived from the ledger entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RestartLedgerStatus {
    /// The order is waiting for one named step.
    InProgress {
        /// The next admissible step.
        expected: RestartStep,
    },
    /// The order stopped at a typed blocker.
    Stopped {
        /// The step that stopped.
        step: RestartStep,
        /// The blocker recorded at that step.
        blocker: LiveInfrastructureBlocker,
    },
    /// All seven ordered steps were accepted.
    Completed,
}

/// What goes wrong when the order is recorded badly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestartOrderRefusal {
    /// A step was recorded before the step that gates it.
    OutOfOrder {
        /// The step the caller tried to record.
        attempted: RestartStep,
        /// The step the order was waiting for.
        expected: RestartStep,
    },
    /// A step was recorded after the order had already stopped. The
    /// later run may have happened; it is not evidence about its own
    /// subject, because its entry condition never held.
    AfterStop {
        /// The step the caller tried to record.
        attempted: RestartStep,
        /// Where the order stopped.
        stopped_at: RestartStep,
    },
    /// A step was offered after all seven ordered steps had accepted.
    AlreadyComplete {
        /// The step the caller tried to record.
        attempted: RestartStep,
    },
}

/// The restart order, under execution.
#[derive(Clone, Debug, Default)]
pub struct RestartLedger {
    entries: Vec<(RestartStep, RecordedStepResult)>,
}

impl RestartLedger {
    /// An order with nothing recorded.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The step the order is waiting for, where it is still running.
    #[must_use]
    pub fn expected_step(&self) -> Option<RestartStep> {
        match self.status() {
            RestartLedgerStatus::InProgress { expected } => Some(expected),
            RestartLedgerStatus::Stopped { .. } | RestartLedgerStatus::Completed => None,
        }
    }

    /// The lifecycle state derived from the recorded entries alone.
    #[must_use]
    pub fn status(&self) -> RestartLedgerStatus {
        if let Some((step, blocker)) = self.entries.iter().find_map(|(step, result)| {
            if let RecordedStepResult::StoppedTyped { blocker, .. } = result {
                Some((*step, *blocker))
            } else {
                None
            }
        }) {
            return RestartLedgerStatus::Stopped { step, blocker };
        }

        let completed = self.entries.len() == RestartStep::ALL.len()
            && self.entries.iter().zip(RestartStep::ALL).all(
                |((recorded_step, result), expected_step)| {
                    *recorded_step == expected_step && result.continues()
                },
            );
        if completed {
            return RestartLedgerStatus::Completed;
        }

        let expected = RestartStep::ALL
            .into_iter()
            .find(|step| {
                !self
                    .entries
                    .iter()
                    .any(|(recorded_step, _)| recorded_step == step)
            })
            .unwrap_or(RestartStep::AcceptedSponsorlessControl);
        RestartLedgerStatus::InProgress { expected }
    }

    /// Record `result` for `step`.
    ///
    /// # Errors
    ///
    /// [`RestartOrderRefusal::OutOfOrder`] where `step` is not the step
    /// the order is waiting for, [`RestartOrderRefusal::AfterStop`]
    /// where the order has already stopped, and
    /// [`RestartOrderRefusal::AlreadyComplete`] where all seven steps
    /// have already accepted.
    pub fn record(
        &mut self,
        step: RestartStep,
        result: RecordedStepResult,
    ) -> Result<(), RestartOrderRefusal> {
        let expected = match self.status() {
            RestartLedgerStatus::Stopped {
                step: stopped_at, ..
            } => {
                return Err(RestartOrderRefusal::AfterStop {
                    attempted: step,
                    stopped_at,
                });
            }
            RestartLedgerStatus::Completed => {
                return Err(RestartOrderRefusal::AlreadyComplete { attempted: step });
            }
            RestartLedgerStatus::InProgress { expected } => expected,
        };
        if step != expected {
            return Err(RestartOrderRefusal::OutOfOrder {
                attempted: step,
                expected,
            });
        }
        self.entries.push((step, result));
        Ok(())
    }

    /// Whether the ledger holds an accepted control for step one.
    ///
    /// The precondition every negative case has, asked as a question
    /// rather than assumed.
    #[must_use]
    pub fn has_accepted_control(&self) -> bool {
        self.entries.first().is_some_and(|(step, result)| {
            *step == RestartStep::AcceptedSponsorlessControl && result.continues()
        })
    }

    /// Where the order stopped, if it did.
    #[must_use]
    pub fn stopped_at(&self) -> Option<RestartStep> {
        if let RestartLedgerStatus::Stopped { step, .. } = self.status() {
            Some(step)
        } else {
            None
        }
    }

    /// Every step and its result, with the unreached steps written in as
    /// [`RenderedStepResult::NotReached`] rather than omitted.
    ///
    /// Omitting them would leave a report whose length depended on how
    /// far the run got, which is the shape in which a stopped run reads
    /// as a shorter complete one.
    #[must_use]
    pub fn entries(&self) -> Vec<(RestartStep, RenderedStepResult)> {
        RestartStep::ALL
            .into_iter()
            .map(|step| {
                let recorded = self
                    .entries
                    .iter()
                    .find(|(recorded, _)| *recorded == step)
                    .map(|(_, result)| RenderedStepResult::from(result.clone()));
                (step, recorded.unwrap_or(RenderedStepResult::NotReached))
            })
            .collect()
    }

    /// The order rendered one step per line, for a run transcript.
    #[must_use]
    pub fn render(&self) -> String {
        use std::fmt::Write as _;

        let mut out = String::new();
        for (step, result) in self.entries() {
            let _ = writeln!(
                out,
                "restart_step {} {} {result:?}",
                step.number(),
                step.name(),
            );
        }
        out
    }
}

/// A control the target accepted, and therefore one whose commitment
/// balance the target itself checked.
///
/// The only constructor takes an observed layer. There is no way to
/// declare a control balance-valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BalanceValidControl {
    accepted_identity: String,
    frozen_bytes: Vec<u8>,
}

impl BalanceValidControl {
    /// A control from one observed submission.
    ///
    /// # Errors
    ///
    /// [`ProofNegativeAttributionRefusal::ControlNotBalanceValid`] for
    /// any layer but [`ObservedOutcomeLayer::Accepted`] — including the
    /// two non-verdict layers, where the target never checked anything
    /// at all.
    pub fn from_observed(
        layer: ObservedOutcomeLayer,
        accepted_identity: impl Into<String>,
        frozen_bytes: impl Into<Vec<u8>>,
    ) -> Result<Self, ProofNegativeAttributionRefusal> {
        if layer != ObservedOutcomeLayer::Accepted {
            return Err(ProofNegativeAttributionRefusal::ControlNotBalanceValid {
                observed_layer: layer,
            });
        }
        Ok(Self {
            accepted_identity: accepted_identity.into(),
            frozen_bytes: frozen_bytes.into(),
        })
    }

    /// The identity the target computed for the accepted control.
    #[must_use]
    pub fn accepted_identity(&self) -> &str {
        &self.accepted_identity
    }

    /// The control's frozen bytes, which every mutant is compared
    /// against.
    #[must_use]
    pub fn frozen_bytes(&self) -> &[u8] {
        &self.frozen_bytes
    }
}

/// A confidential value-fault proof-negative.
///
/// [`Self::ALL`] is the three the restart's fourth step runs. The
/// conservation lane runs those three and one more — [`Self::PrivateCtImbalance`],
/// which the restart order does not — so the enum carries a fourth
/// variant outside `ALL`: a case belongs to `ALL` only if the restart's
/// fourth step submits it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProofNegativeCase {
    /// A value blinder that does not belong to the committed value.
    WrongBlinder,
    /// An output-witness entry whose range proof is absent.
    MissingRangeproof,
    /// An output-witness entry whose range proof is present and does not
    /// verify.
    MalformedRangeproof,
    /// A value commitment that commits a value the transaction's balance
    /// does not close.
    ///
    /// Distinct from [`Self::WrongBlinder`]: the value is wrong rather than
    /// its blinder, and the conservation lane places it at a DIFFERENT
    /// output so its located field range separates it from the
    /// wrong-blinder mutant they otherwise share a verdict with.
    PrivateCtImbalance,
}

impl ProofNegativeCase {
    /// The three the restart's fourth step runs, in the guide's order.
    ///
    /// [`Self::PrivateCtImbalance`] is deliberately absent: it is the
    /// conservation lane's own case and no step of the restart order
    /// submits it.
    pub const ALL: [Self; 3] = [
        Self::WrongBlinder,
        Self::MissingRangeproof,
        Self::MalformedRangeproof,
    ];

    /// The single serialized field this case mutates.
    ///
    /// Two cases share a field and that is not a collision: "absent" and
    /// "present and wrong" are two mutations of the same range-proof
    /// bytes, and the rule is one field per case rather than one case
    /// per field.
    #[must_use]
    pub const fn serialized_field(self) -> SerializedOutputField {
        match self {
            Self::WrongBlinder | Self::PrivateCtImbalance => SerializedOutputField::ValueCommitment,
            Self::MissingRangeproof | Self::MalformedRangeproof => {
                SerializedOutputField::RangeproofBytes
            }
        }
    }

    /// The case's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::WrongBlinder => "wrong-blinder",
            Self::MissingRangeproof => "missing-rangeproof",
            Self::MalformedRangeproof => "malformed-rangeproof",
            Self::PrivateCtImbalance => "private-ct-imbalance",
        }
    }
}

/// One typed proof-negative mutation.
///
/// Its case derives its serialized field, so callers can select an output
/// but cannot pair the case with a different field kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofNegativeMutation {
    case: ProofNegativeCase,
    locator: SerializedFieldLocator,
    mutant: TargetTransaction,
}

impl ProofNegativeMutation {
    /// A mutation at `output_index`, with its field derived from `case`.
    #[must_use]
    pub const fn at_output(
        case: ProofNegativeCase,
        output_index: usize,
        mutant: TargetTransaction,
    ) -> Self {
        Self {
            case,
            locator: SerializedFieldLocator::new(output_index, case.serialized_field()),
            mutant,
        }
    }

    /// The proof-negative case.
    #[must_use]
    pub const fn case(&self) -> ProofNegativeCase {
        self.case
    }

    /// The case-derived serialized-field locator.
    #[must_use]
    pub const fn locator(&self) -> SerializedFieldLocator {
        self.locator
    }

    /// The structured mutant transaction.
    #[must_use]
    pub const fn mutant(&self) -> &TargetTransaction {
        &self.mutant
    }
}

/// One mutation field located independently in control and mutant bytes.
///
/// Its fields are private and it has no public range-taking constructor,
/// so reversed, empty, and out-of-bounds ranges are not public inputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedMutationField {
    locator: SerializedFieldLocator,
    control_range: Range<usize>,
    mutant_range: Range<usize>,
}

impl LocatedMutationField {
    /// The output and serialized field selected by the mutation.
    #[must_use]
    pub const fn locator(&self) -> SerializedFieldLocator {
        self.locator
    }

    /// The field's half-open range in the control encoding.
    #[must_use]
    pub const fn control_range(&self) -> &Range<usize> {
        &self.control_range
    }

    /// The field's half-open range in the mutant encoding.
    #[must_use]
    pub const fn mutant_range(&self) -> &Range<usize> {
        &self.mutant_range
    }
}

/// Which independently encoded transaction failed field location.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerializedMutationSide {
    /// The accepted balance-valid control.
    Control,
    /// The structured mutant.
    Mutant,
}

/// What refuses a proof-negative attribution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofNegativeAttributionRefusal {
    /// The control was not accepted, so its commitment balance was never
    /// checked by the target, so a mutant's rejection cannot be
    /// attributed to the mutation.
    ControlNotBalanceValid {
        /// What the control's own submission actually observed.
        observed_layer: ObservedOutcomeLayer,
    },
    /// The accepted control's frozen bytes do not decode exactly.
    ControlDecodingFailure {
        /// The transaction decoder's typed refusal.
        refusal: TransactionRefusal,
    },
    /// The selected serialized field is absent from one encoding.
    SerializedFieldAbsent {
        /// The output and field that were selected.
        locator: SerializedFieldLocator,
        /// Which encoding lacks the field.
        side: SerializedMutationSide,
    },
    /// The selected output's value form cannot carry the field.
    SerializedFieldWrongKind {
        /// The output and field that were selected.
        locator: SerializedFieldLocator,
        /// Which encoding has the wrong field kind.
        side: SerializedMutationSide,
    },
    /// The independently located field bytes did not change.
    LocatedFieldUnchanged {
        /// The output and field that were checked.
        locator: SerializedFieldLocator,
    },
    /// Bytes outside the independently located field also changed.
    MutationOutsideLocatedField {
        /// The selected field's control and mutant locations.
        located_field: LocatedMutationField,
    },
}

/// One proof-negative, attributed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofNegativeAttribution {
    case: ProofNegativeCase,
    located_field: LocatedMutationField,
    control_identity: String,
    observed_layer: ObservedOutcomeLayer,
    detail: Option<String>,
}

impl ProofNegativeAttribution {
    /// The case.
    #[must_use]
    pub const fn case(&self) -> ProofNegativeCase {
        self.case
    }

    /// The one field the case moved.
    #[must_use]
    pub const fn serialized_field(&self) -> SerializedOutputField {
        self.located_field.locator.field()
    }

    /// The field located independently in the control and mutant.
    #[must_use]
    pub const fn located_field(&self) -> &LocatedMutationField {
        &self.located_field
    }

    /// The accepted control this mutant was derived from.
    #[must_use]
    pub fn control_identity(&self) -> &str {
        &self.control_identity
    }

    /// The layer the target refused the mutant at, recorded and not
    /// graded.
    ///
    /// A caller turning this into evidence must ask
    /// [`ObservedOutcomeLayer::is_target_verdict`] first: a mutant that
    /// failed to build says nothing about the target's opinion of the
    /// mutation, and a mutant the target *accepted* is a finding about
    /// the mutation rather than a failure of the run.
    #[must_use]
    pub const fn observed_layer(&self) -> ObservedOutcomeLayer {
        self.observed_layer
    }

    /// The node's own words, where it gave any.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

/// Attribute one proof-negative to the layer that refused it.
///
/// The former raw-range call is intentionally unavailable:
///
/// ```compile_fail
/// use target_elements_conformance::protocol::ObservedOutcomeLayer;
/// use vectors::live_restart::{
///     BalanceValidControl, ProofNegativeCase, attribute_proof_negative,
/// };
///
/// let control = BalanceValidControl::from_observed(
///     ObservedOutcomeLayer::Accepted,
///     "control",
///     [0_u8, 1, 2],
/// )?;
/// attribute_proof_negative(
///     &control,
///     ProofNegativeCase::WrongBlinder,
///     (0, usize::MAX),
///     &[9_u8, 1, 2],
///     ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
///     None,
/// )?;
/// # Ok::<(), vectors::live_restart::ProofNegativeAttributionRefusal>(())
/// ```
///
/// # Errors
///
/// [`ProofNegativeAttributionRefusal::ControlDecodingFailure`] when the
/// control bytes are not one exact transaction,
/// [`ProofNegativeAttributionRefusal::SerializedFieldAbsent`] or
/// [`ProofNegativeAttributionRefusal::SerializedFieldWrongKind`] when the
/// selected field cannot be located,
/// [`ProofNegativeAttributionRefusal::LocatedFieldUnchanged`] when the
/// selected field did not move, and
/// [`ProofNegativeAttributionRefusal::MutationOutsideLocatedField`] when
/// bytes outside it also moved.
pub fn attribute_proof_negative(
    control: &BalanceValidControl,
    mutation: &ProofNegativeMutation,
    observed_layer: ObservedOutcomeLayer,
    detail: Option<String>,
) -> Result<ProofNegativeAttribution, ProofNegativeAttributionRefusal> {
    let control_bytes = control.frozen_bytes();
    let control_transaction = TargetTransaction::decode(control_bytes)
        .map_err(|refusal| ProofNegativeAttributionRefusal::ControlDecodingFailure { refusal })?;
    let control_location = control_transaction
        .locate_serialized_field(mutation.locator)
        .map_err(|refusal| location_refusal(SerializedMutationSide::Control, &refusal))?;
    let mutant_bytes = mutation.mutant.encode();
    let mutant_location = mutation
        .mutant
        .locate_serialized_field(mutation.locator)
        .map_err(|refusal| location_refusal(SerializedMutationSide::Mutant, &refusal))?;
    let located_field = LocatedMutationField {
        locator: mutation.locator,
        control_range: control_location.range().clone(),
        mutant_range: mutant_location.range().clone(),
    };

    if !located_field_differs(control_bytes, &mutant_bytes, &located_field) {
        return Err(ProofNegativeAttributionRefusal::LocatedFieldUnchanged {
            locator: mutation.locator,
        });
    }
    if !outside_located_field_matches(control_bytes, &mutant_bytes, &located_field) {
        return Err(ProofNegativeAttributionRefusal::MutationOutsideLocatedField { located_field });
    }

    Ok(ProofNegativeAttribution {
        case: mutation.case,
        located_field,
        control_identity: control.accepted_identity().to_owned(),
        observed_layer,
        detail,
    })
}

const fn location_refusal(
    side: SerializedMutationSide,
    refusal: &SerializedFieldLocationRefusal,
) -> ProofNegativeAttributionRefusal {
    let locator = refusal.locator();
    match refusal {
        SerializedFieldLocationRefusal::OutputAbsent { .. }
        | SerializedFieldLocationRefusal::FieldAbsent { .. } => {
            ProofNegativeAttributionRefusal::SerializedFieldAbsent { locator, side }
        }
        SerializedFieldLocationRefusal::WrongFieldKind { .. } => {
            ProofNegativeAttributionRefusal::SerializedFieldWrongKind { locator, side }
        }
    }
}

fn located_field_differs(control: &[u8], mutant: &[u8], located: &LocatedMutationField) -> bool {
    control
        .get(located.control_range.clone())
        .zip(mutant.get(located.mutant_range.clone()))
        .is_some_and(|(control_field, mutant_field)| control_field != mutant_field)
}

fn outside_located_field_matches(
    control: &[u8],
    mutant: &[u8],
    located: &LocatedMutationField,
) -> bool {
    let prefixes_match = control
        .get(..located.control_range.start)
        .zip(mutant.get(..located.mutant_range.start))
        .is_some_and(|(control_prefix, mutant_prefix)| control_prefix == mutant_prefix);
    let suffixes_match = control
        .get(located.control_range.end..)
        .zip(mutant.get(located.mutant_range.end..))
        .is_some_and(|(control_suffix, mutant_suffix)| control_suffix == mutant_suffix);
    prefixes_match && suffixes_match
}

#[cfg(test)]
mod tests {
    use super::{
        BalanceValidControl, ProofNegativeAttribution, ProofNegativeAttributionRefusal,
        ProofNegativeCase, ProofNegativeMutation, RecordedStepResult, RenderedStepResult,
        RestartLedger, RestartLedgerStatus, RestartOrderRefusal, RestartStep,
        SerializedMutationSide, attribute_proof_negative,
    };
    use crate::live_evidence::LiveInfrastructureBlocker;
    use target_elements_conformance::protocol::ObservedOutcomeLayer;
    use transaction::bytes::{
        AssetField, AssetId, COMMITMENT_BYTES, InputWitness, NonceField, Outpoint, OutputWitness,
        SerializedOutputField, TargetInput, TargetOutput, TargetTransaction, Txid, ValueField,
    };

    const CONTROL_COMMITMENT: [u8; COMMITMENT_BYTES] = [0x08; COMMITMENT_BYTES];
    const MUTANT_COMMITMENT: [u8; COMMITMENT_BYTES] = [0x09; COMMITMENT_BYTES];

    fn accepted(identity: &str) -> RecordedStepResult {
        RecordedStepResult::Accepted {
            accepted_identities: vec![identity.to_owned()],
            established: "a control accepted".to_owned(),
        }
    }

    fn confidential_transaction(
        commitments: [[u8; COMMITMENT_BYTES]; 2],
        range_proofs: [Vec<u8>; 2],
    ) -> TargetTransaction {
        let outputs = commitments
            .into_iter()
            .map(|commitment| {
                TargetOutput::new(
                    AssetField::Explicit(AssetId::from_internal([0x11; 32])),
                    ValueField::Commitment(commitment),
                    NonceField::Null,
                    vec![0x51],
                )
            })
            .collect();
        let output_witnesses = range_proofs
            .into_iter()
            .map(OutputWitness::range_proof_only)
            .collect();
        TargetTransaction::with_output_witnesses(
            2,
            vec![TargetInput::new(
                Outpoint::new(Txid::from_internal([0x22; 32]), 0)
                    .expect("the fixture outpoint is in range"),
                u32::MAX,
            )],
            outputs,
            0,
            vec![InputWitness::new(Vec::new())],
            output_witnesses,
        )
        .expect("the confidential fixture is structurally complete")
    }

    fn confidential_control() -> TargetTransaction {
        confidential_transaction(
            [CONTROL_COMMITMENT, CONTROL_COMMITMENT],
            [vec![1_u8, 2, 3, 4], vec![5_u8, 6, 7]],
        )
    }

    fn rebuild(
        transaction: &TargetTransaction,
        outputs: Vec<TargetOutput>,
        output_witnesses: Vec<OutputWitness>,
    ) -> TargetTransaction {
        TargetTransaction::with_output_witnesses(
            transaction.version(),
            transaction.inputs().to_vec(),
            outputs,
            transaction.lock_time(),
            transaction.witnesses().to_vec(),
            output_witnesses,
        )
        .expect("the mutation preserves the transaction census")
    }

    fn replace_value_commitment(
        transaction: &TargetTransaction,
        output_index: usize,
        commitment: [u8; COMMITMENT_BYTES],
    ) -> TargetTransaction {
        let mut outputs = transaction.outputs().to_vec();
        let output = outputs
            .get_mut(output_index)
            .expect("the fixture output exists");
        *output = TargetOutput::new(
            output.asset(),
            ValueField::Commitment(commitment),
            output.nonce(),
            output.program().to_vec(),
        );
        rebuild(
            transaction,
            outputs,
            transaction.output_witnesses().to_vec(),
        )
    }

    fn replace_range_proof(
        transaction: &TargetTransaction,
        output_index: usize,
        range_proof: Vec<u8>,
    ) -> TargetTransaction {
        let mut output_witnesses = transaction.output_witnesses().to_vec();
        let witness = output_witnesses
            .get_mut(output_index)
            .expect("the fixture output witness exists");
        *witness = OutputWitness::new(witness.surjection_proof().to_vec(), range_proof);
        rebuild(
            transaction,
            transaction.outputs().to_vec(),
            output_witnesses,
        )
    }

    fn accepted_control(transaction: &TargetTransaction) -> BalanceValidControl {
        BalanceValidControl::from_observed(
            ObservedOutcomeLayer::Accepted,
            "accepted-control",
            transaction.encode(),
        )
        .expect("the fixture records an accepted control")
    }

    fn attribute(
        control: &BalanceValidControl,
        mutation: &ProofNegativeMutation,
    ) -> Result<ProofNegativeAttribution, ProofNegativeAttributionRefusal> {
        attribute_proof_negative(
            control,
            mutation,
            ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
            None,
        )
    }

    #[test]
    fn a_negative_step_cannot_be_recorded_before_the_control_step() {
        let mut ledger = RestartLedger::new();
        assert_eq!(
            ledger.record(RestartStep::ProofNegatives, accepted("aa")),
            Err(RestartOrderRefusal::OutOfOrder {
                attempted: RestartStep::ProofNegatives,
                expected: RestartStep::AcceptedSponsorlessControl,
            }),
        );
        assert!(!ledger.has_accepted_control());
    }

    #[test]
    fn a_typed_stop_closes_the_order_and_leaves_the_rest_unreached() {
        let mut ledger = RestartLedger::new();
        ledger
            .record(RestartStep::AcceptedSponsorlessControl, accepted("aa"))
            .expect("step one records");
        ledger
            .record(
                RestartStep::BothParitySuccessors,
                RecordedStepResult::StoppedTyped {
                    blocker: LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
                    because: "no signer is wired".to_owned(),
                },
            )
            .expect("the stop records");
        assert_eq!(ledger.stopped_at(), Some(RestartStep::BothParitySuccessors));
        assert_eq!(
            ledger.status(),
            RestartLedgerStatus::Stopped {
                step: RestartStep::BothParitySuccessors,
                blocker: LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
            },
        );
        assert_eq!(
            ledger.record(RestartStep::TargetCtConservation, accepted("bb")),
            Err(RestartOrderRefusal::AfterStop {
                attempted: RestartStep::TargetCtConservation,
                stopped_at: RestartStep::BothParitySuccessors,
            }),
        );

        // The unreached steps are written in rather than omitted.
        let entries = ledger.entries();
        assert_eq!(entries.len(), RestartStep::ALL.len());
        assert!(
            entries[2..]
                .iter()
                .all(|(_, result)| *result == RenderedStepResult::NotReached)
        );
        assert!(ledger.has_accepted_control());
    }

    #[test]
    fn a_full_ledger_refuses_an_eighth_record_without_panicking() {
        let mut ledger = RestartLedger::new();
        for step in RestartStep::ALL {
            ledger
                .record(step, accepted(step.name()))
                .expect("the ordered acceptance records");
        }
        assert_eq!(ledger.status(), RestartLedgerStatus::Completed);
        assert_eq!(
            ledger.record(
                RestartStep::AcceptedSponsorlessControl,
                accepted("an eighth acceptance"),
            ),
            Err(RestartOrderRefusal::AlreadyComplete {
                attempted: RestartStep::AcceptedSponsorlessControl,
            }),
        );
    }

    #[test]
    fn stop_status_excludes_ledger_owned_commentary() {
        fn stopped(because: &str) -> RestartLedger {
            let mut ledger = RestartLedger::new();
            ledger
                .record(
                    RestartStep::AcceptedSponsorlessControl,
                    RecordedStepResult::StoppedTyped {
                        blocker: LiveInfrastructureBlocker::NoAcceptingControlExists,
                        because: because.to_owned(),
                    },
                )
                .expect("the stop records");
            ledger
        }

        assert_eq!(
            stopped("first explanation").status(),
            stopped("different explanation").status(),
        );
    }

    #[test]
    fn a_control_the_target_did_not_accept_cannot_ground_an_attribution() {
        for layer in [
            ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
            ObservedOutcomeLayer::ScriptPathRejection,
            ObservedOutcomeLayer::FixtureConstructionFailure,
            ObservedOutcomeLayer::ExecutorInfrastructureFailure,
            ObservedOutcomeLayer::RelayPolicyRejection,
        ] {
            assert_eq!(
                BalanceValidControl::from_observed(layer, "aa", [1_u8, 2, 3]),
                Err(ProofNegativeAttributionRefusal::ControlNotBalanceValid {
                    observed_layer: layer,
                }),
            );
        }
    }

    #[test]
    fn all_four_honest_mutants_attribute_to_their_located_fields() {
        let transaction = confidential_control();
        let control = accepted_control(&transaction);
        let mutations = [
            ProofNegativeMutation::at_output(
                ProofNegativeCase::WrongBlinder,
                0,
                replace_value_commitment(&transaction, 0, MUTANT_COMMITMENT),
            ),
            ProofNegativeMutation::at_output(
                ProofNegativeCase::MissingRangeproof,
                0,
                replace_range_proof(&transaction, 0, Vec::new()),
            ),
            ProofNegativeMutation::at_output(
                ProofNegativeCase::MalformedRangeproof,
                0,
                replace_range_proof(&transaction, 0, vec![9_u8, 2, 3, 4]),
            ),
            ProofNegativeMutation::at_output(
                ProofNegativeCase::PrivateCtImbalance,
                1,
                replace_value_commitment(&transaction, 1, MUTANT_COMMITMENT),
            ),
        ];

        for mutation in mutations {
            let attribution = attribute(&control, &mutation)
                .expect("a mutation confined to its located field attributes");
            assert_eq!(attribution.case(), mutation.case());
            assert_eq!(
                attribution.serialized_field(),
                mutation.case().serialized_field()
            );
            assert_eq!(attribution.located_field().locator(), mutation.locator());
            assert_eq!(attribution.control_identity(), "accepted-control");
            assert!(attribution.observed_layer().is_target_verdict());
        }
    }

    #[test]
    fn missing_rangeproof_deletion_keeps_distinct_prefix_inclusive_ranges() {
        let transaction = confidential_control();
        let control = accepted_control(&transaction);
        let mutation = ProofNegativeMutation::at_output(
            ProofNegativeCase::MissingRangeproof,
            0,
            replace_range_proof(&transaction, 0, Vec::new()),
        );
        let attribution = attribute(&control, &mutation)
            .expect("deleting only the located range proof attributes");
        let located = attribution.located_field();

        assert_eq!(located.control_range().start, located.mutant_range().start);
        assert_eq!(located.control_range().len(), 5);
        assert_eq!(located.mutant_range().len(), 1);
        assert_eq!(
            &transaction.encode()[located.control_range().clone()],
            &[4_u8, 1, 2, 3, 4],
        );
        assert_eq!(
            &mutation.mutant().encode()[located.mutant_range().clone()],
            &[0_u8],
        );
    }

    #[test]
    fn an_unchanged_selected_field_refuses_case_and_output_mismatches() {
        let transaction = confidential_control();
        let control = accepted_control(&transaction);
        let commitment_mutant = replace_value_commitment(&transaction, 0, MUTANT_COMMITMENT);
        for mutation in [
            ProofNegativeMutation::at_output(
                ProofNegativeCase::MissingRangeproof,
                0,
                commitment_mutant.clone(),
            ),
            ProofNegativeMutation::at_output(ProofNegativeCase::WrongBlinder, 1, commitment_mutant),
        ] {
            assert_eq!(
                attribute(&control, &mutation),
                Err(ProofNegativeAttributionRefusal::LocatedFieldUnchanged {
                    locator: mutation.locator(),
                }),
            );
        }
    }

    #[test]
    fn a_second_changed_field_refuses_an_otherwise_selected_mutation() {
        let transaction = confidential_control();
        let control = accepted_control(&transaction);
        let first_changed = replace_value_commitment(&transaction, 0, MUTANT_COMMITMENT);
        let two_changed = replace_value_commitment(&first_changed, 1, MUTANT_COMMITMENT);
        let mutation =
            ProofNegativeMutation::at_output(ProofNegativeCase::WrongBlinder, 0, two_changed);

        assert!(matches!(
            attribute(&control, &mutation),
            Err(ProofNegativeAttributionRefusal::MutationOutsideLocatedField { .. })
        ));
    }

    #[test]
    fn decoding_and_location_failures_stay_typed() {
        let transaction = confidential_control();
        let ordinary_mutation = ProofNegativeMutation::at_output(
            ProofNegativeCase::WrongBlinder,
            0,
            replace_value_commitment(&transaction, 0, MUTANT_COMMITMENT),
        );
        let malformed_control = BalanceValidControl::from_observed(
            ObservedOutcomeLayer::Accepted,
            "malformed-control",
            [1_u8, 2, 3],
        )
        .expect("the observed acceptance constructs the frozen control");
        assert!(matches!(
            attribute(&malformed_control, &ordinary_mutation),
            Err(ProofNegativeAttributionRefusal::ControlDecodingFailure { .. })
        ));

        let control = accepted_control(&transaction);
        let absent = ProofNegativeMutation::at_output(
            ProofNegativeCase::WrongBlinder,
            2,
            ordinary_mutation.mutant().clone(),
        );
        assert_eq!(
            attribute(&control, &absent),
            Err(ProofNegativeAttributionRefusal::SerializedFieldAbsent {
                locator: absent.locator(),
                side: SerializedMutationSide::Control,
            }),
        );

        let mut explicit_outputs = transaction.outputs().to_vec();
        let output = explicit_outputs
            .first_mut()
            .expect("the fixture output exists");
        *output = TargetOutput::new(
            output.asset(),
            ValueField::Explicit(7),
            output.nonce(),
            output.program().to_vec(),
        );
        let mut explicit_witnesses = transaction.output_witnesses().to_vec();
        explicit_witnesses[0] = OutputWitness::empty();
        let explicit = rebuild(&transaction, explicit_outputs, explicit_witnesses);
        let explicit_control = accepted_control(&explicit);
        let wrong_kind =
            ProofNegativeMutation::at_output(ProofNegativeCase::WrongBlinder, 0, explicit);
        assert_eq!(
            attribute(&explicit_control, &wrong_kind),
            Err(ProofNegativeAttributionRefusal::SerializedFieldWrongKind {
                locator: wrong_kind.locator(),
                side: SerializedMutationSide::Control,
            }),
        );
    }

    #[test]
    fn proof_negative_cases_name_the_serialized_fields_they_change() {
        assert_eq!(
            ProofNegativeCase::WrongBlinder.serialized_field(),
            SerializedOutputField::ValueCommitment,
        );
        assert_eq!(
            ProofNegativeCase::PrivateCtImbalance.serialized_field(),
            SerializedOutputField::ValueCommitment,
        );
        assert_eq!(
            ProofNegativeCase::MissingRangeproof.serialized_field(),
            SerializedOutputField::RangeproofBytes,
        );
        assert_eq!(
            ProofNegativeCase::MalformedRangeproof.serialized_field(),
            SerializedOutputField::RangeproofBytes,
        );
    }

    #[test]
    fn commitment_cases_carry_distinct_output_locators_and_ranges() {
        let transaction = confidential_control();
        let wrong_blinder = ProofNegativeMutation::at_output(
            ProofNegativeCase::WrongBlinder,
            0,
            replace_value_commitment(&transaction, 0, MUTANT_COMMITMENT),
        );
        let imbalance = ProofNegativeMutation::at_output(
            ProofNegativeCase::PrivateCtImbalance,
            1,
            replace_value_commitment(&transaction, 1, MUTANT_COMMITMENT),
        );
        let wrong_location = transaction
            .locate_serialized_field(wrong_blinder.locator())
            .expect("the first value commitment is serialized");
        let imbalance_location = transaction
            .locate_serialized_field(imbalance.locator())
            .expect("the second value commitment is serialized");

        assert_ne!(wrong_blinder.locator(), imbalance.locator());
        assert_eq!(wrong_location.range().len(), COMMITMENT_BYTES);
        assert_eq!(imbalance_location.range().len(), COMMITMENT_BYTES);
        assert_ne!(wrong_location.range(), imbalance_location.range());
    }
}
