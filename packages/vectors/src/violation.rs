//! Which relation one §18 mutation class intends to violate.
//!
//! §19.2 requires a negative case to name its intended violated
//! relation before it is run, alongside the exact changed fields and the
//! collateral closure. That declaration is the missing half of the
//! negative census: the compiler publishes, for every negative
//! requirement, the semantic mutation class it demands, and §18 names
//! classes of concrete change — but nothing yet says which §18 class
//! belongs to which requirement.
//!
//! # The link is resolved, never tabulated
//!
//! An arm declares only what §19.2 makes it declare: the relation it
//! intends to violate, and the semantic mutation class its change falls
//! in. The requirement itself is then *looked up* in the published plan,
//! and the lookup must find exactly one. A table mapping arms to
//! requirement identities would be a third source of truth that could
//! agree with neither side; a resolution that must hit exactly one row
//! fails loudly the moment the plan moves underneath it.
//!
//! # Most arms link to nothing, and say so
//!
//! §18's tables are lists of names. No row there names a relation, a
//! semantic mutation class, or a boundary, so for most arms the guide
//! does not determine a requirement at all. Those arms carry a typed
//! reason instead of a guess: an arm whose change no semantic class
//! describes, an arm several classes fit equally, and an arm refused
//! before any target sees it are three different situations, and none of
//! them is coverage.

use std::collections::BTreeSet;

use architecture::{AssetId, ObjectId, OperationId};
use compiler::operation_plan::{
    CoverageBoundary, CoverageRequirementId, RelationMutation, SponsorCase,
    TargetCoverageObligation, ValidatedTargetOperationPlan,
};
use realization::{RelationId, RelationKind, RelationSubject, TransactionSide};

use crate::error::VectorError;
use crate::mutation::NegativeMutation;

/// Why one §18 class names no relation-indexed requirement.
///
/// Three distinct situations, kept apart because they call for three
/// different repairs and only one of them is this package's to make.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum UnlinkedReason {
    /// The class expects its refusal before any target sees the bytes.
    ///
    /// Every negative requirement the plan publishes sits at one of four
    /// discharge boundaries, and the constructor's is not among them, so
    /// there is no row for such a class to answer. Reaching one would
    /// need the ABI-validation entry point §16.5 names as its own report
    /// role, which does not exist.
    BoundaryPrecedesTarget,
    /// The compiler's negative vocabulary describes no such change.
    ///
    /// §18 asks for the class and the relation inventory has nothing to
    /// index it by. That is a gap between two authorities rather than a
    /// defect in either, and it is reported rather than closed by
    /// filing the class under whichever mutation looked nearest.
    NoSemanticMutationClass,
    /// Several semantic mutation classes fit the guide's wording.
    ///
    /// §18 names the class and nothing narrows it to one requirement, so
    /// picking one would be the discharge-by-intent the census exists to
    /// prevent.
    SemanticClassUnderdetermined,
}

/// What one mutation arm intends to violate, as §19.2 requires.
///
/// The mutation class travels as a predicate rather than a value on
/// purpose. An above-maximum mutation carries the ceiling it must
/// exceed, and that ceiling is the architecture's to state; asking which
/// class a requirement is in leaves the bound where it belongs, where
/// comparing whole values would have copied it here.
#[derive(Clone, Debug)]
pub enum IntendedViolation {
    /// The arm names one relation and one semantic mutation class.
    Declared {
        /// The relation the change is intended to violate.
        relation: RelationId,
        /// Whether one published mutation is the class this arm stages.
        class: fn(&RelationMutation) -> bool,
        /// That class's name, for reports and for the census.
        class_name: &'static str,
    },
    /// The arm names no requirement, for a stated reason.
    Unlinked(UnlinkedReason),
}

impl PartialEq for IntendedViolation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Declared {
                    relation: left,
                    class_name: left_name,
                    ..
                },
                Self::Declared {
                    relation: right,
                    class_name: right_name,
                    ..
                },
            ) => left == right && left_name == right_name,
            (Self::Unlinked(left), Self::Unlinked(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for IntendedViolation {}

/// The compact-ASH relation of one kind and subject.
const fn relation(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::CompactAsh, kind, subject)
}

impl NegativeMutation {
    /// The relation and semantic mutation class this arm intends to violate.
    ///
    /// # Where each answer comes from
    ///
    /// Two arms are determined by their own §18 class name read in the
    /// relation vocabulary, and the resolution against the plan is what
    /// checks that reading. The rest are not determined by anything the
    /// guide says, and carry the reason instead.
    #[must_use]
    pub fn intended_violation(self) -> IntendedViolation {
        match self {
            // Determined by the table the class comes from and the words
            // in its name, in that order. §18.2 is the cardinality table,
            // which fixes the relation kind; "two ASH outputs" names the
            // side and the object family, which fixes the subject; and
            // the plan declares that family's maximum to be one, which
            // leaves exactly one above-maximum mutation for two of them
            // to exceed. The ceiling itself stays the plan's to state.
            //
            // A second ASH output is arguably also an unexpected
            // canonical delta family, and that reading is what the table
            // rules out: a class §18 files under cardinality is a
            // cardinality case, and this does not get to choose again.
            Self::SplitSuccessorInTwo => IntendedViolation::Declared {
                relation: relation(
                    RelationKind::Cardinality,
                    RelationSubject::ObjectFamily {
                        side: TransactionSide::Output,
                        object: ObjectId::Ash,
                    },
                ),
                class: |mutation| {
                    matches!(mutation, RelationMutation::CardinalityAboveMaximum { .. })
                },
                class_name: "CardinalityAboveMaximum",
            },
            // Determined by the mutation class rather than by the table,
            // because §18.4 collects output mutations of several relation
            // kinds and fixes none. "One below the sum" leaves the closed
            // asset short of what the inputs carry, which is what an
            // amount mismatch on that asset is, and exactly one relation
            // in the whole plan publishes that class.
            //
            // The target never reaches it. The arm does not preserve
            // value balance and Elements checks per-asset conservation
            // before running a script, so the refusal arrives before the
            // carrier executes and §19.2's carrier condition fails. The
            // link is still stated: what is missing is a way to reach
            // this relation with the script running, not a relation.
            Self::SuccessorOneBelowTheSum => IntendedViolation::Declared {
                relation: relation(
                    RelationKind::Conservation,
                    RelationSubject::Asset { asset: AssetId::U },
                ),
                class: |mutation| matches!(mutation, RelationMutation::AmountMismatch),
                class_name: "AmountMismatch",
            },
            // Neither change has a member in the negative mutation
            // vocabulary. Canonical input ordering is asked for by §18.3
            // and the relation inventory indexes nothing by ordering; a
            // permuted witness stack is refused by the script, but the
            // only witness-shaped class published is the
            // compiler-static constructibility one, which no target run
            // can answer.
            Self::ReverseAshInputOrder | Self::ReorderWitnessItems => {
                IntendedViolation::Unlinked(UnlinkedReason::NoSemanticMutationClass)
            }
            // Both changes fit two published classes equally well and
            // §18 names neither. Paying the successor to another program
            // is as much a wrongly recognized output object as an
            // undeclared output family; growing an undeclared output
            // while the totals hold is as much an unexpected canonical
            // delta family as an undeclared open flow.
            Self::RedirectSuccessorProgram | Self::RouteUnitIntoUndeclaredOutput => {
                IntendedViolation::Unlinked(UnlinkedReason::SemanticClassUnderdetermined)
            }
            // Both were respecified to the constructor's boundary once
            // the target accepted them, and no requirement is indexed
            // there.
            Self::ChangeInputSequence | Self::ChangeTransactionVersion => {
                IntendedViolation::Unlinked(UnlinkedReason::BoundaryPrecedesTarget)
            }
        }
    }
}

/// Resolve the one requirement an intended violation names.
///
/// `Ok(None)` for an arm that names no requirement — that is an answer,
/// not a failure. For a declared violation the plan must publish exactly
/// one matching negative requirement in the stated case.
///
/// # Errors
///
/// [`VectorError::NegativeLinkUnresolved`] when the plan publishes no
/// matching requirement or more than one, either of which means the
/// declaration and the plan disagree about what exists.
pub fn matching_requirement(
    plan: &ValidatedTargetOperationPlan,
    violation: &IntendedViolation,
    case: SponsorCase,
) -> Result<Option<CoverageRequirementId>, VectorError> {
    let IntendedViolation::Declared {
        relation,
        class,
        class_name,
    } = violation
    else {
        return Ok(None);
    };

    let mut found: BTreeSet<CoverageRequirementId> = BTreeSet::new();
    for requirement in plan.coverage() {
        let TargetCoverageObligation::Negative(negative) = &requirement.obligation else {
            continue;
        };
        // The runtime boundary is the only one a submitted transaction
        // can answer; a compiler-static or backend-structural row is
        // reached by first-party code and never by this.
        if requirement.id.boundary != CoverageBoundary::RuntimeCarrier
            || requirement.id.relation != *relation
            || requirement.id.case.sponsor != case
            || !class(&negative.mutation)
        {
            continue;
        }
        found.insert(requirement.id.clone());
    }

    let mut resolved = found.into_iter();
    let (Some(only), None) = (resolved.next(), resolved.next()) else {
        return Err(VectorError::NegativeLinkUnresolved { class: class_name });
    };
    Ok(Some(only))
}

#[cfg(test)]
mod tests {
    use super::{IntendedViolation, RelationMutation, UnlinkedReason, matching_requirement};
    use crate::bundle::fixture_bundle;
    use crate::mutation::NegativeMutation;
    use compiler::operation_plan::SponsorCase;

    #[test]
    fn every_arm_states_a_violation_or_a_reason() {
        // The point of the enum: no arm is silent about whether it can
        // answer a requirement, so a new arm has to say which it is
        // before it compiles.
        for &arm in NegativeMutation::ALL {
            match arm.intended_violation() {
                IntendedViolation::Declared { class_name, .. } => {
                    assert!(!class_name.is_empty(), "{arm:?} declared an empty class");
                }
                IntendedViolation::Unlinked(_) => {}
            }
        }
    }

    #[test]
    fn exactly_two_arms_resolve_and_each_hits_one_row() {
        // The honest number, recomputed rather than asserted from
        // prose. Each declared arm must hit exactly one published
        // requirement per case, which is what makes the declaration
        // falsifiable instead of decorative.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let mut declared = 0_usize;
        for &arm in NegativeMutation::ALL {
            let violation = arm.intended_violation();
            if matches!(violation, IntendedViolation::Unlinked(_)) {
                continue;
            }
            declared += 1;
            for case in [SponsorCase::Absent, SponsorCase::Present] {
                let resolved = matching_requirement(plan, &violation, case)
                    .expect("a declared violation resolves")
                    .expect("a declared violation names a requirement");
                assert_eq!(
                    resolved.relation,
                    match &violation {
                        IntendedViolation::Declared { relation, .. } => relation.clone(),
                        IntendedViolation::Unlinked(_) => unreachable!(),
                    },
                    "{arm:?} resolved to another relation",
                );
            }
        }
        assert_eq!(declared, 2, "the number of arms the guide determines");
    }

    #[test]
    fn the_two_pre_target_arms_name_the_missing_entry_point() {
        // Their boundary moved to the constructor's in earlier waves,
        // and no requirement is indexed there. The reason is recorded so
        // the rows are not read as merely unrun.
        for arm in [
            NegativeMutation::ChangeInputSequence,
            NegativeMutation::ChangeTransactionVersion,
        ] {
            assert_eq!(
                arm.intended_violation(),
                IntendedViolation::Unlinked(UnlinkedReason::BoundaryPrecedesTarget),
                "{arm:?} should name the boundary that precedes the target",
            );
        }
    }

    #[test]
    fn every_first_party_requirement_has_a_stated_evidence_standing() {
        // All 18 of them, recomputed from the plan rather than counted
        // from prose, and every one classified. A class reaching this
        // without an answer would be a new first-party boundary, which
        // has to be looked at rather than absorbed.
        use compiler::operation_plan::{EvidenceRole, TargetCoverageObligation};

        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let mut counted = 0_usize;
        for requirement in bundle.plan().coverage() {
            let TargetCoverageObligation::Negative(negative) = &requirement.obligation else {
                continue;
            };
            let emitted = match requirement.role {
                EvidenceRole::EmittedStructure => true,
                EvidenceRole::CompilerAnalysisResult => false,
                _ => continue,
            };
            counted += 1;
            assert!(
                super::first_party_evidence(&negative.mutation, emitted).is_some(),
                "{:?} has no stated first-party evidence standing",
                negative.mutation,
            );
        }
        assert_eq!(counted, 18, "the first-party half of the negative census");
    }

    #[test]
    fn each_first_party_class_states_the_standing_the_archaeology_found() {
        // Pinned per class, so that any of them gaining or losing a
        // refusal has to be recorded here on purpose. There is
        // deliberately no variant meaning "refused and tested at this
        // boundary": nothing in the repository is, and a variant nobody
        // could return would invite one to be claimed.
        use super::FirstPartyEvidence as E;

        let expected = [
            (
                RelationMutation::ConstructibilityWitnessUnavailable,
                false,
                E::RefusalReachedButUntested,
            ),
            (
                RelationMutation::PermissionlessPrivateDependency,
                false,
                E::RefusalReachedButUntested,
            ),
            (
                RelationMutation::RequiredLifecycleExitMissing,
                false,
                E::RefusedOnlyAtAnotherLayer,
            ),
            (
                RelationMutation::RequiredLifecycleExitMissing,
                true,
                E::NoTypedRefusal,
            ),
            (
                RelationMutation::UnsupportedRepresentation,
                false,
                E::MadeUnrepresentable,
            ),
            (
                RelationMutation::UnauthenticatedRepresentation,
                true,
                E::NoTypedRefusal,
            ),
            (
                RelationMutation::UnexpectedProtocolSecret,
                true,
                E::NoTypedRefusal,
            ),
        ];
        for (mutation, emitted, standing) in expected {
            assert_eq!(
                super::first_party_evidence(&mutation, emitted),
                Some(standing),
                "{mutation:?} at emitted={emitted} carries another standing",
            );
        }
    }

    #[test]
    fn an_unlinked_arm_resolves_to_no_requirement() {
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let violation = NegativeMutation::ReverseAshInputOrder.intended_violation();
        let resolved = matching_requirement(plan, &violation, SponsorCase::Absent)
            .expect("an unlinked arm resolves without error");
        assert!(
            resolved.is_none(),
            "an unlinked arm must name no requirement",
        );
    }
}

/// What first-party evidence exists for one negative requirement.
///
/// The 18 requirements whose evidence role is the compiler's own
/// analysis or the emitted structure are answered, if at all, by
/// first-party code refusing a condition rather than by a target
/// refusing a transaction. This says what such a refusal actually looks
/// like today, per semantic mutation class.
///
/// # None of these is a discharge, and the guide is why
///
/// §19.1 says positive coverage of a compiler-static or
/// backend-structural relation uses typed structural evidence instead of
/// inventing target execution. §19.2 states no such rule for the
/// negative half: its conditions are a valid source transaction, a
/// complete mutated target transaction, an executed carrier and an
/// observed target rejection, none of which a compiler-static relation
/// can have. So the guide states no condition under which a first-party
/// refusal discharges a negative requirement, and a boundary being
/// first-party is not the same claim as a first-party test discharging
/// the row. These arms therefore record readiness, never coverage.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum FirstPartyEvidence {
    /// A typed refusal exists and is reached, but no test at this
    /// boundary drives it to the error.
    ///
    /// `CompileError::ConstructibilityWitnessUnavailable` and
    /// `CompileError::PermissionlessPrivateDependency` are both
    /// constructed in the compiler's constructibility stage, and the
    /// stage is live. Every test that asserts either error asserts the
    /// realization twin instead, which is a different layer answering a
    /// different requirement.
    RefusalReachedButUntested,
    /// No error names this condition; the nearest one refuses something
    /// else.
    ///
    /// The compiler refuses a missing lifecycle path and a missing
    /// representation choice, and neither is a required exit going
    /// missing. The realization layer does refuse the exact mutation,
    /// and is tested, but answers its own boundary and not this one.
    RefusedOnlyAtAnotherLayer,
    /// The condition is made unrepresentable rather than refused.
    ///
    /// Representation candidates are built from the relation's own
    /// allowed set, so a selection outside it cannot be constructed to
    /// be refused. Nothing checks the membership the static requirement
    /// documents, because nothing can currently violate it.
    MadeUnrepresentable,
    /// No typed refusal exists anywhere, and no site constructs one.
    ///
    /// The backend-structural half of the lifecycle exit is a declared
    /// no-op at the layout stage; an unauthenticated representation and
    /// an unexpected protocol secret have requirement types and
    /// predicates but no error. The secret-freeness assertions that do
    /// exist are positive properties of the emitted program, not
    /// refusals of an offending input.
    NoTypedRefusal,
}

/// The first-party evidence standing of one semantic mutation class.
///
/// `None` for a class no first-party requirement carries, so a boundary
/// that started emitting one would surface here rather than being
/// folded into whichever answer looked closest.
#[must_use]
pub const fn first_party_evidence(
    mutation: &RelationMutation,
    role_is_emitted_structure: bool,
) -> Option<FirstPartyEvidence> {
    match mutation {
        RelationMutation::ConstructibilityWitnessUnavailable
        | RelationMutation::PermissionlessPrivateDependency => {
            Some(FirstPartyEvidence::RefusalReachedButUntested)
        }
        // The compiler-static half has a neighbouring refusal and the
        // emitted half has nothing at all, so the two boundaries of one
        // class answer differently.
        RelationMutation::RequiredLifecycleExitMissing => {
            if role_is_emitted_structure {
                Some(FirstPartyEvidence::NoTypedRefusal)
            } else {
                Some(FirstPartyEvidence::RefusedOnlyAtAnotherLayer)
            }
        }
        RelationMutation::UnsupportedRepresentation => {
            Some(FirstPartyEvidence::MadeUnrepresentable)
        }
        RelationMutation::UnauthenticatedRepresentation
        | RelationMutation::UnexpectedProtocolSecret => Some(FirstPartyEvidence::NoTypedRefusal),
        _ => None,
    }
}
