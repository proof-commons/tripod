//! External consumer of the feature-gated keyed production entry.
#![cfg(feature = "keyed-experimental")]
use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId as O},
};
use compiler::{AnalysisPolicy, ProofSearchLimits, bind_input, keyed_plan::*};

#[test]
fn external_keyed_plans_publish_all_seven_stages_and_universal_clauses() {
    for (scope, relations, evidence, assignments, coverage_nodes) in [
        (vec![K::Source], 4, 4, 0, 28),
        (vec![K::Merge], 5, 3, 2, 33),
        (vec![K::Source, K::Merge], 9, 7, 2, 61),
    ] {
        let manifest = KeyedManifest::contract().expect("manifest");
        let realization = realization::keyed::KeyedRealizationDraft::derive(&manifest)
            .expect("realization")
            .validate(&manifest)
            .expect("validation");
        let declarations = realization.project();
        let states = std::num::NonZeroU64::new(1_000_000).unwrap();
        let candidates = std::num::NonZeroU64::new(100_000).unwrap();
        let input = bind_input(
            &manifest,
            realization,
            scope.into_iter().map(O::Keyed).collect(),
            AnalysisPolicy::strict(ProofSearchLimits::new(states, candidates)),
        )
        .expect("input");
        let limits = PlacementSearchLimits::new(states, candidates);
        let plan = plan(&input, limits).expect("production plan");
        revalidate(&input, limits, &plan).expect("independent validation");
        let projection = plan.project();
        assert_eq!(projection.stages.relations, relations);
        assert_eq!(projection.stages.source_tables, relations);
        assert_eq!(projection.stages.construction_nodes, 0);
        assert_eq!(projection.stages.lifecycle_nodes, 0);
        assert_eq!(projection.stages.proof_plans, 1);
        assert_eq!(projection.stages.placement_assignments, assignments);
        assert_eq!(projection.stages.coverage_nodes, coverage_nodes);
        assert_eq!(projection.evidence.len(), evidence);
        let published = projection
            .plans
            .iter()
            .flat_map(|plan| plan.operations.values())
            .flat_map(|operation| &operation.relations)
            .map(|(id, row)| (*id, row.declaration))
            .collect::<std::collections::BTreeMap<_, _>>();
        let expected = declarations
            .into_iter()
            .filter(|row| projection.scope.contains(&row.id.operation))
            .map(|row| (row.id, row.relation))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(published, expected);
        for obligation in &projection.evidence {
            if let EvidencePremises::MembershipReading {
                inclusion,
                retrieval,
                finality_for_final_reading,
                ..
            } = obligation.premises
            {
                assert!(inclusion && retrieval && finality_for_final_reading);
            }
        }
        for carrier in projection
            .plans
            .iter()
            .flat_map(|p| p.operations.values())
            .flat_map(|o| &o.placements)
            .flat_map(|p| p.values())
            .flatten()
        {
            assert_eq!(carrier.operation, O::Keyed(K::Merge));
            assert_eq!(carrier.quantification, CarrierQuantification::PerMember);
        }
    }
}
