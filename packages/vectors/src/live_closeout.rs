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
//! take on trust. So [`MovedMatrixRow`] carries the identity. Every
//! current [`PositivePrivateClass`] is admissible; the provenance token,
//! rather than a class prohibition, is the boundary that keeps an
//! unobserved row out (tab:guide-ctf-exec:row-delta).

use std::collections::{BTreeMap, BTreeSet};

use crate::live_evidence::LiveInfrastructureBlocker;
use crate::live_restart::{
    RecordedStepResult, RestartLedger, RestartLedgerStatus, RestartOrderRefusal, RestartStep,
};
use crate::live_roles::{
    CandidateEvidenceRole, CeremonyEvidenceRoles, CeremonyEvidenceRolesBuilder,
    EvidenceAssemblyRefusal, RoleEvidence, RoleGround,
};
use crate::recorded_acceptance::RecordedAcceptance;
use transaction::{TransactionIdentityParseError, Txid};

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
    /// Admissible on the same terms as every other current class: it
    /// moves only on a provenance-bearing recorded acceptance.
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
    accepted_identity: Txid,
}

impl MovedMatrixRow {
    /// The class.
    #[must_use]
    pub const fn class(&self) -> PositivePrivateClass {
        self.class
    }

    /// The target-computed identity the acceptance was observed at.
    #[must_use]
    pub const fn accepted_identity(&self) -> Txid {
        self.accepted_identity
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
    /// A committed run-of-record identity is not a target transaction
    /// identity. Callers cannot arrange this refusal because public
    /// evidence boundaries accept only [`RecordedAcceptance`].
    MalformedRecordedIdentity(TransactionIdentityParseError),
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
    /// The restart ledger refused an entry, with its exact cause.
    RestartOrder(RestartOrderRefusal),
    /// The evidence-role map refused an entry or incomplete closeout,
    /// with its exact cause.
    EvidenceAssembly(EvidenceAssemblyRefusal),
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
        /// The lifecycle state derived from the ledger.
        ledger_status: RestartLedgerStatus,
    },
}

impl From<RestartOrderRefusal> for CloseoutRefusal {
    fn from(source: RestartOrderRefusal) -> Self {
        Self::RestartOrder(source)
    }
}

impl From<EvidenceAssemblyRefusal> for CloseoutRefusal {
    fn from(source: EvidenceAssemblyRefusal) -> Self {
        Self::EvidenceAssembly(source)
    }
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
/// Caller-authored text is not an acceptance provenance token and does
/// not type-check at this boundary:
///
/// ```compile_fail
/// use vectors::live_closeout::{PositivePrivateClass, moved_on_acceptance};
///
/// let _ = moved_on_acceptance(PositivePrivateClass::OneToOne, "prose");
/// ```
///
pub const fn moved_on_acceptance(
    class: PositivePrivateClass,
    accepted_identity: RecordedAcceptance,
) -> MovedMatrixRow {
    MovedMatrixRow {
        class,
        accepted_identity: accepted_identity.accepted_identity(),
    }
}

fn recorded_identity(
    mint: fn() -> Result<RecordedAcceptance, TransactionIdentityParseError>,
) -> Result<RecordedAcceptance, CloseoutRefusal> {
    mint().map_err(CloseoutRefusal::MalformedRecordedIdentity)
}

fn settle_role(
    builder: &mut CeremonyEvidenceRolesBuilder,
    role: CandidateEvidenceRole,
    ground: RoleGround,
) -> Result<(), CloseoutRefusal> {
    builder.settle(role, RoleEvidence::new(role, ground))?;
    Ok(())
}

fn complete_roles(
    builder: CeremonyEvidenceRolesBuilder,
) -> Result<CeremonyEvidenceRoles, CloseoutRefusal> {
    let roles = builder.complete()?;
    Ok(roles)
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
            offered: parts.cleared_residuals,
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

    // 4. Every current class is admissible, and no class is twice.
    let mut seen = BTreeSet::new();
    for row in &parts.wave5_matrix_delta {
        if !seen.insert(row.class()) {
            return Err(CloseoutRefusal::ClassMovedTwice(row.class()));
        }
    }

    // 5. The disposition is the ledger's, not the author's.
    let ledger_status = parts.ledger.status();
    let agrees = match (&parts.disposition, ledger_status) {
        (CloseoutDisposition::Completed, RestartLedgerStatus::Completed) => true,
        (
            CloseoutDisposition::TypedStopped { step, blocker },
            RestartLedgerStatus::Stopped {
                step: actual_step,
                blocker: actual_blocker,
            },
        ) => *step == actual_step && *blocker == actual_blocker,
        _ => false,
    };
    if !agrees {
        return Err(CloseoutRefusal::DispositionDisagreesWithTheLedger {
            claimed: parts.disposition,
            ledger_status,
        });
    }

    Ok(ConfidentialFundingCloseoutReport { parts })
}

/// The order as this wave executed it.
///
/// # Why the closeout is assembled here and not written down
///
/// The closeout has three invariants that are checked rather than
/// trusted, and a document cannot be checked. Assembling it in code
/// means the workspace's own suite refuses a closeout that cleared two
/// residuals, that cleared the digest blocker from funding evidence, or
/// whose disposition disagrees with the order that was actually run.
///
/// # What the order reached, and where it stops
///
/// Step two, stopping at step three. Steps one and two accepted, on two
/// runs against a real node. Step three is where this wave's execution
/// stops, and the stop is typed rather than silent.
///
/// Split from the assembly because the order and the report are two
/// readings: what happened, and what is claimed from it. A function
/// that did both would let a reader lose track of which sentences were
/// observations.
fn wave_five_ledger() -> Result<RestartLedger, CloseoutRefusal> {
    use crate::live_private_restart::run_of_record as run;
    use crate::live_restart::RestartStep;

    let mut ledger = RestartLedger::new();
    ledger.record(
        RestartStep::AcceptedSponsorlessControl,
        RecordedStepResult::Accepted {
            accepted_identities: vec![run::ACCEPTED_TXID.to_owned()],
            established: "one sponsorless private one-to-one receipt-covenant control, \
                          accepted, its witness verified from the node's own copy against \
                          an independently recomputed message"
                .to_owned(),
        },
    )?;
    ledger.record(
        RestartStep::BothParitySuccessors,
        RecordedStepResult::Accepted {
            accepted_identities: vec![
                run::ACCEPTED_TXID.to_owned(),
                run::PARITY_ACCEPTED_TXID.to_owned(),
            ],
            established: "both admitted commitment parities, each consumed in its own \
                          complete accepted successor"
                .to_owned(),
        },
    )?;
    ledger.record(
        RestartStep::TargetCtConservation,
        RecordedStepResult::StoppedTyped {
            blocker: LiveInfrastructureBlocker::NoAcceptingControlExists,
            because: "a conservation claim needs a refused non-conserving case beside the \
                      accepted conserving one, and the non-conserving case is the fourth \
                      step's wrong-blinder mutation, which this wave did not run"
                .to_owned(),
        },
    )?;
    Ok(ledger)
}

/// The evidence-role map for the control ceremony.
///
/// Every one of the seven roles is settled on its own ground, and the
/// three that are not observations say which step the order stopped
/// before or which ceremony owns the question instead. There is no
/// entry filled from another entry's evidence, which is the property
/// the builder refuses one call at a time.
fn wave_five_roles() -> Result<CeremonyEvidenceRoles, CloseoutRefusal> {
    use crate::live_private_restart::run_of_record as run;
    use crate::live_restart::RestartStep;

    let mut roles = CeremonyEvidenceRolesBuilder::new();
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Funding,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(run::accepted)?,
            independent_check: "the funding record validated against the first-party \
                                commitment oracle, and both predecessor coins matched"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::OwnerSignature,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(run::accepted)?,
            independent_check: "the witness read back out of the node's own copy verifies \
                                against an independently recomputed message"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Safety,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(run::parity_accepted)?,
            independent_check: "two positive private matrix rows moved, each on an acceptance \
                                of its own shape"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::CtConservation,
        RoleGround::OrderNotReached {
            stopped_before_step: RestartStep::TargetCtConservation.number(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Minimality,
        RoleGround::OrderNotReached {
            stopped_before_step: RestartStep::TargetCtConservation.number(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Resource,
        RoleGround::OutsideThisCeremony {
            owned_by: "the live-transfer resource study".to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Lifecycle,
        RoleGround::OutsideThisCeremony {
            owned_by: "the lifecycle report".to_owned(),
        },
    )?;
    complete_roles(roles)
}

/// The confidential-funding guide's closeout, as this wave's runs
/// settled it.
///
/// # Errors
///
/// Every member of [`CloseoutRefusal`]. It returns a `Result` rather
/// than a value precisely so that the invariants are checked on every
/// call rather than at the moment somebody wrote the numbers down.
pub fn wave_five_closeout() -> Result<ConfidentialFundingCloseoutReport, CloseoutRefusal> {
    use crate::live_private_restart::run_of_record as run;
    use crate::live_restart::RestartStep;

    validate_closeout(CloseoutParts {
        disposition: CloseoutDisposition::TypedStopped {
            step: RestartStep::TargetCtConservation,
            blocker: LiveInfrastructureBlocker::NoAcceptingControlExists,
        },
        ledger: wave_five_ledger()?,
        contracts: BTreeMap::from([(
            "private-one-to-one-control".to_owned(),
            "byte-identity".to_owned(),
        )]),
        funding: BTreeMap::from([(
            "ctf-v1/predecessor-dual-parity".to_owned(),
            run::PREDECESSOR_DIGEST.to_owned(),
        )]),
        target_facts: vec![
            "target Elements Core v28.99.0-b7fc5d080a7e".to_owned(),
            format!("issued_asset {}", run::ISSUED_ASSET),
            format!("successor_digest {}", run::SUCCESSOR_DIGEST),
            format!("parity_successor_digest {}", run::PARITY_SUCCESSOR_DIGEST),
            format!("submitted_bytes {}", run::SUBMITTED_BYTES),
        ],
        sighash_result: Some(
            "the reviewed owner-sighash profile, established over its six-member required set \
             by a verdict this guide consumed and did not produce"
                .to_owned(),
        ),
        roles: BTreeMap::from([("private-one-to-one-control".to_owned(), wave_five_roles()?)]),
        cleared_residuals: BTreeSet::from([CLEARED_BY_FUNDING]),
        blockers: BTreeSet::from([
            LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
            LiveInfrastructureBlocker::PredecessorConstructorAbsent,
        ]),
        // Zero, and it has to be: nothing moved before the external
        // closure, because the external closure was already recorded
        // when this wave began.
        pre_sighash_matrix_delta: 0,
        wave5_matrix_delta: vec![
            moved_on_acceptance(
                PositivePrivateClass::OneToOne,
                recorded_identity(run::accepted)?,
            ),
            moved_on_acceptance(
                PositivePrivateClass::BothCommitmentParityForms,
                recorded_identity(run::parity_accepted)?,
            ),
        ],
        exclusions: vec![
            "private-amounts".to_owned(),
            "fixture-openings".to_owned(),
            "value-blinders".to_owned(),
            "nonce-inputs".to_owned(),
            "proof-inputs".to_owned(),
            "wallet-data".to_owned(),
            "credentials".to_owned(),
            "environment-values".to_owned(),
            "diagnostics".to_owned(),
        ],
        non_claims: vec![
            "production-custody-not-established".to_owned(),
            "production-cryptography-not-established".to_owned(),
            "production-multi-owner-protocol-not-established".to_owned(),
            "privacy-not-established".to_owned(),
            "public-fixtures-not-secret".to_owned(),
            "stock-rpc-hybrid-support-not-established".to_owned(),
            "funding-does-not-prove-transfer-safety-minimality-or-resource".to_owned(),
            "malformed-rejection-does-not-replace-control".to_owned(),
            "owner-sighash-not-established-here".to_owned(),
            "candidate-interface-not-final".to_owned(),
        ],
        wall_times: BTreeMap::from([("private-one-to-one-control".to_owned(), run::WALL_SECONDS)]),
    })
}

/// The follow-up evidence wave's order, as it was executed against a real
/// node.
///
/// # What it reached, and where it stops
///
/// Steps one through four accepted; the order stops typed at step five.
/// Steps one and two re-ran the one-to-one control and the second parity
/// on fresh chains. Step three recorded target CT conservation against the
/// accepted control. Step four ran the three proof-negatives from that
/// control and observed each refused at the mempool boundary.
///
/// # The Wave-5 mis-typing this corrects, named rather than deleted
///
/// Wave 5 stopped at step three citing
/// [`LiveInfrastructureBlocker::NoAcceptingControlExists`], against a
/// ledger whose first entry was an accepted control. That blocker is
/// derived while no positive control exists, and one did — so the stop was
/// mis-typed. This ledger records step three as accepted on the interlock
/// ruling: its conservation record needs no refused case beyond the
/// wrong-blinder mutant step four runs, and §10.5 step 3 asks only for
/// conservation recorded against a balance-valid control.
///
/// # The step-five stop, typed and honest
///
/// Step five's remaining positive shapes need multi-output and multi-input
/// fixtures this wave does not build, which is
/// [`LiveInfrastructureBlocker::MultiOutputShapeConstructorAbsent`]. It is
/// a constructor absence like the predecessor one, not a target verdict:
/// a following wave clears it by building the fixtures.
fn wave_six_ledger() -> Result<RestartLedger, CloseoutRefusal> {
    use crate::live_conservation_negatives::run_of_record as cn;
    use crate::live_private_restart::run_of_record as run;
    use crate::live_restart::RestartStep;

    let mut ledger = RestartLedger::new();
    ledger.record(
        RestartStep::AcceptedSponsorlessControl,
        RecordedStepResult::Accepted {
            accepted_identities: vec![run::ACCEPTED_TXID.to_owned()],
            established: "one sponsorless private one-to-one receipt-covenant control, \
                          re-run on a fresh chain, accepted, its witness verified from the \
                          node's own copy against an independently recomputed message"
                .to_owned(),
        },
    )?;
    ledger.record(
        RestartStep::BothParitySuccessors,
        RecordedStepResult::Accepted {
            accepted_identities: vec![
                run::ACCEPTED_TXID.to_owned(),
                run::PARITY_ACCEPTED_TXID.to_owned(),
            ],
            established: "both admitted commitment parities, each consumed in its own \
                          complete accepted successor, re-run on fresh chains and \
                          reproducing the prior identities byte for byte"
                .to_owned(),
        },
    )?;
    ledger.record(
        RestartStep::TargetCtConservation,
        RecordedStepResult::Accepted {
            accepted_identities: vec![cn::CONTROL_ACCEPTED_TXID.to_owned()],
            established: "target CT conservation, recorded against a balance-valid accepted \
                          control as its conserving half and one wrong-blinder mutant's \
                          balance-layer refusal as its non-conserving half. DISCLOSURE: the \
                          non-conserving half is the SAME observed run the fourth step \
                          records as its wrong-blinder proof-negative — one observed run, not \
                          two — so a reader of two accepted ledger entries does not \
                          double-count it as two observations"
                .to_owned(),
        },
    )?;
    ledger.record(
        RestartStep::ProofNegatives,
        RecordedStepResult::Accepted {
            accepted_identities: vec![cn::CONTROL_ACCEPTED_TXID.to_owned()],
            established: "wrong-blinder, missing-rangeproof, and malformed-rangeproof, all \
                          three submitted to the mempool boundary and refused at \
                          ConsensusRejectionBeforeScript with the one identical string \
                          bad-txns-in-ne-out, each attributed by its mutated field against \
                          the accepted control rather than by a target layer the three could \
                          differ on. The wrong-blinder case is the same observed refusal the \
                          third step records as conservation's non-conserving half"
                .to_owned(),
        },
    )?;
    ledger.record(
        RestartStep::RemainingPositiveShapes,
        RecordedStepResult::StoppedTyped {
            blocker: LiveInfrastructureBlocker::MultiOutputShapeConstructorAbsent,
            because: "the remaining positive shapes — split, many-to-many, several distinct \
                      owners — need multi-output and multi-input fixtures this wave does not \
                      build, and private-merge is additionally structurally unconstructible \
                      under the registry's two-output rule; a following wave builds the \
                      fixtures and moves each shape on its own observed acceptance"
                .to_owned(),
        },
    )?;
    Ok(ledger)
}

/// The evidence-role map for the follow-up wave's control ceremony.
///
/// The conservation role is now an observed acceptance rather than an
/// order-not-reached, which is the whole difference from Wave 5: the run
/// that stopped that entry has been taken.
fn wave_six_roles() -> Result<CeremonyEvidenceRoles, CloseoutRefusal> {
    use crate::live_conservation_negatives::run_of_record as cn;
    use crate::live_private_restart::run_of_record as run;
    use crate::live_restart::RestartStep;

    let mut roles = CeremonyEvidenceRolesBuilder::new();
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Funding,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(run::accepted)?,
            independent_check: "the funding record validated against the first-party \
                                commitment oracle, and both predecessor coins matched"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::OwnerSignature,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(run::accepted)?,
            independent_check: "the witness read back out of the node's own copy verifies \
                                against an independently recomputed message"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Safety,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(run::parity_accepted)?,
            independent_check: "three positive private matrix rows moved, each on an acceptance \
                                of its own shape"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::CtConservation,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(cn::control_accepted)?,
            independent_check: "the target's own commitment-balance rule accepted the conserving \
                                control and refused the wrong-blinder mutant at the balance layer, \
                                the same observed run the proof-negatives record"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Minimality,
        RoleGround::OrderNotReached {
            stopped_before_step: RestartStep::MinimalityPairs.number(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Resource,
        RoleGround::OutsideThisCeremony {
            owned_by: "the live-transfer resource study".to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Lifecycle,
        RoleGround::OutsideThisCeremony {
            owned_by: "the lifecycle report".to_owned(),
        },
    )?;
    complete_roles(roles)
}

fn wave_six_matrix_delta() -> Result<Vec<MovedMatrixRow>, CloseoutRefusal> {
    use crate::live_conservation_negatives::run_of_record as cn;
    use crate::live_private_restart::run_of_record as run;

    Ok(vec![
        moved_on_acceptance(
            PositivePrivateClass::OneToOne,
            recorded_identity(run::accepted)?,
        ),
        moved_on_acceptance(
            PositivePrivateClass::BothCommitmentParityForms,
            recorded_identity(run::parity_accepted)?,
        ),
        moved_on_acceptance(
            PositivePrivateClass::TargetCtConservation,
            recorded_identity(cn::control_accepted)?,
        ),
    ])
}

/// The follow-up evidence wave's closeout, as its runs settled it.
///
/// # Errors
///
/// Every member of [`CloseoutRefusal`]. It returns a `Result` rather than
/// a value precisely so that the invariants are checked on every call.
pub fn wave_six_closeout() -> Result<ConfidentialFundingCloseoutReport, CloseoutRefusal> {
    use crate::live_conservation_negatives::run_of_record as cn;
    use crate::live_private_restart::run_of_record as run;
    use crate::live_restart::RestartStep;

    validate_closeout(CloseoutParts {
        disposition: CloseoutDisposition::TypedStopped {
            step: RestartStep::RemainingPositiveShapes,
            blocker: LiveInfrastructureBlocker::MultiOutputShapeConstructorAbsent,
        },
        ledger: wave_six_ledger()?,
        contracts: BTreeMap::from([
            (
                "private-one-to-one-control".to_owned(),
                "byte-identity".to_owned(),
            ),
            (
                "conservation-and-proof-negatives".to_owned(),
                "byte-identity".to_owned(),
            ),
        ]),
        funding: BTreeMap::from([(
            "ctf-v1/predecessor-dual-parity".to_owned(),
            cn::PREDECESSOR_DIGEST.to_owned(),
        )]),
        target_facts: vec![
            "target Elements Core v28.99.0-b7fc5d080a7e".to_owned(),
            format!("issued_asset {}", cn::ISSUED_ASSET),
            format!("control_accepted_txid {}", cn::CONTROL_ACCEPTED_TXID),
            format!("control_submitted_bytes {}", cn::CONTROL_SUBMITTED_BYTES),
            format!("mutant_reject_detail {}", cn::MUTANT_REJECT_DETAIL),
            "mutants_refused_at consensus-rejection-before-script".to_owned(),
            format!(
                "wrong_blinder_field_range {}..{}",
                cn::WRONG_BLINDER_FIELD_RANGE.0,
                cn::WRONG_BLINDER_FIELD_RANGE.1,
            ),
            format!(
                "rangeproof_field_range {}..{}",
                cn::RANGEPROOF_FIELD_RANGE.0,
                cn::RANGEPROOF_FIELD_RANGE.1,
            ),
        ],
        sighash_result: Some(
            "the reviewed owner-sighash profile, established over its six-member required set \
             by a verdict this guide consumed and did not produce"
                .to_owned(),
        ),
        roles: BTreeMap::from([(
            "conservation-and-proof-negatives".to_owned(),
            wave_six_roles()?,
        )]),
        cleared_residuals: BTreeSet::from([CLEARED_BY_FUNDING]),
        blockers: BTreeSet::from([
            LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
            LiveInfrastructureBlocker::PredecessorConstructorAbsent,
        ]),
        pre_sighash_matrix_delta: 0,
        wave5_matrix_delta: wave_six_matrix_delta()?,
        exclusions: vec![
            "private-amounts".to_owned(),
            "fixture-openings".to_owned(),
            "value-blinders".to_owned(),
            "nonce-inputs".to_owned(),
            "proof-inputs".to_owned(),
            "wallet-data".to_owned(),
            "credentials".to_owned(),
            "environment-values".to_owned(),
            "diagnostics".to_owned(),
        ],
        non_claims: vec![
            "production-custody-not-established".to_owned(),
            "production-cryptography-not-established".to_owned(),
            "production-multi-owner-protocol-not-established".to_owned(),
            "privacy-not-established".to_owned(),
            "public-fixtures-not-secret".to_owned(),
            "stock-rpc-hybrid-support-not-established".to_owned(),
            "funding-does-not-prove-transfer-safety-minimality-or-resource".to_owned(),
            "malformed-rejection-does-not-replace-control".to_owned(),
            "owner-sighash-not-established-here".to_owned(),
            "candidate-interface-not-final".to_owned(),
        ],
        wall_times: BTreeMap::from([
            ("private-one-to-one-control".to_owned(), run::WALL_SECONDS),
            (
                "conservation-and-proof-negatives".to_owned(),
                cn::WALL_SECONDS,
            ),
        ]),
    })
}

/// The shape wave's restart ledger: the fifth step taken, the sixth
/// stopped.
///
/// # What changed from the follow-up wave's ledger
///
/// One entry. Steps one through four are the same four observed results,
/// unchanged and re-stated rather than re-derived. Step five was
/// `StoppedTyped` on
/// [`LiveInfrastructureBlocker::MultiOutputShapeConstructorAbsent`] and is
/// now accepted: the multi-output and multi-input fixtures exist and
/// three of the remaining positive shapes were submitted to a real node
/// and accepted, each moving its own row on its own identity.
///
/// # Why step five is accepted with a shape that did not run
///
/// The step asks for "the remaining positive private shapes, only where
/// each one accepts". The condition is per shape, so a shape that cannot
/// accept does not stop the step; it goes unmoved and typed. Private-merge
/// is that shape, and it is not unbuilt but UNCONSTRUCTIBLE: a merge is
/// one output and the registry refuses a manifest with fewer than two,
/// because a confidential balance needs a balancing output. The conflict
/// between the guide's merge predicate and that floor is filed as an
/// erratum against the guide rather than resolved by relaxing either
/// rule, and the entry says so in its own words so that a reader of an
/// accepted step five does not read it as four shapes out of four.
///
/// # Where it stops
///
/// Step six, on [`LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent`]
/// — a residual this guide does not clear and this wave does not touch.
/// Step seven follows it in the mandatory order and is therefore not
/// reached; the ledger's own `AfterStop` refusal is what makes that
/// structural rather than a matter of this wave's choosing.
///
/// # This ledger is a record, and it is not amended
///
/// The dependency it stops on has since closed: a later wave threaded
/// the deployment's fee-role program digest, submitted a sponsored
/// control, and had it accepted. Nothing here changes on that account.
/// This function states what WAVE SEVEN observed, and wave seven
/// observed a typed stop; editing it to say otherwise would be
/// backdating an acceptance into a wave that did not have one, which is
/// the one thing a ledger of record exists to prevent. Its `because`
/// prose is read as of its own wave, and a reader wanting the current
/// standing reads `carried_residuals` instead.
fn wave_seven_ledger() -> Result<RestartLedger, CloseoutRefusal> {
    use crate::live_restart::RestartStep;

    let mut ledger = wave_seven_steps_one_to_five()?;
    ledger.record(
        RestartStep::SponsorCases,
        RecordedStepResult::StoppedTyped {
            blocker: LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
            because: "the sponsor cases run only after their independent signer dependency \
                      closes by observation, and it has not: no target has accepted a control \
                      carrying a sponsor owner's authorization. The dependency is not this \
                      guide's to close, the private-sponsor-values row may not move, and step \
                      seven's disclosure-minimality pairs follow this step in the mandatory \
                      order and are therefore not reached"
                .to_owned(),
        },
    )?;
    Ok(ledger)
}

/// The restart order's first five steps, as wave seven observed them.
///
/// Extracted so that a later wave can carry the order forward from the
/// same five entries rather than restating them. The extraction is a
/// move and not a rewrite: the entries below are wave seven's own, to
/// the byte, and a wave that wanted to say something different about
/// step three would have to record a different entry rather than edit
/// this one.
///
/// The five results are the same for every wave that has them because
/// they are OBSERVATIONS. A later wave does not re-derive what a node
/// said; it inherits it, which is what "each step's entry is the
/// previous step's observed result" means.
fn wave_seven_steps_one_to_five() -> Result<RestartLedger, CloseoutRefusal> {
    use crate::live_conservation_negatives::run_of_record as cn;
    use crate::live_multi_shapes::run_of_record as ms;
    use crate::live_private_restart::run_of_record as run;
    use crate::live_restart::RestartStep;

    let mut ledger = RestartLedger::new();
    ledger.record(
        RestartStep::AcceptedSponsorlessControl,
        RecordedStepResult::Accepted {
            accepted_identities: vec![run::ACCEPTED_TXID.to_owned()],
            established: "one sponsorless private one-to-one receipt-covenant control, \
                          accepted, its witness verified from the node's own copy against an \
                          independently recomputed message"
                .to_owned(),
        },
    )?;
    ledger.record(
        RestartStep::BothParitySuccessors,
        RecordedStepResult::Accepted {
            accepted_identities: vec![
                run::ACCEPTED_TXID.to_owned(),
                run::PARITY_ACCEPTED_TXID.to_owned(),
            ],
            established: "both admitted commitment parities, each consumed in its own \
                          complete accepted successor"
                .to_owned(),
        },
    )?;
    ledger.record(
        RestartStep::TargetCtConservation,
        RecordedStepResult::Accepted {
            accepted_identities: vec![cn::CONTROL_ACCEPTED_TXID.to_owned()],
            established: "target CT conservation, recorded against a balance-valid accepted \
                          control as its conserving half and one wrong-blinder mutant's \
                          balance-layer refusal as its non-conserving half. DISCLOSURE: the \
                          non-conserving half is the SAME observed run the fourth step \
                          records as its wrong-blinder proof-negative — one observed run, not \
                          two"
            .to_owned(),
        },
    )?;
    ledger.record(
        RestartStep::ProofNegatives,
        RecordedStepResult::Accepted {
            accepted_identities: vec![cn::CONTROL_ACCEPTED_TXID.to_owned()],
            established: "wrong-blinder, missing-rangeproof, and malformed-rangeproof, all \
                          three submitted to the mempool boundary and refused at \
                          ConsensusRejectionBeforeScript with the one identical string \
                          bad-txns-in-ne-out, each attributed by its mutated field"
                .to_owned(),
        },
    )?;
    ledger.record(
        RestartStep::RemainingPositiveShapes,
        RecordedStepResult::Accepted {
            accepted_identities: vec![
                ms::SPLIT_ACCEPTED_TXID.to_owned(),
                ms::MANY_TO_MANY_ACCEPTED_TXID.to_owned(),
                ms::SEVERAL_OWNERS_ACCEPTED_TXID.to_owned(),
            ],
            established: "THREE remaining positive private shapes, each built as its own \
                          multi-output or multi-input fixture and each accepted by a real \
                          node on its own shape: a split of one receipt into three outputs, a \
                          representative many-to-many of two receipts into three outputs, and \
                          a two-receipt transfer under two distinct owners into two outputs. \
                          Every one carries a verified readback witness. DISCLOSURE: \
                          private-merge did NOT run and is not unbuilt but UNCONSTRUCTIBLE — \
                          a merge is one output and the registry refuses fewer than two, \
                          because a confidential balance needs a balancing output — so its \
                          row stays unmoved and the predicate conflict is filed as a guide \
                          erratum rather than resolved here. An accepted fifth step is \
                          therefore three shapes of four, not four of four"
                .to_owned(),
        },
    )?;
    Ok(ledger)
}

/// The evidence-role map for the shape wave's ceremonies.
///
/// The safety role now cites a shape acceptance rather than a parity one:
/// six positive private rows are answered, and the identity carried is the
/// widest shape the wave submitted rather than the earliest.
fn wave_seven_roles() -> Result<CeremonyEvidenceRoles, CloseoutRefusal> {
    use crate::live_conservation_negatives::run_of_record as cn;
    use crate::live_multi_shapes::run_of_record as ms;
    use crate::live_private_restart::run_of_record as run;
    use crate::live_restart::RestartStep;

    let mut roles = CeremonyEvidenceRolesBuilder::new();
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Funding,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(run::accepted)?,
            independent_check: "the funding record validated against the first-party \
                                commitment oracle, and both predecessor coins matched"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::OwnerSignature,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(ms::several_owners_accepted)?,
            independent_check: "two receipts under two distinct owners, each input carrying the \
                                leaf its own position executes, and the witness read back out of \
                                the node's own copy verifies against an independently recomputed \
                                message"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Safety,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(ms::many_to_many_accepted)?,
            independent_check: "six positive private matrix rows moved, each on an acceptance of \
                                its own shape, and the four that did not move are named with \
                                their grounds"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::CtConservation,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(cn::control_accepted)?,
            independent_check: "the target's own commitment-balance rule accepted the conserving \
                                control and refused the wrong-blinder mutant at the balance layer"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Minimality,
        RoleGround::OrderNotReached {
            stopped_before_step: RestartStep::MinimalityPairs.number(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Resource,
        RoleGround::OutsideThisCeremony {
            owned_by: "the live-transfer resource study".to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Lifecycle,
        RoleGround::OutsideThisCeremony {
            owned_by: "the lifecycle report".to_owned(),
        },
    )?;
    complete_roles(roles)
}

/// The target facts the shape wave observed.
///
/// Each shape's identity, successor digest, and submitted byte count, plus
/// the two cardinality arrays, so a reader counts the inputs and outputs
/// of each shape from the record rather than from a shape's name. Split
/// out from the closeout itself because a list this long inside it makes
/// one function of two jobs.
fn wave_seven_target_facts() -> Vec<String> {
    use crate::live_multi_shapes::run_of_record as ms;

    vec![
        "target Elements Core v28.99.0-b7fc5d080a7e".to_owned(),
        format!("issued_asset {}", ms::ISSUED_ASSET),
        format!("split_accepted_txid {}", ms::SPLIT_ACCEPTED_TXID),
        format!("split_successor_digest {}", ms::SPLIT_SUCCESSOR_DIGEST),
        format!("split_submitted_bytes {}", ms::SPLIT_SUBMITTED_BYTES),
        format!(
            "many_to_many_accepted_txid {}",
            ms::MANY_TO_MANY_ACCEPTED_TXID,
        ),
        format!(
            "many_to_many_successor_digest {}",
            ms::MANY_TO_MANY_SUCCESSOR_DIGEST,
        ),
        format!(
            "many_to_many_submitted_bytes {}",
            ms::MANY_TO_MANY_SUBMITTED_BYTES,
        ),
        format!(
            "several_owners_accepted_txid {}",
            ms::SEVERAL_OWNERS_ACCEPTED_TXID,
        ),
        format!(
            "several_owners_successor_digest {}",
            ms::SEVERAL_OWNERS_SUCCESSOR_DIGEST,
        ),
        format!(
            "several_owners_submitted_bytes {}",
            ms::SEVERAL_OWNERS_SUBMITTED_BYTES,
        ),
        format!("shape_receipt_leaves {:?}", ms::RECEIPT_LEAVES),
        format!("shape_output_counts {:?}", ms::OUTPUT_COUNTS),
        "shape_runs_serialized_against_one_node true".to_owned(),
    ]
}

fn wave_seven_matrix_delta() -> Result<Vec<MovedMatrixRow>, CloseoutRefusal> {
    use crate::live_conservation_negatives::run_of_record as cn;
    use crate::live_multi_shapes::run_of_record as ms;
    use crate::live_private_restart::run_of_record as run;

    Ok(vec![
        moved_on_acceptance(
            PositivePrivateClass::OneToOne,
            recorded_identity(run::accepted)?,
        ),
        moved_on_acceptance(
            PositivePrivateClass::BothCommitmentParityForms,
            recorded_identity(run::parity_accepted)?,
        ),
        moved_on_acceptance(
            PositivePrivateClass::TargetCtConservation,
            recorded_identity(cn::control_accepted)?,
        ),
        moved_on_acceptance(
            PositivePrivateClass::Split,
            recorded_identity(ms::split_accepted)?,
        ),
        moved_on_acceptance(
            PositivePrivateClass::ManyToManyRepresentative,
            recorded_identity(ms::many_to_many_accepted)?,
        ),
        moved_on_acceptance(
            PositivePrivateClass::SeveralDistinctOwners,
            recorded_identity(ms::several_owners_accepted)?,
        ),
    ])
}

/// The shape wave's closeout, as its runs settled it.
///
/// # What the cleared set does and does not hold
///
/// Exactly one residual, and it is the one confidential funding clears.
/// `MultiOutputShapeConstructorAbsent` is NOT added to it, and the
/// omission is the site's own rule rather than an oversight: that blocker
/// was never a member of `carried_residuals`, which held two when this
/// wave closed and holds one now. It lived in
/// the previous wave's ledger as the fifth step's stop, and the way a
/// constructor absence leaves is that the step it stopped is recorded
/// accepted instead — which is exactly what `wave_seven_ledger` does. A
/// closeout that also listed it as cleared would be counting one clearance
/// twice and would be refused by `validate_closeout` for holding more than
/// one member.
///
/// # Errors
///
/// Every member of [`CloseoutRefusal`]. It returns a `Result` rather than
/// a value precisely so that the invariants are checked on every call.
pub fn wave_seven_closeout() -> Result<ConfidentialFundingCloseoutReport, CloseoutRefusal> {
    use crate::live_conservation_negatives::run_of_record as cn;
    use crate::live_multi_shapes::run_of_record as ms;
    use crate::live_private_restart::run_of_record as run;
    use crate::live_restart::RestartStep;

    validate_closeout(CloseoutParts {
        disposition: CloseoutDisposition::TypedStopped {
            step: RestartStep::SponsorCases,
            blocker: LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
        },
        ledger: wave_seven_ledger()?,
        contracts: BTreeMap::from([
            (
                "private-one-to-one-control".to_owned(),
                "byte-identity".to_owned(),
            ),
            (
                "conservation-and-proof-negatives".to_owned(),
                "byte-identity".to_owned(),
            ),
            ("multi-output-shapes".to_owned(), "byte-identity".to_owned()),
        ]),
        funding: BTreeMap::from([(
            "ctf-v1/predecessor-dual-parity".to_owned(),
            cn::PREDECESSOR_DIGEST.to_owned(),
        )]),
        target_facts: wave_seven_target_facts(),
        sighash_result: Some(
            "the reviewed owner-sighash profile, established over its six-member required set \
             by a verdict this guide consumed and did not produce"
                .to_owned(),
        ),
        roles: BTreeMap::from([("multi-output-shapes".to_owned(), wave_seven_roles()?)]),
        cleared_residuals: BTreeSet::from([CLEARED_BY_FUNDING]),
        blockers: BTreeSet::from([
            LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
            LiveInfrastructureBlocker::PredecessorConstructorAbsent,
        ]),
        pre_sighash_matrix_delta: 0,
        wave5_matrix_delta: wave_seven_matrix_delta()?,
        exclusions: vec![
            "private-amounts".to_owned(),
            "fixture-openings".to_owned(),
            "value-blinders".to_owned(),
            "nonce-inputs".to_owned(),
            "proof-inputs".to_owned(),
            "wallet-data".to_owned(),
            "credentials".to_owned(),
            "environment-values".to_owned(),
            "diagnostics".to_owned(),
        ],
        non_claims: vec![
            "production-custody-not-established".to_owned(),
            "production-cryptography-not-established".to_owned(),
            "production-multi-owner-protocol-not-established".to_owned(),
            "privacy-not-established".to_owned(),
            "public-fixtures-not-secret".to_owned(),
            "stock-rpc-hybrid-support-not-established".to_owned(),
            "funding-does-not-prove-transfer-safety-minimality-or-resource".to_owned(),
            "malformed-rejection-does-not-replace-control".to_owned(),
            "owner-sighash-not-established-here".to_owned(),
            "candidate-interface-not-final".to_owned(),
        ],
        wall_times: BTreeMap::from([
            ("private-one-to-one-control".to_owned(), run::WALL_SECONDS),
            (
                "conservation-and-proof-negatives".to_owned(),
                cn::WALL_SECONDS,
            ),
            ("private-split".to_owned(), ms::SPLIT_WALL_SECONDS),
            (
                "private-many-to-many".to_owned(),
                ms::MANY_TO_MANY_WALL_SECONDS,
            ),
            (
                "private-several-distinct-owners".to_owned(),
                ms::SEVERAL_OWNERS_WALL_SECONDS,
            ),
        ]),
    })
}

/// The minimality wave's ledger: the restart order carried to its end.
///
/// # What advanced, and on what
///
/// Wave seven typed-stopped at step six, and the ledger recording that
/// stop is a record and stays exactly as it is. This is a separate
/// wave's ledger, which is how the order advances: the stopped wave
/// keeps saying what it observed, and a later wave says what it
/// observed.
///
/// Step six's condition is its own text — the sponsor cases run "only
/// after their independent signer dependency closes by observation" —
/// and that dependency closed by observation four arcs ago. A real node
/// accepted a control carrying a sponsor owner's authorization, then a
/// second carrying sponsor change, then a sponsored CONFIDENTIAL
/// successor built through the private construction lane.
///
/// # The chain rule is obeyed rather than relaxed
///
/// Step seven is reached because step six was DISCHARGED, not because
/// the mandatory chain was read loosely. The chain rule was affirmed
/// strict, and under a strict chain a discharged step six is exactly
/// what makes step seven enterable. The step-7 ordering erratum stays
/// filed against the guide on its own merits — a step typed-stopped on
/// a named external dependency should not consume the entry of the
/// steps after it — and nothing here depends on that erratum being
/// answered, which is why this ledger does not wait for it.
///
/// # Both accepted steps are partial, and say so
///
/// Step five's precedent governs: a step whose condition is per-item is
/// accepted on the items that met it, with the shortfall stated in its
/// own entry. Step seven is three pairs of five and says so.
fn wave_eight_ledger() -> Result<RestartLedger, CloseoutRefusal> {
    use crate::live_explicit_shapes::run_of_record as ex;
    use crate::live_multi_shapes::run_of_record as ms;
    use crate::live_restart::RestartStep;
    use crate::live_sponsor_shapes::sponsored_run_of_record as sp;

    let mut ledger = wave_seven_steps_one_to_five()?;
    ledger.record(
        RestartStep::SponsorCases,
        RecordedStepResult::Accepted {
            accepted_identities: vec![
                sp::SPONSORED_ACCEPTED_TXID.to_owned(),
                sp::SPONSORED_CHANGE_ACCEPTED_TXID.to_owned(),
                sp::SPONSORED_PRIVATE_TXID.to_owned(),
            ],
            established: "the sponsor cases, entered only after their independent signer \
                              dependency closed BY OBSERVATION, which is the condition this step \
                              names: a real node accepted a control carrying a sponsor owner's \
                              authorization. Three cases behind it -- the change-absent explicit \
                              control, the change-present explicit control whose change output \
                              was located in the node's own MINED copy rather than inferred from \
                              the request, and the sponsored CONFIDENTIAL successor built through \
                              the private construction lane at a blinded sponsor coin with \
                              committed change. The private-sponsor-values row moves on the last \
                              of these and cites it"
                .to_owned(),
        },
    )?;
    ledger.record(
        RestartStep::MinimalityPairs,
        RecordedStepResult::Accepted {
            accepted_identities: vec![
                ex::ONE_TO_ONE_ACCEPTED_TXID.to_owned(),
                ms::STRICT_ONE_TO_ONE_ACCEPTED_TXID.to_owned(),
                ex::MERGE_ACCEPTED_TXID.to_owned(),
                ms::MERGE_ACCEPTED_TXID.to_owned(),
                ex::SEVERAL_TO_SEVERAL_ACCEPTED_TXID.to_owned(),
                ms::SEVERAL_OWNERS_ACCEPTED_TXID.to_owned(),
            ],
            established: "the disclosure-minimality pairs, entered after step six discharged \
                              and reached through the mandatory order rather than around it. \
                              THREE of the five paired cases have a recorded acceptance of BOTH \
                              members' shapes -- one-to-one, merge, and many-to-many -- and each \
                              satisfies all ten of the pair-acceptance conjuncts, which is what \
                              that rule's own 'only when' requires. DISCLOSURE: no member of any \
                              pair was SUBMITTED. What is cited per member is an acceptance of \
                              that member's SHAPE, which is this workspace's own rule for moving \
                              a row and is a narrower fact than the conjunct's words; the \
                              registry spells the difference at every one of these identities. \
                              DISCLOSURE: split and sponsor do NOT support minimality and neither \
                              is unbuilt. Each lacks a run of its PRIVATE member's shape -- the \
                              recorded private split creates three outputs where the pair's \
                              member creates two, and the one recorded sponsored private \
                              successor carries a blinded sponsor coin with committed change \
                              where the pair's member states an explicit sponsor coin funded to \
                              the fee and no change role. An accepted seventh step is therefore \
                              three pairs of five, not five of five. DISCLOSURE: the \
                              projection-equality-with-paired-explicit row does NOT move on this \
                              step. Its rule wants an observed acceptance of its own shape, and \
                              no run compares the semantic projections of an accepted private \
                              transaction and its paired accepted explicit one; the equality this \
                              workspace holds is first-party, over registry members that never \
                              reached a target"
                .to_owned(),
        },
    )?;
    Ok(ledger)
}

/// The minimality wave's closeout: the first COMPLETED disposition this
/// guide has had.
///
/// # What moved and what did not
///
/// Eight rows, and the two that did not move are the interesting half.
///
/// `deterministic-public-fixture-openings` is ANSWERED in the safety
/// matrix and absent from this delta, and the two facts are consistent
/// rather than in tension. The delta's own rule is that a row enters it
/// carrying the identity a TARGET computed; that row's gate is the
/// byte-identity contract, its evidence is a recomputation, and it has
/// no such identity to carry. It moved in the matrix on
/// `LiveRowStanding::DeterminismObserved`, which was minted for exactly
/// this. A reader comparing the two artifacts should find the row
/// answered in one and unmoved in the other, and that is the vocabulary
/// difference showing rather than a row nobody is checking.
///
/// `projection-equality-with-paired-explicit` is unmoved in both, and
/// its ground is stated in the seventh step's own entry.
///
/// # Errors
///
/// Every member of [`CloseoutRefusal`].
pub fn wave_eight_closeout() -> Result<ConfidentialFundingCloseoutReport, CloseoutRefusal> {
    use crate::live_conservation_negatives::run_of_record as cn;
    use crate::live_multi_shapes::run_of_record as ms;
    use crate::live_private_restart::run_of_record as run;
    use crate::live_sponsor_shapes::sponsored_run_of_record as sp;

    validate_closeout(CloseoutParts {
        disposition: CloseoutDisposition::Completed,
        ledger: wave_eight_ledger()?,
        contracts: BTreeMap::from([
            (
                "private-one-to-one-control".to_owned(),
                "byte-identity".to_owned(),
            ),
            (
                "conservation-and-proof-negatives".to_owned(),
                "byte-identity".to_owned(),
            ),
            ("multi-output-shapes".to_owned(), "byte-identity".to_owned()),
            ("sponsor-shapes".to_owned(), "byte-identity".to_owned()),
        ]),
        funding: BTreeMap::from([(
            "ctf-v1/predecessor-dual-parity".to_owned(),
            cn::PREDECESSOR_DIGEST.to_owned(),
        )]),
        target_facts: wave_eight_target_facts(),
        sighash_result: Some(
            "the reviewed owner-sighash profile, established over its six-member required set \
             by a verdict this guide consumed and did not produce"
                .to_owned(),
        ),
        roles: BTreeMap::from([("minimality-pairs".to_owned(), wave_eight_roles()?)]),
        cleared_residuals: BTreeSet::from([CLEARED_BY_FUNDING]),
        // One carried residual, down from two. `SponsorEnvelopeSignerAbsent`
        // is gone and it did NOT move to the cleared set: this guide did
        // not clear it, and the cleared set holds exactly the residual
        // confidential funding clears. A blocker leaves the carried set
        // the way `MultiOutputShapeConstructorAbsent` left it, by the
        // step it stopped being recorded Accepted -- which for this one
        // is step six, in the ledger above.
        blockers: BTreeSet::from([LiveInfrastructureBlocker::PredecessorConstructorAbsent]),
        pre_sighash_matrix_delta: 0,
        wave5_matrix_delta: vec![
            moved_on_acceptance(
                PositivePrivateClass::OneToOne,
                recorded_identity(run::accepted)?,
            ),
            moved_on_acceptance(
                PositivePrivateClass::BothCommitmentParityForms,
                recorded_identity(run::parity_accepted)?,
            ),
            moved_on_acceptance(
                PositivePrivateClass::TargetCtConservation,
                recorded_identity(cn::control_accepted)?,
            ),
            moved_on_acceptance(
                PositivePrivateClass::Split,
                recorded_identity(ms::split_accepted)?,
            ),
            moved_on_acceptance(
                PositivePrivateClass::ManyToManyRepresentative,
                recorded_identity(ms::many_to_many_accepted)?,
            ),
            moved_on_acceptance(
                PositivePrivateClass::SeveralDistinctOwners,
                recorded_identity(ms::several_owners_accepted)?,
            ),
            // The two this wave adds.
            moved_on_acceptance(
                PositivePrivateClass::Merge,
                recorded_identity(ms::merge_accepted)?,
            ),
            moved_on_acceptance(
                PositivePrivateClass::PrivateSponsorValues,
                recorded_identity(sp::sponsored_private_accepted)?,
            ),
        ],
        exclusions: vec![
            "private-amounts".to_owned(),
            "fixture-openings".to_owned(),
            "value-blinders".to_owned(),
            "nonce-inputs".to_owned(),
            "proof-inputs".to_owned(),
            "wallet-data".to_owned(),
            "credentials".to_owned(),
            "environment-values".to_owned(),
            "diagnostics".to_owned(),
        ],
        non_claims: vec![
            "production-custody-not-established".to_owned(),
            "production-cryptography-not-established".to_owned(),
            "production-multi-owner-protocol-not-established".to_owned(),
            "privacy-not-established".to_owned(),
            "public-fixtures-not-secret".to_owned(),
            "stock-rpc-hybrid-support-not-established".to_owned(),
            "funding-does-not-prove-transfer-safety-minimality-or-resource".to_owned(),
            "malformed-rejection-does-not-replace-control".to_owned(),
            "owner-sighash-not-established-here".to_owned(),
            "candidate-interface-not-final".to_owned(),
        ],
        wall_times: BTreeMap::new(),
    })
}

/// The target facts the minimality wave adds to the shape wave's.
///
/// Appended rather than restated, because the shape wave's facts are
/// observations and a later wave inherits them.
fn wave_eight_target_facts() -> Vec<String> {
    use crate::live_multi_shapes::run_of_record as ms;
    use crate::live_sponsor_shapes::sponsored_run_of_record as sp;

    let mut facts = wave_seven_target_facts();
    facts.push(format!(
        "sponsored_accepted_txid {}",
        sp::SPONSORED_ACCEPTED_TXID,
    ));
    facts.push(format!(
        "sponsored_change_accepted_txid {}",
        sp::SPONSORED_CHANGE_ACCEPTED_TXID,
    ));
    facts.push(format!(
        "sponsored_private_txid {}",
        sp::SPONSORED_PRIVATE_TXID,
    ));
    facts.push(format!(
        "sponsored_private_submitted_bytes {}",
        sp::SPONSORED_PRIVATE_SUBMITTED_BYTES,
    ));
    facts.push(format!("merge_accepted_txid {}", ms::MERGE_ACCEPTED_TXID));
    facts.push(format!(
        "strict_one_to_one_accepted_txid {}",
        ms::STRICT_ONE_TO_ONE_ACCEPTED_TXID,
    ));
    facts.push("minimality_pairs_supporting 3 of 5".to_owned());
    facts
}

/// The evidence-role map for the minimality wave.
///
/// One role changes from wave seven's and it is the one this wave is
/// about: minimality stood at `OrderNotReached` while step seven was
/// unreachable, and the order has now reached it.
fn wave_eight_roles() -> Result<CeremonyEvidenceRoles, CloseoutRefusal> {
    use crate::live_multi_shapes::run_of_record as ms;
    use crate::live_private_restart::run_of_record as run;
    use crate::live_sponsor_shapes::sponsored_run_of_record as sp;

    let mut roles = CeremonyEvidenceRolesBuilder::new();
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Funding,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(run::accepted)?,
            independent_check: "the funding record validated against the first-party \
                                commitment oracle, and both predecessor coins matched"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::OwnerSignature,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(sp::sponsored_private_accepted)?,
            independent_check: "an owner's signature verified out of the node's own copy against \
                                a message recomputed independently of the candidate, on the \
                                sponsored private successor whose sponsor change was located in \
                                the mined bytes"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Safety,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(ms::merge_accepted)?,
            independent_check: "eight positive private matrix rows moved, each on an acceptance \
                                of its own shape; the openings row is answered in the matrix by \
                                a determinism observation and carries no target identity, and \
                                the projection-equality row is unmoved with its ground stated"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::CtConservation,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(
                crate::live_conservation_negatives::run_of_record::control_accepted,
            )?,
            independent_check: "the target's own commitment-balance rule accepted the conserving \
                                control and refused the wrong-blinder mutant at the balance layer"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Minimality,
        RoleGround::ObservedAcceptance {
            accepted_identity: recorded_identity(ms::strict_one_to_one_accepted)?,
            independent_check: "three of the five paired cases satisfy every pair-acceptance \
                                conjunct, each pair's two members having a recorded acceptance \
                                of its own shape; the identity cited is the private half of the \
                                one-to-one pair, and no member of any pair was itself submitted"
                .to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Resource,
        RoleGround::OutsideThisCeremony {
            owned_by: "the live-transfer resource study".to_owned(),
        },
    )?;
    settle_role(
        &mut roles,
        CandidateEvidenceRole::Lifecycle,
        RoleGround::OutsideThisCeremony {
            owned_by: "the lifecycle report".to_owned(),
        },
    )?;
    complete_roles(roles)
}

#[cfg(test)]
mod tests {
    use super::{
        CLEARED_BY_FUNDING, CloseoutDisposition, CloseoutParts, CloseoutRefusal,
        ConfidentialFundingCloseoutReport, PositivePrivateClass, moved_on_acceptance,
        validate_closeout,
    };
    use crate::live_evidence::LiveInfrastructureBlocker;
    use crate::live_restart::{
        RecordedStepResult, RenderedStepResult, RestartLedger, RestartLedgerStatus,
        RestartOrderRefusal, RestartStep,
    };
    use crate::live_roles::{CandidateEvidenceRole, EvidenceAssemblyRefusal};
    use std::collections::{BTreeMap, BTreeSet};

    fn assert_exact_moved_identities(
        report: &ConfidentialFundingCloseoutReport,
        expected: &[(PositivePrivateClass, &str)],
    ) {
        let observed: Vec<_> = report
            .moved_rows()
            .iter()
            .map(|row| (row.class(), row.accepted_identity().to_string()))
            .collect();
        let expected: Vec<_> = expected
            .iter()
            .map(|(class, identity)| (*class, (*identity).to_owned()))
            .collect();
        assert_eq!(observed, expected);

        let rendered = report.render();
        for (class, identity) in expected {
            let line = format!("moved_row {} on {identity}", class.name());
            assert!(rendered.lines().any(|rendered| rendered == line.as_str()));
        }
    }

    #[test]
    fn all_four_closeouts_render_the_exact_recorded_identities() {
        use crate::live_conservation_negatives::run_of_record as cn;
        use crate::live_multi_shapes::run_of_record as ms;
        use crate::live_private_restart::run_of_record as run;
        use crate::live_sponsor_shapes::sponsored_run_of_record as sp;

        let five = super::wave_five_closeout().expect("wave five validates");
        assert_exact_moved_identities(
            &five,
            &[
                (PositivePrivateClass::OneToOne, run::ACCEPTED_TXID),
                (
                    PositivePrivateClass::BothCommitmentParityForms,
                    run::PARITY_ACCEPTED_TXID,
                ),
            ],
        );

        let six = super::wave_six_closeout().expect("wave six validates");
        assert_exact_moved_identities(
            &six,
            &[
                (PositivePrivateClass::OneToOne, run::ACCEPTED_TXID),
                (
                    PositivePrivateClass::BothCommitmentParityForms,
                    run::PARITY_ACCEPTED_TXID,
                ),
                (
                    PositivePrivateClass::TargetCtConservation,
                    cn::CONTROL_ACCEPTED_TXID,
                ),
            ],
        );

        let seven = super::wave_seven_closeout().expect("wave seven validates");
        assert_exact_moved_identities(
            &seven,
            &[
                (PositivePrivateClass::OneToOne, run::ACCEPTED_TXID),
                (
                    PositivePrivateClass::BothCommitmentParityForms,
                    run::PARITY_ACCEPTED_TXID,
                ),
                (
                    PositivePrivateClass::TargetCtConservation,
                    cn::CONTROL_ACCEPTED_TXID,
                ),
                (PositivePrivateClass::Split, ms::SPLIT_ACCEPTED_TXID),
                (
                    PositivePrivateClass::ManyToManyRepresentative,
                    ms::MANY_TO_MANY_ACCEPTED_TXID,
                ),
                (
                    PositivePrivateClass::SeveralDistinctOwners,
                    ms::SEVERAL_OWNERS_ACCEPTED_TXID,
                ),
            ],
        );

        let eight = super::wave_eight_closeout().expect("wave eight validates");
        assert_exact_moved_identities(
            &eight,
            &[
                (PositivePrivateClass::OneToOne, run::ACCEPTED_TXID),
                (
                    PositivePrivateClass::BothCommitmentParityForms,
                    run::PARITY_ACCEPTED_TXID,
                ),
                (
                    PositivePrivateClass::TargetCtConservation,
                    cn::CONTROL_ACCEPTED_TXID,
                ),
                (PositivePrivateClass::Split, ms::SPLIT_ACCEPTED_TXID),
                (
                    PositivePrivateClass::ManyToManyRepresentative,
                    ms::MANY_TO_MANY_ACCEPTED_TXID,
                ),
                (
                    PositivePrivateClass::SeveralDistinctOwners,
                    ms::SEVERAL_OWNERS_ACCEPTED_TXID,
                ),
                (PositivePrivateClass::Merge, ms::MERGE_ACCEPTED_TXID),
                (
                    PositivePrivateClass::PrivateSponsorValues,
                    sp::SPONSORED_PRIVATE_TXID,
                ),
            ],
        );
    }

    #[test]
    fn the_minimality_wave_completes_the_order_and_leaves_wave_seven_untouched() {
        // The two halves of this wave's claim, and the second is as
        // important as the first. A ledger of record exists to stop an
        // acceptance being backdated into a wave that did not have one,
        // so the wave-seven ledger is asserted to still say what it
        // said before this wave's own is looked at.
        let seven = super::wave_seven_ledger().expect("wave seven's ledger builds");
        assert_eq!(seven.stopped_at(), Some(RestartStep::SponsorCases));
        let seven_entries = seven.entries();
        assert_eq!(seven_entries.len(), RestartStep::ALL.len());
        assert_eq!(
            seven_entries[RestartStep::MinimalityPairs.number() as usize - 1].1,
            RenderedStepResult::NotReached,
        );

        // And the order carried to its end by a LATER wave, which is
        // how a strict chain advances: step six discharged, so step
        // seven is enterable.
        let report = super::wave_eight_closeout().expect("the minimality closeout validates");
        assert_eq!(report.parts().disposition, CloseoutDisposition::Completed);
        assert_eq!(report.parts().ledger.stopped_at(), None);
        for (step, result) in report.parts().ledger.entries() {
            assert!(result.continues(), "step {} did not accept", step.number());
        }

        // Eight rows moved, each carrying a target-computed identity.
        assert_eq!(report.moved_rows().len(), 8);
        let moved: BTreeSet<_> = report
            .moved_rows()
            .iter()
            .map(|row| row.class().name())
            .collect();
        assert!(moved.contains("private-merge"));
        assert!(moved.contains("private-sponsor-values"));
        for row in report.moved_rows() {
            assert_eq!(
                row.accepted_identity().to_string().len(),
                64,
                "{}",
                row.class().name()
            );
        }

        // And the two that did not, each for a reason its own step
        // states. The openings row is answered in the SAFETY MATRIX by
        // a determinism observation and is absent here because this
        // delta admits target-computed identities only; the
        // projection-equality row is unmoved in both.
        for unmoved in [
            "deterministic-public-fixture-openings",
            "projection-equality-with-paired-explicit",
        ] {
            assert!(!moved.contains(unmoved), "{unmoved} entered the delta");
        }

        // One carried residual, down from two, and the one that left
        // did NOT move to the cleared set.
        assert_eq!(
            report.parts().blockers,
            BTreeSet::from([LiveInfrastructureBlocker::PredecessorConstructorAbsent]),
        );
        assert_eq!(
            report.parts().cleared_residuals,
            BTreeSet::from([CLEARED_BY_FUNDING])
        );

        // The seventh step states its own shortfall rather than reading
        // as five pairs of five.
        let rendered = report.render();
        assert!(rendered.contains("restart_step 7"));
        let seventh = &report.parts().ledger.entries()[6].1;
        let RenderedStepResult::Accepted { established, .. } = seventh else {
            panic!("the seventh step did not accept");
        };
        assert!(established.contains("three pairs of five, not five of five"));
        assert!(established.contains("no member of any pair was SUBMITTED"));
        assert!(established.contains("projection-equality-with-paired-explicit row does NOT move"));
    }

    fn stopped_ledger() -> RestartLedger {
        let mut ledger = RestartLedger::new();
        ledger
            .record(
                RestartStep::AcceptedSponsorlessControl,
                RecordedStepResult::StoppedTyped {
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
    fn the_waves_own_closeout_validates_and_stops_where_the_order_stopped() {
        use super::wave_five_closeout;
        use crate::live_restart::RestartStep;

        let report = wave_five_closeout().expect("the wave's closeout validates");
        let rendered = report.render();

        // Two rows moved, each on its own acceptance, and the sponsor
        // row is named as unmoved rather than left out.
        assert_eq!(report.moved_rows().len(), 2);
        assert!(rendered.contains("wave5_matrix_delta 2"));
        assert!(rendered.contains("unmoved_row private-sponsor-values"));
        assert!(rendered.contains("pre_sighash_matrix_delta 0"));

        // The stop is where the ledger says it is.
        assert_eq!(
            report.parts().disposition,
            super::CloseoutDisposition::TypedStopped {
                step: RestartStep::TargetCtConservation,
                blocker: LiveInfrastructureBlocker::NoAcceptingControlExists,
            },
        );
        assert_eq!(
            report.parts().ledger.stopped_at(),
            Some(RestartStep::TargetCtConservation),
        );

        // Exactly one residual cleared, and the two carried ones
        // unchanged.
        assert_eq!(report.parts().cleared_residuals.len(), 1);
        assert_eq!(report.parts().blockers.len(), 2);

        // The whole non-claim census travels with the report.
        assert_eq!(report.parts().non_claims.len(), 10);
        assert_eq!(report.parts().exclusions.len(), 9);
    }

    #[test]
    fn the_follow_up_wave_closeout_moves_conservation_and_stops_at_step_five() {
        use super::wave_six_closeout;
        use crate::live_restart::RestartStep;

        let report = wave_six_closeout().expect("the follow-up wave's closeout validates");
        let rendered = report.render();

        // Three rows moved, including the target-ct-conservation row Wave 5
        // stopped before, each on its own acceptance.
        assert_eq!(report.moved_rows().len(), 3);
        assert!(rendered.contains("wave5_matrix_delta 3"));
        assert!(rendered.contains("moved_row target-ct-conservation"));
        assert!(rendered.contains("unmoved_row private-sponsor-values"));
        assert!(rendered.contains("pre_sighash_matrix_delta 0"));

        // The order stops typed at step five on the shape-constructor
        // absence, not on the mis-typed no-accepting-control blocker.
        assert_eq!(
            report.parts().disposition,
            super::CloseoutDisposition::TypedStopped {
                step: RestartStep::RemainingPositiveShapes,
                blocker: LiveInfrastructureBlocker::MultiOutputShapeConstructorAbsent,
            },
        );
        assert_eq!(
            report.parts().ledger.stopped_at(),
            Some(RestartStep::RemainingPositiveShapes),
        );

        // The mandatory disclosure: step three's non-conserving half is the
        // same observed run step four records as the wrong-blinder case.
        assert!(rendered.contains("SAME observed run"));

        // Exactly one residual cleared and the two carried ones unchanged,
        // as the interlock ruling requires.
        assert_eq!(report.parts().cleared_residuals.len(), 1);
        assert_eq!(report.parts().blockers.len(), 2);
        assert_eq!(report.parts().non_claims.len(), 10);
    }

    #[test]
    fn the_shape_wave_closeout_moves_six_rows_and_stops_at_step_six() {
        use super::wave_seven_closeout;

        let report = wave_seven_closeout().expect("the shape wave's closeout validates");
        let rendered = report.render();

        // Six rows moved, each on an acceptance of its own shape: the
        // three the earlier waves moved and the three shapes this wave
        // built and ran.
        assert_eq!(report.moved_rows().len(), 6);
        assert!(rendered.contains("wave5_matrix_delta 6"));
        for moved in [
            "moved_row private-split",
            "moved_row private-many-to-many-representative",
            "moved_row private-several-distinct-owners",
            "moved_row target-ct-conservation",
        ] {
            assert!(rendered.contains(moved), "{moved} is not in the delta");
        }

        // Every moved row cites a target-computed identity.
        let identities: BTreeSet<_> = report
            .moved_rows()
            .iter()
            .map(super::MovedMatrixRow::accepted_identity)
            .collect();
        for identity in &identities {
            assert_eq!(
                identity.to_string().len(),
                64,
                "{identity} is not a target identity"
            );
        }

        // Six rows cite FIVE distinct identities, and the collision is a
        // disclosed fact rather than a defect: the conservation row's
        // conserving half is the SAME observed run as the one-to-one
        // control, which is what the previous wave's record discloses in
        // its own words. Spelling five here is what keeps that disclosure
        // true — a sixth identity appearing would mean conservation had
        // quietly been re-grounded on some other run.
        assert_eq!(identities.len(), 5, "six rows over five observed runs");
        assert_eq!(
            crate::live_conservation_negatives::run_of_record::CONTROL_ACCEPTED_TXID,
            crate::live_private_restart::run_of_record::ACCEPTED_TXID,
            "the conserving control is the one-to-one control",
        );

        // The three shapes this wave added are three DIFFERENT identities:
        // three runs reported once each rather than one run reported three
        // times.
        let shapes: BTreeSet<&str> = BTreeSet::from([
            crate::live_multi_shapes::run_of_record::SPLIT_ACCEPTED_TXID,
            crate::live_multi_shapes::run_of_record::MANY_TO_MANY_ACCEPTED_TXID,
            crate::live_multi_shapes::run_of_record::SEVERAL_OWNERS_ACCEPTED_TXID,
        ]);
        assert_eq!(shapes.len(), 3, "three shapes, three identities");
        assert!(
            shapes.is_disjoint(&BTreeSet::from([
                crate::live_private_restart::run_of_record::ACCEPTED_TXID,
                crate::live_private_restart::run_of_record::PARITY_ACCEPTED_TXID,
            ])),
            "a shape claims an identity an earlier wave's run produced",
        );

        // The four that did not move are still named as unmoved, so a
        // reader counts ten either way.
        for unmoved in [
            "unmoved_row private-merge",
            "unmoved_row private-sponsor-values",
            "unmoved_row deterministic-public-fixture-openings",
            "unmoved_row projection-equality-with-paired-explicit",
        ] {
            assert!(rendered.contains(unmoved), "{unmoved} is not named unmoved");
        }

        // The order now stops at step SIX on the sponsor signer, not at
        // step five on the shape constructor: the constructor absence left
        // by the fifth step being recorded accepted.
        assert_eq!(
            report.parts().disposition,
            CloseoutDisposition::TypedStopped {
                step: RestartStep::SponsorCases,
                blocker: LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
            },
        );
        assert_eq!(
            report.parts().ledger.stopped_at(),
            Some(RestartStep::SponsorCases),
        );

        // The fifth step's disclosure: an accepted step five is three
        // shapes of four, and the fourth is unconstructible rather than
        // merely unrun.
        assert!(rendered.contains("UNCONSTRUCTIBLE"));
        assert!(rendered.contains("three shapes of four, not four of four"));

        // Exactly one residual cleared and the two carried ones unchanged.
        // The shape-constructor blocker is NOT in the cleared set; it left
        // by the step it stopped being recorded accepted.
        assert_eq!(report.parts().cleared_residuals.len(), 1);
        assert!(
            !report
                .parts()
                .cleared_residuals
                .contains(&LiveInfrastructureBlocker::MultiOutputShapeConstructorAbsent),
        );
        assert_eq!(report.parts().blockers.len(), 2);
        assert_eq!(report.parts().non_claims.len(), 10);
    }

    /// Step seven is unreachable while step six stops, by the ledger's own
    /// rule and not by this wave's choosing.
    ///
    /// The mandatory order says each step's entry is the previous step's
    /// observed result, and the ledger enforces it: a step recorded after
    /// a stop is [`crate::live_restart::RestartOrderRefusal::AfterStop`],
    /// whose own words are that the later run "is not evidence about its
    /// own subject, because its entry condition never held". This asserts
    /// the structural fact rather than restating it in prose.
    #[test]
    fn the_minimality_pairs_cannot_be_recorded_after_the_sponsor_stop() {
        use crate::live_restart::RestartOrderRefusal;

        let report = super::wave_seven_closeout().expect("the closeout validates");
        let mut ledger = report.parts().ledger.clone();
        assert_eq!(
            ledger.record(
                RestartStep::MinimalityPairs,
                RecordedStepResult::Accepted {
                    accepted_identities: vec!["ab".repeat(32)],
                    established: "a pair that never had an entry condition".to_owned(),
                },
            ),
            Err(RestartOrderRefusal::AfterStop {
                attempted: RestartStep::MinimalityPairs,
                stopped_at: RestartStep::SponsorCases,
            }),
        );
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
    fn the_sponsor_row_moves_on_an_acceptance_of_its_own_shape() {
        // This test used to assert the opposite, and the reversal is the
        // point rather than an edit to keep a suite green. The row was
        // shut because no run of ITS OWN shape existed: an explicit
        // sponsored acceptance answers nothing about a confidential one,
        // so the constructor refused the class outright.
        //
        // A sponsored CONFIDENTIAL with-change successor has since been
        // accepted and mined, so the ground the refusal stood on is
        // gone. What has NOT changed is that the row moves on an
        // acceptance and on nothing else -- the constructor still takes
        // an identity, and a caller with no run has none to hand it.
        let recorded =
            crate::live_sponsor_shapes::sponsored_run_of_record::sponsored_private_accepted()
                .expect("the committed identity parses");
        let moved = moved_on_acceptance(PositivePrivateClass::PrivateSponsorValues, recorded);
        assert_eq!(moved.class(), PositivePrivateClass::PrivateSponsorValues);
        assert_eq!(
            moved.accepted_identity().to_string(),
            crate::live_sponsor_shapes::sponsored_run_of_record::SPONSORED_PRIVATE_TXID,
        );
    }

    #[test]
    fn every_class_may_now_enter_the_delta() {
        // Counted over the whole vocabulary rather than asserted of the
        // one that changed, so a class added later with no run behind it
        // is caught here.
        let recorded = crate::live_private_restart::run_of_record::accepted()
            .expect("the committed primary identity parses");
        let moved: Vec<_> = PositivePrivateClass::ALL
            .into_iter()
            .map(|class| moved_on_acceptance(class, recorded))
            .collect();
        assert_eq!(
            moved
                .iter()
                .map(super::MovedMatrixRow::class)
                .collect::<Vec<_>>(),
            PositivePrivateClass::ALL,
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
                ledger_status: RestartLedgerStatus::Stopped {
                    step: RestartStep::AcceptedSponsorlessControl,
                    blocker: LiveInfrastructureBlocker::NoAcceptingControlExists,
                },
            }),
        );
    }

    fn accepted_result(step: RestartStep) -> RecordedStepResult {
        RecordedStepResult::Accepted {
            accepted_identities: vec![step.name().to_owned()],
            established: format!("{} accepted", step.name()),
        }
    }

    fn completed_ledger() -> RestartLedger {
        let mut ledger = RestartLedger::new();
        for step in RestartStep::ALL {
            ledger
                .record(step, accepted_result(step))
                .expect("the ordered acceptance records");
        }
        ledger
    }

    #[test]
    fn completed_refuses_an_empty_ledger() {
        let mut offered = parts(RestartLedger::new());
        offered.disposition = CloseoutDisposition::Completed;
        assert_eq!(
            validate_closeout(offered).err(),
            Some(CloseoutRefusal::DispositionDisagreesWithTheLedger {
                claimed: CloseoutDisposition::Completed,
                ledger_status: RestartLedgerStatus::InProgress {
                    expected: RestartStep::AcceptedSponsorlessControl,
                },
            }),
        );
    }

    #[test]
    fn completed_refuses_an_accepted_prefix() {
        let mut ledger = RestartLedger::new();
        ledger
            .record(
                RestartStep::AcceptedSponsorlessControl,
                accepted_result(RestartStep::AcceptedSponsorlessControl),
            )
            .expect("the first acceptance records");
        let mut offered = parts(ledger);
        offered.disposition = CloseoutDisposition::Completed;
        assert_eq!(
            validate_closeout(offered).err(),
            Some(CloseoutRefusal::DispositionDisagreesWithTheLedger {
                claimed: CloseoutDisposition::Completed,
                ledger_status: RestartLedgerStatus::InProgress {
                    expected: RestartStep::BothParitySuccessors,
                },
            }),
        );
    }

    #[test]
    fn completed_accepts_all_seven_ordered_acceptances() {
        let mut offered = parts(completed_ledger());
        offered.disposition = CloseoutDisposition::Completed;
        validate_closeout(offered).expect("the complete ledger agrees with completion");
    }

    #[test]
    fn a_typed_stop_requires_the_exact_step_and_blocker() {
        let mut different_blocker = parts(stopped_ledger());
        different_blocker.disposition = CloseoutDisposition::TypedStopped {
            step: RestartStep::AcceptedSponsorlessControl,
            blocker: LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
        };
        assert!(matches!(
            validate_closeout(different_blocker),
            Err(CloseoutRefusal::DispositionDisagreesWithTheLedger { .. }),
        ));

        let mut different_step = parts(stopped_ledger());
        different_step.disposition = CloseoutDisposition::TypedStopped {
            step: RestartStep::BothParitySuccessors,
            blocker: LiveInfrastructureBlocker::NoAcceptingControlExists,
        };
        assert!(matches!(
            validate_closeout(different_step),
            Err(CloseoutRefusal::DispositionDisagreesWithTheLedger { .. }),
        ));
    }

    #[test]
    fn closeout_wrappers_preserve_restart_order_causes() {
        for source in [
            RestartOrderRefusal::OutOfOrder {
                attempted: RestartStep::ProofNegatives,
                expected: RestartStep::AcceptedSponsorlessControl,
            },
            RestartOrderRefusal::AfterStop {
                attempted: RestartStep::MinimalityPairs,
                stopped_at: RestartStep::SponsorCases,
            },
        ] {
            assert_eq!(
                CloseoutRefusal::from(source.clone()),
                CloseoutRefusal::RestartOrder(source),
            );
        }
    }

    #[test]
    fn closeout_wrappers_preserve_evidence_assembly_causes() {
        for source in [
            EvidenceAssemblyRefusal::RoleSubstitution {
                offered_role: CandidateEvidenceRole::Funding,
                required_role: CandidateEvidenceRole::Safety,
            },
            EvidenceAssemblyRefusal::RoleSettledTwice(CandidateEvidenceRole::Lifecycle),
            EvidenceAssemblyRefusal::RoleUnsettled(CandidateEvidenceRole::Resource),
        ] {
            assert_eq!(
                CloseoutRefusal::from(source.clone()),
                CloseoutRefusal::EvidenceAssembly(source),
            );
        }
    }

    #[test]
    fn one_class_cannot_move_twice() {
        let mut offered = parts(stopped_ledger());
        offered.wave5_matrix_delta = vec![
            moved_on_acceptance(
                PositivePrivateClass::OneToOne,
                crate::live_private_restart::run_of_record::accepted()
                    .expect("the committed primary identity parses"),
            ),
            moved_on_acceptance(
                PositivePrivateClass::OneToOne,
                crate::live_private_restart::run_of_record::parity_accepted()
                    .expect("the committed parity identity parses"),
            ),
        ];
        assert_eq!(
            validate_closeout(offered).err(),
            Some(CloseoutRefusal::ClassMovedTwice(
                PositivePrivateClass::OneToOne
            )),
        );
    }
}
