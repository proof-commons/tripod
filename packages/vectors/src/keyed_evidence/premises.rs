//! Named offers of evidence provenance; local computation proves none of them.
use super::Refusal;
use crate::keyed_history_reader::{Key, Obligation, ParentFact, Reading};
use std::collections::BTreeSet;
/// Deployment identity; no activation is performed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deployment(pub String);
/// Version of caller policy readings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyVersion(pub String);
/// Opaque external chain position. Matching binds caller provenance only,
/// never a position authenticated by the supplied transaction bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryAnchor(pub String);
/// Named source of an external assertion, not its truth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceSource(pub String);
/// All caller readings stay distinct from byte-derived evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provenance {
    pub deployment: Deployment,
    pub policy_version: PolicyVersion,
    pub anchor: HistoryAnchor,
    pub history_source: EvidenceSource,
    pub policy_source: EvidenceSource,
    /// Opaque named readings, not native policy observations or acceptance rules.
    pub policy_readings: std::collections::BTreeMap<String, String>,
}
/// Sources name obligations even when evidence remains owed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PremiseSources {
    pub inclusion: EvidenceSource,
    pub unspentness: EvidenceSource,
    pub retrieval: EvidenceSource,
    pub finality: EvidenceSource,
}
/// A depth or replacement rule is deployment data, never chosen by this crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FinalityRule {
    /// Zero means final at inclusion: a deployment choice, not library advice.
    Depth(u64),
    Replacement(String),
}
/// Published declaration form. An offer is not independently authenticated here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    /// Named external acceptance; not established by this library.
    pub acceptance_source: EvidenceSource,
    pub deployment: Deployment,
    pub policy_version: PolicyVersion,
    pub anchor: HistoryAnchor,
    pub rule: FinalityRule,
    pub inclusion_source: EvidenceSource,
    pub retrieval_source: EvidenceSource,
    pub unspentness_scope: String,
}
/// Fixture-only declaration type; cannot confer deployment finality.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HypotheticalDeclaration(pub Declaration);
/// Externally accepted deployment declaration, supplied at activation.
///
/// Its provenance/acceptance is an assumption, not verified by this repository.
/// No public producer exists until an activation route is specified.
/// ```compile_fail,E0451
/// use vectors::keyed_evidence::DeploymentDeclaration;
/// let _ = DeploymentDeclaration { 0: todo!() };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeploymentDeclaration(Declaration);
impl DeploymentDeclaration {
    // No activation route exists. Synthetic in-crate tests are the only producer.
    #[cfg(test)]
    pub(crate) const fn new(declaration: Declaration) -> Self {
        Self(declaration)
    }
}
/// Distinct origins prevent a hypothetical match from reading as deployment finality.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclarationOffer {
    Hypothetical(HypotheticalDeclaration),
    Deployment(DeploymentDeclaration),
}
/// Absence is nonfinal; requiring finality invokes the matching contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinalityRequest {
    NotRequested,
    Required,
}
/// Even deployment finality is assumed, never established by finite local tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finality {
    Nonfinal,
    Hypothetical(HypotheticalDeclaration),
    Assumed(DeploymentDeclaration),
}
/// Every report carries these assumptions, never as byte-proven facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Assumption {
    Parent {
        obligation: Obligation,
        source: EvidenceSource,
    },
    /// Excluding a handle of an unsupplied key relies on inclusion in one valid history.
    NoUnsuppliedKeyHandle {
        supplied_keys: BTreeSet<Key>,
        inclusion_source: EvidenceSource,
    },
    Finality(Finality),
}
pub fn assumptions(
    reading: &Reading,
    keys: &BTreeSet<Key>,
    sources: &PremiseSources,
    finality: &Finality,
) -> Vec<Assumption> {
    let mut result: Vec<_> = reading
        .obligations()
        .iter()
        .map(|obligation| {
            let source = match obligation.fact {
                ParentFact::Inclusion => &sources.inclusion,
                ParentFact::Unspentness => &sources.unspentness,
                ParentFact::Retrieval => &sources.retrieval,
                ParentFact::Finality => &sources.finality,
            };
            Assumption::Parent {
                obligation: obligation.clone(),
                source: source.clone(),
            }
        })
        .collect();
    result.push(Assumption::NoUnsuppliedKeyHandle {
        supplied_keys: keys.clone(),
        inclusion_source: sources.inclusion.clone(),
    });
    result.push(Assumption::Finality(finality.clone()));
    result
}
pub fn finality(
    provenance: &Provenance,
    sources: &PremiseSources,
    request: FinalityRequest,
    offer: Option<&DeclarationOffer>,
) -> Result<Finality, Refusal> {
    let Some(offer) = offer else {
        return match request {
            FinalityRequest::NotRequested => Ok(Finality::Nonfinal),
            FinalityRequest::Required => Err(Refusal::FinalityMissing),
        };
    };
    if request == FinalityRequest::Required && matches!(offer, DeclarationOffer::Hypothetical(_)) {
        return Err(Refusal::FinalityHypothetical);
    }
    let declaration = match offer {
        DeclarationOffer::Hypothetical(d) => &d.0,
        DeclarationOffer::Deployment(d) => &d.0,
    };
    if declaration.acceptance_source.0.is_empty() {
        return Err(Refusal::FinalityAcceptanceSource);
    }
    if declaration.deployment != provenance.deployment {
        return Err(Refusal::FinalityDeployment);
    }
    if declaration.anchor != provenance.anchor {
        return Err(Refusal::FinalityAnchor);
    }
    if declaration.policy_version != provenance.policy_version {
        return Err(Refusal::FinalityPolicy);
    }
    if declaration.inclusion_source.0.is_empty() {
        return Err(Refusal::FinalityInclusionSource);
    }
    if declaration.retrieval_source.0.is_empty() {
        return Err(Refusal::FinalityRetrievalSource);
    }
    if declaration.inclusion_source != sources.inclusion {
        return Err(Refusal::FinalityInclusionMismatch);
    }
    if declaration.retrieval_source != sources.retrieval {
        return Err(Refusal::FinalityRetrievalMismatch);
    }
    if declaration.unspentness_scope.is_empty() {
        return Err(Refusal::FinalityUnspentnessScope);
    }
    if matches!(&declaration.rule, FinalityRule::Replacement(rule) if rule.is_empty()) {
        return Err(Refusal::FinalityRule);
    }
    Ok(match offer {
        DeclarationOffer::Hypothetical(d) => Finality::Hypothetical(d.clone()),
        DeclarationOffer::Deployment(d) => Finality::Assumed(d.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn synthetic_deployment_is_required_and_hypothetical_cannot_promote() {
        // Synthetic internal declaration, not activation data or authenticated policy.
        let sources = PremiseSources {
            inclusion: EvidenceSource("i".into()),
            retrieval: EvidenceSource("r".into()),
            unspentness: EvidenceSource("u".into()),
            finality: EvidenceSource("f".into()),
        };
        let provenance = Provenance {
            deployment: Deployment("synthetic".into()),
            policy_version: PolicyVersion("1".into()),
            anchor: HistoryAnchor("external position".into()),
            history_source: EvidenceSource("h".into()),
            policy_source: EvidenceSource("p".into()),
            policy_readings: std::collections::BTreeMap::new(),
        };
        let form = Declaration {
            acceptance_source: EvidenceSource("synthetic acceptance".into()),
            deployment: provenance.deployment.clone(),
            policy_version: provenance.policy_version.clone(),
            anchor: provenance.anchor.clone(),
            rule: FinalityRule::Depth(0),
            inclusion_source: sources.inclusion.clone(),
            retrieval_source: sources.retrieval.clone(),
            unspentness_scope: "synthetic observation".into(),
        };
        let deployment = DeploymentDeclaration::new(form.clone());
        let offer = DeclarationOffer::Deployment(deployment.clone());
        assert_eq!(
            finality(
                &provenance,
                &sources,
                FinalityRequest::Required,
                Some(&offer)
            ),
            Ok(Finality::Assumed(deployment.clone()))
        );
        let hypothetical = DeclarationOffer::Hypothetical(HypotheticalDeclaration(form.clone()));
        assert_eq!(
            finality(
                &provenance,
                &sources,
                FinalityRequest::Required,
                Some(&hypothetical)
            ),
            Err(Refusal::FinalityHypothetical)
        );
        let original = crate::keyed_evidence::CompleteReading {
            scope: crate::keyed_evidence::Scope::HandleClosure,
            finality: Finality::Hypothetical(HypotheticalDeclaration(form)),
            membership: std::collections::BTreeSet::new(),
        };
        let mut promoted = original.clone();
        promoted.finality = Finality::Assumed(deployment);
        assert_eq!(
            crate::keyed_evidence::validation::complete(&original, &original),
            Ok(())
        );
        assert_eq!(
            crate::keyed_evidence::validation::complete(&original, &promoted),
            Err(Refusal::OutcomeMismatch)
        );
    }
}
