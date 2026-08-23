//! Guide-13 §18's resource study, exercised at the package boundary.
//!
//! The module tests inside `vectors` check each half of the study against
//! the machinery beside it. These check what a *consumer* of the package
//! gets: that the study is reachable through the public surface, that the
//! only route to a validated resource report runs through a validator,
//! and that the document that comes out says what §18.5 permits and
//! nothing further.

use std::collections::BTreeSet;

// The forbidden-key scan is reached through its own module rather than
// through the crate root. Each report owns a scan of that name, and the
// two lists are deliberately different — a crate-root re-export would
// have had to rename one of them and hide which document a caller was
// checking.
use vectors::live_resource_report::canonical_bytes_publish_no_forbidden_key;
use vectors::{
    CandidateBoundsResult, ComparisonStanding, DimensionStanding, LiveResourceCase,
    LiveResourceRecord, ResourceNonClaim, assemble_live_resource_report, assignments_realized_by,
    compare_run, measure_resource_cases, observed_run_of_record, render_live_resource_report,
    research_bound_assignments, run_agreements, run_failures, validate_live_resource_report,
};

/// The reviewed target's projection.
fn projection() -> target_elements::TargetProjection {
    target_elements::reviewed_elements_tapscript()
        .expect("the reviewed target binds")
        .projection()
}

#[test]
fn the_study_is_reachable_and_validated_through_the_public_surface() {
    // §18.5's document, obtained the only way a consumer can obtain one.
    // A caller cannot construct a validated report; it has to assemble a
    // report and hand it to a validator that rebuilds every carrier.
    let target = projection();
    let report = assemble_live_resource_report(target.clone()).expect("the study measures");
    let validated =
        validate_live_resource_report(report, &target).expect("the resource report validates");

    let census = validated.report().census();
    assert_eq!(census.enumerated(), research_bound_assignments().len());
    assert_eq!(census.cases(), LiveResourceCase::ALL.len());
    assert_eq!(census.dimensions(), LiveResourceRecord::ALL.len());
    assert_eq!(census.mismatches(), 0);
    assert_ne!(census.agreements(), 0);

    assert_eq!(
        validated.report().result(),
        CandidateBoundsResult::CandidateBoundsFitThisTestedCandidateBundleAndAbi,
    );
    assert!(canonical_bytes_publish_no_forbidden_key(&validated));

    let rendered = render_live_resource_report(&validated);
    assert!(rendered.starts_with("schema live-transfer-resource/1\n"));
    assert_eq!(rendered, render_live_resource_report(&validated));
}

#[test]
fn the_enumeration_is_wide_and_what_the_candidate_realizes_is_narrow() {
    // §18.1's enumeration against §18.5's fitting relation, from outside.
    // The gap between the two is this study's headline: two hundred and
    // ninety-four assignments were costed and the tested bundle has a
    // program for every shape of eight of them.
    let enumerated = research_bound_assignments();
    let realized = assignments_realized_by(&tapscript::demonstration_live_shape_set());

    assert_eq!(enumerated.len(), 294);
    assert_eq!(realized.len(), 8);
    assert!(realized.len() < enumerated.len());

    let enumerated: BTreeSet<_> = enumerated.into_iter().collect();
    for bounds in &realized {
        assert!(
            enumerated.contains(bounds),
            "a realized assignment must be one the enumeration holds",
        );
    }
}

#[test]
fn every_case_records_every_dimension_and_the_non_claims_travel_with_them() {
    // §18.3's "record separately", checked at the package boundary, and
    // the non-claims checked to be in the document rather than in a doc
    // comment — a summary can be compared against a carried non-claim and
    // cannot be compared against a sentence in prose.
    let cases = measure_resource_cases().expect("the study measures");
    assert_eq!(cases.len(), LiveResourceCase::ALL.len());

    let mut transactions = 0;
    for case in &cases {
        for member in case.members() {
            transactions += 1;
            assert_eq!(member.dimensions().len(), LiveResourceRecord::ALL.len());

            // The three dimensions no measurement here carries a figure
            // for. Each is a typed value rather than a missing key, which
            // is what stops a consumer reading it as zero.
            for blocked in [
                LiveResourceRecord::ConsensusVerdict,
                LiveResourceRecord::RelayPolicyVerdict,
            ] {
                assert!(matches!(
                    member.standing(blocked),
                    Some(DimensionStanding::NotClaimable(_)),
                ));
            }
            assert_eq!(
                member.standing(LiveResourceRecord::ConstructionAndExecutionTime),
                Some(DimensionStanding::NoncanonicalDiagnostic),
            );
            assert_eq!(
                member
                    .standing(LiveResourceRecord::OwnerSignatureWitnessBytes)
                    .and_then(DimensionStanding::measured),
                None,
                "a signature slot is not a measurement of a signature",
            );
        }
    }
    assert_eq!(transactions, 17);

    let target = projection();
    let report = assemble_live_resource_report(target).expect("the study measures");
    assert_eq!(report.non_claims().len(), ResourceNonClaim::ALL.len());
}

#[test]
fn the_comparison_names_an_absence_for_every_dimension_it_could_not_observe() {
    // §18.4's rule, from outside: no absent observation is read as zero
    // or as agreement. Every dimension appears in the table, one of them
    // was compared, and every other one names which kind of absence it
    // is.
    let comparisons = compare_run(&observed_run_of_record());
    assert_eq!(run_failures(&comparisons), vec![]);
    assert_eq!(run_agreements(&comparisons), 1);

    for comparison in &comparisons {
        assert_eq!(comparison.standings().len(), LiveResourceRecord::ALL.len());
        for (dimension, standing) in comparison.standings() {
            assert!(
                !standing.is_mismatch(),
                "{} disagreed at the package boundary",
                dimension.name(),
            );
            // An agreement carries a real figure. Every other standing
            // carries no figure at all, which is the whole point: the
            // wildcard is the absences, and they are checked to be
            // absences by the assertion above rather than here.
            if let ComparisonStanding::Agree { figure } = standing {
                assert_ne!(*figure, 0);
            }
        }
    }
}
