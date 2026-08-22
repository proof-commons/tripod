//! Semantic fixtures and their independently derived expectations.
//!
//! Guide-12 §17.2 states what a semantic fixture must *not* contain:
//! target input index, target output index, target program, tapleaf,
//! control block, transaction byte, sponsor wallet state. This module
//! takes that list literally. A [`CompactAshSemanticCase`] is a list of
//! protocol amounts, a sponsor case, and a name — there is no field a
//! target fact could be put in, and a test checks the rendering for the
//! vocabulary anyway.
//!
//! # Where the expected result comes from
//!
//! §17.3 forbids the candidate backend from generating the expectation
//! used to judge it. The successor amount here is therefore derived with
//! [`ProtocolAmount::checked_sum`] — the realization layer's own
//! arithmetic over its own domain — and never by asking the constructor
//! what it settled on. The construction's answer is compared against
//! this one later; if the two were the same computation there would be
//! nothing to compare.

use realization::ProtocolAmount;

use crate::error::VectorError;
use crate::matrix::{VectorClass, VectorPolarity};
use crate::projection::{
    AcceptedProjection, SemanticObjectId, SponsorChange, SponsorRegion, TransitionCertificate,
};

/// The exact operation identity every fixture in this package claims.
pub const OPERATION: &str = "compact-ash";

/// A semantic fixture's identity.
///
/// §17's fixture identity binds the architecture and realization, the
/// operation, the initial state, the request, the order context, and the
/// explicit parameters — and excludes target bytes and bundle identity.
/// The architecture and realization are constant across this package, so
/// they are bound once by the bundle rather than repeated per row; what
/// varies is carried here.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticFixtureId {
    name: &'static str,
    ordinal: u32,
}

impl SemanticFixtureId {
    /// The fixture named `name` at `ordinal` in the canonical order.
    #[must_use]
    pub const fn new(name: &'static str, ordinal: u32) -> Self {
        Self { name, ordinal }
    }

    /// The fixture's stable name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Its position in the canonical order.
    #[must_use]
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }
}

/// Whether the fixture's world carries a sponsor region, and what shape
/// that region has.
///
/// # Why the change role lives here
///
/// §18.1 names `sponsor-change-present` and `sponsor-change-absent` as
/// two classes. While this type recorded only membership, the two rows
/// answering those classes were identical values under two names, so
/// neither row witnessed the property its name asserts and a run of
/// either was equally good evidence for both (`G13-R10`).
///
/// The dimension sits on the sponsor case rather than beside it because
/// a change role is part of what a sponsor region *is*: §9.1 refuses one
/// with no region to carry it, and [`Self::region`] can only stay a
/// total function of this value while both facts are in it.
///
/// It is stated in this package's own vocabulary
/// ([`SponsorChange`]) and never the backend's. §17.2 keeps target and
/// backend terms out of a semantic fixture, and the mapping onto the
/// candidate's shape vocabulary happens at materialization, where the
/// target is.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SponsorCase {
    /// No sponsor region.
    Absent,
    /// A sponsor region with this many members and this change role.
    Present {
        /// How many members the region has.
        members: u16,
        /// Whether the region returns a residual to its sponsor.
        change: SponsorChange,
    },
}

impl SponsorCase {
    /// A sponsored case whose region returns nothing.
    #[must_use]
    pub const fn sponsored(members: u16) -> Self {
        Self::Present {
            members,
            change: SponsorChange::Absent,
        }
    }

    /// A sponsored case whose region returns a residual.
    #[must_use]
    pub const fn sponsored_with_change(members: u16) -> Self {
        Self::Present {
            members,
            change: SponsorChange::Present,
        }
    }

    /// The region this case projects.
    #[must_use]
    pub const fn region(self) -> SponsorRegion {
        match self {
            Self::Absent => SponsorRegion::ABSENT,
            Self::Present { members, change } => SponsorRegion::present(members, change),
        }
    }

    /// Whether a sponsor region exists.
    #[must_use]
    pub const fn is_present(self) -> bool {
        matches!(self, Self::Present { .. })
    }

    /// How many members the region has, which is zero where it is
    /// absent.
    ///
    /// A count rather than a presence flag, because the shape a row
    /// names is a function of the number and a target either has a
    /// program for that shape or does not.
    #[must_use]
    pub const fn members(self) -> u16 {
        match self {
            Self::Absent => 0,
            Self::Present { members, .. } => members,
        }
    }

    /// Whether the region returns a residual to its sponsor.
    ///
    /// Absent where there is no region, which is the only value §9.1
    /// admits there rather than a default chosen here.
    #[must_use]
    pub const fn change(self) -> SponsorChange {
        match self {
            Self::Absent => SponsorChange::Absent,
            Self::Present { change, .. } => change,
        }
    }
}

/// One positive semantic case: a model-valid compact-ASH world, its
/// request, and the projection the realization layer's own definitions
/// say an accepted transaction must exhibit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactAshSemanticCase {
    id: SemanticFixtureId,
    class: VectorClass,
    inputs: Vec<ProtocolAmount>,
    sponsor: SponsorCase,
    expected: AcceptedProjection,
}

impl CompactAshSemanticCase {
    /// The fixture's identity.
    #[must_use]
    pub const fn id(&self) -> SemanticFixtureId {
        self.id
    }

    /// The §18 class this fixture answers.
    #[must_use]
    pub const fn class(&self) -> VectorClass {
        self.class
    }

    /// The ASH input family's amounts, in the fixture's own order.
    #[must_use]
    pub fn inputs(&self) -> &[ProtocolAmount] {
        &self.inputs
    }

    /// The sponsor case.
    #[must_use]
    pub const fn sponsor(&self) -> SponsorCase {
        self.sponsor
    }

    /// The expected accepted projection.
    #[must_use]
    pub const fn expected(&self) -> &AcceptedProjection {
        &self.expected
    }

    /// The number of ASH inputs, which the target shape must match.
    #[must_use]
    pub const fn ash_inputs(&self) -> usize {
        self.inputs.len()
    }
}

/// Build one positive semantic case from stated facts.
///
/// # Errors
///
/// [`VectorError::InvalidSemanticFixture`] when the stated world is not
/// a model-valid compact-ASH world — fewer than the two inputs §12.3
/// requires, or an amount outside the realization layer's domain —
/// [`VectorError::FixtureContradictsItsClass`] when the facts stated do
/// not witness the §18.1 class the row is filed under, and
/// [`VectorError::ExpectationNotDerivable`] when the realization layer's
/// own arithmetic refuses the sum.
pub fn semantic_case(
    id: SemanticFixtureId,
    class: VectorClass,
    amounts: &[u64],
    sponsor: SponsorCase,
) -> Result<CompactAshSemanticCase, VectorError> {
    // §12.3's family cardinality: a compact-ASH consolidation of fewer
    // than two members is not the operation. Refusing here keeps an
    // invalid world out of the positive census rather than letting the
    // constructor discover it later, which would file a fixture defect
    // as a construction result.
    if amounts.len() < 2 {
        return Err(VectorError::InvalidSemanticFixture(id));
    }
    // And the row must actually witness what it is filed under. Checked
    // at the one place a case is built, so no route — the canonical
    // census, a test, or a later caller — can produce a row carrying a
    // class name without the property that name asserts.
    witnesses_its_class(id, class, sponsor)?;

    let mut inputs = Vec::with_capacity(amounts.len());
    for value in amounts {
        let amount = ProtocolAmount::new(*value)
            .map_err(|cause| VectorError::ExpectationNotDerivable { fixture: id, cause })?;
        inputs.push(amount);
    }

    // The realization layer's own checked arithmetic over its own
    // domain. Nothing about the candidate backend enters here.
    let successor = ProtocolAmount::checked_sum(inputs.iter().copied())
        .map_err(|cause| VectorError::ExpectationNotDerivable { fixture: id, cause })?;

    let family = inputs
        .iter()
        .enumerate()
        .map(|(ordinal, amount)| {
            (
                SemanticObjectId::new(u32::try_from(ordinal).unwrap_or(u32::MAX)),
                *amount,
            )
        })
        .collect::<Vec<_>>();
    let successor_handle = SemanticObjectId::new(u32::try_from(inputs.len()).unwrap_or(u32::MAX));

    let expected = AcceptedProjection::new(
        family,
        (successor_handle, successor),
        successor,
        successor,
        sponsor.region(),
        TransitionCertificate::DerivedFromRelations,
        OPERATION,
    );

    Ok(CompactAshSemanticCase {
        id,
        class,
        inputs,
        sponsor,
        expected,
    })
}

/// Whether a row's stated facts witness the §18.1 class it is filed
/// under.
///
/// # Why this is a check and not a convention
///
/// A class name is an assertion about the row, and until `G13-R10` the
/// only thing keeping a row honest about its own name was that whoever
/// wrote the census meant it. The two sponsor-change rows were the proof
/// that this is not enough: they were the same value twice, filed under
/// two names, and nothing anywhere noticed.
///
/// Only the classes whose names assert a fact this type can see are
/// listed. A class named for something a [`SponsorCase`] does not record
/// — the canonical ordering, the candidate maximum — is not silently
/// passed here: it is answered by a row whose *facts* make the property
/// checkable, which is a different obligation and belongs where those
/// facts are.
///
/// # Errors
///
/// [`VectorError::FixtureContradictsItsClass`] naming the row and the
/// class it does not witness.
fn witnesses_its_class(
    id: SemanticFixtureId,
    class: VectorClass,
    sponsor: SponsorCase,
) -> Result<(), VectorError> {
    // Spelled out name by name rather than by a prefix. Two of these
    // names begin with the same seven letters and assert opposite
    // things, so a rule written over the prefix would have passed the
    // sponsorless row for being sponsored.
    let holds = match class.name() {
        "sponsor-change-present" => {
            sponsor.is_present() && sponsor.change() == SponsorChange::Present
        }
        "sponsor-change-absent" => {
            sponsor.is_present() && sponsor.change() == SponsorChange::Absent
        }
        "sponsorless-consensus-transaction" => !sponsor.is_present(),
        "sponsored-transaction" => sponsor.is_present(),
        "one-sponsor-input" => sponsor.members() == 1,
        "multiple-sponsor-inputs" => sponsor.members() > 1,
        _ => true,
    };
    if holds {
        Ok(())
    } else {
        Err(VectorError::FixtureContradictsItsClass {
            fixture: id,
            class: class.name(),
        })
    }
}

/// The §18.1 positive semantic census, in the guide's own order.
///
/// Each row answers one named §18.1 class. Four of the fourteen classes
/// are not amount-and-sponsor facts at all — canonical input-order
/// normalization, the candidate maximum, the exact successor amount, and
/// report-byte repeatability — and they are answered by rows whose facts
/// make the property checkable rather than by rows that merely carry the
/// name.
///
/// # Errors
///
/// Propagates any [`VectorError`] from [`semantic_case`]; every row here
/// is a model-valid world, so a refusal is a regression in the
/// realization layer's domain rather than a fixture defect.
pub fn positive_semantic_census() -> Result<Vec<CompactAshSemanticCase>, VectorError> {
    let classes = crate::matrix::POSITIVE_SEMANTIC;
    // The guide's own §18.1 order, paired with the facts that witness
    // each class. The maximum row uses four inputs because the fixture
    // bundle's demonstration shape bounds admit one to four.
    let rows: &[(&[u64], SponsorCase)] = &[
        (&[120, 180], SponsorCase::Absent),
        (&[100, 200, 300], SponsorCase::Absent),
        (&[1, 2, 3, 4], SponsorCase::Absent),
        (&[0xffff_ffff, 1], SponsorCase::Absent),
        (&[(1 << 51) - 2, 1], SponsorCase::Absent),
        (&[300, 200, 100], SponsorCase::Absent),
        (&[7, 11], SponsorCase::Absent),
        (&[7, 11], SponsorCase::sponsored(1)),
        (&[13, 17], SponsorCase::sponsored(1)),
        (&[13, 17, 19], SponsorCase::sponsored(2)),
        // The two rows §18.1 distinguishes, now distinguished by a
        // fact: the first returns a residual to its sponsor and the
        // second does not. They were the same value under two names
        // until `G13-R10`.
        (&[23, 29], SponsorCase::sponsored_with_change(1)),
        (&[31, 37], SponsorCase::sponsored(1)),
        (&[41, 43], SponsorCase::Absent),
        (&[47, 53], SponsorCase::Absent),
    ];

    if rows.len() != classes.len() {
        return Err(VectorError::MatrixCoverageMismatch {
            class: "18.1 positive semantic census",
        });
    }

    let mut census = Vec::with_capacity(rows.len());
    for (ordinal, (class, (amounts, sponsor))) in classes.iter().zip(rows.iter()).enumerate() {
        let id = SemanticFixtureId::new(class.name(), u32::try_from(ordinal).unwrap_or(u32::MAX));
        census.push(semantic_case(id, *class, amounts, *sponsor)?);
    }
    Ok(census)
}

/// Whether a §18 class can be answered by a positive semantic fixture.
#[must_use]
pub fn is_positive_semantic(class: VectorClass) -> bool {
    class.polarity() == VectorPolarity::Positive
}

#[cfg(test)]
mod tests {
    use super::{
        OPERATION, SemanticFixtureId, SponsorCase, positive_semantic_census, semantic_case,
    };
    use crate::error::VectorError;
    use crate::matrix::POSITIVE_SEMANTIC;
    use crate::projection::SponsorChange;
    use realization::ProtocolAmount;
    use std::collections::BTreeSet;

    #[test]
    fn the_positive_census_answers_every_named_class_exactly_once() {
        let census = positive_semantic_census().expect("the positive census builds");
        assert_eq!(census.len(), POSITIVE_SEMANTIC.len());
        assert_eq!(census.len(), 14);

        let answered: BTreeSet<&str> = census.iter().map(|case| case.class().name()).collect();
        let named: BTreeSet<&str> = POSITIVE_SEMANTIC
            .iter()
            .map(super::VectorClass::name)
            .collect();
        assert_eq!(answered, named);
    }

    #[test]
    fn the_successor_is_the_realization_layers_own_sum() {
        // Recomputed here by a different route than the fixture used, so
        // the assertion is a comparison rather than a restatement.
        let census = positive_semantic_census().expect("the positive census builds");
        for case in &census {
            let by_hand: u64 = case.inputs().iter().map(|amount| amount.get()).sum();
            assert_eq!(
                case.expected().successor().1.get(),
                by_hand,
                "{:?}",
                case.id()
            );
            assert_eq!(case.expected().aggregate().get(), by_hand);
            assert_eq!(case.expected().explicit_u().get(), by_hand);
        }
    }

    #[test]
    fn a_fixture_carries_no_target_vocabulary() {
        // §17.2's exclusion list, checked against the rendering rather
        // than trusted to the field list, because a later field would
        // pass a field-list review and fail this.
        let census = positive_semantic_census().expect("the positive census builds");
        let rendered = format!("{census:?}").to_lowercase();
        for forbidden in [
            "tapleaf",
            "controlblock",
            "control_block",
            "witness",
            "outpoint",
            "txid",
            "scriptpubkey",
            "program",
            "wallet",
            "input_index",
            "output_index",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "a semantic fixture rendered the target term {forbidden}"
            );
        }
    }

    #[test]
    fn the_maximum_row_reaches_the_domain_bound_without_crossing_it() {
        let census = positive_semantic_census().expect("the positive census builds");
        let boundary = census
            .iter()
            .find(|case| case.class().name() == "values-summing-to-two-pow-51-minus-one")
            .expect("the boundary row is present");
        assert_eq!(
            boundary.expected().successor().1,
            ProtocolAmount::new((1 << 51) - 1).expect("the bound minus one is in the domain")
        );
    }

    #[test]
    fn a_one_member_world_is_not_a_compact_ash_world() {
        let id = SemanticFixtureId::new("one-member", 0);
        let refused = semantic_case(id, POSITIVE_SEMANTIC[0], &[1], SponsorCase::Absent);
        assert_eq!(refused, Err(VectorError::InvalidSemanticFixture(id)));
    }

    #[test]
    fn a_sum_leaving_the_domain_is_refused_by_the_realization_layer() {
        let id = SemanticFixtureId::new("overflowing", 0);
        let refused = semantic_case(
            id,
            POSITIVE_SEMANTIC[0],
            &[(1 << 51) - 1, 1],
            SponsorCase::Absent,
        );
        assert!(
            matches!(refused, Err(VectorError::ExpectationNotDerivable { .. })),
            "the sum must be refused by the layer that owns the domain"
        );
    }

    #[test]
    fn every_fixture_claims_the_one_operation_identity() {
        let census = positive_semantic_census().expect("the positive census builds");
        for case in &census {
            assert_eq!(case.expected().operation(), OPERATION);
            assert_eq!(
                case.expected().sponsor().is_present(),
                case.sponsor().is_present()
            );
        }
    }

    /// The fixture named by a §18.1 class.
    fn named(name: &str) -> super::CompactAshSemanticCase {
        positive_semantic_census()
            .expect("the positive census builds")
            .into_iter()
            .find(|case| case.class().name() == name)
            .unwrap_or_else(|| panic!("the census answers {name}"))
    }

    /// `G13-R10`: the two sponsor-change classes are told apart by facts.
    ///
    /// The census names `sponsor-change-present` and
    /// `sponsor-change-absent` as distinct classes, but [`SponsorCase`]
    /// records only whether a sponsor region exists and how many members
    /// it has. Both rows are therefore `Present(1)`, and what separates
    /// them is the class name and their amounts — neither of which is a
    /// sponsor-change fact.
    ///
    /// A row that carries a class name without carrying the property the
    /// name asserts cannot witness that property, so a run of either row
    /// would be equally good evidence for both.
    #[test]
    fn the_sponsor_change_classes_are_distinguished_by_a_sponsor_change_fact() {
        let present = named("sponsor-change-present");
        let absent = named("sponsor-change-absent");

        // Both are sponsored at all: the distinction under test is about
        // change, not about the presence of a sponsor region.
        assert!(present.sponsor().is_present());
        assert!(absent.sponsor().is_present());

        assert_ne!(
            present.sponsor(),
            absent.sponsor(),
            "the semantic fixture records nothing that tells the two sponsor-change classes apart",
        );
    }

    /// `G13-R10`: the accepted projection retains sponsor-change presence.
    ///
    /// [`SponsorRegion`](crate::projection::SponsorRegion) carries
    /// presence and a member count and nothing else, so the projection a
    /// run is compared against cannot detect that a transaction named
    /// `sponsor-change-present` carried no sponsor change. Both rows
    /// project the same region.
    #[test]
    fn the_accepted_projection_retains_sponsor_change_presence() {
        let present = named("sponsor-change-present");
        let absent = named("sponsor-change-absent");

        assert_ne!(
            present.expected().sponsor(),
            absent.expected().sponsor(),
            "the accepted projection of the two classes is identical, so a matching projection is no evidence of which class ran",
        );
    }

    /// `G13-R10`: the change dimension is the *only* thing separating
    /// the two rows.
    ///
    /// Stated so that a later edit cannot make the two tests above pass
    /// by moving some other fact instead. If the rows differed in their
    /// member count, or in their sponsor presence, they would be told
    /// apart by something that is not what their names assert.
    #[test]
    fn the_two_sponsor_change_rows_differ_in_the_change_role_and_nowhere_else() {
        let present = named("sponsor-change-present");
        let absent = named("sponsor-change-absent");

        assert!(present.sponsor().is_present());
        assert!(absent.sponsor().is_present());
        assert_eq!(present.sponsor().members(), absent.sponsor().members());
        assert_eq!(present.sponsor().change(), SponsorChange::Present);
        assert_eq!(absent.sponsor().change(), SponsorChange::Absent);
    }

    /// `G13-R10`: a row filed under a class it does not witness is
    /// refused where it is built.
    ///
    /// The swap, both ways. Nothing about either world is invalid — both
    /// are model-valid sponsored compact-ASH worlds — and that is the
    /// point: the refusal is about the row's name not matching its
    /// facts, which is a different defect from an unbuildable world and
    /// is reported as one.
    #[test]
    fn swapping_the_two_sponsor_change_fixtures_refuses_the_case() {
        for (name, wrong) in [
            ("sponsor-change-present", SponsorCase::sponsored(1)),
            (
                "sponsor-change-absent",
                SponsorCase::sponsored_with_change(1),
            ),
        ] {
            let class = POSITIVE_SEMANTIC
                .iter()
                .find(|class| class.name() == name)
                .copied()
                .unwrap_or_else(|| panic!("§18.1 names {name}"));
            let id = SemanticFixtureId::new(name, 0);
            assert_eq!(
                semantic_case(id, class, &[23, 29], wrong),
                Err(VectorError::FixtureContradictsItsClass {
                    fixture: id,
                    class: name,
                }),
                "{name} accepted a world that does not witness it",
            );
        }
    }

    /// A sponsorless row filed under a sponsored class is refused too.
    ///
    /// The check is written name by name rather than over the shared
    /// prefix, and this is what would catch a rule that had been written
    /// over the prefix instead: `sponsorless-consensus-transaction` and
    /// `sponsored-transaction` begin alike and assert opposites.
    #[test]
    fn a_sponsorship_claim_a_row_does_not_carry_is_refused() {
        let class = POSITIVE_SEMANTIC
            .iter()
            .find(|class| class.name() == "sponsored-transaction")
            .copied()
            .expect("§18.1 names sponsored-transaction");
        let id = SemanticFixtureId::new("sponsored-transaction", 0);
        assert!(matches!(
            semantic_case(id, class, &[7, 11], SponsorCase::Absent),
            Err(VectorError::FixtureContradictsItsClass { .. })
        ));

        let sponsorless = POSITIVE_SEMANTIC
            .iter()
            .find(|class| class.name() == "sponsorless-consensus-transaction")
            .copied()
            .expect("§18.1 names sponsorless-consensus-transaction");
        assert!(
            semantic_case(
                SemanticFixtureId::new("sponsorless-consensus-transaction", 0),
                sponsorless,
                &[7, 11],
                SponsorCase::Absent,
            )
            .is_ok(),
            "the sponsorless row must still be admitted by a rule about sponsored ones",
        );
    }
}
