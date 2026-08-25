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
//! attribution is given the control's frozen bytes, the mutant's bytes,
//! and the byte range the named field occupies in the control, and it
//! refuses when any byte outside that range moved. Insertions and
//! deletions are handled by comparing a common prefix and a common
//! suffix, so a removed range proof is a change inside the range and not
//! a change to everything after it.
//!
//! # Nothing here submits, mutates, or observes
//!
//! This module records an order and attributes an outcome. The runs are
//! the ceremony's and the layers are the target's.

use crate::live_evidence::LiveInfrastructureBlocker;
use target_elements_conformance::protocol::ObservedOutcomeLayer;

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

/// How one step came out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestartStepResult {
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
    /// This is the honest stop. Every later step is `NotReached`.
    StoppedTyped {
        /// The blocker.
        blocker: LiveInfrastructureBlocker,
        /// Why this blocker stops this step, stated at the stop rather
        /// than left for a reader to infer.
        because: String,
    },
    /// The order stopped before this step.
    NotReached,
}

impl RestartStepResult {
    /// Whether the order may continue past a step with this result.
    #[must_use]
    pub const fn continues(&self) -> bool {
        matches!(self, Self::Accepted { .. })
    }
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
}

/// The restart order, under execution.
#[derive(Clone, Debug, Default)]
pub struct RestartLedger {
    entries: Vec<(RestartStep, RestartStepResult)>,
    stopped_at: Option<RestartStep>,
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
        if self.stopped_at.is_some() {
            return None;
        }
        RestartStep::ALL.get(self.entries.len()).copied()
    }

    /// Record `result` for `step`.
    ///
    /// # Errors
    ///
    /// [`RestartOrderRefusal::OutOfOrder`] where `step` is not the step
    /// the order is waiting for, and [`RestartOrderRefusal::AfterStop`]
    /// where the order has already stopped.
    ///
    /// # Panics
    ///
    /// Never: a ledger that has neither stopped nor recorded all seven
    /// steps always expects one, and the two conditions are checked
    /// above in that order.
    pub fn record(
        &mut self,
        step: RestartStep,
        result: RestartStepResult,
    ) -> Result<(), RestartOrderRefusal> {
        if let Some(stopped_at) = self.stopped_at {
            return Err(RestartOrderRefusal::AfterStop {
                attempted: step,
                stopped_at,
            });
        }
        let expected = self
            .expected_step()
            .expect("a ledger that has not stopped and is not full expects a step");
        if step != expected {
            return Err(RestartOrderRefusal::OutOfOrder {
                attempted: step,
                expected,
            });
        }
        let continues = result.continues();
        self.entries.push((step, result));
        if !continues {
            self.stopped_at = Some(step);
        }
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
    pub const fn stopped_at(&self) -> Option<RestartStep> {
        self.stopped_at
    }

    /// Every step and its result, with the unreached steps written in as
    /// [`RestartStepResult::NotReached`] rather than omitted.
    ///
    /// Omitting them would leave a report whose length depended on how
    /// far the run got, which is the shape in which a stopped run reads
    /// as a shorter complete one.
    #[must_use]
    pub fn entries(&self) -> Vec<(RestartStep, RestartStepResult)> {
        RestartStep::ALL
            .into_iter()
            .map(|step| {
                let recorded = self
                    .entries
                    .iter()
                    .find(|(recorded, _)| *recorded == step)
                    .map(|(_, result)| result.clone());
                (step, recorded.unwrap_or(RestartStepResult::NotReached))
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

/// The one field a proof-negative moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MutatedField {
    /// The value blinder of one output.
    ValueBlinder,
    /// The range-proof bytes of one output-witness entry.
    RangeproofBytes,
}

impl MutatedField {
    /// The field's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ValueBlinder => "value-blinder",
            Self::RangeproofBytes => "rangeproof-bytes",
        }
    }
}

/// The three proof-negatives the restart's fourth step runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProofNegativeCase {
    /// A value blinder that does not belong to the committed value.
    WrongBlinder,
    /// An output-witness entry whose range proof is absent.
    MissingRangeproof,
    /// An output-witness entry whose range proof is present and does not
    /// verify.
    MalformedRangeproof,
}

impl ProofNegativeCase {
    /// All three, in the guide's order.
    pub const ALL: [Self; 3] = [
        Self::WrongBlinder,
        Self::MissingRangeproof,
        Self::MalformedRangeproof,
    ];

    /// The single field this case mutates.
    ///
    /// Two cases share a field and that is not a collision: "absent" and
    /// "present and wrong" are two mutations of the same range-proof
    /// bytes, and the rule is one field per case rather than one case
    /// per field.
    #[must_use]
    pub const fn mutated_field(self) -> MutatedField {
        match self {
            Self::WrongBlinder => MutatedField::ValueBlinder,
            Self::MissingRangeproof | Self::MalformedRangeproof => MutatedField::RangeproofBytes,
        }
    }

    /// The case's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::WrongBlinder => "wrong-blinder",
            Self::MissingRangeproof => "missing-rangeproof",
            Self::MalformedRangeproof => "malformed-rangeproof",
        }
    }
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
    /// The mutant differs from the control outside the declared field.
    ///
    /// This module's own member, and it is here because the guide's rule
    /// says "exactly one field" and a rule nobody checks is a comment.
    /// The reported bounds are in the control's byte coordinates.
    MutationOutsideDeclaredField {
        /// The field the case declared it was mutating.
        field: MutatedField,
        /// The declared range's start, inclusive.
        declared_start: usize,
        /// The declared range's end, exclusive.
        declared_end: usize,
        /// The first byte index at which the two actually differ.
        observed_start: usize,
        /// One past the last control byte the difference covers.
        observed_end: usize,
    },
    /// The mutant's bytes equal the control's. Nothing was mutated, so
    /// whatever the target said is a second observation of the control.
    MutantIdenticalToControl,
}

/// One proof-negative, attributed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofNegativeAttribution {
    case: ProofNegativeCase,
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
    pub const fn mutated_field(&self) -> MutatedField {
        self.case.mutated_field()
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
/// `declared_field_range` is the half-open byte range, in the control's
/// coordinates, that the mutated field occupies.
///
/// # Errors
///
/// [`ProofNegativeAttributionRefusal::MutantIdenticalToControl`] where
/// nothing moved, and
/// [`ProofNegativeAttributionRefusal::MutationOutsideDeclaredField`]
/// where something outside the declared field did.
pub fn attribute_proof_negative(
    control: &BalanceValidControl,
    case: ProofNegativeCase,
    declared_field_range: (usize, usize),
    mutant_bytes: &[u8],
    observed_layer: ObservedOutcomeLayer,
    detail: Option<String>,
) -> Result<ProofNegativeAttribution, ProofNegativeAttributionRefusal> {
    let control_bytes = control.frozen_bytes();
    if control_bytes == mutant_bytes {
        return Err(ProofNegativeAttributionRefusal::MutantIdenticalToControl);
    }

    // The common prefix and the common suffix bound the change from
    // both ends, so an insertion or a deletion is a change inside a
    // range rather than a change to everything downstream of it.
    let prefix = control_bytes
        .iter()
        .zip(mutant_bytes)
        .take_while(|(left, right)| left == right)
        .count();
    let suffix = control_bytes
        .iter()
        .rev()
        .zip(mutant_bytes.iter().rev())
        .take_while(|(left, right)| left == right)
        .count()
        .min(control_bytes.len() - prefix)
        .min(mutant_bytes.len().saturating_sub(prefix));
    let observed_start = prefix;
    let observed_end = control_bytes.len() - suffix;

    let (declared_start, declared_end) = declared_field_range;
    if observed_start < declared_start || observed_end > declared_end {
        return Err(
            ProofNegativeAttributionRefusal::MutationOutsideDeclaredField {
                field: case.mutated_field(),
                declared_start,
                declared_end,
                observed_start,
                observed_end,
            },
        );
    }

    Ok(ProofNegativeAttribution {
        case,
        control_identity: control.accepted_identity().to_owned(),
        observed_layer,
        detail,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        BalanceValidControl, ProofNegativeAttributionRefusal, ProofNegativeCase, RestartLedger,
        RestartOrderRefusal, RestartStep, RestartStepResult, attribute_proof_negative,
    };
    use crate::live_evidence::LiveInfrastructureBlocker;
    use target_elements_conformance::protocol::ObservedOutcomeLayer;

    fn accepted(identity: &str) -> RestartStepResult {
        RestartStepResult::Accepted {
            accepted_identities: vec![identity.to_owned()],
            established: "a control accepted".to_owned(),
        }
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
                RestartStepResult::StoppedTyped {
                    blocker: LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
                    because: "no signer is wired".to_owned(),
                },
            )
            .expect("the stop records");
        assert_eq!(ledger.stopped_at(), Some(RestartStep::BothParitySuccessors));
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
                .all(|(_, result)| *result == RestartStepResult::NotReached)
        );
        assert!(ledger.has_accepted_control());
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
    fn a_mutation_reaching_outside_its_declared_field_is_refused() {
        let control = BalanceValidControl::from_observed(
            ObservedOutcomeLayer::Accepted,
            "cc",
            [0_u8, 1, 2, 3, 4, 5, 6, 7],
        )
        .expect("an accepted control builds");

        // Inside the declared range: accepted.
        let inside = [0_u8, 1, 9, 9, 4, 5, 6, 7];
        let attribution = attribute_proof_negative(
            &control,
            ProofNegativeCase::MalformedRangeproof,
            (2, 4),
            &inside,
            ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
            None,
        )
        .expect("a single-field mutation attributes");
        assert_eq!(attribution.control_identity(), "cc");
        assert!(attribution.observed_layer().is_target_verdict());

        // One byte outside it: refused, with both ranges reported.
        let outside = [0_u8, 1, 9, 9, 4, 5, 6, 8];
        assert_eq!(
            attribute_proof_negative(
                &control,
                ProofNegativeCase::MalformedRangeproof,
                (2, 4),
                &outside,
                ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
                None,
            ),
            Err(
                ProofNegativeAttributionRefusal::MutationOutsideDeclaredField {
                    field: ProofNegativeCase::MalformedRangeproof.mutated_field(),
                    declared_start: 2,
                    declared_end: 4,
                    observed_start: 2,
                    observed_end: 8,
                }
            ),
        );
    }

    #[test]
    fn a_removed_field_is_a_change_inside_its_range_and_not_after_it() {
        // The missing-rangeproof case shortens the bytes. A comparison
        // that only walked forwards would call every later byte changed
        // and refuse a mutation that is in fact confined.
        let control = BalanceValidControl::from_observed(
            ObservedOutcomeLayer::Accepted,
            "dd",
            [0_u8, 1, 2, 3, 4, 5],
        )
        .expect("an accepted control builds");
        let removed = [0_u8, 1, 4, 5];
        attribute_proof_negative(
            &control,
            ProofNegativeCase::MissingRangeproof,
            (2, 4),
            &removed,
            ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
            None,
        )
        .expect("a deletion inside the declared range attributes");
    }

    #[test]
    fn an_unmutated_mutant_is_a_second_look_at_the_control() {
        let control =
            BalanceValidControl::from_observed(ObservedOutcomeLayer::Accepted, "ee", [7_u8, 7])
                .expect("an accepted control builds");
        assert_eq!(
            attribute_proof_negative(
                &control,
                ProofNegativeCase::WrongBlinder,
                (0, 2),
                &[7_u8, 7],
                ObservedOutcomeLayer::Accepted,
                None,
            ),
            Err(ProofNegativeAttributionRefusal::MutantIdenticalToControl),
        );
    }
}
