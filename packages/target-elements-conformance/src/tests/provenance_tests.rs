//! The admitted provenance syntax, the matching rule, and the
//! command-argument rule (`G11-R06`, `G11-R07`).

use std::collections::BTreeSet;

use crate::provenance::{
    ExpectedExecutorProvenance, FULL_REVISION_WIDTH, FullRevisionId, MAXIMUM_TOPIC_NAME_BYTES,
    MINIMUM_REVISION_PREFIX_WIDTH, ProvenanceArgumentDefect, ProvenanceSyntaxDefect, RevisionId,
    TopicName, expected_provenance_from_arguments,
};

/// A full object identifier, in the admitted syntax.
const FULL: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678";

/// A second one, differing from [`FULL`] in its first digit.
const OTHER: &str = "b1b2c3d4e5f60718293a4b5c6d7e8f9012345678";

#[test]
fn the_admitted_revision_syntax_is_lowercase_hexadecimal_of_bounded_width() {
    assert_eq!(FULL.len(), FULL_REVISION_WIDTH);
    RevisionId::new(FULL).expect("a full identifier is admitted");
    RevisionId::new(&FULL[..MINIMUM_REVISION_PREFIX_WIDTH])
        .expect("the narrowest abbreviation is admitted");

    assert_eq!(
        RevisionId::new(&FULL[..MINIMUM_REVISION_PREFIX_WIDTH - 1]),
        Err(ProvenanceSyntaxDefect::RevisionTooShort),
    );
    assert_eq!(
        RevisionId::new(""),
        Err(ProvenanceSyntaxDefect::RevisionTooShort),
        "a blank field is not a revision",
    );
    assert_eq!(
        RevisionId::new(&"a".repeat(FULL_REVISION_WIDTH + 1)),
        Err(ProvenanceSyntaxDefect::RevisionTooLong),
    );
    for rejected in [
        "A1B2C3D",
        "g1b2c3d",
        "a1b2c3 ",
        "a1b2-c3d",
        "not a revision",
    ] {
        assert_eq!(
            RevisionId::new(rejected),
            Err(ProvenanceSyntaxDefect::RevisionNotLowercaseHex),
            "{rejected} is not an admitted revision",
        );
    }
}

#[test]
fn an_expectation_must_be_a_full_identifier_rather_than_a_prefix() {
    FullRevisionId::new(FULL).expect("a full identifier is a full identifier");
    assert_eq!(
        FullRevisionId::new(&FULL[..12]),
        Err(ProvenanceSyntaxDefect::RevisionNotFullWidth),
        "an expectation stated as an abbreviation compares abbreviations",
    );
    // Every width below the full one, refused by the same rule rather
    // than by the syntax that admits abbreviations generally.
    for width in MINIMUM_REVISION_PREFIX_WIDTH..FULL_REVISION_WIDTH {
        assert_eq!(
            FullRevisionId::new(&FULL[..width]),
            Err(ProvenanceSyntaxDefect::RevisionNotFullWidth),
            "a {width}-digit expectation is not a full identifier",
        );
        RevisionId::new(&FULL[..width]).expect("the same text is an admitted reported revision");
    }
}

/// `G12-R01`: the expectation has no constructible members.
///
/// The row was a public-API weakness rather than a logic error: the
/// members were public, so a caller wrote the struct literal and put an
/// abbreviation where a full identifier was required. The repair is
/// structural, and this test states the surface that makes it hold.
///
/// Two of the three parts cannot be asserted at runtime, because they
/// are now compile errors — the struct literal, and passing a
/// [`RevisionId`] where a [`FullRevisionId`] is required. What is
/// checked here is that the type is reached only through its
/// constructor and read only through accessors, so any future edit
/// restoring a public member or a lenient constructor breaks this test.
#[test]
fn an_expectation_is_reachable_only_through_its_validating_constructor() {
    let expectation = ExpectedExecutorProvenance::new(FULL, OTHER, ["fix/one"])
        .expect("the full identifiers state an expectation");

    // Read-only accessors, each carrying the width in its type.
    let tip: &FullRevisionId = expectation.intended_tip();
    let base: &FullRevisionId = expectation.upstream_base();
    assert_eq!(tip.as_str(), FULL);
    assert_eq!(base.as_str(), OTHER);
    assert_eq!(tip.as_str().len(), FULL_REVISION_WIDTH);
    assert_eq!(base.as_str().len(), FULL_REVISION_WIDTH);
    assert_eq!(expectation.included_local_topics().len(), 1);

    // The constructor is total over its refusals: an abbreviation in
    // either revision position is refused by width, whatever the other
    // arguments say.
    for (tip_text, base_text) in [(&FULL[..12], OTHER), (FULL, &OTHER[..7])] {
        assert_eq!(
            ExpectedExecutorProvenance::new(tip_text, base_text, ["fix/one"]),
            Err(ProvenanceSyntaxDefect::RevisionNotFullWidth),
        );
    }

    // And the one full-width value a caller can hold is the wrapper, so
    // the matching rule cannot be handed a prefix as its expectation.
    let full = FullRevisionId::new(FULL).expect("a full identifier");
    assert!(
        RevisionId::new(&FULL[..MINIMUM_REVISION_PREFIX_WIDTH])
            .expect("an abbreviation is an admitted reported revision")
            .matches_full(&full),
    );
}

/// The one matching rule, stated as a test rather than as prose.
#[test]
fn the_matching_rule_is_minimum_width_exact_prefix() {
    let expected = FullRevisionId::new(FULL).expect("full");

    // Equality is the width-forty case of the same rule.
    assert!(RevisionId::new(FULL).expect("full").matches_full(&expected));
    // Every admitted abbreviation of it matches.
    for width in MINIMUM_REVISION_PREFIX_WIDTH..=FULL_REVISION_WIDTH {
        assert!(
            RevisionId::new(&FULL[..width])
                .expect("an abbreviation is admitted")
                .matches_full(&expected),
            "an exact {width}-digit prefix matches",
        );
    }
    // A different object does not, however wide.
    assert!(
        !RevisionId::new(OTHER)
            .expect("full")
            .matches_full(&expected),
        "a different identifier is not a prefix",
    );
    assert!(
        !RevisionId::new(&OTHER[..MINIMUM_REVISION_PREFIX_WIDTH])
            .expect("abbreviation")
            .matches_full(&expected),
        "a different abbreviation is not a prefix",
    );
    // And a prefix that ends one digit early somewhere in the middle is
    // still a prefix, which is the whole cost of the rule: it names the
    // object less precisely, never a different one that shares it.
    let mut nearly = FULL[..12].to_owned();
    nearly.pop();
    nearly.push(if FULL.as_bytes()[11] == b'f' {
        'a'
    } else {
        'f'
    });
    assert!(
        !RevisionId::new(&nearly)
            .expect("abbreviation")
            .matches_full(&expected),
        "a prefix differing in any digit does not match",
    );
}

#[test]
fn the_admitted_topic_syntax_refuses_blanks_and_stray_bytes() {
    for admitted in ["fix/example", "topic-1", "a_b.c", "feature/x+y"] {
        TopicName::new(admitted).expect("an admitted topic name");
    }
    assert_eq!(TopicName::new(""), Err(ProvenanceSyntaxDefect::TopicBlank));
    assert_eq!(
        TopicName::new("   "),
        Err(ProvenanceSyntaxDefect::TopicBlank),
    );
    assert_eq!(
        TopicName::new(&"a".repeat(MAXIMUM_TOPIC_NAME_BYTES + 1)),
        Err(ProvenanceSyntaxDefect::TopicTooLong),
    );
    for rejected in ["fix example", "fix\texample", "fix;rm -rf", "fix\nexample"] {
        assert_eq!(
            TopicName::new(rejected),
            Err(ProvenanceSyntaxDefect::TopicCharacterNotAdmitted),
            "{rejected:?} is not an admitted topic name",
        );
    }
}

#[test]
fn an_expectation_carries_its_topic_census_as_a_set() {
    let expectation = ExpectedExecutorProvenance::new(FULL, OTHER, ["b/two", "a/one", "b/two"])
        .expect("the expectation states");
    let names: BTreeSet<String> = expectation
        .included_local_topics()
        .iter()
        .map(|topic| topic.as_str().to_owned())
        .collect();
    assert_eq!(
        names,
        BTreeSet::from(["a/one".to_owned(), "b/two".to_owned()]),
        "a census is a set, so a repeated declaration is one topic",
    );
    assert_eq!(expectation.intended_tip().as_str(), FULL);
    assert_eq!(expectation.upstream_base().as_str(), OTHER);
}

// -- G11-R07: the command-argument rule --------------------------------

#[test]
fn a_reviewed_run_requires_both_binding_revisions() {
    assert_eq!(
        expected_provenance_from_arguments(true, None, Some(OTHER), &[]),
        Err(ProvenanceArgumentDefect::MissingIntendedTip),
    );
    assert_eq!(
        expected_provenance_from_arguments(true, Some(FULL), None, &[]),
        Err(ProvenanceArgumentDefect::MissingUpstreamBase),
    );
    assert_eq!(
        expected_provenance_from_arguments(true, None, None, &[]),
        Err(ProvenanceArgumentDefect::MissingIntendedTip),
    );
}

#[test]
fn a_reviewed_run_refuses_arguments_outside_the_admitted_syntax() {
    assert_eq!(
        expected_provenance_from_arguments(true, Some(&FULL[..12]), Some(OTHER), &[]),
        Err(ProvenanceArgumentDefect::Malformed(
            ProvenanceSyntaxDefect::RevisionNotFullWidth
        )),
    );
    assert_eq!(
        expected_provenance_from_arguments(true, Some(FULL), Some("not-a-revision"), &[]),
        Err(ProvenanceArgumentDefect::Malformed(
            ProvenanceSyntaxDefect::RevisionNotLowercaseHex
        )),
    );
    assert_eq!(
        expected_provenance_from_arguments(
            true,
            Some(FULL),
            Some(OTHER),
            &["fix example".to_owned()],
        ),
        Err(ProvenanceArgumentDefect::Malformed(
            ProvenanceSyntaxDefect::TopicCharacterNotAdmitted
        )),
    );
}

#[test]
fn a_reviewed_run_with_complete_arguments_states_its_expectation() {
    let expectation =
        expected_provenance_from_arguments(true, Some(FULL), Some(OTHER), &["fix/one".to_owned()])
            .expect("the arguments state an expectation")
            .expect("a reviewed run always states one");
    assert_eq!(expectation.intended_tip().as_str(), FULL);
    assert_eq!(expectation.upstream_base().as_str(), OTHER);
    assert_eq!(expectation.included_local_topics().len(), 1);

    // No topic at all is a statement — a tip that folded in no local
    // branch — rather than an omission.
    let none = expected_provenance_from_arguments(true, Some(FULL), Some(OTHER), &[])
        .expect("the arguments state an expectation")
        .expect("a reviewed run always states one");
    assert!(none.included_local_topics().is_empty());
}

#[test]
fn a_mock_run_states_no_expectation_and_refuses_one() {
    assert_eq!(
        expected_provenance_from_arguments(false, None, None, &[]),
        Ok(None),
        "a mock run needs no expectation, because none makes it evidence",
    );
    for stated in [
        expected_provenance_from_arguments(false, Some(FULL), None, &[]),
        expected_provenance_from_arguments(false, None, Some(OTHER), &[]),
        expected_provenance_from_arguments(false, None, None, &["fix/one".to_owned()]),
    ] {
        assert_eq!(
            stated,
            Err(ProvenanceArgumentDefect::ProvenanceStatedForMockRun),
            "a mock run stating provenance is a caller believing they configured evidence",
        );
    }
}
