//! What discharges a negative requirement no target can answer.
//!
//! Eighteen of the plan's seventy-two negative requirements are answered
//! by first-party code refusing a condition, because their boundary is
//! the compiler's own analysis or the emitted structure and no
//! transaction exists at either. §19.2's conditions — a complete mutated
//! target transaction, an executed carrier, an observed target rejection
//! — cannot be met by any of them, and none had ever been discharged.
//!
//! # The policy, and why it needs six things rather than one
//!
//! §4.2 states the condition explicitly, and every clause of it removes
//! a way of appearing to have evidence. A canonical malformed input, so
//! the subject is a focused change of something the workspace really
//! publishes rather than an invention. The *exact owning* validator, so
//! a refusal from a neighbouring layer cannot stand in — which is the
//! trap the archaeology found, where every test asserting a compiler
//! error asserted the realization twin instead. A typed refusal naming
//! the intended class, so the requirement's own semantic mutation is
//! what came back and not merely some failure. A focused positive
//! control, so the refusal is attributable to the malformation and not
//! to the input being unacceptable all along. And a validated report
//! whose constructor is private, so a caller cannot state the conclusion
//! without passing through the two runs that establish it.
//!
//! # The same argument the negative half already makes
//!
//! This is [`crate::mutation`]'s un-mutated sibling, moved to a boundary
//! that has no bytes. There, a refusal means something because the
//! accepted transaction and the mutated one differ by exactly one thing.
//! Here, a compiler refusal means something because the published
//! dependency and the malformed one differ by exactly one thing, and the
//! published one is accepted by the same validator in the same call.

use std::collections::BTreeSet;

use architecture::{ARCHITECTURE, OperationId};
use compiler::operation_plan::{
    CoverageRequirementId, EvidenceRole, RelationMutation, TargetCoverageObligation,
    ValidatedTargetOperationPlan,
};
use compiler::{CompileError, validate_required_dependency};
use realization::{AvailabilityClass, ConstructibilityNodeId, RealizationScope, derive};

/// Which first-party validator owns one negative requirement.
///
/// A name for the exact entry point a discharge drove, carried into the
/// evidence so a reader of a coverage row can see *what* refused. There
/// is deliberately no arm for "the compiler", because the compiler
/// refuses many things and a row answered by one of them is not answered
/// by the others.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum FirstPartyValidator {
    /// The compiler's check that an operation's authorization case
    /// discharges the availability class of a required dependency.
    ///
    /// `compiler::validate_required_dependency`, which is the sole site
    /// constructing either constructibility refusal and is the same call
    /// the scoped analysis makes for every required ancestor.
    CompilerRequiredDependency,
}

/// One first-party refusal, as a coverage row records it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FirstPartyRefusal {
    validator: FirstPartyValidator,
    class: RelationMutation,
}

impl FirstPartyRefusal {
    /// Which validator refused.
    #[must_use]
    pub const fn validator(self) -> FirstPartyValidator {
        self.validator
    }

    /// The semantic mutation class its refusal named.
    #[must_use]
    pub const fn class(self) -> RelationMutation {
        self.class
    }
}

/// One canonical malformed typed input, and its control.
///
/// The malformation is a single field, and the control is the same value
/// without it. Stating the pair as one value rather than two is what
/// keeps a caller from offering a control that is not the malformed
/// input's sibling.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FirstPartyNegativeCase {
    /// A required dependency whose availability class was changed from
    /// the one the realization publishes to a private one.
    ///
    /// The published node is the control. The malformed input is that
    /// node with one field replaced, and the authorization offered to
    /// both is the operation's own — looked up rather than authored, so
    /// the case cannot be made easier by choosing a weaker one.
    PrivateRequiredDependency {
        /// The operation whose authorization case is offered.
        operation: OperationId,
        /// The node exactly as the realization publishes it.
        published: ConstructibilityNodeId,
        /// The one changed field.
        availability: AvailabilityClass,
    },
}

/// Why one offered case does not discharge its requirement.
///
/// Every arm is a way the offered evidence falls short of §4.2, and none
/// of them is a defect in the validator: a validator that refused the
/// control has said something true about the control, and what fails is
/// the claim that the refusal was about the malformation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FirstPartyEvidenceRefusal {
    /// The plan publishes no such requirement.
    RequirementNotPublished,
    /// The requirement is not one first-party code answers.
    ///
    /// A runtime or external row reaching here would be answered by a
    /// target or by a report, and a first-party refusal is not either.
    NotAFirstPartyRequirement,
    /// The realization the malformed input is checked against could not
    /// be derived.
    RealizationUnavailable,
    /// The control is not a node the realization publishes.
    ///
    /// Then the pair is two inventions rather than a published value and
    /// a focused change of it, and a refusal of the second says nothing
    /// about the workspace.
    ControlIsNotPublished,
    /// The malformation left the input unchanged.
    ///
    /// The whole argument is that the two inputs differ by exactly one
    /// thing; two identical inputs differ by nothing, and the validator
    /// would have to answer them the same way.
    MalformationChangedNothing,
    /// The validator accepted the malformed input.
    MalformedInputWasAccepted,
    /// The validator refused, naming a class other than the
    /// requirement's.
    RefusalNamesAnotherClass(RelationMutation),
    /// The validator refused, naming no semantic mutation class at all.
    ///
    /// The guard rather than an observed state: the validator raises two
    /// errors and both name a class. It is here because the mapping
    /// returns an option, and a validator that grew a third error would
    /// otherwise have its refusal filed under whichever class the match
    /// reached first.
    RefusalNamesNoClass,
    /// The validator refused the control too.
    ///
    /// Then the refusal is not attributable to the malformation: the
    /// input was unacceptable before it was malformed.
    ControlWasRefused,
}

/// One first-party negative requirement, discharged.
///
/// # What holding one of these establishes
///
/// That the exact validator the requirement's boundary belongs to was
/// run twice — once on a canonical malformed input and once on the
/// published value that input is a single change of — that it refused
/// the first naming the requirement's own semantic mutation class, and
/// that it accepted the second. All of that is recomputed by
/// [`validate_first_party_negative`], which is the only constructor;
/// every field here is a conclusion of that run and none of them is a
/// value a caller offered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedFirstPartyNegativeEvidence {
    requirement: CoverageRequirementId,
    refusal: FirstPartyRefusal,
    control: ConstructibilityNodeId,
    malformed: ConstructibilityNodeId,
}

impl ValidatedFirstPartyNegativeEvidence {
    /// The requirement this evidence answers.
    #[must_use]
    pub const fn requirement(&self) -> &CoverageRequirementId {
        &self.requirement
    }

    /// What refused, and what class it named.
    #[must_use]
    pub const fn refusal(&self) -> FirstPartyRefusal {
        self.refusal
    }

    /// The published value the validator accepted.
    #[must_use]
    pub const fn control(&self) -> &ConstructibilityNodeId {
        &self.control
    }

    /// The malformed value it refused.
    #[must_use]
    pub const fn malformed(&self) -> &ConstructibilityNodeId {
        &self.malformed
    }
}

/// Discharge one first-party negative requirement, per §4.2.
///
/// # Errors
///
/// [`FirstPartyEvidenceRefusal`], naming which of §4.2's conditions the
/// offered case does not meet.
pub fn validate_first_party_negative(
    plan: &ValidatedTargetOperationPlan,
    requirement: &CoverageRequirementId,
    case: &FirstPartyNegativeCase,
) -> Result<ValidatedFirstPartyNegativeEvidence, FirstPartyEvidenceRefusal> {
    let published = plan
        .coverage_requirement(requirement)
        .ok_or(FirstPartyEvidenceRefusal::RequirementNotPublished)?;
    let TargetCoverageObligation::Negative(negative) = &published.obligation else {
        return Err(FirstPartyEvidenceRefusal::NotAFirstPartyRequirement);
    };
    if !matches!(
        published.role,
        EvidenceRole::CompilerAnalysisResult | EvidenceRole::EmittedStructure
    ) {
        return Err(FirstPartyEvidenceRefusal::NotAFirstPartyRequirement);
    }

    let FirstPartyNegativeCase::PrivateRequiredDependency {
        operation,
        published: control,
        availability,
    } = case;

    // The control has to be a value the workspace really publishes, or
    // the pair is two inventions rather than a published value and one
    // focused change of it.
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots())
        .map_err(|_| FirstPartyEvidenceRefusal::RealizationUnavailable)?;
    let nodes: BTreeSet<ConstructibilityNodeId> = realization
        .project()
        .constructibility
        .nodes
        .into_iter()
        .map(|node| node.id)
        .collect();
    if !nodes.contains(control) {
        return Err(FirstPartyEvidenceRefusal::ControlIsNotPublished);
    }

    let malformed = with_availability(control, *availability)
        .ok_or(FirstPartyEvidenceRefusal::ControlIsNotPublished)?;
    if malformed == *control {
        return Err(FirstPartyEvidenceRefusal::MalformationChangedNothing);
    }

    // The authorization is the operation's own, looked up rather than
    // authored: a case offering an authorization the operation does not
    // have would be refusing something the analysis never asks.
    let authorization = realization
        .constructibility_authorizations(*operation)
        .map_err(|_| FirstPartyEvidenceRefusal::RealizationUnavailable)?
        .first()
        .ok_or(FirstPartyEvidenceRefusal::RealizationUnavailable)?
        .clone();

    // The negative half. Nothing below reads a list: the class comes
    // back out of the validator's own error. The dependency walk is
    // empty because this case states a dependency rather than building a
    // graph to reach it through; the walk is diagnostic detail on the
    // error and no part of the class it names.
    let Err(error) = validate_required_dependency(*operation, &authorization, &malformed, Vec::new)
    else {
        return Err(FirstPartyEvidenceRefusal::MalformedInputWasAccepted);
    };
    let class = refused_class(&error).ok_or(FirstPartyEvidenceRefusal::RefusalNamesNoClass)?;
    if class != negative.mutation {
        return Err(FirstPartyEvidenceRefusal::RefusalNamesAnotherClass(class));
    }

    // The positive control, through the same call with the same
    // authorization. A validator that refused this one refused the
    // input rather than the malformation.
    if validate_required_dependency(*operation, &authorization, control, Vec::new).is_err() {
        return Err(FirstPartyEvidenceRefusal::ControlWasRefused);
    }

    Ok(ValidatedFirstPartyNegativeEvidence {
        requirement: requirement.clone(),
        refusal: FirstPartyRefusal {
            validator: FirstPartyValidator::CompilerRequiredDependency,
            class,
        },
        control: control.clone(),
        malformed,
    })
}

/// The same node under another availability class.
///
/// `None` for an operation node, which carries no availability at all
/// and therefore has no such field to change.
fn with_availability(
    node: &ConstructibilityNodeId,
    availability: AvailabilityClass,
) -> Option<ConstructibilityNodeId> {
    match node {
        ConstructibilityNodeId::Operation(_) => None,
        ConstructibilityNodeId::Fact {
            operation, fact, ..
        } => Some(ConstructibilityNodeId::Fact {
            operation: *operation,
            fact: fact.clone(),
            availability,
        }),
        ConstructibilityNodeId::Witness {
            operation, role, ..
        } => Some(ConstructibilityNodeId::Witness {
            operation: *operation,
            role: *role,
            availability,
        }),
    }
}

/// The semantic mutation class one compiler refusal names.
///
/// `None` where the refusal names none, which is every other compiler
/// error: an error outside this validator's two is a failure of
/// something else and answers no coverage requirement here.
const fn refused_class(error: &CompileError) -> Option<RelationMutation> {
    match error {
        CompileError::PermissionlessPrivateDependency { .. } => {
            Some(RelationMutation::PermissionlessPrivateDependency)
        }
        CompileError::ConstructibilityWitnessUnavailable { .. } => {
            Some(RelationMutation::ConstructibilityWitnessUnavailable)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FirstPartyEvidenceRefusal, FirstPartyNegativeCase, FirstPartyValidator,
        validate_first_party_negative,
    };
    use crate::bundle::fixture_bundle;
    use architecture::{ObjectId, OperationId};
    use compiler::operation_plan::{
        CoverageRequirementId, EvidenceRole, RelationMutation, SponsorCase,
        TargetCoverageObligation,
    };
    use realization::{AvailabilityClass, ConstructibilityNodeId};

    /// The first-party requirement of one class, in one sponsor case.
    fn requirement(
        plan: &compiler::operation_plan::ValidatedTargetOperationPlan,
        wanted: RelationMutation,
        sponsor: SponsorCase,
    ) -> CoverageRequirementId {
        plan.coverage()
            .find(|candidate| {
                let TargetCoverageObligation::Negative(negative) = &candidate.obligation else {
                    return false;
                };
                candidate.role == EvidenceRole::CompilerAnalysisResult
                    && candidate.id.case.sponsor == sponsor
                    && negative.mutation == wanted
            })
            .expect("the class is published as a compiler-static negative")
            .id
            .clone()
    }

    /// The compact-ASH input-family count, exactly as the realization
    /// publishes it: a required dependency of this operation whose
    /// availability is public, which is why the real analysis passes.
    fn published_public_fact() -> ConstructibilityNodeId {
        let realization = realization::derive(
            &architecture::ARCHITECTURE,
            realization::RealizationScope::phase1_pilots(),
        )
        .expect("the realization derives");
        realization
            .project()
            .constructibility
            .nodes
            .into_iter()
            .map(|node| node.id)
            .find(|id| {
                matches!(
                    id,
                    ConstructibilityNodeId::Fact {
                        operation: OperationId::CompactAsh,
                        availability: AvailabilityClass::Public,
                        ..
                    }
                )
            })
            .expect("compact-ash publishes a public required fact")
    }

    /// The canonical malformed input: that fact under a private class.
    fn private_dependency() -> FirstPartyNegativeCase {
        FirstPartyNegativeCase::PrivateRequiredDependency {
            operation: OperationId::CompactAsh,
            published: published_public_fact(),
            availability: AvailabilityClass::InputOwners {
                object: ObjectId::Ash,
            },
        }
    }

    #[test]
    fn the_permissionless_private_dependency_row_is_discharged_end_to_end() {
        // The proving instance. Compact-ASH's own authorization case is
        // the permissionless one, and every required dependency it
        // publishes is public — which is exactly why the real analysis
        // passes and why one changed availability class is the whole
        // malformation.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        for sponsor in [SponsorCase::Absent, SponsorCase::Present] {
            let id = requirement(
                plan,
                RelationMutation::PermissionlessPrivateDependency,
                sponsor,
            );
            let evidence = validate_first_party_negative(plan, &id, &private_dependency())
                .expect("the case discharges the row");
            assert_eq!(evidence.requirement(), &id);
            assert_eq!(
                evidence.refusal().validator(),
                FirstPartyValidator::CompilerRequiredDependency,
            );
            assert_eq!(
                evidence.refusal().class(),
                RelationMutation::PermissionlessPrivateDependency,
            );
            // And the pair really is a pair: one field apart, and the
            // control is the published value.
            assert_ne!(evidence.control(), evidence.malformed());
            assert_eq!(evidence.control(), &published_public_fact());
        }
    }

    #[test]
    fn the_same_case_does_not_discharge_the_neighbouring_class() {
        // The sibling refusal `ConstructibilityWitnessUnavailable` is
        // raised by the same validator, and this case cannot reach it:
        // compact-ASH publishes only a permissionless authorization,
        // and that is the arm the error's own construction selects on.
        // The row therefore stays outstanding rather than being filled
        // by whichever refusal was nearest.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let id = requirement(
            plan,
            RelationMutation::ConstructibilityWitnessUnavailable,
            SponsorCase::Absent,
        );
        assert_eq!(
            validate_first_party_negative(plan, &id, &private_dependency()),
            Err(FirstPartyEvidenceRefusal::RefusalNamesAnotherClass(
                RelationMutation::PermissionlessPrivateDependency
            )),
        );
    }

    #[test]
    fn a_control_the_workspace_does_not_publish_is_refused() {
        // Without this, the pair could be two inventions and the second
        // one's refusal would say nothing about the workspace.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let id = requirement(
            plan,
            RelationMutation::PermissionlessPrivateDependency,
            SponsorCase::Absent,
        );
        let case = FirstPartyNegativeCase::PrivateRequiredDependency {
            operation: OperationId::CompactAsh,
            published: ConstructibilityNodeId::Witness {
                operation: OperationId::CompactAsh,
                role: realization::WitnessRole::OperatorAuthorization,
                availability: AvailabilityClass::Public,
            },
            availability: AvailabilityClass::Operator,
        };
        assert_eq!(
            validate_first_party_negative(plan, &id, &case),
            Err(FirstPartyEvidenceRefusal::ControlIsNotPublished),
        );
    }

    #[test]
    fn a_malformation_that_changes_nothing_is_refused() {
        // Two identical inputs differ by nothing, so the validator would
        // answer them the same way and the control would establish the
        // opposite of what it is for.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let id = requirement(
            plan,
            RelationMutation::PermissionlessPrivateDependency,
            SponsorCase::Absent,
        );
        let case = FirstPartyNegativeCase::PrivateRequiredDependency {
            operation: OperationId::CompactAsh,
            published: published_public_fact(),
            availability: AvailabilityClass::Public,
        };
        assert_eq!(
            validate_first_party_negative(plan, &id, &case),
            Err(FirstPartyEvidenceRefusal::MalformationChangedNothing),
        );
    }

    #[test]
    fn a_malformation_the_validator_accepts_discharges_nothing() {
        // A sponsor-local availability is discharged by every
        // authorization case, so the "malformed" input is not malformed
        // and the validator says so. Without this arm a case could
        // change a field, be accepted, and still be offered as evidence
        // of a refusal.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let id = requirement(
            plan,
            RelationMutation::PermissionlessPrivateDependency,
            SponsorCase::Absent,
        );
        let case = FirstPartyNegativeCase::PrivateRequiredDependency {
            operation: OperationId::CompactAsh,
            published: published_public_fact(),
            availability: AvailabilityClass::SponsorLocal,
        };
        assert_eq!(
            validate_first_party_negative(plan, &id, &case),
            Err(FirstPartyEvidenceRefusal::MalformedInputWasAccepted),
        );
    }

    #[test]
    fn a_control_the_validator_also_refuses_discharges_nothing() {
        // The attributability guard, staged against a node the
        // realization really publishes: the live-transfer owner witness
        // is private to that operation's owners, so compact-ASH's
        // permissionless case refuses it before anything is changed. A
        // refusal of the malformed sibling then establishes nothing the
        // refusal of the control did not already.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let id = requirement(
            plan,
            RelationMutation::PermissionlessPrivateDependency,
            SponsorCase::Absent,
        );
        let case = FirstPartyNegativeCase::PrivateRequiredDependency {
            operation: OperationId::CompactAsh,
            published: ConstructibilityNodeId::Witness {
                operation: OperationId::TransferLive,
                role: realization::WitnessRole::ProtocolOwnerAuthorization,
                availability: AvailabilityClass::InputOwners {
                    object: ObjectId::ReceiptLive,
                },
            },
            availability: AvailabilityClass::InputOwners {
                object: ObjectId::Ash,
            },
        };
        assert_eq!(
            validate_first_party_negative(plan, &id, &case),
            Err(FirstPartyEvidenceRefusal::ControlWasRefused),
        );
    }

    #[test]
    fn a_runtime_requirement_is_not_answerable_here() {
        // §4.2's policy is about the first-party half only. A row a
        // target must answer stays a target's, whatever a compiler
        // refuses.
        let bundle = fixture_bundle().expect("the fixture bundle builds");
        let plan = bundle.plan();
        let runtime = plan
            .coverage()
            .find(|candidate| {
                matches!(candidate.obligation, TargetCoverageObligation::Negative(_))
                    && candidate.role == EvidenceRole::TargetExecution
            })
            .expect("the plan publishes a runtime negative row")
            .id
            .clone();
        assert_eq!(
            validate_first_party_negative(plan, &runtime, &private_dependency()),
            Err(FirstPartyEvidenceRefusal::NotAFirstPartyRequirement),
        );
    }
}
