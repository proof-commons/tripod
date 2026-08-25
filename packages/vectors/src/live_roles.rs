//! Evidence roles, typed, so one ceremony's success cannot discharge
//! another ceremony's question.
//!
//! # The sentence this module exists to make unsayable
//!
//! "The funding ceremony succeeded, so the transfer is evidenced."
//!
//! That sentence is false, and it is false in a way prose cannot catch:
//! both halves are about the same run, both halves are true of something,
//! and the word "so" is the whole error. Funding evidence is evidence
//! that a confidential predecessor can exist on chain. It is not evidence
//! that a transfer spending that predecessor was authorized, that any
//! safety relation holds, that any disclosure is minimal, or that any
//! resource result was measured.
//!
//! So the roles are seven separate entries rather than one summary, each
//! settled on its own ground, and a ceremony that tries to fill one
//! role's entry from another's evidence is refused by name
//! (rule:guide-ctf-exec:evidence-roles, rule:guide-ctf-exec:role-separation).
//!
//! # Why the disposition is a projection rather than a field
//!
//! [`EvidenceDisposition`] is the guide's four-member census and this
//! module never stores one. It is computed from [`RoleGround`], which is
//! what the settling caller actually supplies, so a settled entry whose
//! disposition disagrees with its ground is not a refusal — it is a
//! value that cannot be constructed.
//!
//! The alternative shape, a disposition field beside a ground field with
//! a validator between them, was considered and rejected on the census
//! doctrine this workspace already applies to the funding output form: a
//! record naming three of four facts is the thing the census exists to
//! make unsayable, and the same reasoning applies to a record naming a
//! disposition its own ground does not support.
//!
//! # What a `Blocked` entry names, and why it is a live blocker
//!
//! [`RoleGround::CarriedBlocker`] takes a
//! [`LiveInfrastructureBlocker`], the Guide-13 vocabulary, rather than a
//! string or a fresh enum of this guide's own. That coupling is
//! deliberate: the closeout report has to state which residuals are
//! still carried, unchanged, and a role blocked by a residual that is
//! not one of those residuals would be a blocker nobody is tracking.
//!
//! # Nothing here observes anything
//!
//! This module is vocabulary. It reaches no target, submits nothing, and
//! settles no entry by itself. A caller that has observed nothing can
//! still build a complete map — every entry `NotRun` — and that map is a
//! correct report of a ceremony that ran nothing.

use std::collections::BTreeMap;

use crate::live_evidence::LiveInfrastructureBlocker;

/// The seven questions a candidate ceremony can be evidence about.
///
/// Exhaustive by the guide's own listing. A new kind of evidence is a
/// new member here and not a reuse of the nearest one, because the whole
/// point of the census is that a reader can tell which question a run
/// answered without reading the run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CandidateEvidenceRole {
    /// That a confidential predecessor can be funded, mined, and read
    /// back, with a retained opening reference. Nothing about spending
    /// it.
    Funding,
    /// That the target's own commitment-balance rule accepted a
    /// conserving private transaction and refused a non-conserving one.
    CtConservation,
    /// That an owner's authorization over the protected bytes was
    /// accepted, and verifies against an independently recomputed
    /// message.
    OwnerSignature,
    /// That a safety-matrix relation was exercised against a real
    /// target and came out as the relation's polarity requires.
    Safety,
    /// That a disclosure-minimality pair's two sides were both accepted
    /// and their projections compared.
    Minimality,
    /// That a resource measurement was taken against a real target.
    Resource,
    /// That a lifecycle transition was observed rather than modelled.
    Lifecycle,
}

impl CandidateEvidenceRole {
    /// Every role, in the guide's order.
    pub const ALL: [Self; 7] = [
        Self::Funding,
        Self::CtConservation,
        Self::OwnerSignature,
        Self::Safety,
        Self::Minimality,
        Self::Resource,
        Self::Lifecycle,
    ];

    /// The role's wire spelling, stable across renderings.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Funding => "funding",
            Self::CtConservation => "ct-conservation",
            Self::OwnerSignature => "owner-signature",
            Self::Safety => "safety",
            Self::Minimality => "minimality",
            Self::Resource => "resource",
            Self::Lifecycle => "lifecycle",
        }
    }
}

/// How one role's entry came out.
///
/// Four members, and `NotRun` is not a failure while `NotApplicable` is
/// not a success. The two are separated because a ceremony that never
/// reached a step and a ceremony for which the step was never in scope
/// are two different reports, and a reader who cannot tell them apart
/// cannot tell whether more running would change the answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceDisposition {
    /// Observed, and validated against an independent origin.
    Validated,
    /// Attempted or attemptable, and stopped by a named carried
    /// blocker.
    Blocked,
    /// In scope, not stopped, and not reached — the order had not got
    /// there.
    NotRun,
    /// Not this ceremony's question at all.
    NotApplicable,
}

impl EvidenceDisposition {
    /// The disposition's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Validated => "validated",
            Self::Blocked => "blocked",
            Self::NotRun => "not-run",
            Self::NotApplicable => "not-applicable",
        }
    }
}

/// Why a role's entry says what it says.
///
/// The ground is the thing a caller supplies and the disposition is read
/// off it, so the two cannot disagree. Each variant carries the fact a
/// reader would otherwise have to go and find.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RoleGround {
    /// The target accepted, at this identity, and an independent origin
    /// agreed. The identity is the target's own, recorded as the target
    /// computed it.
    ObservedAcceptance {
        /// The identity the target computed for the accepted
        /// transaction.
        accepted_identity: String,
        /// What the second origin checked, in that origin's own words.
        independent_check: String,
    },
    /// A residual this workspace carries and does not clear stopped the
    /// role.
    CarriedBlocker(LiveInfrastructureBlocker),
    /// The mandatory restart order did not reach the step that would
    /// have settled this role.
    OrderNotReached {
        /// The step the order stopped before, one-based, as
        /// task:guide-ctf-exec:restart-order numbers them.
        stopped_before_step: u8,
    },
    /// The role is not a question about this ceremony.
    OutsideThisCeremony {
        /// Which ceremony owns the question instead.
        owned_by: String,
    },
}

impl RoleGround {
    /// The disposition this ground supports, and the only one it does.
    #[must_use]
    pub const fn disposition(&self) -> EvidenceDisposition {
        match self {
            Self::ObservedAcceptance { .. } => EvidenceDisposition::Validated,
            Self::CarriedBlocker(_) => EvidenceDisposition::Blocked,
            Self::OrderNotReached { .. } => EvidenceDisposition::NotRun,
            Self::OutsideThisCeremony { .. } => EvidenceDisposition::NotApplicable,
        }
    }
}

/// What goes wrong when a role map is assembled badly.
///
/// [`Self::RoleSubstitution`] is the guide's own member and the reason
/// the type exists. The other two are this module's, and they are here
/// because a map that silently overwrote an entry, or that was read
/// while incomplete, would produce exactly the summary
/// [`CeremonyEvidenceRoles`] exists to prevent — one that looks
/// exhaustive and is not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvidenceAssemblyRefusal {
    /// Evidence about one role was offered to settle another's entry.
    RoleSubstitution {
        /// The role the offered evidence is actually about.
        offered_role: CandidateEvidenceRole,
        /// The role whose entry the caller was settling.
        required_role: CandidateEvidenceRole,
    },
    /// A role's entry was settled twice. The second answer is refused
    /// rather than applied, because a role settled twice has two grounds
    /// and the map can only carry one.
    RoleSettledTwice(CandidateEvidenceRole),
    /// The map was completed with a role still unsettled. A partial map
    /// read as a whole one is a census with a silent hole.
    RoleUnsettled(CandidateEvidenceRole),
}

/// Evidence offered for exactly one role.
///
/// The role travels with the evidence rather than being supplied at the
/// call site, which is what makes [`EvidenceAssemblyRefusal::RoleSubstitution`]
/// detectable at all: if the caller named the role twice, the two names
/// could be compared and a mistake caught, and if the caller named it
/// once there would be nothing to compare.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleEvidence {
    role: CandidateEvidenceRole,
    ground: RoleGround,
}

impl RoleEvidence {
    /// Evidence about `role`, on `ground`.
    #[must_use]
    pub const fn new(role: CandidateEvidenceRole, ground: RoleGround) -> Self {
        Self { role, ground }
    }

    /// The role this evidence is about.
    #[must_use]
    pub const fn role(&self) -> CandidateEvidenceRole {
        self.role
    }

    /// The ground.
    #[must_use]
    pub const fn ground(&self) -> &RoleGround {
        &self.ground
    }
}

/// One ceremony's role map, under assembly.
///
/// Settled entries accumulate; the map becomes readable only through
/// [`Self::complete`], which refuses while any role is unsettled. So
/// there is no route by which a caller reads seven entries and gets
/// five.
#[derive(Clone, Debug, Default)]
pub struct CeremonyEvidenceRolesBuilder {
    settled: BTreeMap<CandidateEvidenceRole, RoleGround>,
}

impl CeremonyEvidenceRolesBuilder {
    /// An empty map, with every role unsettled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Settle `required_role` from `evidence`.
    ///
    /// # Errors
    ///
    /// [`EvidenceAssemblyRefusal::RoleSubstitution`] where the evidence
    /// is about a different role, and
    /// [`EvidenceAssemblyRefusal::RoleSettledTwice`] where the entry
    /// already has a ground.
    pub fn settle(
        &mut self,
        required_role: CandidateEvidenceRole,
        evidence: RoleEvidence,
    ) -> Result<(), EvidenceAssemblyRefusal> {
        if evidence.role() != required_role {
            return Err(EvidenceAssemblyRefusal::RoleSubstitution {
                offered_role: evidence.role(),
                required_role,
            });
        }
        if self.settled.contains_key(&required_role) {
            return Err(EvidenceAssemblyRefusal::RoleSettledTwice(required_role));
        }
        self.settled.insert(required_role, evidence.ground);
        Ok(())
    }

    /// Close the map.
    ///
    /// # Errors
    ///
    /// [`EvidenceAssemblyRefusal::RoleUnsettled`] naming the first role,
    /// in the census's own order, that has no ground.
    pub fn complete(self) -> Result<CeremonyEvidenceRoles, EvidenceAssemblyRefusal> {
        for role in CandidateEvidenceRole::ALL {
            if !self.settled.contains_key(&role) {
                return Err(EvidenceAssemblyRefusal::RoleUnsettled(role));
            }
        }
        Ok(CeremonyEvidenceRoles {
            settled: self.settled,
        })
    }
}

/// One ceremony's complete role map.
///
/// Every one of the seven roles has a ground, and therefore a
/// disposition. There is no constructor but
/// [`CeremonyEvidenceRolesBuilder::complete`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CeremonyEvidenceRoles {
    settled: BTreeMap<CandidateEvidenceRole, RoleGround>,
}

impl CeremonyEvidenceRoles {
    /// The ground settled for `role`.
    ///
    /// # Panics
    ///
    /// Never: [`CeremonyEvidenceRolesBuilder::complete`] is the only
    /// constructor and it refuses while any role is unsettled, so a
    /// value of this type has a ground for all seven.
    #[must_use]
    pub fn ground(&self, role: CandidateEvidenceRole) -> &RoleGround {
        self.settled
            .get(&role)
            .expect("a completed role map has a ground for every role")
    }

    /// The disposition of `role`, read off its ground.
    #[must_use]
    pub fn disposition(&self, role: CandidateEvidenceRole) -> EvidenceDisposition {
        self.ground(role).disposition()
    }

    /// Every role that came out `Validated`.
    ///
    /// Offered as the only aggregate, and deliberately not as a count of
    /// "roles discharged": a caller who wants to know whether the
    /// transfer is evidenced has to ask about the transfer's role by
    /// name.
    #[must_use]
    pub fn validated_roles(&self) -> Vec<CandidateEvidenceRole> {
        CandidateEvidenceRole::ALL
            .into_iter()
            .filter(|role| self.disposition(*role) == EvidenceDisposition::Validated)
            .collect()
    }

    /// The map, rendered one role per line, for a run transcript.
    #[must_use]
    pub fn render(&self) -> String {
        use std::fmt::Write as _;

        let mut out = String::new();
        for role in CandidateEvidenceRole::ALL {
            let _ = writeln!(
                out,
                "role {} {} {:?}",
                role.name(),
                self.disposition(role).name(),
                self.ground(role),
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CandidateEvidenceRole, CeremonyEvidenceRolesBuilder, EvidenceAssemblyRefusal,
        EvidenceDisposition, RoleEvidence, RoleGround,
    };
    use crate::live_evidence::LiveInfrastructureBlocker;

    fn outside(owner: &str) -> RoleGround {
        RoleGround::OutsideThisCeremony {
            owned_by: owner.to_owned(),
        }
    }

    #[test]
    fn funding_evidence_cannot_settle_the_safety_entry() {
        // The sentence the module exists to refuse, as a call.
        let mut builder = CeremonyEvidenceRolesBuilder::new();
        let funding = RoleEvidence::new(
            CandidateEvidenceRole::Funding,
            RoleGround::ObservedAcceptance {
                accepted_identity: "aa".repeat(32),
                independent_check: "the funding record validated".to_owned(),
            },
        );
        assert_eq!(
            builder.settle(CandidateEvidenceRole::Safety, funding),
            Err(EvidenceAssemblyRefusal::RoleSubstitution {
                offered_role: CandidateEvidenceRole::Funding,
                required_role: CandidateEvidenceRole::Safety,
            }),
        );
    }

    #[test]
    fn a_role_settled_twice_is_refused_rather_than_overwritten() {
        let mut builder = CeremonyEvidenceRolesBuilder::new();
        builder
            .settle(
                CandidateEvidenceRole::Resource,
                RoleEvidence::new(
                    CandidateEvidenceRole::Resource,
                    outside("the resource lane"),
                ),
            )
            .expect("the first settling is accepted");
        assert_eq!(
            builder.settle(
                CandidateEvidenceRole::Resource,
                RoleEvidence::new(
                    CandidateEvidenceRole::Resource,
                    RoleGround::OrderNotReached {
                        stopped_before_step: 6,
                    },
                ),
            ),
            Err(EvidenceAssemblyRefusal::RoleSettledTwice(
                CandidateEvidenceRole::Resource
            )),
        );
    }

    #[test]
    fn an_incomplete_map_cannot_be_read_as_a_whole_one() {
        let mut builder = CeremonyEvidenceRolesBuilder::new();
        builder
            .settle(
                CandidateEvidenceRole::Funding,
                RoleEvidence::new(CandidateEvidenceRole::Funding, outside("the funding lane")),
            )
            .expect("the entry settles");
        assert_eq!(
            builder.complete(),
            Err(EvidenceAssemblyRefusal::RoleUnsettled(
                CandidateEvidenceRole::CtConservation
            )),
        );
    }

    #[test]
    fn every_disposition_is_read_off_its_own_ground() {
        // Four grounds, four dispositions, and the map is complete only
        // because all seven roles are settled.
        let mut builder = CeremonyEvidenceRolesBuilder::new();
        let grounds = [
            (
                CandidateEvidenceRole::Funding,
                RoleGround::ObservedAcceptance {
                    accepted_identity: "bb".repeat(32),
                    independent_check: "two origins agreed".to_owned(),
                },
                EvidenceDisposition::Validated,
            ),
            (
                CandidateEvidenceRole::CtConservation,
                RoleGround::OrderNotReached {
                    stopped_before_step: 3,
                },
                EvidenceDisposition::NotRun,
            ),
            (
                CandidateEvidenceRole::OwnerSignature,
                RoleGround::CarriedBlocker(LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent),
                EvidenceDisposition::Blocked,
            ),
            (
                CandidateEvidenceRole::Safety,
                outside("the safety matrix restart"),
                EvidenceDisposition::NotApplicable,
            ),
        ];
        for (role, ground, expected) in grounds {
            assert_eq!(ground.disposition(), expected);
            builder
                .settle(role, RoleEvidence::new(role, ground))
                .expect("the entry settles");
        }
        for role in [
            CandidateEvidenceRole::Minimality,
            CandidateEvidenceRole::Resource,
            CandidateEvidenceRole::Lifecycle,
        ] {
            builder
                .settle(role, RoleEvidence::new(role, outside("a later ceremony")))
                .expect("the entry settles");
        }

        let map = builder.complete().expect("the map is complete");
        assert_eq!(map.validated_roles(), vec![CandidateEvidenceRole::Funding]);
        assert_eq!(
            map.disposition(CandidateEvidenceRole::OwnerSignature),
            EvidenceDisposition::Blocked,
        );
        assert_eq!(
            map.render().lines().count(),
            CandidateEvidenceRole::ALL.len()
        );
    }
}
