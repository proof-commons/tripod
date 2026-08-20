//! The §7.3 backend policy, and every way its selection refuses.
//!
//! The interesting assertions here are the refusals. A selection rule
//! is only worth stating if it can decline, and §7.3 attaches three
//! separate conditions to a lexicographic least-key selection — the
//! policy states it, the objective has already narrowed the field, and
//! the survivors' semantic equivalence is established. Each is reached
//! by a fixture that fails exactly that one.

use std::collections::{BTreeMap, BTreeSet};

use target_elements::{
    FieldForm, SponsorInspectedField, SponsorProgramClass, TransactionForm, ZeroFeeRepresentation,
};

use crate::policy::{
    AshRepresentationSelection, CompactAshBackendPolicy, ConcreteCandidate, LayoutFamily,
    SelectionObjective, SelectionRefusal, SemanticEquivalence, TieBreak,
    reviewed_target_projection,
};
use crate::shape::{CandidateShapeSet, demonstration_shape_set};

/// A policy with the stated tie-break.
fn policy(tie_break: TieBreak) -> CompactAshBackendPolicy {
    CompactAshBackendPolicy::new(
        reviewed_target_projection(),
        AshRepresentationSelection::Explicit,
        demonstration_shape_set(),
        LayoutFamily::CoordinatorPrefixedSponsorSuffix,
        Vec::new(),
        SelectionObjective::EncodedProgramBytes,
        tie_break,
    )
}

/// The stated least-key tie-break over the accepted objective.
const fn least_key() -> TieBreak {
    TieBreak::LexicographicLeastKey {
        objective: SelectionObjective::EncodedProgramBytes,
    }
}

/// A candidate over `shapes` measuring `measure`.
fn candidate(shapes: CandidateShapeSet, measure: u64) -> ConcreteCandidate {
    ConcreteCandidate::new(BTreeSet::new(), shapes, measure)
}

/// The demonstration set, and a strictly smaller one to tie against it.
fn two_shape_sets() -> (CandidateShapeSet, CandidateShapeSet) {
    let full = demonstration_shape_set();
    let narrow = CandidateShapeSet::new(full.bounds(), full.shapes().take(1).collect(), true);
    (full, narrow)
}

#[test]
fn the_policy_states_every_ground_section_seven_three_lists() {
    // Each of the seven grounds is readable, and the sponsor profile is
    // the reviewed one rather than a second spelling of it.
    let policy = policy(least_key());

    assert_eq!(
        policy.representation(),
        AshRepresentationSelection::Explicit,
    );
    assert_eq!(
        policy.layout(),
        LayoutFamily::CoordinatorPrefixedSponsorSuffix,
    );
    assert_eq!(policy.objective(), SelectionObjective::EncodedProgramBytes);
    assert_eq!(policy.tie_break(), least_key());
    assert_eq!(policy.cardinality(), &demonstration_shape_set());
    assert_eq!(
        policy.sponsor_profile(),
        policy.projection().sponsor(),
        "the profile is read from the projection, not stored twice",
    );

    // No pattern is preferred yet where none has been admitted: a
    // preference naming a pattern that does not exist would be the
    // speculative identity §1.10 refuses.
    assert_eq!(policy.pattern_preference(), []);
}

#[test]
fn the_projection_carries_the_reviewed_wave_five_facts_unchanged() {
    // The projection is the target package's own reviewed values. This
    // checks the three facts the later patterns depend on: the fee role
    // is not a protocol object, a zero fee is the absence of the
    // output, and the sponsor asset is forced explicit while the
    // sponsor value is never inspected.
    let projection = reviewed_target_projection();

    assert!(!projection.fee().protocol_object());
    assert_eq!(
        projection.fee().zero_fee(),
        ZeroFeeRepresentation::AbsentOutput
    );
    assert_eq!(
        projection.fee().refused_zero_fee(),
        ZeroFeeRepresentation::ZeroValuedOutput,
    );
    assert!(!projection.fee().blindable());

    let sponsor = projection.sponsor();
    assert_eq!(sponsor.asset_form(), FieldForm::ExplicitOnly);
    assert_eq!(
        sponsor.inspected(),
        &BTreeSet::from([
            SponsorInspectedField::RegionMembership,
            SponsorInspectedField::RegionExtent,
            SponsorInspectedField::Asset,
        ]),
    );
    assert_eq!(
        sponsor.admitted_programs(),
        &BTreeSet::from([SponsorProgramClass::WitnessV0KeyHash]),
    );
    assert!(!sponsor.arbitrary_program_support_claimed());

    // Both forms are reviewed, so a sponsorless candidate is not
    // silently made to depend on sponsorship.
    let forms = projection.forms();
    assert_eq!(
        forms.keys().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([TransactionForm::Sponsorless, TransactionForm::Sponsored]),
    );
    assert_eq!(forms.len(), 2);
    assert!(!projection.zero_value().is_empty());
    let _: &BTreeMap<TransactionForm, _> = forms;
}

#[test]
fn the_objective_selects_outright_when_it_separates_the_candidates() {
    // No tie-break is consulted, and the policy that states none still
    // selects: a rule needed only for ties must not be needed for
    // anything else.
    let (full, narrow) = two_shape_sets();
    let candidates = BTreeSet::from([candidate(full, 900), candidate(narrow.clone(), 400)]);

    let selected = policy(TieBreak::NoneStated)
        .select(&candidates, SemanticEquivalence::NotEstablished)
        .expect("the objective separates them");

    assert_eq!(selected.measure(), 400);
    assert_eq!(selected.shapes(), &narrow);
}

#[test]
fn nothing_offered_is_a_refusal_rather_than_an_empty_answer() {
    assert_eq!(
        policy(least_key()).select(
            &BTreeSet::new(),
            SemanticEquivalence::EstablishedByRelationAndShapeIdentity,
        ),
        Err(SelectionRefusal::NoCandidates),
    );
}

#[test]
fn a_tie_with_no_stated_rule_refuses() {
    // §7.3 permits a least-key selection only where the policy states
    // it. Without one there is no ground to prefer either candidate,
    // and taking the first would be choosing on the container's order.
    let (full, narrow) = two_shape_sets();
    let candidates = BTreeSet::from([candidate(full, 500), candidate(narrow, 500)]);

    assert_eq!(
        policy(TieBreak::NoneStated).select(
            &candidates,
            SemanticEquivalence::EstablishedByRelationAndShapeIdentity,
        ),
        Err(SelectionRefusal::TiedWithNoTieBreak { tied: 2 }),
    );
}

#[test]
fn a_tie_whose_equivalence_is_not_established_refuses() {
    // Two programs of equal size can enforce different relations, so
    // equal measures are not equivalence. The rule requires the
    // equivalence to be established, and the policy refuses rather than
    // inferring it from the tie it is trying to break.
    let (full, narrow) = two_shape_sets();
    let candidates = BTreeSet::from([candidate(full, 500), candidate(narrow, 500)]);

    assert_eq!(
        policy(least_key()).select(&candidates, SemanticEquivalence::NotEstablished,),
        Err(SelectionRefusal::TiedWithoutEstablishedEquivalence { tied: 2 }),
    );
}

#[test]
fn an_established_tie_selects_the_lexicographically_least_typed_key() {
    // The key is the typed candidate itself — patterns, then shapes —
    // compared exactly, with no digest and no rendering in between
    // (§1.10). The narrow set orders below the full one because it is a
    // strict prefix of it under the derived ordering, so the selection
    // is predictable rather than incidental.
    let (full, narrow) = two_shape_sets();
    let candidates = BTreeSet::from([candidate(full.clone(), 500), candidate(narrow.clone(), 500)]);

    let selected = policy(least_key())
        .select(
            &candidates,
            SemanticEquivalence::EstablishedByRelationAndShapeIdentity,
        )
        .expect("an established tie is breakable");

    let expected = candidate(full, 500).min(candidate(narrow, 500));
    assert_eq!(selected, &expected);
}

#[test]
fn a_worse_candidate_never_wins_a_tie_break_it_was_not_in() {
    // The tie-break applies to the survivors of the objective, never to
    // the whole field. A candidate the objective already rejected must
    // not be selected because its typed key happens to sort first.
    let (full, narrow) = two_shape_sets();
    let candidates = BTreeSet::from([
        candidate(full.clone(), 500),
        candidate(narrow.clone(), 500),
        candidate(narrow.clone(), 900),
    ]);

    let selected = policy(least_key())
        .select(
            &candidates,
            SemanticEquivalence::EstablishedByRelationAndShapeIdentity,
        )
        .expect("the tie is breakable");

    assert_eq!(selected.measure(), 500);
    assert_ne!(full, narrow);
}
