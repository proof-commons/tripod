//! Determinism of the stable projection under declaration
//! permutation.
//!
//! The projection is a comparison surface. If the order in which a
//! contract's parts happened to be declared could change it, then two
//! reviewers who typed the same facts in a different order would get
//! two different answers to "is this the same target contract", and
//! the projection would be worthless for the one job it has.

use std::collections::BTreeMap;

use crate::definition::{
    TargetDefinition, TargetDefinitionParts, reviewed_elements_tapscript,
    validate_target_definition,
};
use crate::opcode::{OpcodeId, OpcodeSpec};

/// The reviewed contract's parts.
fn reviewed_parts() -> TargetDefinitionParts {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let source = reviewed.definition();
    TargetDefinitionParts {
        version: source.version(),
        execution_domain: source.execution_domain(),
        leaf_version: source.leaf_version(),
        opcodes: source.opcodes().clone(),
        encodings: source.encodings().clone(),
        authorization: source.authorization().clone(),
        confidential_values: source.confidential_values().clone(),
        issuance: source.issuance().clone(),
        resources: source.resources().clone(),
        capabilities: source.capabilities().clone(),
        evidence_requirements: source.evidence_requirements().clone(),
    }
}

/// Rebuilds the registry, inserting entries in the given order.
fn registry_in_order(order: &[OpcodeId]) -> BTreeMap<OpcodeId, OpcodeSpec> {
    let reviewed = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let source = reviewed.definition().opcodes();

    let mut built = BTreeMap::new();
    for id in order {
        let spec = source.get(id).expect("every reviewed identity is present");
        built.insert(*id, spec.clone());
    }
    assert_eq!(built.len(), source.len(), "the permutation is a bijection");
    built
}

/// A deterministic permutation of the identity census.
///
/// Written as an explicit stride rather than a shuffle so that a
/// failure is reproducible from the test source alone.
fn strided_order(stride: usize) -> Vec<OpcodeId> {
    let census = OpcodeId::ALL;
    let mut order = Vec::with_capacity(census.len());
    let mut index = 0;
    let mut visited = vec![false; census.len()];
    for _ in 0..census.len() {
        while visited[index] {
            index = (index + 1) % census.len();
        }
        visited[index] = true;
        order.push(census[index]);
        index = (index + stride) % census.len();
    }
    order
}

#[test]
fn the_projection_survives_declaration_permutation() {
    let expected = reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .projection();

    for stride in [1_usize, 3, 5, 7, 11, 13, 17] {
        let order = strided_order(stride);
        let definition = TargetDefinition::new(TargetDefinitionParts {
            opcodes: registry_in_order(&order),
            ..reviewed_parts()
        });
        let validated =
            validate_target_definition(definition).expect("a permutation is still valid");
        assert_eq!(
            validated.projection(),
            expected,
            "stride {stride} changed the projection"
        );
    }
}

#[test]
fn the_projection_survives_reverse_declaration() {
    let expected = reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .projection();

    let mut order: Vec<OpcodeId> = OpcodeId::ALL.to_vec();
    order.reverse();

    let definition = TargetDefinition::new(TargetDefinitionParts {
        opcodes: registry_in_order(&order),
        ..reviewed_parts()
    });
    let validated = validate_target_definition(definition).expect("a permutation is still valid");
    assert_eq!(validated.projection(), expected);
}

#[test]
fn repeated_construction_is_equal() {
    let first = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let second = reviewed_elements_tapscript().expect("the reviewed contract validates");
    assert_eq!(first, second);
    assert_eq!(first.projection(), second.projection());
}

#[test]
fn the_projection_is_ordered_by_stable_identity_without_duplicates() {
    // Checking a canonical vector by converting it to a set would
    // silently accept a duplicate. The order and the count are both
    // part of the contract, so both are asserted directly.
    let projection = reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .projection();

    let ids: Vec<OpcodeId> = projection.opcodes().iter().map(OpcodeSpec::id).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();

    assert_eq!(ids, sorted, "the projection is sorted and duplicate-free");
    assert_eq!(ids.len(), OpcodeId::ALL.len(), "no entry is missing");
}

#[test]
fn the_strided_orders_really_are_permutations() {
    // Guards the test apparatus itself: a stride that failed to visit
    // every identity would make the determinism tests vacuous.
    for stride in [1_usize, 3, 5, 7, 11, 13, 17] {
        let order = strided_order(stride);
        let mut sorted = order.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), OpcodeId::ALL.len(), "stride {stride}");
        assert_eq!(order.len(), OpcodeId::ALL.len(), "stride {stride}");
    }
}
