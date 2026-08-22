//! Oracles for the candidate live-receipt constructor (Guide-13 §7).
//!
//! # What is checked against what
//!
//! Nothing here asks the constructor to confirm its own reasoning. The
//! owner-metadata refusals are checked against the *target's* encoding
//! registry, so a test that passed because this module and the registry
//! agreed on a wrong width could not exist. The §7.6 determinism
//! properties are checked by varying one input at a time and comparing
//! whole constructors, so a field that stopped depending on the owner
//! would fail rather than go unnoticed. And the mutation census is not
//! read as a table: every case it says the constructor refuses is
//! *executed*, and the refusal it produces is required to be the one
//! that names the mutation.
//!
//! # The material below is public test material
//!
//! The owner keys are fixed byte patterns: distinguishable, meaningless,
//! and — this being the batch's first secret-adjacent surface — worth
//! being explicit about. They authorize nothing anywhere, they are not
//! derived from anything, and none of them is a key in the sense of
//! having a secret behind it. They are public disposable test data in
//! exactly the sense `(´[ADR015-rule:security:test-material]´)` fixes,
//! and the one that stands in for private key material stands in for it
//! by carrying the scalar *encoding class*, not by being a scalar.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU8;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
use target_elements::{
    CanonicalEncodingRule, EncodingClass, EncodingDomain, PayloadWidth,
    ReviewedElementsTapscriptDefinition,
};

use super::{live_transfer_plan, reviewed_target};
use crate::authorization::{OwnerKeyEncodingClosure, OwnerKeyNegative, owner_key_encoding_closure};
use crate::bundle::{ConstructorAssumption, InternalKeyPolicy, KeyPathPolicy};
use crate::live_constructor::{
    BindingStatus, ConstructorBinding, ConstructorDisposition, ConstructorFacet,
    ConstructorMutationCase, ConstructorMutationCaseId, KeyPathClosure, LiveConstructorRefusal,
    LiveProgramRole, LiveSpendingRoute, LiveTransferLeafRole, MutationResidual, OwnerKey,
    OwnerKeyEstablishment, OwnerKeyRejection, OwnerKeyResidual, PendingBinding,
    StaticLiveReceiptConstructor, constructor_mutation_cases, derive_live_receipt_constructor,
    key_path_closure, mutation_census_defects, static_transfer_leaf_set,
};
use crate::live_shape::{
    LiveTransferShape, LiveTransferShapeBounds, LiveTransferShapeSet, demonstration_live_shape_set,
};
use crate::shape::SponsorChangePresence;

// --- Fixtures ---------------------------------------------------------

/// The reviewed contract's owner-key encoding closure.
fn closure() -> OwnerKeyEncodingClosure {
    owner_key_encoding_closure(reviewed_target().definition().authorization())
}

/// The approved owner-key encoding, read from the reviewed contract.
fn approved() -> EncodingClass {
    closure().approved()
}

/// The approved encoding's exact width, read from the target's registry.
///
/// # Panics
///
/// If the approved owner-key encoding stops fixing one exact width, in
/// which case the fixtures below have nothing to be the right width of
/// and the module's own [`OwnerKeyRejection::UnfixedApprovedWidth`]
/// guard is the thing under test rather than these oracles.
fn approved_width() -> usize {
    match approved().v1_shape().payload() {
        PayloadWidth::Exact(width) => width.get(),
        other => panic!("the approved owner-key encoding fixes no exact width: {other:?}"),
    }
}

/// One owner key of the approved encoding, filled with `fill`.
fn owner(fill: u8) -> OwnerKey {
    OwnerKey::new(&closure(), approved(), vec![fill; approved_width()])
        .expect("the fixture is the approved encoding at its exact width")
}

/// A nonzero count for the shape fixtures.
fn count(value: u8) -> NonZeroU8 {
    NonZeroU8::new(value).expect("the fixture counts are nonzero")
}

/// The reference shape set: the Phase-5 demonstration window.
fn shapes() -> LiveTransferShapeSet {
    demonstration_live_shape_set()
}

/// The reference constructor, and the pieces a mutation replaces.
struct Subject {
    target: ReviewedElementsTapscriptDefinition,
    representation: LiveTransferRepresentationPlan,
    owner: OwnerKey,
    shapes: LiveTransferShapeSet,
    leaves: BTreeSet<LiveTransferLeafRole>,
}

impl Subject {
    /// The reference subject: explicit representation, one owner, the
    /// demonstration shapes, and exactly the leaf set they call for.
    fn new() -> Self {
        let representation = LiveTransferRepresentationPlan::Explicit;
        let shapes = shapes();

        Self {
            target: reviewed_target(),
            representation,
            owner: owner(0x11),
            leaves: static_transfer_leaf_set(representation, &shapes),
            shapes,
        }
    }

    /// Derive the constructor these parts describe.
    fn derive(&self) -> Result<StaticLiveReceiptConstructor, LiveConstructorRefusal> {
        derive_live_receipt_constructor(
            &self.target,
            &live_transfer_plan(),
            self.representation,
            self.owner.clone(),
            self.shapes.clone(),
            self.leaves.clone(),
        )
    }

    /// The constructor, which the reference subject always yields.
    fn constructor(&self) -> StaticLiveReceiptConstructor {
        self.derive().expect("the reference subject derives")
    }
}

// --- §7.2: canonical owner metadata -----------------------------------

#[test]
fn the_approved_encoding_at_its_exact_width_is_the_canonical_metadata() {
    let key = owner(0x11);

    assert_eq!(key.encoding(), approved());
    assert_eq!(key.width(), approved_width());
    assert_eq!(key.bytes(), vec![0x11; approved_width()]);
}

#[test]
fn an_omitted_owner_is_refused_as_absent_rather_than_as_a_width() {
    // §7.2 lists the omitted owner separately from the wrong width, and
    // the target agrees: it represents an absent field as a form of its
    // own rather than as a zero-length payload.
    assert_eq!(
        OwnerKey::new(&closure(), approved(), Vec::new()),
        Err(OwnerKeyRejection::OwnerOmitted)
    );
}

#[test]
fn owner_bytes_of_the_wrong_width_are_refused_on_both_sides_of_the_exact_one() {
    for offered in [approved_width() - 1, approved_width() + 1] {
        assert_eq!(
            OwnerKey::new(&closure(), approved(), vec![0x11; offered]),
            Err(OwnerKeyRejection::WrongWidth {
                offered,
                required: approved_width(),
            })
        );
    }
}

#[test]
fn the_targets_other_public_key_encoding_is_refused_as_an_alternate_encoding() {
    // The target names more than one way to write a public key. §7.2
    // rejects the alternate encoding of the same key, because one owner
    // written two ways is two owners to every byte comparison.
    let alternate = EncodingClass::ALL
        .iter()
        .copied()
        .find(|class| class.v1_shape().domain() == EncodingDomain::Key && *class != approved())
        .expect("the target names a second public-key encoding");

    assert_eq!(
        OwnerKey::new(&closure(), alternate, vec![0x11; approved_width()]),
        Err(OwnerKeyRejection::AlternateEncodingOfApprovedKey {
            offered: alternate,
            approved: approved(),
        })
    );
}

#[test]
fn a_scalar_is_refused_although_nothing_about_its_bytes_differs_from_a_key() {
    // §1.10's line, where it is thinnest. `EcScalar` is the encoding
    // class private key material is written in, and against the approved
    // public encoding it is the same width and the same opaque payload:
    // no byte-level check could separate them. The domain does, and that
    // is the whole reason the domain gate exists.
    let scalar = EncodingClass::EcScalar;

    assert_eq!(scalar.v1_shape().payload(), approved().v1_shape().payload());
    assert_eq!(scalar.interpretation(), approved().interpretation());
    assert_ne!(scalar.v1_shape().domain(), EncodingDomain::Key);

    assert_eq!(
        OwnerKey::new(&closure(), scalar, vec![0x11; approved_width()]),
        Err(OwnerKeyRejection::NotAKeyEncoding {
            offered: scalar,
            domain: scalar.v1_shape().domain(),
        })
    );
}

#[test]
fn the_domain_gate_is_reached_before_the_class_gate() {
    // The ordering is what makes the test above say what it says. A
    // scalar also fails the approved-class comparison, and reporting it
    // that way would leave the §1.10 refusal unreachable while still
    // refusing the offering.
    let scalar_rejection = OwnerKey::new(
        &closure(),
        EncodingClass::EcScalar,
        vec![0x11; approved_width()],
    )
    .expect_err("a scalar is not owner metadata");

    assert!(matches!(
        scalar_rejection,
        OwnerKeyRejection::NotAKeyEncoding { .. }
    ));
}

#[test]
fn the_approved_encoding_is_the_uniquely_canonical_fixed_width_form() {
    // The two target-side guards, checked against the registry rather
    // than against this module: if either stopped holding, §7.2's "one
    // canonical encoding" would no longer exist and the constructor
    // would be refusing offerings for the wrong reason.
    let shape = approved().v1_shape();

    assert_eq!(shape.domain(), EncodingDomain::Key);
    assert_eq!(shape.canonicality(), CanonicalEncodingRule::Unique);
    assert!(matches!(shape.payload(), PayloadWidth::Exact(_)));
}

#[test]
fn every_owner_key_residual_names_where_a_run_discharges_it() {
    // The residual census is the honest half of §7.2: what accepting the
    // encoding does *not* establish. Two of the three are Wave 3's own
    // negatives; the third is the signature check's business and says so
    // by naming none.
    assert_eq!(
        OwnerKeyResidual::CurvePointMembership.negative(),
        Some(OwnerKeyNegative::MalformedApprovedKey)
    );
    assert_eq!(
        OwnerKeyResidual::KeyIsTheIntendedPartys.negative(),
        Some(OwnerKeyNegative::ApprovedKeyOfAnotherOwner)
    );
    assert_eq!(
        OwnerKeyResidual::OwnerHoldsTheCorrespondingSecret.negative(),
        None
    );
}

#[test]
fn the_establishment_and_residual_censuses_are_both_populated() {
    // A statement of what an accepted key means needs both halves. One
    // of them empty would be either a claim with no limits or a limit
    // with no claim.
    assert_eq!(OwnerKeyEstablishment::ALL.len(), 4);
    assert_eq!(OwnerKeyResidual::ALL.len(), 3);
}

// --- §7.1 and §10.3: the static transfer leaf set ---------------------

#[test]
fn the_one_to_one_shape_contributes_no_member_leaf() {
    // §10.3 makes member leaves valid only at nonzero receipt positions,
    // and a one-input transfer has none: input 0 is the coordinator and
    // nothing follows it. A member leaf here would be unspendable weight
    // in the tree.
    let bounds = LiveTransferShapeBounds::new(count(1), count(2), 0);
    let unary = LiveTransferShapeSet::new(
        bounds,
        BTreeSet::from([
            LiveTransferShape::new(bounds, count(1), count(1), 0, SponsorChangePresence::Absent)
                .expect("one-to-one"),
            LiveTransferShape::new(bounds, count(1), count(2), 0, SponsorChangePresence::Absent)
                .expect("one-to-two"),
        ]),
        false,
    )
    .expect("a set of unary-input shapes");

    let leaves = static_transfer_leaf_set(LiveTransferRepresentationPlan::Explicit, &unary);

    assert!(
        leaves
            .iter()
            .all(|leaf| leaf.program_role() == LiveProgramRole::Coordinator)
    );
    assert_eq!(leaves.len(), 2);
}

#[test]
fn every_admitted_shape_has_a_coordinator_and_every_batch_a_member() {
    let set = shapes();
    let leaves = static_transfer_leaf_set(LiveTransferRepresentationPlan::Explicit, &set);

    for shape in set.shapes() {
        assert!(leaves.contains(&LiveTransferLeafRole::Coordinator {
            representation: LiveTransferRepresentationPlan::Explicit,
            shape,
        }));
        assert_eq!(
            shape.receipt_inputs() > 1,
            leaves.contains(&LiveTransferLeafRole::Member {
                representation: LiveTransferRepresentationPlan::Explicit,
                receipt_inputs: shape.receipt_inputs(),
            })
        );
    }
}

#[test]
fn the_two_representations_share_no_leaf() {
    // §11.3's fail-closed default: two representations get two leaf sets
    // until a typed proof says they may share one.
    let set = shapes();
    let explicit = static_transfer_leaf_set(LiveTransferRepresentationPlan::Explicit, &set);
    let private = static_transfer_leaf_set(LiveTransferRepresentationPlan::PrivateCommitted, &set);

    assert_eq!(explicit.len(), private.len());
    assert!(explicit.is_disjoint(&private));
}

// --- §7.5: no key-path escape -----------------------------------------

#[test]
fn an_empty_transfer_leaf_set_is_refused_as_the_key_path_escape() {
    // The enforcement with teeth. A taproot output carrying no script
    // leaf can be spent only through its key path, so the empty leaf set
    // is not a narrow candidate — it is §7.5's accepted escape, arrived
    // at by omission. The refusal names it as that rather than as a
    // bookkeeping complaint about missing leaves.
    let mut subject = Subject::new();
    subject.leaves = BTreeSet::new();

    assert_eq!(
        subject.derive(),
        Err(LiveConstructorRefusal::KeyPathWouldBeTheOnlySpendingRoute)
    );
}

#[test]
fn the_key_path_closure_holds_for_a_constructor_that_exists() {
    let constructor = Subject::new().constructor();
    let leaves = constructor.leaves().count();

    match key_path_closure(&constructor) {
        KeyPathClosure::Closed { script_paths } => assert_eq!(script_paths.get(), leaves),
        KeyPathClosure::OpenForLackOfScriptPath => {
            panic!("a derived constructor has script paths")
        }
    }
    assert!(key_path_closure(&constructor).is_closed());
}

#[test]
fn every_spending_route_is_a_script_path_of_a_leaf_the_constructor_carries() {
    let constructor = Subject::new().constructor();
    let leaves = constructor.leaves().collect::<BTreeSet<_>>();
    let routes = constructor.spending_routes().collect::<Vec<_>>();

    // Exactly as many routes as leaves, and each one reaches a leaf this
    // constructor actually holds: a route to a leaf outside the set
    // would be a spending path the candidate does not claim to support.
    assert_eq!(routes.len(), leaves.len());
    for route in routes {
        let LiveSpendingRoute::ScriptPath { leaf } = route;
        assert!(leaves.contains(&leaf));
    }
}

#[test]
fn the_inherited_policies_are_the_compact_ash_constructors_own() {
    // §7.5 says to use the inherited policy, so the values here are the
    // sibling constructor's types rather than new ones saying the same
    // thing. A second one-variant enum would be one more place for an
    // escape to be admitted.
    let constructor = Subject::new().constructor();

    assert_eq!(
        constructor.internal_key(),
        InternalKeyPolicy::UnspendableWithResidualDiscreteLogAssumption
    );
    assert_eq!(constructor.key_path(), KeyPathPolicy::NoAcceptedEscape);
    assert_eq!(
        *constructor.assumptions(),
        BTreeSet::from([
            ConstructorAssumption::ResidualDiscreteLogOnUnspendableInternalKey,
            ConstructorAssumption::InternalKeyUnspendabilityVerifiableFromPublicData,
        ])
    );
}

// --- §7.1: the leaf set is exactly the one the shapes call for --------

#[test]
fn a_leaf_of_the_other_representation_is_refused() {
    let mut subject = Subject::new();
    let intruder = LiveTransferLeafRole::Member {
        representation: LiveTransferRepresentationPlan::PrivateCommitted,
        receipt_inputs: 2,
    };
    subject.leaves.insert(intruder);

    assert_eq!(
        subject.derive(),
        Err(LiveConstructorRefusal::LeafOfAnotherRepresentation {
            leaf: intruder,
            selected: LiveTransferRepresentationPlan::Explicit,
        })
    );
}

#[test]
fn removing_a_coordinator_leaf_leaves_a_shape_the_candidate_cannot_spend() {
    let mut subject = Subject::new();
    let removed = subject
        .leaves
        .iter()
        .copied()
        .find(|leaf| leaf.program_role() == LiveProgramRole::Coordinator)
        .expect("the reference leaf set has coordinators");
    subject.leaves.remove(&removed);

    assert_eq!(
        subject.derive(),
        Err(LiveConstructorRefusal::LeafSetIncomplete {
            missing: BTreeSet::from([removed]),
        })
    );
}

#[test]
fn removing_a_member_leaf_is_refused_the_same_way() {
    let mut subject = Subject::new();
    let removed = subject
        .leaves
        .iter()
        .copied()
        .find(|leaf| leaf.program_role() == LiveProgramRole::Member)
        .expect("the demonstration window has batches above one");
    subject.leaves.remove(&removed);

    assert_eq!(
        subject.derive(),
        Err(LiveConstructorRefusal::LeafSetIncomplete {
            missing: BTreeSet::from([removed]),
        })
    );
}

#[test]
fn a_leaf_for_an_unadmitted_shape_is_refused_as_an_orphan() {
    let mut subject = Subject::new();
    let wider = LiveTransferShapeBounds::new(count(8), count(8), 1);
    let unadmitted =
        LiveTransferShape::new(wider, count(8), count(8), 0, SponsorChangePresence::Absent)
            .expect("admissible under a wider window");
    let orphan = LiveTransferLeafRole::Coordinator {
        representation: subject.representation,
        shape: unadmitted,
    };
    assert!(!subject.shapes.admits(unadmitted));
    subject.leaves.insert(orphan);

    assert_eq!(
        subject.derive(),
        Err(LiveConstructorRefusal::LeafServesNoAdmittedShape {
            orphans: BTreeSet::from([orphan]),
        })
    );
}

#[test]
fn the_shape_leaves_of_an_admitted_shape_come_back_together() {
    let subject = Subject::new();
    let constructor = subject.constructor();

    for shape in subject.shapes.shapes() {
        let (coordinator, member) = constructor
            .shape_leaves(shape)
            .expect("an admitted shape has a coordinator");

        assert_eq!(coordinator.program_role(), LiveProgramRole::Coordinator);
        assert_eq!(member.is_some(), shape.receipt_inputs() > 1);
    }
}

// --- §7.4: the candidate lifecycle ------------------------------------

#[test]
fn the_lifecycle_is_structurally_incomplete() {
    let constructor = Subject::new().constructor();
    let lifecycle = constructor.lifecycle();

    assert!(!lifecycle.closure().release_complete());
    assert_eq!(
        lifecycle.outstanding().get(),
        lifecycle.closure().outstanding().count()
    );
    // The exits partition: nothing is both done and outstanding.
    let implemented = lifecycle.closure().implemented().collect::<Vec<_>>();
    assert!(!implemented.is_empty());
    assert!(
        !lifecycle
            .closure()
            .outstanding()
            .any(|exit| implemented.contains(&exit))
    );
}

#[test]
fn no_leaf_role_can_name_an_outstanding_exit() {
    // §7.4's "no spendable placeholder for burn, redemption, or
    // normalization" needs no check, because it needs no value: the leaf
    // role census is a coordinator and a member. What is checked is the
    // consequence — every leaf of every constructor is a transfer leaf.
    let constructor = Subject::new().constructor();

    for leaf in constructor.leaves() {
        assert!(matches!(
            leaf.program_role(),
            LiveProgramRole::Coordinator | LiveProgramRole::Member
        ));
    }
}

// --- §7.1: what the constructor binds ---------------------------------

#[test]
fn the_binding_census_is_the_guides_list_with_one_element_outstanding() {
    let outstanding = ConstructorBinding::ALL
        .iter()
        .filter(|binding| binding.status() != BindingStatus::Bound)
        .copied()
        .collect::<Vec<_>>();

    assert_eq!(ConstructorBinding::ALL.len(), 10);
    assert_eq!(outstanding, vec![ConstructorBinding::CandidateAbiSchema]);
    assert_eq!(
        ConstructorBinding::CandidateAbiSchema.status(),
        BindingStatus::Outstanding {
            pending: PendingBinding::CandidateTransactionAbi,
        }
    );
}

// --- §7.6: the constructor derivation ---------------------------------

#[test]
fn the_same_parts_derive_the_same_constructor() {
    assert_eq!(Subject::new().constructor(), Subject::new().constructor());
}

#[test]
fn changing_the_owner_changes_the_constructor_and_nothing_else() {
    // §7.6: changing owner changes the constructor deterministically.
    // Deterministically, and *only* the owner: a constructor whose class
    // or key-path policy moved with the owner would be inferring
    // something from it, which §7.3 forbids.
    let first = Subject::new();
    let mut second = Subject::new();
    second.owner = owner(0x22);

    let (left, right) = (first.constructor(), second.constructor());

    assert_ne!(left, right);
    assert_ne!(left.owner(), right.owner());
    assert_eq!(left.class(), right.class());
    assert_eq!(left.owner_family(), right.owner_family());
    assert_eq!(left.value(), right.value());
    assert_eq!(left.representation(), right.representation());
    assert_eq!(left.contract(), right.contract());
    assert_eq!(left.leaf_version(), right.leaf_version());
    assert_eq!(left.internal_key(), right.internal_key());
    assert_eq!(left.key_path(), right.key_path());
    assert_eq!(left.lifecycle(), right.lifecycle());
    assert_eq!(
        left.leaves().collect::<Vec<_>>(),
        right.leaves().collect::<Vec<_>>()
    );
}

#[test]
fn changing_the_representation_changes_only_the_leaves_it_is_permitted_to() {
    // §7.6: changing representation changes the constructor only where
    // the selected ABI permits it. What it is permitted to change is the
    // program set — §11.3 keeps the two representations' leaves
    // distinct — and nothing else.
    let first = Subject::new();
    let mut second = Subject::new();
    second.representation = LiveTransferRepresentationPlan::PrivateCommitted;
    second.leaves = static_transfer_leaf_set(second.representation, &second.shapes);

    let (left, right) = (first.constructor(), second.constructor());

    assert_ne!(left, right);
    assert_ne!(left.representation(), right.representation());
    assert!(
        left.leaves()
            .collect::<BTreeSet<_>>()
            .is_disjoint(&right.leaves().collect::<BTreeSet<_>>())
    );

    assert_eq!(left.owner(), right.owner());
    assert_eq!(left.class(), right.class());
    assert_eq!(left.owner_family(), right.owner_family());
    assert_eq!(left.contract(), right.contract());
    assert_eq!(left.leaf_version(), right.leaf_version());
    assert_eq!(left.internal_key(), right.internal_key());
    assert_eq!(left.key_path(), right.key_path());
    assert_eq!(left.lifecycle(), right.lifecycle());
    assert_eq!(left.shapes(), right.shapes());
}

#[test]
fn the_class_is_not_inferred_from_owner_representation_or_shape() {
    // §7.3: class is not inferred from target position, value
    // representation, amount, or owner. Position and amount are not
    // constructor inputs at all; the other two are, and varying them
    // leaves the class where the plan put it.
    let reference = Subject::new().constructor();

    let mut owner_changed = Subject::new();
    owner_changed.owner = owner(0x33);

    let mut representation_changed = Subject::new();
    representation_changed.representation = LiveTransferRepresentationPlan::PrivateCommitted;
    representation_changed.leaves = static_transfer_leaf_set(
        representation_changed.representation,
        &representation_changed.shapes,
    );

    let bounds = LiveTransferShapeBounds::new(count(2), count(2), 0);
    let mut shapes_changed = Subject::new();
    shapes_changed.shapes = crate::live_shape::dense_live_shape_set(bounds);
    shapes_changed.leaves =
        static_transfer_leaf_set(shapes_changed.representation, &shapes_changed.shapes);

    for variant in [owner_changed, representation_changed, shapes_changed] {
        let derived = variant.constructor();

        assert_ne!(derived, reference);
        assert_eq!(derived.class(), reference.class());
        assert_eq!(derived.owner_family(), reference.owner_family());
    }
}

#[test]
fn the_owner_encoding_closure_travels_with_the_constructor() {
    // Wave 3's obligation is what §10.2's recognition fragments owe
    // next, so the constructor keeps what its owner metadata was checked
    // against rather than discarding it.
    let constructor = Subject::new().constructor();

    assert_eq!(*constructor.owner_encoding(), closure());
    assert_eq!(constructor.owner().encoding(), closure().approved());
}

// --- The constructor mutation census ----------------------------------

#[test]
fn the_mutation_census_stands_up() {
    assert_eq!(mutation_census_defects(&constructor_mutation_cases()), []);
}

#[test]
fn every_facet_of_the_constructor_is_disturbed_by_some_case() {
    let cases = constructor_mutation_cases();
    let disturbed = cases
        .values()
        .map(ConstructorMutationCase::disturbs)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        disturbed,
        ConstructorFacet::ALL
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
    );
}

#[test]
fn no_case_records_a_verdict_and_every_expressible_one_still_needs_a_run() {
    // §1.11: a target-negative claim requires a complete target
    // transaction and an observed target verdict. Every case that
    // reaches past this constructor says so, and none of them says what
    // the verdict would be — the type has no field one could occupy.
    for case in constructor_mutation_cases().values() {
        match case.disposition() {
            ConstructorDisposition::RefusedByTheConstructor => {
                assert_eq!(case.residuals().count(), 0, "{:?}", case.id());
            }
            ConstructorDisposition::ExpressibleByRawSurgery => {
                assert!(
                    case.residuals()
                        .any(|residual| residual == MutationResidual::TargetNativeRunRequired),
                    "{:?} claims a target outcome with no run",
                    case.id()
                );
                assert!(
                    case.residuals()
                        .any(|residual| residual != MutationResidual::TargetNativeRunRequired),
                    "{:?} names no component that has to exist first",
                    case.id()
                );
            }
        }
    }
}

#[test]
fn every_refused_case_is_actually_refused_and_by_the_variant_that_names_it() {
    // The census is not read as a table. Each case it says the
    // constructor refuses is built and run, and the refusal it produces
    // has to be the one that names the mutation — a case refused for
    // some other reason would be evidence about a different mistake.
    let cases = constructor_mutation_cases();

    for (id, case) in &cases {
        if case.disposition() != ConstructorDisposition::RefusedByTheConstructor {
            continue;
        }
        assert!(
            refusal_matches(*id),
            "{id:?} is not refused by the variant that names it"
        );
    }

    // Every refused case was reached: a helper that silently answered
    // true for an unhandled case would make the loop above vacuous.
    let refused = cases
        .values()
        .filter(|case| case.disposition() == ConstructorDisposition::RefusedByTheConstructor)
        .count();
    assert_eq!(refused, 11);
}

/// Whether one refused mutation case produces the refusal that names it.
///
/// Exhaustive over the case census with no wildcard arm, so a case added
/// later stops this file compiling until somebody decides how it is
/// refused — which is the only mechanism that keeps the census and the
/// oracle from drifting apart.
#[expect(
    clippy::too_many_lines,
    reason = "one arm per refused case, and merging arms would lose which \
              mutation each refusal names"
)]
fn refusal_matches(id: ConstructorMutationCaseId) -> bool {
    use ConstructorMutationCaseId as Case;

    let bounds = LiveTransferShapeBounds::new(count(2), count(2), 1);

    match id {
        Case::OwnerOmitted => matches!(
            OwnerKey::new(&closure(), approved(), Vec::new()),
            Err(OwnerKeyRejection::OwnerOmitted)
        ),
        Case::OwnerKeyWrongWidth => matches!(
            OwnerKey::new(&closure(), approved(), vec![0x11; approved_width() + 1]),
            Err(OwnerKeyRejection::WrongWidth { .. })
        ),
        Case::OwnerKeyAlternateEncoding => {
            let alternate = EncodingClass::ALL
                .iter()
                .copied()
                .find(|class| {
                    class.v1_shape().domain() == EncodingDomain::Key && *class != approved()
                })
                .expect("the target names a second public-key encoding");

            matches!(
                OwnerKey::new(&closure(), alternate, vec![0x11; approved_width()]),
                Err(OwnerKeyRejection::AlternateEncodingOfApprovedKey { .. })
            )
        }
        Case::OwnerKeyScalarMaterial => matches!(
            OwnerKey::new(
                &closure(),
                EncodingClass::EcScalar,
                vec![0x11; approved_width()],
            ),
            Err(OwnerKeyRejection::NotAKeyEncoding { .. })
        ),

        Case::LeafOfTheOtherRepresentation => {
            let mut subject = Subject::new();
            subject.leaves.insert(LiveTransferLeafRole::Member {
                representation: LiveTransferRepresentationPlan::PrivateCommitted,
                receipt_inputs: 2,
            });

            matches!(
                subject.derive(),
                Err(LiveConstructorRefusal::LeafOfAnotherRepresentation { .. })
            )
        }
        Case::CoordinatorLeafRemoved | Case::MemberLeafRemoved => {
            let wanted = if id == Case::CoordinatorLeafRemoved {
                LiveProgramRole::Coordinator
            } else {
                LiveProgramRole::Member
            };
            let mut subject = Subject::new();
            let removed = subject
                .leaves
                .iter()
                .copied()
                .find(|leaf| leaf.program_role() == wanted)
                .expect("the reference leaf set has both roles");
            subject.leaves.remove(&removed);

            matches!(
                subject.derive(),
                Err(LiveConstructorRefusal::LeafSetIncomplete { .. })
            )
        }
        Case::LeafForAnUnadmittedShape => {
            let wider = LiveTransferShapeBounds::new(count(8), count(8), 0);
            let mut subject = Subject::new();
            subject.leaves.insert(LiveTransferLeafRole::Coordinator {
                representation: subject.representation,
                shape: LiveTransferShape::new(
                    wider,
                    count(8),
                    count(8),
                    0,
                    SponsorChangePresence::Absent,
                )
                .expect("admissible under a wider window"),
            });

            matches!(
                subject.derive(),
                Err(LiveConstructorRefusal::LeafServesNoAdmittedShape { .. })
            )
        }
        Case::EveryTransferLeafRemoved => {
            let mut subject = Subject::new();
            subject.leaves = BTreeSet::new();

            subject.derive() == Err(LiveConstructorRefusal::KeyPathWouldBeTheOnlySpendingRoute)
        }

        // Refused where the shape is built, before any constructor sees
        // it: an inadmissible shape has no value, so there is nothing
        // for the leaf set to be built over.
        Case::ShapeOutsideTheCandidateBounds => {
            LiveTransferShape::new(bounds, count(4), count(1), 0, SponsorChangePresence::Absent)
                .is_err()
        }
        Case::SponsorChangeWithoutSponsorRegion => LiveTransferShape::new(
            bounds,
            count(1),
            count(1),
            0,
            SponsorChangePresence::Present,
        )
        .is_err(),

        Case::OwnerBytesReplacedInTheLinkedOutput
        | Case::TimeLockedPredecessorOfferedToATransferLeaf
        | Case::KeyPathSpendAttemptedOnTheLinkedOutput
        | Case::InternalKeyReplacedWithASpendableOne
        | Case::BurnLeafAddedToTheTaptree
        | Case::RepresentationSwappedInTheLinkedTree => {
            unreachable!("{id:?} is an expressible case, not a refused one")
        }
    }
}

#[test]
fn the_case_census_and_the_cases_map_agree() {
    let cases: BTreeMap<_, _> = constructor_mutation_cases();

    assert_eq!(cases.len(), ConstructorMutationCaseId::ALL.len());
    for id in ConstructorMutationCaseId::ALL {
        assert_eq!(cases[id].id(), *id);
    }
}
