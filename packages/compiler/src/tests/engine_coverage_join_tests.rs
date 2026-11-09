//! Placement and projection joins retain evidence without runtime carriers.
use crate::{engine_coverage_join as core, engine_placement};
use architecture::semantic::KeyedOperationId;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn keyed_coverage_joins_refuse_missing_and_invented_carriers_and_projections() {
    let input = super::engine_coverage_tests::bound(vec![
        KeyedOperationId::Source,
        KeyedOperationId::Merge,
    ]);
    let records = crate::engine_keyed::lower_placements(input.owner());
    let coverage = crate::engine_keyed::lower_coverage(input.owner());
    let limits = crate::placement::PlacementSearchLimits::new(
        std::num::NonZeroU64::new(100).expect("states"),
        std::num::NonZeroU64::new(10).expect("candidates"),
    );
    let placements = engine_placement::enumerate(
        &records,
        limits,
        &mut crate::placement::PlacementSearchReport::default(),
    )
    .expect("placements");
    let entries = records
        .eligibility
        .iter()
        .map(|row| (row.key.clone(), row.entries.clone()))
        .collect::<BTreeMap<_, _>>();
    let expected = core::alternatives(
        &entries,
        placements
            .iter()
            .flat_map(|placement| placement.assignments.iter().cloned()),
    );
    let mut joined = coverage
        .into_iter()
        .map(|plan| core::Joined {
            carrier: expected.get(&plan.key).cloned(),
            projections: core::projection_boundaries(&plan, &plan.boundaries),
            plan,
        })
        .collect::<Vec<_>>();
    for plan in &mut joined {
        core::attach_requirements(plan);
    }
    let carried = entries.keys().cloned().collect();
    assert_eq!(
        core::validate(&joined, &carried, &BTreeSet::new(), &expected),
        Ok(())
    );
    assert_eq!(
        joined.iter().filter(|plan| plan.carrier.is_some()).count(),
        2
    );
    assert_eq!(
        joined
            .iter()
            .map(|plan| plan.projections.len())
            .sum::<usize>(),
        9
    );
    check_graph(&joined);
    let runtime = joined
        .iter()
        .position(|plan| plan.carrier.is_some())
        .expect("runtime");
    let evidence = joined
        .iter()
        .position(|plan| !plan.plan.evidence.is_empty())
        .expect("evidence");
    let mut missing = joined.clone();
    missing[runtime].carrier = None;
    assert_eq!(
        core::validate(&missing, &carried, &BTreeSet::new(), &expected),
        Err(core::Error::MissingCarrier(
            joined[runtime].plan.key.clone()
        ))
    );
    let mut unprojected = joined.clone();
    unprojected[runtime].projections.clear();
    assert_eq!(
        core::validate(&unprojected, &carried, &BTreeSet::new(), &expected),
        Err(core::Error::MissingProjection(
            joined[runtime].plan.key.clone()
        ))
    );
    let mut uncovered = joined.clone();
    uncovered[runtime].carrier = Some(BTreeSet::new());
    assert_eq!(
        core::validate(&uncovered, &carried, &BTreeSet::new(), &expected),
        Err(core::Error::MissingLayout(joined[runtime].plan.key.clone()))
    );
    let mut invented = joined.clone();
    invented[evidence]
        .carrier
        .clone_from(&joined[runtime].carrier);
    assert_eq!(
        core::validate(&invented, &carried, &BTreeSet::new(), &expected),
        Err(core::Error::UnexpectedCarrier(
            joined[evidence].plan.key.clone()
        ))
    );
}

fn check_graph(joined: &[core::Joined]) {
    let symbols = joined.iter().map(core::symbols).collect::<Vec<_>>();
    let nodes =
        crate::engine_coverage_graph::definition_census(&symbols).expect("definition census");
    let edges = crate::engine_coverage_graph::derive_dependencies(&symbols, &[]);
    let graph = crate::engine_coverage_graph::build(&nodes, &edges).expect("resolved graph");
    assert!(graph.validate_acyclic().is_ok());
    assert_eq!(nodes.len(), 61);
    assert_eq!(edges.len(), 18);
}
