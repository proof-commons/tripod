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
    CarrierQuantification, CarrierRole, CollateralPolicy, CoverageBoundary, CoverageRequirementId,
    EvidenceRole, PlacedCarrier, RelationMutation, SponsorCase, TargetCoverageObligation,
    ValidatedTargetOperationPlan,
};
use realization::{RelationId, RelationKind, RelationSubject, RepresentationMode, TransactionSide};
use transaction::{TargetInput, TargetTransaction, ValueField};

use crate::comparison::ProjectionTerm;
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

/// One target-transaction field a mutation moves.
///
/// The vocabulary is deliberately structural rather than positional: an
/// arm says *what kind of thing* it changes, and the check reads the
/// accepted transaction and the mutated one and works out which of these
/// actually moved. A field named by index would have to be restated
/// every time the ABI rearranged a layout.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum TargetField {
    /// The transaction version.
    Version,
    /// Some input's sequence, at its own position.
    InputSequence,
    /// The order of the inputs, as a permutation of the same outpoints.
    InputOrder,
    /// Some input's witness stack.
    WitnessStack,
    /// How many outputs there are.
    OutputCensus,
    /// The multiset of explicit output amounts.
    OutputAmount,
    /// The set of output witness programs.
    OutputProgram,
}

impl TargetField {
    /// Every field this vocabulary distinguishes.
    pub const ALL: &'static [Self] = &[
        Self::Version,
        Self::InputSequence,
        Self::InputOrder,
        Self::WitnessStack,
        Self::OutputCensus,
        Self::OutputAmount,
        Self::OutputProgram,
    ];

    /// Whether this field differs between an accepted transaction and a
    /// mutated one.
    #[must_use]
    pub fn moved(self, before: &TargetTransaction, after: &TargetTransaction) -> bool {
        match self {
            Self::Version => before.version() != after.version(),
            Self::InputSequence => before
                .inputs()
                .iter()
                .zip(after.inputs())
                .any(|(left, right)| left.sequence() != right.sequence()),
            Self::InputOrder => {
                let left: Vec<_> = before.inputs().iter().map(TargetInput::outpoint).collect();
                let right: Vec<_> = after.inputs().iter().map(TargetInput::outpoint).collect();
                let ordered: BTreeSet<_> = left.iter().copied().collect();
                left != right && ordered == right.iter().copied().collect()
            }
            Self::WitnessStack => before.witnesses() != after.witnesses(),
            Self::OutputCensus => before.outputs().len() != after.outputs().len(),
            Self::OutputAmount => explicit_amounts(before) != explicit_amounts(after),
            Self::OutputProgram => output_programs(before) != output_programs(after),
        }
    }
}

/// Every explicit output amount, as a sorted multiset.
fn explicit_amounts(transaction: &TargetTransaction) -> Vec<u64> {
    let mut amounts: Vec<u64> = transaction
        .outputs()
        .iter()
        .filter_map(|output| match output.value() {
            ValueField::Explicit(amount) => Some(amount),
            _ => None,
        })
        .collect();
    amounts.sort_unstable();
    amounts
}

/// Every distinct output witness program.
fn output_programs(transaction: &TargetTransaction) -> BTreeSet<Vec<u8>> {
    transaction
        .outputs()
        .iter()
        .map(|output| output.program().to_vec())
        .collect()
}

/// What one change does to the §17.4 semantic projection.
///
/// Not a set on its own, because a transaction that no longer reads as
/// this operation at all has not disagreed about any term — it has
/// stopped being comparable, which is a different fact from every term
/// still matching.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticChange {
    /// The mutated transaction still projects, and exactly these terms
    /// differ from the fixture's own expectation.
    Terms(BTreeSet<ProjectionTerm>),
    /// The mutated transaction no longer reads as this operation.
    Unreadable,
}

/// What one arm needs of the semantic fixture it starts from.
///
/// The shape conditions [`crate::mutation::apply`] refuses on, stated as
/// data rather than discovered by trying: an arm that needs two inputs
/// to reverse says so, and a plan can then say in advance which vectors
/// it has a subject for.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceFixtureRequirement {
    minimum_ash_inputs: u8,
    minimum_witness_items: u8,
    needs_explicit_successor: bool,
}

impl SourceFixtureRequirement {
    /// How many ASH inputs the subject must carry.
    #[must_use]
    pub const fn minimum_ash_inputs(self) -> u8 {
        self.minimum_ash_inputs
    }

    /// How many items the first witness stack must hold.
    #[must_use]
    pub const fn minimum_witness_items(self) -> u8 {
        self.minimum_witness_items
    }

    /// Whether the successor's amount must be an explicit one.
    #[must_use]
    pub const fn needs_explicit_successor(self) -> bool {
        self.needs_explicit_successor
    }

    /// Whether one decoded accepted transaction meets these conditions.
    #[must_use]
    pub fn admits(self, transaction: &TargetTransaction, successor: Option<usize>) -> bool {
        if transaction.inputs().len() < usize::from(self.minimum_ash_inputs) {
            return false;
        }
        let items = transaction
            .witnesses()
            .first()
            .map_or(0, |witness| witness.stack().len());
        if items < usize::from(self.minimum_witness_items) {
            return false;
        }
        if !self.needs_explicit_successor {
            return true;
        }
        successor.is_some_and(|index| {
            matches!(
                transaction.outputs()[index].value(),
                ValueField::Explicit(_)
            )
        })
    }
}

/// Everything §4.1 makes one canonical negative vector state.
///
/// Eight declarations, each of which the plan can contradict. The point
/// is that none of them is a name: the relation and the mutation class
/// are resolved against the published requirement, the carrier and the
/// collateral policy and the representation are compared against what
/// that requirement carries, and the changed fields are compared against
/// what the mutation actually did to the bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NegativeVectorDeclaration {
    arm: NegativeMutation,
    source: SourceFixtureRequirement,
    violation: IntendedViolation,
    target_fields: BTreeSet<TargetField>,
    semantic: SemanticChange,
    boundary: crate::matrix::EvidenceBoundary,
    carrier: PlacedCarrier,
    collateral: CollateralPolicy,
    representation: RepresentationMode,
    sponsor: SponsorCase,
}

impl NegativeVectorDeclaration {
    /// The arm this declaration belongs to.
    #[must_use]
    pub const fn arm(&self) -> NegativeMutation {
        self.arm
    }

    /// What the arm needs of its source semantic fixture.
    #[must_use]
    pub const fn source(&self) -> SourceFixtureRequirement {
        self.source
    }

    /// The relation and semantic mutation class, or the reason there is
    /// none.
    #[must_use]
    pub const fn violation(&self) -> &IntendedViolation {
        &self.violation
    }

    /// The exact target fields the change moves.
    #[must_use]
    pub const fn target_fields(&self) -> &BTreeSet<TargetField> {
        &self.target_fields
    }

    /// What the change does to the semantic projection.
    #[must_use]
    pub const fn semantic(&self) -> &SemanticChange {
        &self.semantic
    }

    /// The §1.5 boundary the refusal is expected at.
    #[must_use]
    pub const fn boundary(&self) -> crate::matrix::EvidenceBoundary {
        self.boundary
    }

    /// The carrier that must have executed for the refusal to be about
    /// this relation.
    #[must_use]
    pub const fn carrier(&self) -> &PlacedCarrier {
        &self.carrier
    }

    /// The dependency collateral the requirement is expected to demand.
    #[must_use]
    pub const fn collateral(&self) -> CollateralPolicy {
        self.collateral
    }

    /// The representation the case fixes for the ASH object.
    #[must_use]
    pub const fn representation(&self) -> RepresentationMode {
        self.representation
    }

    /// The sponsor case this declaration is made in.
    #[must_use]
    pub const fn sponsor(&self) -> SponsorCase {
        self.sponsor
    }
}

/// The covenant's own coordinator, which is the carrier every runtime
/// compact-ASH relation executes at.
const fn covenant_carrier() -> PlacedCarrier {
    PlacedCarrier {
        carrier: CarrierRole::OperationGlobal {
            operation: OperationId::CompactAsh,
            anchor: ObjectId::Ash,
        },
        quantification: CarrierQuantification::Single,
    }
}

impl NegativeMutation {
    /// This arm's complete §4.1 declaration, in one sponsor case.
    ///
    /// # Errors
    ///
    /// Whatever [`Self::expected_boundary`] refuses, which is the matrix
    /// and this module having drifted apart about a class name.
    pub fn declaration(
        self,
        sponsor: SponsorCase,
    ) -> Result<NegativeVectorDeclaration, VectorError> {
        use ProjectionTerm as T;
        use TargetField as F;

        let terms =
            |terms: &[ProjectionTerm]| SemanticChange::Terms(terms.iter().copied().collect());
        let (source, target_fields, semantic) = match self {
            // A second output at the successor's own program leaves the
            // reading with two candidates and no rule to pick between
            // them, which is the cardinality violation seen from the
            // projection's side rather than a separate fact.
            Self::SplitSuccessorInTwo => (
                source_requirement(1, 0, true),
                [F::OutputCensus, F::OutputAmount].as_slice(),
                SemanticChange::Unreadable,
            ),
            Self::ReverseAshInputOrder => (
                source_requirement(2, 0, false),
                [F::InputOrder, F::WitnessStack].as_slice(),
                terms(&[]),
            ),
            // The successor still reads, at a program no constructor in
            // this bundle emits: the object is no longer recognized as
            // the covenant's own, and the movement it records is no
            // longer the canonical one. Both terms move, which is the
            // measured form of the guide's own ambiguity about this
            // class.
            Self::RedirectSuccessorProgram => (
                source_requirement(1, 0, false),
                [F::OutputProgram].as_slice(),
                terms(&[T::Ownership, T::Flow]),
            ),
            Self::RouteUnitIntoUndeclaredOutput => (
                source_requirement(1, 0, true),
                [F::OutputCensus, F::OutputAmount, F::OutputProgram].as_slice(),
                SemanticChange::Unreadable,
            ),
            Self::ChangeInputSequence => (
                source_requirement(1, 0, false),
                [F::InputSequence].as_slice(),
                terms(&[]),
            ),
            Self::ChangeTransactionVersion => (
                source_requirement(1, 0, false),
                [F::Version].as_slice(),
                terms(&[]),
            ),
            Self::ReorderWitnessItems => (
                source_requirement(1, 2, false),
                [F::WitnessStack].as_slice(),
                terms(&[]),
            ),
            // The successor reads, one unit short. Three terms move
            // together because all three are the same amount seen from
            // three places, and declaring only the aggregate would have
            // been an under-statement the recomputation catches.
            Self::SuccessorOneBelowTheSum => (
                source_requirement(1, 0, true),
                [F::OutputAmount].as_slice(),
                terms(&[T::Successor, T::ExplicitU, T::Aggregate]),
            ),
        };

        Ok(NegativeVectorDeclaration {
            arm: self,
            source,
            violation: self.intended_violation(),
            target_fields: target_fields.iter().copied().collect(),
            semantic,
            boundary: self.expected_boundary()?,
            carrier: covenant_carrier(),
            // Every runtime negative requirement this plan publishes
            // demands the intended relation *and* its typed dependency
            // closure be reported blocked. The closure itself is not
            // restated here: it travels with the requirement, and
            // copying it would be a third source of truth.
            collateral: CollateralPolicy::RequireIntendedAndDependencyClosure,
            // Phase 4 selects `Explicit` for the ASH object and the
            // execution case carries that choice; a declaration stating
            // another mode would resolve to no requirement at all.
            representation: RepresentationMode::Explicit,
            sponsor,
        })
    }
}

/// One source-fixture requirement, spelled once.
const fn source_requirement(
    minimum_ash_inputs: u8,
    minimum_witness_items: u8,
    needs_explicit_successor: bool,
) -> SourceFixtureRequirement {
    SourceFixtureRequirement {
        minimum_ash_inputs,
        minimum_witness_items,
        needs_explicit_successor,
    }
}

/// Which requirement one complete declaration names.
///
/// Two answers and no third. A declaration either resolves to exactly
/// one published requirement whose every stated expectation agreed, or
/// it names none for a reason it states; a declaration that resolved to
/// something the plan describes differently is neither, and refuses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclarationLink {
    /// Exactly one requirement matched and agreed with the declaration.
    Resolved(CoverageRequirementId),
    /// The declaration names no requirement, for a stated reason.
    Blocked(UnlinkedReason),
}

/// Which of a declaration's statements the published requirement
/// contradicts.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ContradictedExpectation {
    /// The requirement is answered by an artifact other than a target
    /// execution.
    EvidenceRole,
    /// The requirement's case fixes another representation for the ASH
    /// object.
    Representation,
    /// No carrier alternative the requirement admits contains the
    /// declared carrier.
    Carrier,
    /// The requirement demands another dependency-collateral policy.
    Collateral,
}

/// Resolve one complete declaration against the published plan.
///
/// The relation and the mutation class select the requirement; the rest
/// of the declaration is then compared against what that requirement
/// carries, so a link established by two fields cannot survive the other
/// six disagreeing.
///
/// # Errors
///
/// [`VectorError::NegativeLinkUnresolved`] when the plan publishes no
/// matching requirement or more than one, and
/// [`VectorError::NegativeLinkContradicted`] when the one it publishes
/// describes something the declaration does not.
pub fn resolve_declaration(
    plan: &ValidatedTargetOperationPlan,
    declaration: &NegativeVectorDeclaration,
) -> Result<DeclarationLink, VectorError> {
    let IntendedViolation::Declared { class_name, .. } = declaration.violation() else {
        let IntendedViolation::Unlinked(reason) = declaration.violation() else {
            unreachable!("an intended violation is declared or unlinked")
        };
        return Ok(DeclarationLink::Blocked(*reason));
    };

    let Some(id) = matching_requirement(plan, declaration.violation(), declaration.sponsor)? else {
        return Err(VectorError::NegativeLinkUnresolved { class: class_name });
    };
    let requirement = plan
        .coverage_requirement(&id)
        .ok_or(VectorError::NegativeLinkUnresolved { class: class_name })?;
    let TargetCoverageObligation::Negative(negative) = &requirement.obligation else {
        return Err(VectorError::NegativeLinkUnresolved { class: class_name });
    };

    let refuse = |expectation| {
        Err(VectorError::NegativeLinkContradicted {
            class: class_name,
            expectation,
        })
    };
    if requirement.role != EvidenceRole::TargetExecution {
        return refuse(ContradictedExpectation::EvidenceRole);
    }
    if id.case.representations.get(&ObjectId::Ash) != Some(&declaration.representation) {
        return refuse(ContradictedExpectation::Representation);
    }
    if !requirement
        .carrier
        .iter()
        .any(|alternative| alternative.carriers.contains(&declaration.carrier))
    {
        return refuse(ContradictedExpectation::Carrier);
    }
    if negative.collateral.policy != declaration.collateral {
        return refuse(ContradictedExpectation::Collateral);
    }
    Ok(DeclarationLink::Resolved(id))
}

#[cfg(test)]
mod tests {
    use super::{
        BTreeSet, IntendedViolation, RelationMutation, UnlinkedReason, matching_requirement,
    };
    use crate::bundle::fixture_bundle;
    use crate::error::VectorError;
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

    /// The multi-input subject, its coins, and its accepted bytes.
    ///
    /// The placeholder coins are the ones the canonical plan uses, so
    /// what is measured below is a property of the mutation over the
    /// canonical subject rather than of a chain nobody ran.
    fn subject() -> (
        crate::bundle::FixtureBundle,
        crate::fixture::CompactAshSemanticCase,
        crate::materialize::MaterializedTargetVector,
        std::collections::BTreeMap<transaction::Outpoint, u64>,
    ) {
        use crate::materialize::{AshFunding, is_materializable, materialize, vector_id};

        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let census = crate::fixture::positive_semantic_census().expect("the census builds");
        let case = census
            .iter()
            .filter(|case| is_materializable(case))
            .find(|case| vector_id(case).ash_inputs() >= 2)
            .expect("a multi-input row exists")
            .clone();
        let funding = AshFunding::unexecutable_placeholder(vector_id(&case));
        let coins = funding
            .outpoints()
            .iter()
            .copied()
            .zip(case.inputs().iter().map(|amount| amount.get()))
            .collect();
        let vector = materialize(&bundle, &case, &funding).expect("the subject materializes");
        (bundle, case, vector, coins)
    }

    #[test]
    fn every_arm_declares_all_eight_facts_in_both_cases() {
        // §4.1's list, checked as a list: an arm that gained a ninth
        // fact or lost one has to change this, and the two facts the
        // declaration does not invent — the boundary and the violation —
        // must be the ones the other two authorities already state.
        use compiler::operation_plan::CollateralPolicy;

        for &arm in NegativeMutation::ALL {
            for case in [SponsorCase::Absent, SponsorCase::Present] {
                let declaration = arm.declaration(case).expect("the arm declares");
                assert_eq!(declaration.arm(), arm);
                assert_eq!(declaration.sponsor(), case);
                assert_eq!(
                    declaration.boundary(),
                    arm.expected_boundary().expect("the class is named"),
                    "{arm:?} declared a boundary its §18 class does not",
                );
                assert_eq!(declaration.violation(), &arm.intended_violation());
                assert!(
                    !declaration.target_fields().is_empty(),
                    "{arm:?} claims to change nothing about the transaction",
                );
                assert_eq!(
                    declaration.collateral(),
                    CollateralPolicy::RequireIntendedAndDependencyClosure,
                );
            }
        }
    }

    #[test]
    fn each_declaration_names_exactly_the_target_fields_its_change_moves() {
        // The declaration made falsifiable. Every field of the
        // vocabulary is asked of the accepted transaction and the
        // mutated one, and the answer must be the declared set exactly —
        // so an arm that under-declares fails here just as loudly as one
        // that over-declares.
        use transaction::TargetTransaction;

        let (bundle, _, vector, _) = subject();
        let before = TargetTransaction::decode(vector.bytes()).expect("the subject decodes");
        for &arm in NegativeMutation::ALL {
            let declaration = arm.declaration(SponsorCase::Absent).expect("declares");
            let mutated =
                crate::mutation::apply(&vector, bundle.closed_asset(), arm).expect("applies");
            let after = TargetTransaction::decode(mutated.bytes()).expect("the mutation decodes");
            let observed: BTreeSet<super::TargetField> = super::TargetField::ALL
                .iter()
                .copied()
                .filter(|field| field.moved(&before, &after))
                .collect();
            assert_eq!(
                &observed,
                declaration.target_fields(),
                "{arm:?} moved another set of fields than it declared",
            );
        }
    }

    #[test]
    fn each_declaration_names_the_semantic_change_the_comparison_reads() {
        // The other half of "exact changed semantic and target fields",
        // recomputed through the same §17.4 comparison a validated
        // report performs. A mutated transaction that no longer reads as
        // this operation is recorded as such rather than as a run of
        // terms that all happened to match.
        use super::SemanticChange;
        use crate::comparison::{compare, read_accepted};
        use crate::fixture::OPERATION;

        let (bundle, case, vector, coins) = subject();
        let program = bundle
            .pin()
            .output_script(bundle.target())
            .expect("the pinned program derives");
        for &arm in NegativeMutation::ALL {
            let declaration = arm.declaration(SponsorCase::Absent).expect("declares");
            let mutated =
                crate::mutation::apply(&vector, bundle.closed_asset(), arm).expect("applies");
            let observed = read_accepted(
                mutated.bytes(),
                &coins,
                bundle.closed_asset(),
                bundle.reserve_asset(),
                &program,
                OPERATION,
            )
            .map_or(SemanticChange::Unreadable, |projection| {
                SemanticChange::Terms(compare(case.expected(), &projection).into_iter().collect())
            });
            assert_eq!(
                &observed,
                declaration.semantic(),
                "{arm:?} moved another semantic change than it declared",
            );
        }
    }

    #[test]
    fn the_source_requirement_predicts_which_subjects_an_arm_has() {
        // The declaration's first item, made checkable: an arm states
        // what it needs of its source fixture, and the statement must
        // agree with what the mutation can actually be applied to over
        // every vector the plan materialized. A requirement nothing
        // consulted would be prose.
        use transaction::TargetTransaction;

        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = crate::plan::derive_evidence_plan(&bundle).expect("the plan derives");
        let asset = bundle.closed_asset();
        let mut checked = 0_usize;
        for subject in plan.target_cases() {
            let vector = subject.subject();
            let decoded = TargetTransaction::decode(vector.bytes()).expect("a vector decodes");
            let successor = crate::mutation::outputs_carrying(&decoded, asset)
                .first()
                .copied();
            for &arm in NegativeMutation::ALL {
                let declaration = arm.declaration(SponsorCase::Absent).expect("declares");
                let admitted = declaration.source().admits(&decoded, successor);
                let applied = crate::mutation::apply(vector, asset, arm).is_ok();
                assert_eq!(
                    admitted,
                    applied,
                    "{arm:?} disagreed with its own source requirement on {:?}",
                    vector.id(),
                );
                checked += 1;
            }
        }
        assert!(checked > 0, "the plan materialized nothing to check");
    }

    #[test]
    fn exactly_two_declarations_resolve_and_the_rest_state_a_reason() {
        // The whole §4.1 resolution, over both cases. The honest number
        // is recomputed rather than asserted from prose, and a blocked
        // arm carries the reason its own declaration gave rather than a
        // silence a reader could mistake for merely unrun.
        use super::{DeclarationLink, resolve_declaration};

        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let mut resolved = 0_usize;
        let mut blocked = 0_usize;
        for &arm in NegativeMutation::ALL {
            for case in [SponsorCase::Absent, SponsorCase::Present] {
                let declaration = arm.declaration(case).expect("declares");
                match resolve_declaration(plan, &declaration).expect("the declaration resolves") {
                    DeclarationLink::Resolved(id) => {
                        resolved += 1;
                        assert_eq!(id.case.sponsor, case, "{arm:?} resolved into another case");
                    }
                    DeclarationLink::Blocked(reason) => {
                        blocked += 1;
                        assert_eq!(
                            super::IntendedViolation::Unlinked(reason),
                            arm.intended_violation(),
                            "{arm:?} was blocked for a reason it did not declare",
                        );
                    }
                }
            }
        }
        assert_eq!(resolved, 4, "two arms resolve, in two cases each");
        assert_eq!(blocked, 12, "six arms state a reason, in two cases each");
    }

    #[test]
    fn a_declaration_the_published_requirement_contradicts_is_refused() {
        // The reason the other six statements are compared at all: a
        // link established by the relation and the class alone would
        // survive the plan describing a different carrier, a different
        // representation, or a different collateral policy. Each of
        // those is staged here against a requirement that really exists.
        use super::{ContradictedExpectation, NegativeVectorDeclaration, resolve_declaration};
        use compiler::operation_plan::{
            CarrierQuantification, CarrierRole, CollateralPolicy, PlacedCarrier,
        };
        use realization::RepresentationMode;

        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let honest = NegativeMutation::SplitSuccessorInTwo
            .declaration(SponsorCase::Absent)
            .expect("declares");

        let staged: [(NegativeVectorDeclaration, ContradictedExpectation); 3] = [
            (
                NegativeVectorDeclaration {
                    representation: RepresentationMode::PrivateCommitted,
                    ..honest.clone()
                },
                ContradictedExpectation::Representation,
            ),
            (
                NegativeVectorDeclaration {
                    carrier: PlacedCarrier {
                        carrier: CarrierRole::EveryInputFamilyMember {
                            object: architecture::ObjectId::Ash,
                        },
                        quantification: CarrierQuantification::PerMember,
                    },
                    ..honest.clone()
                },
                ContradictedExpectation::Carrier,
            ),
            (
                NegativeVectorDeclaration {
                    collateral: CollateralPolicy::ReportAdditional,
                    ..honest
                },
                ContradictedExpectation::Collateral,
            ),
        ];

        for (declaration, expectation) in staged {
            assert_eq!(
                resolve_declaration(plan, &declaration),
                Err(VectorError::NegativeLinkContradicted {
                    class: "CardinalityAboveMaximum",
                    expectation,
                }),
                "a contradicted declaration was admitted",
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
