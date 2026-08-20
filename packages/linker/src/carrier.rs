//! Relation-carrier closure (§14.6).
//!
//! # Three censuses, compared rather than assumed
//!
//! §14.6 names them: what the compiler required, what the backend
//! emitted, and what remains reachable after linking. The first two are
//! read from the bundle — the validated plan it carries and the
//! placements it emitted — and the third is recomputed here from the
//! committed tree, because reachability is a fact about the tree and
//! the tree did not exist until this wave.
//!
//! # Why the placement is re-derived from the plan
//!
//! The bundle already selected one carrier assignment per relation-case
//! and recorded the sites. Believing that record would make this module
//! a restatement. So each recorded site set is checked against the
//! plan's own alternatives: there must be an accepted alternative every
//! one of whose abstract carriers maps onto the recorded concrete
//! sites. A placement matching no alternative is a placement the
//! compiler never offered, whatever the bundle says about it.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::upstream::{CarrierRole, RelationCaseKey};
use tapscript::{CandidateRelocatableTapscriptBundle, ConcreteCarrierSite, LeafRole, ProgramRole};

use crate::error::LinkRefusal;

/// The linked carrier census (§14.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CarrierClosure {
    required: BTreeSet<RelationCaseKey>,
    emitted: BTreeSet<RelationCaseKey>,
    reachable: BTreeMap<RelationCaseKey, BTreeSet<ConcreteCarrierSite>>,
    uniquely_carried: BTreeMap<RelationCaseKey, ConcreteCarrierSite>,
}

impl CarrierClosure {
    /// Every relation-case the compiler required a carrier for.
    #[must_use]
    pub const fn required(&self) -> &BTreeSet<RelationCaseKey> {
        &self.required
    }

    /// Every relation-case the backend placed.
    #[must_use]
    pub const fn emitted(&self) -> &BTreeSet<RelationCaseKey> {
        &self.emitted
    }

    /// Every relation-case's reachable sites after linking.
    #[must_use]
    pub const fn reachable(&self) -> &BTreeMap<RelationCaseKey, BTreeSet<ConcreteCarrierSite>> {
        &self.reachable
    }

    /// Every relation-case carried by exactly one concrete site.
    ///
    /// The set §14.6's last clause is about: these are the placements
    /// with no second carrier to fall back on, so the program holding
    /// each of them cannot be removed or made unreachable.
    #[must_use]
    pub const fn uniquely_carried(&self) -> &BTreeMap<RelationCaseKey, ConcreteCarrierSite> {
        &self.uniquely_carried
    }
}

/// Close the carrier census against the committed leaf set (§14.6).
///
/// # Errors
///
/// [`LinkRefusal::CarrierCensusMismatch`] when the required and emitted
/// sets differ, [`LinkRefusal::MissingRelationCarrier`] when a
/// placement names no site, [`LinkRefusal::UnreachableRelationCarrier`]
/// when a named site has no committed program,
/// [`LinkRefusal::DuplicateIncompatibleCarrier`] when a recorded
/// placement matches no alternative the compiler accepted, and
/// [`LinkRefusal::UniqueCarrierRemoved`] when the sole carrier of a
/// relation-case is not committed.
pub fn close(
    bundle: &CandidateRelocatableTapscriptBundle,
    committed: &BTreeSet<LeafRole>,
) -> Result<CarrierClosure, LinkRefusal> {
    let required: BTreeSet<RelationCaseKey> = bundle
        .plan()
        .carriers()
        .map(|requirement| requirement.relation_case.clone())
        .collect();
    let emitted: BTreeSet<RelationCaseKey> = bundle.placements().keys().cloned().collect();

    if required != emitted {
        return Err(LinkRefusal::CarrierCensusMismatch {
            required_only: required.difference(&emitted).cloned().collect(),
            emitted_only: emitted.difference(&required).cloned().collect(),
        });
    }

    let anchor = bundle
        .plan()
        .carriers()
        .flat_map(|requirement| requirement.alternatives.iter())
        .flat_map(|alternative| alternative.carriers.iter())
        .find_map(|placed| match &placed.carrier {
            CarrierRole::OperationGlobal { anchor, .. } => Some(*anchor),
            _ => None,
        });

    // The abstract-to-concrete mapping, stated once as a closure so
    // the anchor comparison is written against the plan's own object
    // identity without this crate having to name that type. A family
    // outside the anchor is the sponsor region, which carries no
    // protocol leaf in this candidate, so it maps onto nothing.
    let concrete_site = |carrier: &CarrierRole| -> Option<ConcreteCarrierSite> {
        match carrier {
            CarrierRole::OperationGlobal { .. } => Some(ConcreteCarrierSite::CoordinatorLeaf),
            CarrierRole::InputFamilyCoordinator { object } if Some(*object) == anchor => {
                Some(ConcreteCarrierSite::CoordinatorLeaf)
            }
            CarrierRole::EveryInputFamilyMember { object } if Some(*object) == anchor => {
                Some(ConcreteCarrierSite::MemberLeaf)
            }
            CarrierRole::InputFamilyCoordinator { .. }
            | CarrierRole::EveryInputFamilyMember { .. } => None,
            CarrierRole::BackendStructural { .. } => Some(ConcreteCarrierSite::BundleStructure),
            CarrierRole::ExternalEvidence { .. } => Some(ConcreteCarrierSite::OutsideBundle),
        }
    };

    let mut reachable = BTreeMap::new();
    let mut uniquely_carried = BTreeMap::new();

    for requirement in bundle.plan().carriers() {
        let key = &requirement.relation_case;
        let placement = bundle
            .placements()
            .get(key)
            .ok_or_else(|| LinkRefusal::MissingRelationCarrier(key.clone()))?;

        let sites = placement.sites();
        if sites.is_empty() {
            return Err(LinkRefusal::MissingRelationCarrier(key.clone()));
        }

        // The recorded sites must be one of the alternatives the
        // compiler accepted, mapped through the same anchor the backend
        // placed against. Nothing here reads the bundle's own answer.
        let matched = requirement.alternatives.iter().any(|alternative| {
            let mapped: Option<BTreeSet<ConcreteCarrierSite>> = alternative
                .carriers
                .iter()
                .map(|placed| concrete_site(&placed.carrier))
                .collect();
            mapped.as_ref() == Some(sites)
        });
        if !matched {
            return Err(LinkRefusal::DuplicateIncompatibleCarrier(key.clone()));
        }

        for site in sites {
            if !site_is_reachable(*site, committed) {
                return Err(LinkRefusal::UnreachableRelationCarrier(key.clone()));
            }
        }

        if let Some(single) = sole_site(sites) {
            if let Some(missing) = uncommitted_leaf(single, bundle, committed) {
                return Err(LinkRefusal::UniqueCarrierRemoved {
                    relation_case: key.clone(),
                    leaf: missing,
                });
            }
            uniquely_carried.insert(key.clone(), single);
        }

        reachable.insert(key.clone(), sites.clone());
    }

    Ok(CarrierClosure {
        required,
        emitted,
        reachable,
        uniquely_carried,
    })
}

/// Whether one concrete site has a committed program behind it.
fn site_is_reachable(site: ConcreteCarrierSite, committed: &BTreeSet<LeafRole>) -> bool {
    match site {
        ConcreteCarrierSite::CoordinatorLeaf => committed
            .iter()
            .any(|leaf| leaf.program_role() == ProgramRole::Coordinator),
        ConcreteCarrierSite::MemberLeaf => committed
            .iter()
            .any(|leaf| leaf.program_role() == ProgramRole::Member),
        // Neither is a program, so neither can be removed from a tree.
        ConcreteCarrierSite::BundleStructure | ConcreteCarrierSite::OutsideBundle => true,
    }
}

/// The one site a placement names, where it names exactly one.
fn sole_site(sites: &BTreeSet<ConcreteCarrierSite>) -> Option<ConcreteCarrierSite> {
    let mut iterator = sites.iter();
    let first = *iterator.next()?;
    iterator.next().is_none().then_some(first)
}

/// A leaf of one site's role that the bundle emitted and the tree does
/// not commit.
fn uncommitted_leaf(
    site: ConcreteCarrierSite,
    bundle: &CandidateRelocatableTapscriptBundle,
    committed: &BTreeSet<LeafRole>,
) -> Option<LeafRole> {
    let role = match site {
        ConcreteCarrierSite::CoordinatorLeaf => ProgramRole::Coordinator,
        ConcreteCarrierSite::MemberLeaf => ProgramRole::Member,
        ConcreteCarrierSite::BundleStructure | ConcreteCarrierSite::OutsideBundle => return None,
    };

    bundle
        .constructor()
        .leaves()
        .keys()
        .find(|leaf| leaf.program_role() == role && !committed.contains(leaf))
        .copied()
}
