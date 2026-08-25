//! The confidential-funding closeout, written for Guide 14 to consume.
//!
//! # What this type is for
//!
//! One value carrying everything a following guide needs in order to
//! start without reopening anything: what was ruled, what ran, what each
//! ceremony was evidence about, which residual left, which stayed, and
//! which matrix rows moved and on what observation
//! (sec:guide-ctf-exec:closeout).
//!
//! Its most important property is that it can say a small thing. A
//! closeout whose only expressible disposition were `Completed` would
//! make an honest stop unreportable, and an unreportable stop is a stop
//! that gets reported as something else.
//!
//! # Three invariants, checked rather than trusted
//!
//! The guide names them and this module refuses rather than asserts:
//!
//! 1. `cleared_residuals` holds exactly one member. Confidential funding
//!    clears `NoConfidentialPredecessorCanBeFunded` and nothing else, so
//!    a closeout clearing two has cleared something it did not earn and
//!    a closeout clearing none has lost the one thing it did.
//! 2. Funding never clears `OwnerSighashNotComputable`. The two are
//!    separate questions and the whole role census exists to keep them
//!    separate; a closeout that cleared the digest blocker from funding
//!    evidence would be the exact substitution
//!    [`crate::live_roles`] refuses one call at a time.
//! 3. `pre_sighash_matrix_delta` is zero. Constructibility is not
//!    acceptance, and no row may move before the external closure.
//!
//! # Why the matrix delta is a list of grounds and not a count
//!
//! Because a count cannot be audited. A row moved on an observed
//! acceptance is a row whose mover can be named — a class, and the
//! target-computed identity the acceptance was observed at — and a delta
//! that carried only its own size would be a number a reader has to
//! take on trust. So [`MovedMatrixRow`] carries the identity, and the
//! sponsor class cannot appear in the delta at all: it is refused by
//! name, because that row is blocked by a residual this guide does not
//! clear (tab:guide-ctf-exec:row-delta).

use std::collections::{BTreeMap, BTreeSet};

use crate::live_evidence::LiveInfrastructureBlocker;
use crate::live_restart::{RestartLedger, RestartStep};
use crate::live_roles::CeremonyEvidenceRoles;

/// The ten positive private classes, as the row delta names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PositivePrivateClass {
    /// One receipt in, one out.
    OneToOne,
    /// One receipt in, several out.
    Split,
    /// Several receipts in, one out.
    Merge,
    /// The representative many-to-many shape.
    ManyToManyRepresentative,
    /// A transfer whose receipts have several distinct owners.
    SeveralDistinctOwners,
    /// Both admitted commitment parities.
    BothCommitmentParityForms,
    /// Deterministic public fixture openings.
    DeterministicPublicFixtureOpenings,
    /// The target's own commitment-balance rule.
    TargetCtConservation,
    /// Projection equality against the paired explicit case.
    ProjectionEqualityWithPairedExplicit,
    /// Private sponsor values.
    ///
    /// Present in the census and forbidden in the delta. The census has
    /// to be able to NAME it — a row that could not be named could not
    /// be refused either — and
    /// [`CloseoutRefusal::SponsorRowMoved`] is what happens when it is
    /// offered.
    PrivateSponsorValues,
}

impl PositivePrivateClass {
    /// All ten, in the guide's table order.
    pub const ALL: [Self; 10] = [
        Self::OneToOne,
        Self::Split,
        Self::Merge,
        Self::ManyToManyRepresentative,
        Self::SeveralDistinctOwners,
        Self::BothCommitmentParityForms,
        Self::DeterministicPublicFixtureOpenings,
        Self::TargetCtConservation,
        Self::ProjectionEqualityWithPairedExplicit,
        Self::PrivateSponsorValues,
    ];

    /// Whether this guide may move the class at all.
    ///
    /// Nine of ten, and the tenth is not a matter of effort: it is
    /// blocked independently by
    /// [`LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent`], which
    /// this guide does not clear.
    #[must_use]
    pub const fn may_enter_the_delta(self) -> bool {
        !matches!(self, Self::PrivateSponsorValues)
    }

    /// The class's wire spelling, which is the safety matrix's own row
    /// name where the matrix has one.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::OneToOne => "private-one-to-one",
            Self::Split => "private-split",
            Self::Merge => "private-merge",
            Self::ManyToManyRepresentative => "private-many-to-many-representative",
            Self::SeveralDistinctOwners => "private-several-distinct-owners",
            Self::BothCommitmentParityForms => "both-commitment-parity-forms",
            Self::DeterministicPublicFixtureOpenings => "deterministic-public-fixture-openings",
            Self::TargetCtConservation => "target-ct-conservation",
            Self::ProjectionEqualityWithPairedExplicit => {
                "projection-equality-with-paired-explicit"
            }
            Self::PrivateSponsorValues => "private-sponsor-values",
        }
    }
}

/// One row that moved, and what moved it.
///
/// The ground is not optional and not a string of the author's
/// choosing: it is the identity the TARGET computed for a transaction
/// the target accepted. A row cannot be entered into the delta without
/// one, which is the whole of "a row moves on an observed acceptance and
/// nothing else" made structural.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MovedMatrixRow {
    class: PositivePrivateClass,
    accepted_identity: String,
}

impl MovedMatrixRow {
    /// The class.
    #[must_use]
    pub const fn class(&self) -> PositivePrivateClass {
        self.class
    }

    /// The target-computed identity the acceptance was observed at.
    #[must_use]
    pub fn accepted_identity(&self) -> &str {
        &self.accepted_identity
    }
}

/// How the guide came out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloseoutDisposition {
    /// Every step of the restart order accepted.
    Completed,
    /// The order stopped, at a named step, on a named blocker.
    ///
    /// A valid closeout and not a failure to produce one. The guide's
    /// own closing statement says that a typed stopped result is the
    /// honest outcome where the work this guide does not own has not
    /// closed, and that it is a smaller claim than the guide set out to
    /// make.
    TypedStopped {
        /// Where the order stopped.
        step: RestartStep,
        /// What stopped it.
        blocker: LiveInfrastructureBlocker,
    },
}

/// What refuses a closeout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloseoutRefusal {
    /// The cleared set does not hold exactly one member.
    ClearedResidualsAreNotExactlyOne {
        /// What was offered.
        offered: BTreeSet<LiveInfrastructureBlocker>,
    },
    /// The cleared set holds a member other than the one confidential
    /// funding clears.
    ClearedTheWrongResidual {
        /// The member that should not be there.
        offered: LiveInfrastructureBlocker,
    },
    /// The closeout claimed funding cleared the digest blocker.
    FundingClearedTheSighashBlocker,
    /// A row moved before the external sighash closure.
    PreSighashMatrixDeltaIsNotZero {
        /// How many rows the closeout claimed moved before it.
        offered: usize,
    },
    /// The sponsor row was entered into the delta.
    SponsorRowMoved,
    /// The delta names one class twice.
    ClassMovedTwice(PositivePrivateClass),
    /// The disposition disagrees with the ledger it was built beside.
    ///
    /// This module's own member. A closeout is assembled from a ledger
    /// and a disposition, and letting the two be written independently
    /// would let a report say `Completed` over an order that stopped.
    DispositionDisagreesWithTheLedger {
        /// What the closeout claimed.
        claimed: CloseoutDisposition,
        /// Where the ledger says the order actually stopped.
        ledger_stopped_at: Option<RestartStep>,
    },
}

/// The residual confidential funding clears, and the only one.
pub const CLEARED_BY_FUNDING: LiveInfrastructureBlocker =
    LiveInfrastructureBlocker::NoConfidentialPredecessorCanBeFunded;

/// The parts a closeout is assembled from.
///
/// A struct rather than a long argument list, because every member has
/// to be supplied and a positional call of this width is a call whose
/// arguments transpose silently.
#[derive(Clone, Debug)]
pub struct CloseoutParts {
    /// How the guide came out.
    pub disposition: CloseoutDisposition,
    /// The restart order as executed.
    pub ledger: RestartLedger,
    /// Which reproducibility contract each ceremony ran under.
    pub contracts: BTreeMap<String, String>,
    /// The validated funding records, by fixture handle.
    pub funding: BTreeMap<String, String>,
    /// Target build, provenance, deployment binding, mined identities,
    /// block heights.
    pub target_facts: Vec<String>,
    /// The external accepted sighash result consumed, or its absence.
    pub sighash_result: Option<String>,
    /// The evidence-role map, per ceremony.
    pub roles: BTreeMap<String, CeremonyEvidenceRoles>,
    /// The residuals this guide cleared.
    pub cleared_residuals: BTreeSet<LiveInfrastructureBlocker>,
    /// The residuals still carried, unchanged.
    pub blockers: BTreeSet<LiveInfrastructureBlocker>,
    /// How many rows moved before the external closure. Must be zero.
    pub pre_sighash_matrix_delta: usize,
    /// The rows that moved on observed evidence.
    pub wave5_matrix_delta: Vec<MovedMatrixRow>,
    /// The canonical exclusions applied.
    pub exclusions: Vec<String>,
    /// The complete non-claim census.
    pub non_claims: Vec<String>,
    /// Per-wave and per-gate durations.
    pub wall_times: BTreeMap<String, f64>,
}

/// The closeout, validated.
#[derive(Clone, Debug)]
pub struct ConfidentialFundingCloseoutReport {
    parts: CloseoutParts,
}

impl ConfidentialFundingCloseoutReport {
    /// The report's members.
    #[must_use]
    pub const fn parts(&self) -> &CloseoutParts {
        &self.parts
    }

    /// The rows that moved.
    #[must_use]
    pub fn moved_rows(&self) -> &[MovedMatrixRow] {
        &self.parts.wave5_matrix_delta
    }

    /// The report rendered as one line per fact, for an artifact.
    #[must_use]
    pub fn render(&self) -> String {
        use std::fmt::Write as _;

        let mut out = String::new();
        let _ = writeln!(out, "disposition {:?}", self.parts.disposition);
        out.push_str(&self.parts.ledger.render());
        for (ceremony, roles) in &self.parts.roles {
            let _ = writeln!(out, "ceremony {ceremony}");
            out.push_str(&roles.render());
        }
        for (handle, record) in &self.parts.funding {
            let _ = writeln!(out, "funding {handle} {record}");
        }
        for (ceremony, contract) in &self.parts.contracts {
            let _ = writeln!(out, "contract {ceremony} {contract}");
        }
        for fact in &self.parts.target_facts {
            let _ = writeln!(out, "target_fact {fact}");
        }
        let _ = writeln!(
            out,
            "sighash_result {}",
            self.parts.sighash_result.as_deref().unwrap_or("absent"),
        );
        for cleared in &self.parts.cleared_residuals {
            let _ = writeln!(out, "cleared_residual {cleared:?}");
        }
        for blocker in &self.parts.blockers {
            let _ = writeln!(out, "carried_blocker {blocker:?}");
        }
        let _ = writeln!(
            out,
            "pre_sighash_matrix_delta {}",
            self.parts.pre_sighash_matrix_delta,
        );
        let _ = writeln!(
            out,
            "wave5_matrix_delta {}",
            self.parts.wave5_matrix_delta.len(),
        );
        for row in &self.parts.wave5_matrix_delta {
            let _ = writeln!(
                out,
                "moved_row {} on {}",
                row.class().name(),
                row.accepted_identity(),
            );
        }
        for class in PositivePrivateClass::ALL {
            if !self
                .parts
                .wave5_matrix_delta
                .iter()
                .any(|row| row.class() == class)
            {
                let _ = writeln!(out, "unmoved_row {}", class.name());
            }
        }
        for exclusion in &self.parts.exclusions {
            let _ = writeln!(out, "exclusion {exclusion}");
        }
        for non_claim in &self.parts.non_claims {
            let _ = writeln!(out, "non_claim {non_claim}");
        }
        for (step, seconds) in &self.parts.wall_times {
            let _ = writeln!(out, "wall_seconds {step} {seconds:.1}");
        }
        out
    }
}

/// One row moved, on one observed acceptance.
///
/// # Errors
///
/// [`CloseoutRefusal::SponsorRowMoved`] for the class this guide may not
/// move.
pub fn moved_on_acceptance(
    class: PositivePrivateClass,
    accepted_identity: impl Into<String>,
) -> Result<MovedMatrixRow, CloseoutRefusal> {
    if !class.may_enter_the_delta() {
        return Err(CloseoutRefusal::SponsorRowMoved);
    }
    Ok(MovedMatrixRow {
        class,
        accepted_identity: accepted_identity.into(),
    })
}

/// Validate a closeout.
///
/// The three invariants the guide names, plus the two this module adds
/// for reasons stated at their own refusals.
///
/// # Errors
///
/// Every member of [`CloseoutRefusal`].
pub fn validate_closeout(
    parts: CloseoutParts,
) -> Result<ConfidentialFundingCloseoutReport, CloseoutRefusal> {
    // 1. Exactly one residual cleared, and it is the one funding clears.
    if parts.cleared_residuals.len() != 1 {
        return Err(CloseoutRefusal::ClearedResidualsAreNotExactlyOne {
            offered: parts.cleared_residuals.clone(),
        });
    }
    for cleared in &parts.cleared_residuals {
        if *cleared != CLEARED_BY_FUNDING {
            return Err(CloseoutRefusal::ClearedTheWrongResidual { offered: *cleared });
        }
    }

    // 2. Funding never clears the digest blocker. Checked on both sides,
    //    because a closeout could claim it by clearing it or by leaving
    //    it out of what it still carries.
    if parts
        .cleared_residuals
        .contains(&LiveInfrastructureBlocker::OwnerSighashNotComputable)
    {
        return Err(CloseoutRefusal::FundingClearedTheSighashBlocker);
    }

    // 3. No row moved before the external closure.
    if parts.pre_sighash_matrix_delta != 0 {
        return Err(CloseoutRefusal::PreSighashMatrixDeltaIsNotZero {
            offered: parts.pre_sighash_matrix_delta,
        });
    }

    // 4. The sponsor row is not in the delta, and no class is twice.
    let mut seen = BTreeSet::new();
    for row in &parts.wave5_matrix_delta {
        if !row.class().may_enter_the_delta() {
            return Err(CloseoutRefusal::SponsorRowMoved);
        }
        if !seen.insert(row.class()) {
            return Err(CloseoutRefusal::ClassMovedTwice(row.class()));
        }
    }

    // 5. The disposition is the ledger's, not the author's.
    let ledger_stopped_at = parts.ledger.stopped_at();
    let agrees = match (&parts.disposition, ledger_stopped_at) {
        (CloseoutDisposition::Completed, None) => true,
        (CloseoutDisposition::TypedStopped { step, .. }, Some(stopped)) => *step == stopped,
        _ => false,
    };
    if !agrees {
        return Err(CloseoutRefusal::DispositionDisagreesWithTheLedger {
            claimed: parts.disposition.clone(),
            ledger_stopped_at,
        });
    }

    Ok(ConfidentialFundingCloseoutReport { parts })
}

#[cfg(test)]
mod tests {
    use super::{
        CLEARED_BY_FUNDING, CloseoutDisposition, CloseoutParts, CloseoutRefusal,
        PositivePrivateClass, moved_on_acceptance, validate_closeout,
    };
    use crate::live_evidence::LiveInfrastructureBlocker;
    use crate::live_restart::{RestartLedger, RestartStep, RestartStepResult};
    use std::collections::{BTreeMap, BTreeSet};

    fn stopped_ledger() -> RestartLedger {
        let mut ledger = RestartLedger::new();
        ledger
            .record(
                RestartStep::AcceptedSponsorlessControl,
                RestartStepResult::StoppedTyped {
                    blocker: LiveInfrastructureBlocker::NoAcceptingControlExists,
                    because: "no control was submitted".to_owned(),
                },
            )
            .expect("the stop records");
        ledger
    }

    fn parts(ledger: RestartLedger) -> CloseoutParts {
        CloseoutParts {
            disposition: CloseoutDisposition::TypedStopped {
                step: RestartStep::AcceptedSponsorlessControl,
                blocker: LiveInfrastructureBlocker::NoAcceptingControlExists,
            },
            ledger,
            contracts: BTreeMap::new(),
            funding: BTreeMap::new(),
            target_facts: Vec::new(),
            sighash_result: None,
            roles: BTreeMap::new(),
            cleared_residuals: BTreeSet::from([CLEARED_BY_FUNDING]),
            blockers: BTreeSet::from([
                LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
                LiveInfrastructureBlocker::PredecessorConstructorAbsent,
            ]),
            pre_sighash_matrix_delta: 0,
            wave5_matrix_delta: Vec::new(),
            exclusions: Vec::new(),
            non_claims: Vec::new(),
            wall_times: BTreeMap::new(),
        }
    }

    #[test]
    fn a_typed_stop_is_a_valid_closeout() {
        let report = validate_closeout(parts(stopped_ledger())).expect("a typed stop validates");
        let rendered = report.render();
        assert!(rendered.contains("pre_sighash_matrix_delta 0"));
        assert!(rendered.contains("wave5_matrix_delta 0"));
        // Every class the delta did not move is named as unmoved rather
        // than left out, so a reader counts ten either way.
        assert_eq!(
            rendered
                .lines()
                .filter(|line| line.starts_with("unmoved_row "))
                .count(),
            PositivePrivateClass::ALL.len(),
        );
        assert!(rendered.contains("unmoved_row private-sponsor-values"));
    }

    #[test]
    fn the_sponsor_row_cannot_be_moved_even_on_an_acceptance() {
        // Not by the constructor, which is where a ceremony would try.
        assert_eq!(
            moved_on_acceptance(PositivePrivateClass::PrivateSponsorValues, "aa".repeat(32)).err(),
            Some(CloseoutRefusal::SponsorRowMoved),
        );
    }

    #[test]
    fn a_closeout_clearing_two_residuals_is_refused() {
        let mut offered = parts(stopped_ledger());
        offered.cleared_residuals = BTreeSet::from([
            CLEARED_BY_FUNDING,
            LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
        ]);
        assert!(matches!(
            validate_closeout(offered),
            Err(CloseoutRefusal::ClearedResidualsAreNotExactlyOne { .. }),
        ));
    }

    #[test]
    fn funding_may_not_clear_the_digest_blocker() {
        let mut offered = parts(stopped_ledger());
        offered.cleared_residuals =
            BTreeSet::from([LiveInfrastructureBlocker::OwnerSighashNotComputable]);
        // It is refused as the wrong residual before the dedicated
        // check is reached, and that is the correct order: the set is
        // exhaustively wrong rather than wrong in one interesting way.
        assert_eq!(
            validate_closeout(offered).err(),
            Some(CloseoutRefusal::ClearedTheWrongResidual {
                offered: LiveInfrastructureBlocker::OwnerSighashNotComputable,
            }),
        );
    }

    #[test]
    fn a_row_moved_before_the_external_closure_is_refused() {
        let mut offered = parts(stopped_ledger());
        offered.pre_sighash_matrix_delta = 1;
        assert_eq!(
            validate_closeout(offered).err(),
            Some(CloseoutRefusal::PreSighashMatrixDeltaIsNotZero { offered: 1 }),
        );
    }

    #[test]
    fn a_completed_disposition_over_a_stopped_order_is_refused() {
        // The invariant that stops a closeout from being written beside
        // its own evidence rather than out of it.
        let mut offered = parts(stopped_ledger());
        offered.disposition = CloseoutDisposition::Completed;
        assert_eq!(
            validate_closeout(offered).err(),
            Some(CloseoutRefusal::DispositionDisagreesWithTheLedger {
                claimed: CloseoutDisposition::Completed,
                ledger_stopped_at: Some(RestartStep::AcceptedSponsorlessControl),
            }),
        );
    }

    #[test]
    fn one_class_cannot_move_twice() {
        let mut offered = parts(stopped_ledger());
        offered.wave5_matrix_delta = vec![
            moved_on_acceptance(PositivePrivateClass::OneToOne, "aa".repeat(32))
                .expect("a movable class"),
            moved_on_acceptance(PositivePrivateClass::OneToOne, "bb".repeat(32))
                .expect("a movable class"),
        ];
        assert_eq!(
            validate_closeout(offered).err(),
            Some(CloseoutRefusal::ClassMovedTwice(
                PositivePrivateClass::OneToOne
            )),
        );
    }
}
