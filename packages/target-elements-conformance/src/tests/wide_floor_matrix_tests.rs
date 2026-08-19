//! The wide-floor case matrix: coherence, coverage, and honesty.

use std::collections::BTreeSet;

use crate::prototype::{
    CompoundPrototypeFixture, ExpectedPrototypeOutcome, PrototypeCaseId, PrototypeClaim,
    PrototypeRelation, wide_floor_bearing_cases, wide_floor_case_matrix,
    wide_floor_residual_threats,
};
use crate::tests::support::reviewed_target;
use crate::wide_floor::oracle::WideFloorInstance;

/// The matrix, or a panic naming why it could not be authored.
fn matrix() -> Vec<CompoundPrototypeFixture> {
    let target = reviewed_target();
    wide_floor_case_matrix(&target)
        .expect("the reviewed contract and the oracle determine the rows")
        .rows()
        .to_vec()
}

/// The wide-floor claims, which are exactly those of the relation.
fn wide_floor_claims() -> BTreeSet<PrototypeClaim> {
    PrototypeClaim::ALL
        .iter()
        .copied()
        .filter(|claim| claim.relation() == PrototypeRelation::WideFloorRelation)
        .collect()
}

#[test]
fn every_row_states_a_coherent_case() {
    let target = reviewed_target();
    for fixture in matrix() {
        assert!(
            fixture.is_coherent(&target),
            "{} is incoherent: {:?}",
            fixture.case,
            fixture.defect(&target)
        );
    }
}

#[test]
fn every_row_belongs_to_the_wide_floor_relation_and_names_a_unique_case() {
    let rows = matrix();
    let mut seen: BTreeSet<PrototypeCaseId> = BTreeSet::new();
    for fixture in &rows {
        assert_eq!(fixture.case.relation, PrototypeRelation::WideFloorRelation);
        assert!(!fixture.claims.is_empty());
        for claim in &fixture.claims {
            assert_eq!(claim.relation(), PrototypeRelation::WideFloorRelation);
        }
        assert!(
            seen.insert(fixture.case.clone()),
            "{} is duplicated",
            fixture.case
        );
    }
    assert_eq!(seen.len(), rows.len());
}

#[test]
fn no_wide_floor_row_states_an_output_of_any_role() {
    // The pattern reads no transaction field, so an output the fixture
    // required would be a requirement its program never checks.
    for fixture in matrix() {
        assert!(fixture.construction.outputs.is_empty(), "{}", fixture.case);
    }
}

#[test]
fn every_wide_floor_claim_has_a_bearing_case() {
    let rows = matrix();
    let bearing = wide_floor_bearing_cases(&rows);
    for claim in wide_floor_claims() {
        let cases = bearing
            .get(&claim)
            .unwrap_or_else(|| panic!("{claim:?} has no bearing case"));
        assert!(!cases.is_empty(), "{claim:?} has an empty bearing set");
    }
}

#[test]
fn no_constructor_claim_is_borne_by_a_wide_floor_row() {
    let rows = matrix();
    let bearing = wide_floor_bearing_cases(&rows);
    for claim in bearing.keys() {
        assert_eq!(claim.relation(), PrototypeRelation::WideFloorRelation);
    }
}

#[test]
fn the_matrix_states_both_verdicts_and_neither_trivially() {
    let rows = matrix();
    let accepted = rows
        .iter()
        .filter(|row| row.expected == ExpectedPrototypeOutcome::Accepted)
        .count();
    let rejected = rows.len() - accepted;
    // Guide 10 requires the boundary vectors and every mutation, so the
    // shape of the matrix is a check in itself: a matrix that had lost
    // its accepting half would still pass every coherence test above.
    assert!(accepted >= 15, "only {accepted} accepting rows");
    assert!(rejected >= 20, "only {rejected} rejecting rows");
}

#[test]
fn every_accepting_row_carries_the_witness_the_oracle_states() {
    // The expectation is the oracle's, not the schedule's: a row that
    // accepted a witness the oracle disagreed with would be an
    // expectation produced by the thing under test.
    for fixture in matrix() {
        if fixture.expected != ExpectedPrototypeOutcome::Accepted {
            continue;
        }
        assert_eq!(fixture.initial_stack.len(), 5, "{}", fixture.case);
        let value = |index: usize| -> u64 {
            let mut bytes = [0_u8; 8];
            bytes.copy_from_slice(&fixture.initial_stack[index]);
            u64::from_le_bytes(bytes)
        };
        let instance = WideFloorInstance::solve(value(0), value(1), value(3))
            .unwrap_or_else(|defect| panic!("{} is out of domain: {defect:?}", fixture.case));
        assert_eq!(instance.quotient(), value(2), "{}", fixture.case);
        assert_eq!(instance.remainder(), value(4), "{}", fixture.case);
        assert!(instance.limbs_agree(), "{}", fixture.case);
    }
}

#[test]
fn every_rejecting_row_differs_from_every_accepting_one() {
    // A rejecting row whose witness happened to be a valid one would be
    // a row the target must accept, stated as a rejection.
    let rows = matrix();
    let accepting: BTreeSet<(Vec<u8>, Vec<Vec<u8>>)> = rows
        .iter()
        .filter(|row| row.expected == ExpectedPrototypeOutcome::Accepted)
        .map(|row| (row.script.clone(), row.initial_stack.clone()))
        .collect();
    for fixture in &rows {
        if fixture.expected != ExpectedPrototypeOutcome::Rejected {
            continue;
        }
        assert!(
            !accepting.contains(&(fixture.script.clone(), fixture.initial_stack.clone())),
            "{} restates an accepting row",
            fixture.case
        );
    }
}

#[test]
fn exactly_one_row_mutates_the_script_rather_than_the_witness() {
    let rows = matrix();
    let scripts: BTreeSet<Vec<u8>> = rows.iter().map(|row| row.script.clone()).collect();
    assert_eq!(
        scripts.len(),
        2,
        "the matrix states one script and one variant"
    );
    let variant = rows
        .iter()
        .filter(|row| row.case.name == "arithmetic_flag_left_unchecked")
        .count();
    assert_eq!(variant, 1);
}

#[test]
fn the_flag_unchecked_variant_is_also_refused_statically() {
    // Guide 10 admits this row as a static *or* a target rejection. It
    // is both: the emitted program's own bytes are not the variant's,
    // so a schedule missing the verification is not something this
    // package can emit at all.
    let target = reviewed_target();
    let rows = matrix();
    let emitted = crate::prototype_program::PrototypeProgram::wide_floor(&target)
        .expect("the schedule is admitted")
        .encode(&target);
    let variant = rows
        .iter()
        .find(|row| row.case.name == "arithmetic_flag_left_unchecked")
        .expect("the row exists");
    assert_ne!(variant.script, emitted);
    assert!(variant.script.len() < emitted.len());
}

#[test]
fn the_residual_threats_are_enumerated_with_reasons() {
    let residuals = wide_floor_residual_threats();
    assert!(residuals.len() >= 6);
    let named: BTreeSet<&str> = residuals.iter().map(|(name, _)| *name).collect();
    assert_eq!(named.len(), residuals.len());
    for (name, reason) in residuals {
        assert!(!name.is_empty());
        assert!(reason.len() > 40, "{name} has no stated reason");
    }
}

#[test]
fn each_claim_is_borne_by_the_verdict_it_is_about() {
    // A claim about a refusal borne only by accepting rows would be
    // established by a run in which the program checked nothing, and a
    // claim about an acceptance borne only by refusals would be
    // established by a program that refused everything.
    let rows = matrix();
    for claim in wide_floor_claims() {
        let bearing: Vec<&CompoundPrototypeFixture> = rows
            .iter()
            .filter(|fixture| fixture.claims.contains(&claim))
            .collect();
        assert!(!bearing.is_empty(), "{claim:?} has no bearing row");
        let wanted = match claim {
            PrototypeClaim::WideFloorExactDivisionObserved
            | PrototypeClaim::WideFloorNonzeroRemainderObserved
            | PrototypeClaim::WideFloorLimbDerivationObserved => ExpectedPrototypeOutcome::Accepted,
            _ => ExpectedPrototypeOutcome::Rejected,
        };
        assert!(
            bearing.iter().all(|fixture| fixture.expected == wanted),
            "{claim:?} is borne by a row of the wrong verdict"
        );
    }
}

#[test]
fn every_wide_floor_claim_is_required() {
    // This asserted the opposite for as long as no runner existed to
    // execute the matrix. One exists now, so a claim recorded as
    // unresolved would be the project understating what it attempted.
    for claim in wide_floor_claims() {
        assert!(
            matches!(
                claim.requirement(),
                crate::claim::ClaimRequirement::Required
            ),
            "{claim:?} is not required, though its matrix is executed"
        );
    }
}
