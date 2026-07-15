//! Clause-weld tests.
//!
//! Implements `´test:verification:clause-weld´`: the reasons→clauses
//! mapping is total (compile-enforced by the exhaustive match) and
//! surjective onto the exported clause registry — with one deliberate
//! exception, asserted rather than discovered.

use std::collections::BTreeSet;

use architecture::InvariantClauseId;

use crate::*;

#[test]
fn every_reason_maps_onto_a_registered_clause() {
    let registered = architecture::ARCHITECTURE
        .clauses
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    for reason in InvariantError::ALL {
        assert!(
            registered.contains(&clause_of(*reason)),
            "reason {reason:?} maps to an unregistered clause",
        );
    }
}

#[test]
fn every_clause_is_the_image_of_a_reason_except_consensus_value() {
    let image = InvariantError::ALL
        .iter()
        .map(|reason| clause_of(*reason))
        .collect::<BTreeSet<_>>();

    for clause in InvariantClauseId::ALL {
        if *clause == InvariantClauseId::ConsensusValue {
            // Deliberate emptiness, asserted rather than discovered:
            // committed-vs-consensus divergence is unrepresentable in
            // this model, so no runtime reason can produce the
            // consensus-value clause. Its discharge is structural
            // (`ExplicitValueIntrospection` + issuance introspection); see
            // `clause_of` and the `Utxo` doc comment.
            continue;
        }

        assert!(
            image.contains(clause),
            "clause {clause} has no failure reason mapping onto it",
        );
    }

    // The image contains ConsensusValue only through the declared,
    // never-produced reason — remove it and the clause is empty.
    assert!(image.contains(&InvariantClauseId::ConsensusValue));

    let produced_image = InvariantError::ALL
        .iter()
        .filter(|reason| !matches!(reason, InvariantError::ConsensusValueAuthority))
        .map(|reason| clause_of(*reason))
        .collect::<BTreeSet<_>>();

    assert!(!produced_image.contains(&InvariantClauseId::ConsensusValue));
}
