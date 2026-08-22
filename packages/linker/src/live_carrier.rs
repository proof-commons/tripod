//! Relation-carrier closure, per execution case and per plan (§11.5).
//!
//! # Three censuses, compared per representation rather than in union
//!
//! §11.5 names the three — compiler-required, backend-emitted, and
//! linked-reachable — and then says the thing that makes this module
//! different from [`crate::carrier`]: *for every execution case and
//! representation plan*, and a carrier reachable only in the explicit
//! plan does not satisfy the private plan.
//!
//! So the comparison is not run over a union and then reported per plan.
//! Each plan's closure is computed against that plan's own compiler
//! projection and that plan's own committed tree, and the cross-plan
//! comparison afterwards is a *check* rather than a summary:
//! [`LinkRefusal::PlanStarvedOfCarrier`] names the plan that starves, and
//! it fires exactly when a requirement of one plan has no reachable site
//! in that plan's own tree while the other plan's tree has one. A closure
//! that had merged the two would have reported that case as carried,
//! which is the mistake §11.5 exists to name.
//!
//! # Why the two plans really do differ here
//!
//! Not hypothetically. The compiler's own projections differ by exactly
//! two relation-cases: the explicit plan requires a carrier for the
//! conservation of the protocol asset in both its sponsor cases, and the
//! private plan requires none — because under §6.3 that equation is the
//! target's confidential-transaction rule, published as
//! [`ExternalEvidenceRole::ConfidentialValueConservation`] instead. The
//! explicit plan carries it on a coordinator leaf; the private plan
//! carries it nowhere local, and must not.
//!
//! # An external requirement is never reassigned to a program
//!
//! §11.5's last sentence, as a check with teeth rather than a mapping
//! that happens not to produce a leaf. For each plan the module collects
//! the relation-cases whose rows name external evidence and the
//! relation-cases the plan gives a local carrier, and requires the two
//! sets to be disjoint. Today they are — the private plan's conservation
//! is external and locally uncarried, and the explicit plan's is locally
//! carried and not external — and the day a change made the private
//! plan's conservation locally carried while the external requirement
//! stood, [`LinkRefusal::ExternalRequirementCarriedLocally`] is what
//! would say so. That is the reassignment §11.5 forbids, and it is the
//! one a program would be most tempted to make, because a coordinator
//! that summed commitments would *look* like it had proved something.
//!
//! # The sponsor region carries no protocol leaf
//!
//! [`crate::carrier`]'s own observation, unchanged and for the same
//! reason: §10.7 has the coordinator prove the sponsor region rather than
//! placing a protocol program inside it, so a carrier anchored on a
//! family other than the protocol family maps onto no concrete site here.
//! An alternative made only of such carriers is one this candidate does
//! not serve, and the requirement is carried by one of its other
//! alternatives or not at all.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::upstream::{
    CarrierRole, ExecutionCaseId, ExternalEvidenceRole, LiveTransferRepresentationPlan,
    LiveTransferRepresentationProjection, RelationCaseKey,
};
use tapscript::{LiveProgramRole, LiveTransferLeafRole};

use crate::error::LinkRefusal;

/// Where one live relation-case's carrier concretely sits.
///
/// The live counterpart of [`tapscript::ConcreteCarrierSite`], and a
/// separate type because the leaf vocabulary is separate: a site naming
/// a compact-ASH leaf role would be naming a program no live constructor
/// holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConcreteLiveCarrierSite {
    /// The coordinator program of one representation.
    CoordinatorLeaf,
    /// The member program of one representation.
    MemberLeaf,
    /// The emitted bundle's own structure, which is not a program.
    BundleStructure,
    /// Outside the bundle entirely — the external-evidence boundary.
    OutsideBundle,
}

impl ConcreteLiveCarrierSite {
    /// The program role behind this site, where there is one.
    ///
    /// `None` for the two sites that are not programs, which is what
    /// makes them unremovable from a tree: there is nothing there to
    /// take out.
    #[must_use]
    pub const fn program_role(self) -> Option<LiveProgramRole> {
        match self {
            Self::CoordinatorLeaf => Some(LiveProgramRole::Coordinator),
            Self::MemberLeaf => Some(LiveProgramRole::Member),
            Self::BundleStructure | Self::OutsideBundle => None,
        }
    }
}

/// One relation-case obligation, identified across representations.
///
/// The key §11.5's cross-plan comparison has to be made on, and it is
/// worth saying why the obvious key is the wrong one. A
/// [`RelationCaseKey`] names a relation and an execution case, and an
/// execution case names the representation each object took — so the
/// explicit and the private plan's keys are *disjoint by construction*,
/// every one of them. A comparison run on those keys would find that no
/// plan requires anything another plan requires, report a difference of
/// everything, and never once fire the starvation check §11.5 exists to
/// make. It would look like a comparison and be a tautology.
///
/// What is genuinely the same obligation across the two plans is the
/// relation and the sponsor case, with the representation choice
/// removed. That is exactly this key: the relation-case with its
/// representation map cleared. Nothing is rendered, concatenated, or
/// indexed — the identity is the compiler's own typed key with one typed
/// field emptied.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlanNeutralRelationCase(RelationCaseKey);

impl PlanNeutralRelationCase {
    /// The plan-neutral identity of one relation-case.
    #[must_use]
    pub fn of(key: &RelationCaseKey) -> Self {
        let mut identity = key.clone();
        identity.case.representations.clear();
        Self(identity)
    }

    /// The relation and execution case, with no representation named.
    #[must_use]
    pub const fn key(&self) -> &RelationCaseKey {
        &self.0
    }
}

/// One representation plan's carrier closure (§11.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanCarrierClosure {
    plan: LiveTransferRepresentationPlan,
    cases: BTreeSet<ExecutionCaseId>,
    required: BTreeSet<RelationCaseKey>,
    emitted: BTreeMap<RelationCaseKey, BTreeSet<ConcreteLiveCarrierSite>>,
    reachable: BTreeMap<RelationCaseKey, BTreeSet<ConcreteLiveCarrierSite>>,
    uniquely_carried: BTreeMap<RelationCaseKey, ConcreteLiveCarrierSite>,
    external: BTreeSet<ExternalEvidenceRole>,
    externally_carried: BTreeSet<RelationCaseKey>,
}

impl PlanCarrierClosure {
    /// The representation plan this closure is for.
    #[must_use]
    pub const fn plan(&self) -> LiveTransferRepresentationPlan {
        self.plan
    }

    /// Every execution case the plan projects, in canonical order.
    #[must_use]
    pub const fn cases(&self) -> &BTreeSet<ExecutionCaseId> {
        &self.cases
    }

    /// Every relation-case the compiler required a carrier for.
    #[must_use]
    pub const fn required(&self) -> &BTreeSet<RelationCaseKey> {
        &self.required
    }

    /// Every relation-case the emitted bundle places, with its sites.
    #[must_use]
    pub const fn emitted(&self) -> &BTreeMap<RelationCaseKey, BTreeSet<ConcreteLiveCarrierSite>> {
        &self.emitted
    }

    /// Every relation-case's reachable sites after linking.
    #[must_use]
    pub const fn reachable(&self) -> &BTreeMap<RelationCaseKey, BTreeSet<ConcreteLiveCarrierSite>> {
        &self.reachable
    }

    /// Every relation-case carried by exactly one concrete site.
    ///
    /// The placements with no second carrier to fall back on, so the
    /// program holding each of them cannot be removed or made
    /// unreachable.
    #[must_use]
    pub const fn uniquely_carried(&self) -> &BTreeMap<RelationCaseKey, ConcreteLiveCarrierSite> {
        &self.uniquely_carried
    }

    /// Every external-evidence role this plan leaves open.
    ///
    /// Carried rather than summarized away, because §6.3's whole point is
    /// that the private plan's value equation is a named requirement on
    /// the target rather than a promise about this crate.
    #[must_use]
    pub const fn external_evidence(&self) -> &BTreeSet<ExternalEvidenceRole> {
        &self.external
    }

    /// Every relation-case whose rows name external evidence.
    ///
    /// Disjoint from the locally carried ones by construction — a
    /// closure in which they overlapped is [`close_live`]'s refusal.
    #[must_use]
    pub const fn externally_carried(&self) -> &BTreeSet<RelationCaseKey> {
        &self.externally_carried
    }
}

/// Both plans' closures and the comparison between them (§11.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveCarrierClosure {
    plans: BTreeMap<LiveTransferRepresentationPlan, PlanCarrierClosure>,
    plan_only: BTreeMap<LiveTransferRepresentationPlan, BTreeSet<PlanNeutralRelationCase>>,
}

impl LiveCarrierClosure {
    /// Every plan's closure, in canonical order.
    #[must_use]
    pub const fn plans(&self) -> &BTreeMap<LiveTransferRepresentationPlan, PlanCarrierClosure> {
        &self.plans
    }

    /// One plan's closure.
    #[must_use]
    pub fn plan(&self, plan: LiveTransferRepresentationPlan) -> Option<&PlanCarrierClosure> {
        self.plans.get(&plan)
    }

    /// Every obligation one plan requires and the others do not.
    ///
    /// The difference §11.5 is about, kept as data rather than derived on
    /// demand: a reader asking what the two representations disagree
    /// about reads it here, and the answer for this candidate is the
    /// conservation of the protocol asset in each of the two sponsor
    /// cases — locally carried under the explicit plan and the target's
    /// own confidential-transaction rule under the private one.
    ///
    /// Keyed by [`PlanNeutralRelationCase`] rather than by
    /// [`RelationCaseKey`], because the latter names the representation
    /// and would report every obligation as unique to its own plan.
    #[must_use]
    pub const fn plan_only(
        &self,
    ) -> &BTreeMap<LiveTransferRepresentationPlan, BTreeSet<PlanNeutralRelationCase>> {
        &self.plan_only
    }
}

/// Close the carrier census per plan, against each plan's own tree
/// (§11.5).
///
/// `committed` gives one plan's committed leaf set per linked
/// constructor. Reachability is computed against the plan's *own* entry,
/// never against the union, which is what makes
/// [`LinkRefusal::PlanStarvedOfCarrier`] reachable at all.
///
/// # Errors
///
/// [`LinkRefusal::MissingLivePlanProjection`] when a plan offered a
/// committed tree has no compiler projection,
/// [`LinkRefusal::LiveCarrierCensusMismatch`] when a required
/// relation-case has no site the emitted bundle serves,
/// [`LinkRefusal::UnreachableLiveRelationCarrier`] when a placed site has
/// no committed program under its own plan,
/// [`LinkRefusal::UniqueLiveCarrierRemoved`] when the sole carrier of a
/// relation-case is not committed,
/// [`LinkRefusal::PlanStarvedOfCarrier`] when a requirement is reachable
/// only in another plan's tree, and
/// [`LinkRefusal::ExternalRequirementCarriedLocally`] when a
/// relation-case named as external evidence is also given a local
/// program.
pub fn close_live(
    plan: &tapscript::upstream::ValidatedLiveTransferOperationPlan,
    emitted: &BTreeMap<LiveTransferRepresentationPlan, BTreeSet<LiveTransferLeafRole>>,
    committed: &BTreeMap<LiveTransferRepresentationPlan, BTreeSet<LiveTransferLeafRole>>,
) -> Result<LiveCarrierClosure, LinkRefusal> {
    let mut plans = BTreeMap::new();

    for (representation, committed_leaves) in committed {
        let projection =
            plan.projection(*representation)
                .ok_or(LinkRefusal::MissingLivePlanProjection {
                    plan: *representation,
                })?;
        let emitted_leaves =
            emitted
                .get(representation)
                .ok_or(LinkRefusal::MissingLivePlanProjection {
                    plan: *representation,
                })?;
        plans.insert(
            *representation,
            close_one_plan(projection, emitted_leaves, committed_leaves)?,
        );
    }

    // Every plan's reachable obligations, in the one vocabulary the two
    // plans share. Built once here rather than inside the loops below,
    // because both the starvation check and the difference census read
    // it and two derivations of it could disagree.
    let reached: BTreeMap<LiveTransferRepresentationPlan, BTreeSet<PlanNeutralRelationCase>> =
        plans
            .iter()
            .map(|(representation, closure)| {
                (
                    *representation,
                    closure
                        .reachable
                        .keys()
                        .map(PlanNeutralRelationCase::of)
                        .collect(),
                )
            })
            .collect();

    // The cross-plan comparison, and the reason the closures above are
    // computed separately. A requirement of one plan that its own tree
    // does not reach is not rescued by another plan's tree reaching it,
    // and the refusal names which plan starves rather than reporting a
    // count.
    // Both refusals live here rather than inside the per-plan pass, and
    // that ordering is the point. A per-plan pass that refused an
    // unreachable case on its own would fire before anyone had looked at
    // the other plan, and §11.5's starvation — the case reachable only in
    // the *other* plan's tree — would never be reported as what it is.
    // So the per-plan pass records what its own tree reaches and this
    // pass decides which of the two refusals the absence deserves.
    for (representation, closure) in &plans {
        for key in &closure.required {
            let identity = PlanNeutralRelationCase::of(key);
            if reached
                .get(representation)
                .is_some_and(|reached| reached.contains(&identity))
            {
                continue;
            }
            if let Some((other, _)) = reached
                .iter()
                .find(|(name, reached)| *name != representation && reached.contains(&identity))
            {
                return Err(LinkRefusal::PlanStarvedOfCarrier {
                    starved: *representation,
                    reachable_in: *other,
                    relation_case: Box::new(key.clone()),
                });
            }
            return Err(LinkRefusal::UnreachableLiveRelationCarrier {
                plan: *representation,
                relation_case: Box::new(key.clone()),
            });
        }
    }

    let plan_only = plans
        .iter()
        .map(|(representation, closure)| {
            let others: BTreeSet<PlanNeutralRelationCase> = plans
                .iter()
                .filter(|(name, _)| *name != representation)
                .flat_map(|(_, other)| other.required.iter().map(PlanNeutralRelationCase::of))
                .collect();
            (
                *representation,
                closure
                    .required
                    .iter()
                    .map(PlanNeutralRelationCase::of)
                    .filter(|identity| !others.contains(identity))
                    .collect(),
            )
        })
        .collect();

    Ok(LiveCarrierClosure { plans, plan_only })
}

/// One plan's three censuses, closed against its own leaves.
fn close_one_plan(
    projection: &LiveTransferRepresentationProjection,
    emitted_leaves: &BTreeSet<LiveTransferLeafRole>,
    committed_leaves: &BTreeSet<LiveTransferLeafRole>,
) -> Result<PlanCarrierClosure, LinkRefusal> {
    let plan = projection.plan();

    let required: BTreeSet<RelationCaseKey> = projection
        .carriers()
        .map(|requirement| requirement.relation_case.clone())
        .collect();
    let emitted = emitted_sites(projection, emitted_leaves)?;

    let mut reachable = BTreeMap::new();
    let mut uniquely_carried = BTreeMap::new();
    for (key, emitted_sites) in &emitted {
        let committed_sites: BTreeSet<ConcreteLiveCarrierSite> = emitted_sites
            .iter()
            .copied()
            .filter(|site| site_is_served(*site, committed_leaves))
            .collect();
        if committed_sites.is_empty() {
            // Nothing this plan's own tree reaches. Recorded as absent
            // rather than refused here: `close_live` is what
            // distinguishes a plan that starves — a case another plan's
            // tree does reach — from a case nothing anywhere carries,
            // and a refusal raised here would fire before anyone had
            // looked at the other plan.
            continue;
        }

        if let Some(single) = sole_site(&committed_sites) {
            if let Some(missing) = uncommitted_leaf(single, emitted_leaves, committed_leaves) {
                return Err(LinkRefusal::UniqueLiveCarrierRemoved {
                    plan,
                    relation_case: Box::new(key.clone()),
                    leaf: missing,
                });
            }
            uniquely_carried.insert(key.clone(), single);
        }
        reachable.insert(key.clone(), committed_sites);
    }

    let external: BTreeSet<ExternalEvidenceRole> = projection.external_evidence().collect();
    let externally_carried = externally_carried_cases(projection);

    // §11.5's last sentence. A relation-case the plan names as external
    // evidence and also gives a local program to is an external
    // requirement reassigned to a target program, which no closure may
    // record however the bytes happen to be arranged.
    if let Some(key) = externally_carried
        .iter()
        .find(|key| reachable.contains_key(*key))
    {
        return Err(LinkRefusal::ExternalRequirementCarriedLocally {
            plan,
            relation_case: Box::new(key.clone()),
            roles: external,
        });
    }

    Ok(PlanCarrierClosure {
        plan,
        cases: projection.cases().map(|case| case.id.clone()).collect(),
        required,
        emitted,
        reachable,
        uniquely_carried,
        external,
        externally_carried,
    })
}

/// Every relation-case's sites, as the emitted bundle serves them.
///
/// The backend-emitted census of §11.5, and the one place the abstract
/// carrier vocabulary is mapped onto concrete live sites.
///
/// # Errors
///
/// [`LinkRefusal::LiveCarrierCensusMismatch`] when a required
/// relation-case has no alternative the emitted leaf set serves — the
/// compiler-required and backend-emitted censuses failing to meet, which
/// is the relation vanishing at the boundary that §1.3 forbids.
fn emitted_sites(
    projection: &LiveTransferRepresentationProjection,
    emitted_leaves: &BTreeSet<LiveTransferLeafRole>,
) -> Result<BTreeMap<RelationCaseKey, BTreeSet<ConcreteLiveCarrierSite>>, LinkRefusal> {
    let plan = projection.plan();

    // The protocol family the plan's global carriers are anchored on,
    // read from the plan's own carriers rather than named here — so a
    // realization that moved the anchor moves this without an edit, and
    // so this crate never has to name a type whose defining package
    // §14.1 keeps out of its dependencies.
    let anchor = projection
        .carriers()
        .flat_map(|requirement| requirement.alternatives.iter())
        .flat_map(|alternative| alternative.carriers.iter())
        .find_map(|placed| match &placed.carrier {
            CarrierRole::OperationGlobal { anchor, .. } => Some(*anchor),
            _ => None,
        });

    // Every concrete site one abstract carrier maps onto — a *set*
    // rather than one site, and that is the correction the live
    // candidate forces. Compact ASH maps `EveryInputFamilyMember` onto
    // the member leaf alone, because its batch minimum is two and it
    // never meets a protocol family with no member position. The live
    // candidate's one-to-one shape is exactly such a family: it has a
    // single receipt input, that input is position zero, and position
    // zero runs the coordinator. Both roles carry
    // `LiveFragmentId::OwnerAuthorization` and
    // `LiveFragmentId::LocalRecognition` for that reason, so a
    // per-member obligation is discharged at whichever of the two
    // programs a receipt position runs — and the mapping says so rather
    // than naming one and hoping the other never comes up.
    //
    // A carrier anchored on a family other than the protocol family is
    // the sponsor region, which carries no protocol leaf in this
    // candidate (§10.7 has the coordinator prove the region), so it maps
    // onto nothing.
    let concrete_sites = |carrier: &CarrierRole| -> BTreeSet<ConcreteLiveCarrierSite> {
        match carrier {
            CarrierRole::OperationGlobal { .. } => {
                BTreeSet::from([ConcreteLiveCarrierSite::CoordinatorLeaf])
            }
            CarrierRole::InputFamilyCoordinator { object } if Some(*object) == anchor => {
                BTreeSet::from([ConcreteLiveCarrierSite::CoordinatorLeaf])
            }
            CarrierRole::EveryInputFamilyMember { object } if Some(*object) == anchor => {
                BTreeSet::from([
                    ConcreteLiveCarrierSite::CoordinatorLeaf,
                    ConcreteLiveCarrierSite::MemberLeaf,
                ])
            }
            CarrierRole::InputFamilyCoordinator { .. }
            | CarrierRole::EveryInputFamilyMember { .. } => BTreeSet::new(),
            CarrierRole::BackendStructural { .. } => {
                BTreeSet::from([ConcreteLiveCarrierSite::BundleStructure])
            }
            CarrierRole::ExternalEvidence { .. } => {
                BTreeSet::from([ConcreteLiveCarrierSite::OutsideBundle])
            }
        }
    };

    let mut emitted = BTreeMap::new();
    for requirement in projection.carriers() {
        let key = &requirement.relation_case;

        // Every accepted alternative, mapped through the anchor the
        // compiler placed against. An alternative whose carriers all map
        // onto nothing is one this candidate does not serve — it named
        // the sponsor region, which carries no protocol leaf — and it
        // contributes no site rather than an empty one.
        let mut emitted_sites: BTreeSet<ConcreteLiveCarrierSite> = BTreeSet::new();
        for alternative in &requirement.alternatives {
            let mapped: BTreeSet<ConcreteLiveCarrierSite> = alternative
                .carriers
                .iter()
                .flat_map(|placed| concrete_sites(&placed.carrier))
                .collect();
            if mapped.is_empty() {
                continue;
            }
            if mapped
                .iter()
                .all(|site| site_is_served(*site, emitted_leaves))
            {
                emitted_sites.extend(mapped);
            }
        }

        if emitted_sites.is_empty() {
            return Err(LinkRefusal::LiveCarrierCensusMismatch {
                plan,
                relation_case: Box::new(key.clone()),
            });
        }
        emitted.insert(key.clone(), emitted_sites);
    }

    Ok(emitted)
}

/// Every relation-case whose rows name external evidence.
fn externally_carried_cases(
    projection: &LiveTransferRepresentationProjection,
) -> BTreeSet<RelationCaseKey> {
    projection
        .relations()
        .flat_map(|relation| relation.cases.values())
        .filter(|case| !case.external_evidence.is_empty())
        .map(|case| case.key.clone())
        .collect()
}

/// Whether one concrete site has a program in the given leaf set.
fn site_is_served(site: ConcreteLiveCarrierSite, leaves: &BTreeSet<LiveTransferLeafRole>) -> bool {
    // A site that is not a program is served by every leaf set, because
    // there is nothing there to remove from a tree.
    site.program_role()
        .is_none_or(|role| leaves.iter().any(|leaf| leaf.program_role() == role))
}

/// The one site a placement names, where it names exactly one.
fn sole_site(sites: &BTreeSet<ConcreteLiveCarrierSite>) -> Option<ConcreteLiveCarrierSite> {
    let mut iterator = sites.iter();
    let first = *iterator.next()?;
    iterator.next().is_none().then_some(first)
}

/// A leaf of one site's role that the bundle emitted and the tree does
/// not commit.
fn uncommitted_leaf(
    site: ConcreteLiveCarrierSite,
    emitted_leaves: &BTreeSet<LiveTransferLeafRole>,
    committed_leaves: &BTreeSet<LiveTransferLeafRole>,
) -> Option<LiveTransferLeafRole> {
    let role = site.program_role()?;
    emitted_leaves
        .iter()
        .find(|leaf| leaf.program_role() == role && !committed_leaves.contains(leaf))
        .copied()
}
