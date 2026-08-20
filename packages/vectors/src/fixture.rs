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
    AcceptedProjection, SemanticObjectId, SponsorRegion, TransitionCertificate,
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

/// Whether the fixture's world carries a sponsor region.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SponsorCase {
    /// No sponsor region.
    Absent,
    /// A sponsor region with this many members.
    Present(u16),
}

impl SponsorCase {
    /// The region this case projects.
    #[must_use]
    pub const fn region(self) -> SponsorRegion {
        match self {
            Self::Absent => SponsorRegion::ABSENT,
            Self::Present(members) => SponsorRegion::present(members),
        }
    }

    /// Whether a sponsor region exists.
    #[must_use]
    pub const fn is_present(self) -> bool {
        matches!(self, Self::Present(_))
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
/// requires, or an amount outside the realization layer's domain — and
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
        (&[7, 11], SponsorCase::Present(1)),
        (&[13, 17], SponsorCase::Present(1)),
        (&[13, 17, 19], SponsorCase::Present(2)),
        (&[23, 29], SponsorCase::Present(1)),
        (&[31, 37], SponsorCase::Present(1)),
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
}
