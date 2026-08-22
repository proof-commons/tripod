//! Oracles for the candidate relocatable explicit live-transfer bundle.
//!
//! # What is checked against what
//!
//! The bundle is emitted once and interrogated, and nothing here asks it
//! to confirm its own bookkeeping. The leaf census is compared with the
//! constructor's own leaf set rather than with a second list; every
//! relocation is required to name a symbol the table describes and to sit
//! at a pushed literal of the program it claims; and the candidate-only
//! status is checked by the absence of any route to another one rather
//! than by reading the field back.
//!
//! # The symbols below are public test material
//!
//! The shared fixture set, in the sense
//! `(´[ADR015-rule:security:test-material]´)` fixes: distinguishable
//! byte strings standing in for values a later wave resolves, and never a
//! key anybody holds.

use std::collections::BTreeSet;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;

use super::{live_transfer_plan, live_transfer_symbols, reviewed_target};
use crate::authorization::{OwnerProfileDisposition, owner_key_encoding_closure};
use crate::bundle::{
    BackendArtifactStatus, FieldSide, InternalKeyPolicy, KeyPathPolicy, RelocationEncoding,
    SubstitutionMode, SymbolBinding, SymbolWidth, TargetRole, WitnessComponent,
};
use crate::instruction::TapscriptInstruction;
use crate::live_bundle::{
    CandidateRelocatableLiveTransferBundle, LiveBundleSymbol, LiveRelocationSite,
    emit_candidate_live_bundle,
};
use crate::live_constructor::{
    LiveTransferLeafRole, OwnerKey, StaticLiveReceiptConstructor, derive_live_receipt_constructor,
    static_transfer_leaf_set,
};
use crate::live_pattern::{LiveTransferPatternId, RecognitionResidual, patterns_for};
use crate::live_plan::{family_range_defects, opens_an_amount};
use crate::live_private::opens_no_value_payload;
use crate::live_shape::demonstration_live_shape_set;

// --- Fixtures ---------------------------------------------------------

/// The reference constructor for one representation plan.
///
/// # Panics
///
/// If the reference parts stop deriving a constructor, which Wave 4's own
/// oracles would have caught first.
fn constructor(representation: LiveTransferRepresentationPlan) -> StaticLiveReceiptConstructor {
    let target = reviewed_target();
    let shapes = demonstration_live_shape_set();
    let leaves = static_transfer_leaf_set(representation, &shapes);
    let closure = owner_key_encoding_closure(target.definition().authorization());
    let owner = OwnerKey::new(&closure, closure.approved(), vec![0x11; 32])
        .expect("the fixture is the approved encoding at its exact width");

    derive_live_receipt_constructor(
        &target,
        &live_transfer_plan(),
        representation,
        owner,
        shapes,
        leaves,
    )
    .expect("the reference subject derives")
}

/// The emitted candidate bundle.
///
/// # Panics
///
/// If emission refuses, which is the finding rather than a fixture
/// problem: every refusal names something wrong with what the backend was
/// asked to build.
fn bundle() -> CandidateRelocatableLiveTransferBundle {
    emit_candidate_live_bundle(
        &reviewed_target(),
        &live_transfer_plan(),
        &constructor(LiveTransferRepresentationPlan::Explicit),
        live_transfer_symbols(&reviewed_target()),
    )
    .expect("the candidate bundle emits")
}

// --- Emission ---------------------------------------------------------

#[test]
fn the_bundle_carries_exactly_the_constructors_leaf_set() {
    let bundle = bundle();
    let constructor = constructor(LiveTransferRepresentationPlan::Explicit);

    // Not a second list: the leaves the bundle emitted programs for are
    // required to be the leaves the constructor says its outputs can be
    // spent through, in both directions.
    assert_eq!(
        bundle.leaves().keys().copied().collect::<BTreeSet<_>>(),
        constructor.leaves().collect::<BTreeSet<_>>(),
    );
    assert!(!bundle.leaves().is_empty());
}

#[test]
fn every_leaf_consumes_exactly_the_owner_signature() {
    // The witness role is established by scheduling, not declared: every
    // live-transfer leaf is spent with the leaf script, its control
    // block, and one witness datum, which is §10.2's signature and
    // nothing else.
    for leaf in bundle().leaves().values() {
        assert_eq!(
            leaf.data_items(),
            1,
            "{:?} wants a second datum",
            leaf.leaf()
        );
        assert_eq!(
            leaf.witness(),
            &BTreeSet::from([WitnessComponent::LeafScript, WitnessComponent::ControlBlock]),
        );
        assert!(
            leaf.resources()
                .contains_key(&target_elements::ResourceDimension::ScriptBytes)
        );
    }
}

#[test]
fn every_admitted_shape_has_a_coordinator_and_complete_family_ranges() {
    let bundle = bundle();
    let representation = LiveTransferRepresentationPlan::Explicit;

    for shape in demonstration_live_shape_set().shapes() {
        assert!(
            bundle
                .leaf(LiveTransferLeafRole::Coordinator {
                    representation,
                    shape,
                })
                .is_some(),
            "{shape:?} is admitted and has no coordinator program",
        );
        let ranges = bundle
            .family_ranges()
            .get(&shape)
            .expect("every admitted shape has a family-range census");
        assert_eq!(family_range_defects(ranges), Vec::new());
    }
}

#[test]
fn a_member_leaf_serving_several_shapes_carries_a_recomputed_sharing_proof() {
    // The demonstration set carries several shapes of each receipt-input
    // count above one, so the sharing is real and §11.3's condition is
    // something emission had to establish rather than assume.
    let bundle = bundle();

    assert!(!bundle.sharing().is_empty());
    for proof in bundle.sharing() {
        assert!(proof.shapes().len() > 1);
        assert!(matches!(proof.leaf(), LiveTransferLeafRole::Member { .. }));
        // And the leaf record agrees with the proof about which shapes it
        // serves, so the two cannot drift.
        assert_eq!(
            bundle
                .leaf(proof.leaf())
                .expect("the shared leaf has a record")
                .shapes(),
            proof.shapes(),
        );
    }
}

#[test]
fn the_private_constructor_emits_a_bundle_of_its_own() {
    // The refusal this wave lifted. §10.6's obligation is built, so the
    // private constructor emits — and it emits under exactly the checks
    // the explicit one passed: every leaf walked and held to §10.9, every
    // shape's positions covered exactly once.
    let private = emit_candidate_live_bundle(
        &reviewed_target(),
        &live_transfer_plan(),
        &constructor(LiveTransferRepresentationPlan::PrivateCommitted),
        live_transfer_symbols(&reviewed_target()),
    )
    .expect("the private plan emits a candidate bundle");

    assert_eq!(
        private.representation(),
        LiveTransferRepresentationPlan::PrivateCommitted,
    );
    assert!(!private.leaves().is_empty());
    assert_eq!(private.status(), BackendArtifactStatus::Prototype);

    // Two bundles and not one. §11.3 keeps the leaf sets disjoint, so no
    // leaf of either bundle is a leaf of the other — which is the check
    // that would fail first if emitting for both had quietly become
    // emitting one shared coordinator.
    let explicit = bundle();
    assert_eq!(
        explicit.representation(),
        LiveTransferRepresentationPlan::Explicit
    );
    for leaf in private.leaves().keys() {
        assert_eq!(
            leaf.representation(),
            LiveTransferRepresentationPlan::PrivateCommitted
        );
        assert!(!explicit.leaves().contains_key(leaf));
    }

    // And the private bundle's programs are the private plan's: none of
    // them opens an amount, over every leaf the bundle carries.
    for leaf in private.leaves().values() {
        assert!(!opens_an_amount(leaf.program()));
        assert!(opens_no_value_payload(leaf.program()));
    }
}

// --- Symbols and relocations ------------------------------------------

#[test]
fn every_relocation_names_a_symbol_the_table_describes() {
    let bundle = bundle();

    for relocation in bundle.relocations() {
        let entry = bundle
            .symbols()
            .get(&relocation.symbol())
            .expect("a relocation names a symbol the table does not describe");
        assert_eq!(entry.width(), relocation.width());
        assert_eq!(entry.encoding(), relocation.encoding());
        // Every relocation this bundle emits substitutes before
        // serialization, so no byte patch and no placeholder exists to be
        // resolved against the wrong width.
        assert_eq!(
            relocation.substitution(),
            &SubstitutionMode::StructuredBeforeSerialization,
        );
    }
}

#[test]
fn every_program_site_is_a_pushed_literal_of_the_leaf_it_names() {
    // The relocations are located by rebuilding with one symbol replaced,
    // so the sites are the positions that moved. Checked here against the
    // programs themselves: a site that was not a push would be a symbol
    // the linker could not substitute into.
    let bundle = bundle();

    for relocation in bundle.relocations() {
        let LiveRelocationSite::ProgramInstructions { leaf, indices } = relocation.site() else {
            continue;
        };
        assert_eq!(indices.len(), relocation.multiplicity().get());
        let program = bundle
            .leaf(*leaf)
            .expect("a relocation names a leaf the bundle emitted")
            .program();
        for index in indices {
            assert!(matches!(
                program.instructions().get(*index),
                Some(TapscriptInstruction::Push(_)),
            ));
        }
    }
}

#[test]
fn the_protocol_asset_holds_both_ends_of_the_output_closure_induction() {
    // One symbol, two target roles: compared with an input's asset by
    // §10.1's recognition and with a destination's by §10.4's closure.
    // Those are the two ends of the induction §10.4 licenses, and the
    // relocation census keeps them apart rather than merging them into
    // one site list.
    let roles: BTreeSet<TargetRole> = bundle()
        .relocations_for(LiveBundleSymbol::ProtocolAsset)
        .map(crate::live_bundle::LiveRelocation::role)
        .collect();

    assert_eq!(
        roles,
        BTreeSet::from([
            TargetRole::AssetComparand {
                side: FieldSide::Input,
            },
            TargetRole::AssetComparand {
                side: FieldSide::Output,
            },
        ]),
    );
}

#[test]
fn the_owner_key_is_placed_as_a_signature_operand_and_defined_by_the_bundle() {
    let bundle = bundle();

    // The role compact ASH had no member for: the key is an operand of
    // the signature primitive rather than a comparand of an introspected
    // field.
    let roles: BTreeSet<TargetRole> = bundle
        .relocations_for(LiveBundleSymbol::OwnerPublicKey)
        .map(crate::live_bundle::LiveRelocation::role)
        .collect();
    assert_eq!(roles, BTreeSet::from([TargetRole::SignatureKeyOperand]));

    // And it is settled here rather than at link: the constructor commits
    // one owner, and §7.6 gives a request no parameter through which
    // another could arrive.
    assert_eq!(
        bundle.symbols()[&LiveBundleSymbol::OwnerPublicKey].binding(),
        SymbolBinding::DefinedByBundle,
    );
    assert!(matches!(
        bundle.symbols()[&LiveBundleSymbol::OwnerPublicKey].width(),
        SymbolWidth::Fixed { .. },
    ));
}

#[test]
fn the_link_time_symbols_are_the_ones_a_later_layer_settles() {
    let bundle = bundle();

    let resolved: BTreeSet<LiveBundleSymbol> = bundle
        .symbols()
        .iter()
        .filter(|(_, entry)| entry.binding() == SymbolBinding::ResolvedAtLink)
        .map(|(symbol, _)| *symbol)
        .collect();

    assert_eq!(
        resolved,
        BTreeSet::from([
            LiveBundleSymbol::ProtocolAsset,
            LiveBundleSymbol::ReserveAsset,
            LiveBundleSymbol::DestinationProgramVersion,
            LiveBundleSymbol::SponsorChangeProgram,
            LiveBundleSymbol::SponsorChangeProgramVersion,
            LiveBundleSymbol::TargetFeeRoleProgramDigest,
            LiveBundleSymbol::UnspendableInternalKey,
        ]),
    );

    // And no symbol is read from the target at spend time, because no
    // live-transfer fragment reads one: recognition rests on the leaf
    // commitment instead of on a comparison.
    assert!(
        !bundle
            .symbols()
            .values()
            .any(|entry| entry.binding() == SymbolBinding::ReadFromTargetAtSpendTime),
    );
}

#[test]
fn every_emitted_leaf_has_its_own_committed_program_symbol() {
    let bundle = bundle();

    for leaf in bundle.leaves().keys() {
        let symbol = match leaf {
            LiveTransferLeafRole::Coordinator { shape, .. } => {
                LiveBundleSymbol::CoordinatorProgram { shape: *shape }
            }
            LiveTransferLeafRole::Member { receipt_inputs, .. } => {
                LiveBundleSymbol::MemberProgram {
                    receipt_inputs: *receipt_inputs,
                }
            }
        };
        assert_eq!(
            bundle.symbols()[&symbol].encoding(),
            RelocationEncoding::TapscriptLeafScript,
        );
        assert!(
            bundle
                .relocations_for(symbol)
                .any(|relocation| relocation.role() == TargetRole::CommittedLeafScript),
        );
    }
}

// --- Candidate state --------------------------------------------------

#[test]
fn the_bundle_is_a_candidate_and_carries_its_outstanding_lifecycle() {
    let bundle = bundle();

    assert_eq!(bundle.status(), BackendArtifactStatus::Prototype);
    // Structurally incomplete: the outstanding count is a nonzero, so a
    // bundle whose lifecycle was complete has no representation at all.
    assert!(bundle.lifecycle().outstanding().get() > 0);
    assert_eq!(
        bundle.representation(),
        LiveTransferRepresentationPlan::Explicit
    );
    assert_eq!(
        bundle.internal_key(),
        InternalKeyPolicy::UnspendableWithResidualDiscreteLogAssumption,
    );
    assert_eq!(bundle.key_path(), KeyPathPolicy::NoAcceptedEscape);
    assert!(!bundle.assumptions().is_empty());
    assert!(bundle.total_script_bytes() > 0);
}

#[test]
fn the_bundle_publishes_the_residuals_its_patterns_carry() {
    let bundle = bundle();

    // The two that decide how far a consumer may trust it: §10.4 is not
    // whole, and the profile a signature is taken under is not reviewed.
    for residual in [
        RecognitionResidual::LinkedDestinationConstructorIdentity,
        RecognitionResidual::SighashProfileUnreviewed,
    ] {
        assert!(
            bundle.residuals().contains(&residual),
            "the bundle reads as though {residual:?} were discharged",
        );
    }
    assert!(matches!(
        bundle.sighash_profile(),
        OwnerProfileDisposition::ReviewIncomplete { .. },
    ));
    // §11.2 lists the profile among the symbol roles, and the table says
    // what it is: settled here, a typed parameter, and reaching no
    // serialized field — which is why it has no relocation.
    let profile = bundle.symbols()[&LiveBundleSymbol::SelectedSighashProfile];
    assert_eq!(profile.binding(), SymbolBinding::DefinedByBundle);
    assert_eq!(profile.encoding(), RelocationEncoding::TypedParameter);
    assert_eq!(profile.width(), SymbolWidth::Unserialized);
    assert_eq!(
        bundle
            .relocations_for(LiveBundleSymbol::SelectedSighashProfile)
            .count(),
        0,
    );
    assert!(!bundle.target_evidence().is_empty());
}

#[test]
fn the_selected_patterns_are_exactly_what_the_admitted_shapes_call_for() {
    // Both bundles, because the value obligation is the one identity the
    // representation moves and a check that ran over the explicit bundle
    // alone would never see it move.
    for representation in [
        LiveTransferRepresentationPlan::Explicit,
        LiveTransferRepresentationPlan::PrivateCommitted,
    ] {
        let bundle = emit_candidate_live_bundle(
            &reviewed_target(),
            &live_transfer_plan(),
            &constructor(representation),
            live_transfer_symbols(&reviewed_target()),
        )
        .expect("the candidate bundle emits");
        let expected: BTreeSet<LiveTransferPatternId> = demonstration_live_shape_set()
            .shapes()
            .flat_map(|shape| patterns_for(shape, representation))
            .collect();

        assert_eq!(bundle.selected_patterns(), &expected);
    }

    // And each carries its own value obligation and not the other's.
    assert!(
        bundle()
            .selected_patterns()
            .contains(&LiveTransferPatternId::LiveExplicitConservationV1),
    );
    let private = emit_candidate_live_bundle(
        &reviewed_target(),
        &live_transfer_plan(),
        &constructor(LiveTransferRepresentationPlan::PrivateCommitted),
        live_transfer_symbols(&reviewed_target()),
    )
    .expect("the private bundle emits");
    assert!(
        !private
            .selected_patterns()
            .contains(&LiveTransferPatternId::LiveExplicitConservationV1),
    );
    assert!(
        private
            .selected_patterns()
            .contains(&LiveTransferPatternId::LivePrivateDestinationFormV1),
    );
}
