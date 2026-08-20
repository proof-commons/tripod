//! The candidate relocatable bundle, checked by recomputation
//! (Guide-12 §11, §13).
//!
//! # Nothing here reads the bundle's own answer back
//!
//! Every figure below is either recomputed from something the bundle
//! does not own — the shape set, the reviewed contract, the compiler's
//! plan — or written out as an exact number. A test that asked the
//! bundle what its layout was and then agreed with it would pass for a
//! bundle whose layout was wrong in the same way twice.
//!
//! The relocation checks are the sharpest case. The bundle finds its
//! sites by rebuilding a program with one symbol replaced; the tests
//! below check those sites by *value*, against the symbol items the
//! bundle was laid out with, which is an independent route to the same
//! answer. The reverse direction — that no site was missed — is checked
//! only for the wide byte-string symbols, because that is exactly the
//! set for which a byte scan is unambiguous, and the ambiguity of the
//! narrow ones is why the bundle does not use a scan at all.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use target_elements::{OpcodeId, ResourceDimension};

use crate::bundle::{
    BackendArtifactStatus, BundleRefusal, BundleSymbol, CandidateRelocatableTapscriptBundle,
    ConcreteCarrierSite, FieldSide, InputRole, InternalKeyPolicy, KeyPathPolicy, LeafRole,
    OutputRole, ProgramRole, Relocation, RelocationEncoding, RelocationSite, ResourceModel,
    SharingGround, SubstitutionMode, SymbolBinding, SymbolWidth, TargetRole, WitnessComponent,
    emit_candidate_bundle,
};
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::pattern::{carries_authorization, coordinator_program, member_program};
use crate::policy::demonstration_policy;
use crate::shape::{CompactAshShape, SponsorChangePresence, demonstration_shape_set};
use crate::tests::{compact_ash_plan, pattern_symbols, reviewed_target};

/// The demonstration bundle.
fn bundle() -> CandidateRelocatableTapscriptBundle {
    let target = reviewed_target();
    emit_candidate_bundle(
        &target,
        &compact_ash_plan(),
        demonstration_policy(),
        pattern_symbols(&target),
    )
    .expect("the demonstration bundle emits")
}

/// Every demonstration shape, in canonical order.
fn shapes() -> Vec<CompactAshShape> {
    demonstration_shape_set().shapes().collect()
}

#[test]
fn the_leaf_census_is_nine_coordinators_and_three_shared_members() {
    // §11.2 requires a coordinator leaf and a member leaf per admitted
    // shape. The coordinator program is a function of the whole shape,
    // so there are as many as there are shapes; the member program is a
    // function of the batch size alone, so three batch sizes give three
    // member leaves and nine shapes share them. The figures are written
    // out, so a set that grew or a sharing that silently stopped fails
    // here rather than passing quietly.
    let bundle = bundle();
    let constructor = bundle.constructor();

    assert_eq!(shapes().len(), 9);
    assert_eq!(constructor.leaves().len(), 12);

    let coordinators = constructor
        .leaves()
        .keys()
        .filter(|leaf| leaf.program_role() == ProgramRole::Coordinator)
        .count();
    let members = constructor
        .leaves()
        .keys()
        .filter(|leaf| leaf.program_role() == ProgramRole::Member)
        .count();
    assert_eq!(coordinators, 9);
    assert_eq!(members, 3);

    // Every shape reaches both of its leaves, and the pair is returned
    // together so neither can be read without the other.
    for shape in shapes() {
        let (coordinator, member) = constructor
            .shape_leaves(shape)
            .expect("every admitted shape has its pair");
        assert_eq!(coordinator.leaf(), LeafRole::Coordinator { shape });
        assert_eq!(
            member.leaf(),
            LeafRole::Member {
                ash_inputs: shape.ash_inputs()
            },
        );
        assert!(member.shapes().contains(&shape));
    }

    assert_eq!(constructor.sharing().len(), 3);
    for proof in constructor.sharing() {
        assert_eq!(proof.shapes().len(), 3);
        assert_eq!(
            proof.ground(),
            SharingGround::IdenticalTypedProgramRecomputedPerShape,
        );
    }
}

#[test]
fn every_shared_member_leaf_is_the_program_every_sharing_shape_builds() {
    // The typed proof §11.2 demands, recomputed here rather than read
    // back: each sharing shape's own member program is built from that
    // shape and compared with the shared leaf's, instruction by
    // instruction. Equal instructions is equal enforcement.
    let target = reviewed_target();
    let symbols = pattern_symbols(&target);
    let bundle = bundle();

    for proof in bundle.constructor().sharing() {
        let leaf = bundle
            .constructor()
            .leaf(proof.leaf())
            .expect("the shared leaf is in the census");
        for shape in proof.shapes() {
            let rebuilt = member_program(&target, &symbols, *shape).expect("the member schedules");
            assert_eq!(leaf.program(), &rebuilt, "{shape:?}");
        }
    }
}

#[test]
fn every_coordinator_leaf_is_the_program_its_shape_builds() {
    let target = reviewed_target();
    let symbols = pattern_symbols(&target);
    let bundle = bundle();

    for shape in shapes() {
        let leaf = bundle
            .constructor()
            .leaf(LeafRole::Coordinator { shape })
            .expect("every shape has a coordinator leaf");
        let rebuilt = coordinator_program(&target, &symbols, shape).expect("it schedules");
        assert_eq!(leaf.program(), &rebuilt, "{shape:?}");
    }
}

#[test]
fn every_layout_position_is_claimed_exactly_once_and_by_the_right_role() {
    // §10.1 and §10.2, recomputed from the shape rather than from the
    // layout: input 0 is the coordinator, the members fill the rest of
    // the ASH range, the sponsor suffix follows, and the outputs are
    // the successor, then the optional change, then the fee role.
    let bundle = bundle();

    for shape in shapes() {
        let layout = bundle.layout(shape).expect("every shape has a layout");
        let mut inputs: Vec<Option<InputRole>> = vec![None; usize::from(shape.inputs())];
        for placement in layout.inputs() {
            for position in placement.first()..placement.end() {
                let slot = &mut inputs[usize::from(position)];
                assert!(slot.is_none(), "position claimed twice");
                *slot = Some(placement.role());
            }
        }

        let expected = (0..shape.inputs())
            .map(|position| {
                if position == 0 {
                    Some(InputRole::Coordinator)
                } else if position < u16::from(shape.ash_inputs()) {
                    Some(InputRole::Member)
                } else {
                    Some(InputRole::Sponsor)
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(inputs, expected, "{shape:?}");

        let mut outputs: Vec<Option<OutputRole>> = vec![None; usize::from(shape.outputs())];
        for placement in layout.outputs() {
            let slot = &mut outputs[usize::from(placement.position())];
            assert!(slot.is_none(), "position claimed twice");
            *slot = Some(placement.role());
        }

        let mut expected = vec![Some(OutputRole::Successor)];
        if shape.sponsor_change() == SponsorChangePresence::Present {
            expected.push(Some(OutputRole::SponsorChange));
        }
        if shape.sponsored() {
            expected.push(Some(OutputRole::TargetFee));
        }
        assert_eq!(outputs, expected, "{shape:?}");

        // The member run stops at the ASH bound, so no member leaf is
        // ever placed on a sponsor input (§12.7, §12.9).
        let members = layout
            .input_run(InputRole::Member)
            .expect("every shape has a member run");
        assert_eq!(members.end(), u16::from(shape.ash_inputs()));
        assert_eq!(members.first(), 1);
    }
}

#[test]
fn no_leaf_consumes_a_witness_datum() {
    // The witness role every protocol leaf declares: the leaf script
    // and its control block, and no data item at all. Established by
    // the bundle scheduling each program from the empty stack, which is
    // what makes the permissionless path something anybody can spend
    // rather than something a witness supplier gates.
    let bundle = bundle();

    for leaf in bundle.constructor().leaves().values() {
        assert_eq!(leaf.witness().data_items(), 0, "{:?}", leaf.leaf());
        assert_eq!(
            leaf.witness().components(),
            &BTreeSet::from([WitnessComponent::LeafScript, WitnessComponent::ControlBlock]),
        );
        assert_eq!(
            leaf.resources()
                .charged(ResourceDimension::InitialStackItems),
            Some(0),
        );
        assert!(!carries_authorization(leaf.program()), "{:?}", leaf.leaf());
    }
}

#[test]
fn every_relocation_site_holds_the_symbol_it_names() {
    // The independent check on the differential probe. The bundle
    // located these positions by rebuilding each program with one
    // symbol replaced; here each position is opened and its literal
    // compared with the symbol's own item. Two routes, one answer.
    let target = reviewed_target();
    let bundle = bundle();
    let symbols = bundle.unresolved_symbols();

    let expected = |symbol: BundleSymbol| -> Option<StackItem> {
        Some(match symbol {
            BundleSymbol::ClosedAsset => symbols.closed_asset().clone(),
            BundleSymbol::ReserveAsset => symbols.reserve_asset().clone(),
            BundleSymbol::AshConstructorProgram => symbols.ash_program().clone(),
            BundleSymbol::SponsorChangeProgram => symbols.sponsor_change_program().clone(),
            BundleSymbol::TargetFeeRoleProgramDigest => symbols.fee_program_digest().clone(),
            BundleSymbol::AshConstructorProgramVersion => {
                StackItem::script_number(&target, symbols.ash_program_version()).ok()?
            }
            BundleSymbol::SponsorChangeProgramVersion => {
                StackItem::script_number(&target, symbols.sponsor_change_version()).ok()?
            }
            _ => return None,
        })
    };

    let mut program_sites = 0;
    for relocation in bundle.relocations() {
        let RelocationSite::ProgramInstructions { leaf, indices } = relocation.site() else {
            continue;
        };
        let item = expected(relocation.symbol()).expect("a program symbol has an item");
        let program = bundle
            .constructor()
            .leaf(*leaf)
            .expect("the relocation names a leaf of this bundle")
            .program();

        assert_eq!(relocation.multiplicity().get(), indices.len());
        for index in indices {
            let TapscriptInstruction::Push(pushed) = &program.instructions()[*index] else {
                panic!("a relocation site is not a push");
            };
            assert_eq!(pushed, &item, "{:?} at {index}", relocation.symbol());
            program_sites += 1;
        }
    }
    assert!(program_sites > 0);
}

#[test]
fn no_wide_symbol_occurrence_is_left_without_a_relocation() {
    // The other direction, for the symbols a byte scan can settle. A
    // thirty-two byte placeholder cannot be confused with an index or a
    // version, so every push of one in every emitted program must be
    // covered by a relocation. The narrow symbols are deliberately
    // outside this check: a version push and an index push are the same
    // bytes, which is the whole reason the bundle locates sites by
    // rebuild rather than by scan.
    let bundle = bundle();
    let symbols = bundle.unresolved_symbols();

    let wide = [
        (BundleSymbol::ClosedAsset, symbols.closed_asset()),
        (BundleSymbol::ReserveAsset, symbols.reserve_asset()),
        (BundleSymbol::AshConstructorProgram, symbols.ash_program()),
        (
            BundleSymbol::SponsorChangeProgram,
            symbols.sponsor_change_program(),
        ),
        (
            BundleSymbol::TargetFeeRoleProgramDigest,
            symbols.fee_program_digest(),
        ),
    ];

    for (symbol, item) in wide {
        assert!(item.len() >= 20, "the scan is only unambiguous when wide");
        for (leaf, program) in bundle.constructor().leaves() {
            let scanned: BTreeSet<usize> = program
                .program()
                .instructions()
                .iter()
                .enumerate()
                .filter(|(_, instruction)| {
                    matches!(instruction, TapscriptInstruction::Push(pushed) if pushed == item)
                })
                .map(|(index, _)| index)
                .collect();

            let mut relocated = BTreeSet::new();
            for relocation in bundle.relocations_for(symbol) {
                if let RelocationSite::ProgramInstructions {
                    leaf: named,
                    indices,
                } = relocation.site()
                    && named == leaf
                {
                    relocated.extend(indices.iter().copied());
                }
            }
            assert_eq!(scanned, relocated, "{symbol:?} in {leaf:?}");
        }
    }
}

#[test]
fn the_symbol_table_is_exactly_the_guide_list_and_every_relocation_is_backed() {
    // §13.4's roles, all of them, and both censuses exact: every symbol
    // the table carries is relocated somewhere, and every relocation
    // names a symbol the table carries.
    let bundle = bundle();

    let mut expected: BTreeSet<BundleSymbol> = BTreeSet::from([
        BundleSymbol::ClosedAsset,
        BundleSymbol::ReserveAsset,
        BundleSymbol::AshConstructorProgram,
        BundleSymbol::AshConstructorProgramVersion,
        BundleSymbol::SponsorChangeProgram,
        BundleSymbol::SponsorChangeProgramVersion,
        BundleSymbol::TargetFeeRoleProgramDigest,
        BundleSymbol::UnspendableInternalKey,
        BundleSymbol::TargetLeafVersion,
        BundleSymbol::CandidateAshBound,
        BundleSymbol::CandidateSponsorBound,
    ]);
    for shape in shapes() {
        expected.insert(BundleSymbol::CoordinatorProgram { shape });
        expected.insert(BundleSymbol::MemberProgram {
            ash_inputs: shape.ash_inputs(),
        });
    }

    let carried: BTreeSet<BundleSymbol> = bundle.symbols().keys().copied().collect();
    assert_eq!(carried, expected);
    assert_eq!(bundle.symbols().len(), 11 + 9 + 3);

    let relocated: BTreeSet<BundleSymbol> = bundle
        .relocations()
        .iter()
        .map(Relocation::symbol)
        .collect();
    assert_eq!(
        relocated, expected,
        "a symbol nobody places is not a symbol"
    );

    // The eight symbols a later layer settles, and the rest this bundle
    // settles itself. The split is what the linker's work is.
    let unresolved: BTreeSet<BundleSymbol> = bundle
        .symbols()
        .iter()
        .filter(|(_, entry)| entry.binding() == SymbolBinding::ResolvedAtLink)
        .map(|(symbol, _)| *symbol)
        .collect();
    assert_eq!(
        unresolved,
        BTreeSet::from([
            BundleSymbol::ClosedAsset,
            BundleSymbol::ReserveAsset,
            BundleSymbol::AshConstructorProgram,
            BundleSymbol::AshConstructorProgramVersion,
            BundleSymbol::SponsorChangeProgram,
            BundleSymbol::SponsorChangeProgramVersion,
            BundleSymbol::TargetFeeRoleProgramDigest,
            BundleSymbol::UnspendableInternalKey,
        ]),
    );
}

#[test]
fn the_asset_and_program_roles_are_read_from_the_introspection_they_sit_under() {
    // One symbol can hold two roles: the closed asset is compared with
    // an input asset in every recognition fragment and with output 0's
    // asset in the successor fragment. The bundle keeps those apart,
    // and a reader can tell which side a relocation is about.
    let bundle = bundle();

    let roles: BTreeSet<TargetRole> = bundle
        .relocations_for(BundleSymbol::ClosedAsset)
        .map(Relocation::role)
        .collect();
    assert_eq!(
        roles,
        BTreeSet::from([
            TargetRole::AssetComparand {
                side: FieldSide::Input
            },
            TargetRole::AssetComparand {
                side: FieldSide::Output
            },
        ]),
    );

    // The fee role reaches a script as a digest under a program
    // introspection, never as an amount test (§10.6).
    let fee: BTreeSet<TargetRole> = bundle
        .relocations_for(BundleSymbol::TargetFeeRoleProgramDigest)
        .map(Relocation::role)
        .collect();
    assert_eq!(fee, BTreeSet::from([TargetRole::FeeProgramDigestComparand]));

    // The reserve asset is read on both sides too: the sponsor inputs
    // and the change and fee outputs.
    let reserve: BTreeSet<TargetRole> = bundle
        .relocations_for(BundleSymbol::ReserveAsset)
        .map(Relocation::role)
        .collect();
    assert_eq!(
        reserve,
        BTreeSet::from([
            TargetRole::AssetComparand {
                side: FieldSide::Input
            },
            TargetRole::AssetComparand {
                side: FieldSide::Output
            },
        ]),
    );
}

#[test]
fn every_relocation_substitutes_before_serialization() {
    // §13.4 prefers structured substitution, and this bundle uses
    // nothing else. A byte patch anywhere here would be a place where a
    // resolution of a different width could corrupt a program.
    let bundle = bundle();

    for relocation in bundle.relocations() {
        assert_eq!(
            relocation.substitution(),
            &SubstitutionMode::StructuredBeforeSerialization,
            "{:?}",
            relocation.symbol(),
        );
    }
}

#[test]
fn a_byte_patch_is_admissible_only_against_a_contract_fixed_width() {
    // §13.4's prohibition, run rather than described. A patch against a
    // width only the resolved value settles is exactly the variable
    // width the rule forbids, and a placeholder of the wrong width is
    // the other way the same corruption arrives.
    let one = NonZeroUsize::MIN;
    let target = reviewed_target();
    let placeholder = StackItem::new(&target, vec![0x00; 32]).expect("a literal within the bound");

    assert_eq!(
        Relocation::byte_patch(
            BundleSymbol::AshConstructorProgram,
            TargetRole::ProgramComparand {
                side: FieldSide::Input
            },
            RelocationSite::ConstructorBinding,
            SymbolWidth::ValueDetermined { bytes: 32 },
            RelocationEncoding::TypedParameter,
            one,
            placeholder.clone(),
        ),
        Err(BundleRefusal::VariableWidthBytePatch {
            symbol: BundleSymbol::AshConstructorProgram
        }),
    );

    assert_eq!(
        Relocation::byte_patch(
            BundleSymbol::ClosedAsset,
            TargetRole::AssetComparand {
                side: FieldSide::Input
            },
            RelocationSite::ConstructorBinding,
            SymbolWidth::Fixed { bytes: 31 },
            RelocationEncoding::TypedParameter,
            one,
            placeholder.clone(),
        ),
        Err(BundleRefusal::PlaceholderWidthMismatch {
            symbol: BundleSymbol::ClosedAsset
        }),
    );

    let admitted = Relocation::byte_patch(
        BundleSymbol::ClosedAsset,
        TargetRole::AssetComparand {
            side: FieldSide::Input,
        },
        RelocationSite::ConstructorBinding,
        SymbolWidth::Fixed { bytes: 32 },
        RelocationEncoding::TypedParameter,
        one,
        placeholder.clone(),
    )
    .expect("a fixed width with a matching placeholder is admissible");
    assert_eq!(
        admitted.substitution(),
        &SubstitutionMode::FixedWidthBytePatch { placeholder },
    );
}

#[test]
fn the_contract_fixed_widths_come_from_the_reviewed_contract() {
    // A width the contract fixes is fixed for every resolution, and a
    // width the class leaves bounded is settled only by the value. The
    // split decides where a byte patch could ever be stated, so it is
    // read from the contract rather than written down.
    let bundle = bundle();
    let table = bundle.symbols();

    for symbol in [
        BundleSymbol::ClosedAsset,
        BundleSymbol::ReserveAsset,
        BundleSymbol::TargetFeeRoleProgramDigest,
        BundleSymbol::UnspendableInternalKey,
    ] {
        assert_eq!(
            table[&symbol].width(),
            SymbolWidth::Fixed { bytes: 32 },
            "{symbol:?}",
        );
    }
    assert_eq!(
        table[&BundleSymbol::TargetLeafVersion].width(),
        SymbolWidth::Fixed { bytes: 1 },
    );

    // A witness program's width is bounded by the class, not fixed by
    // it, so a resolution may be a different width and no patch can be
    // stated against it.
    assert!(matches!(
        table[&BundleSymbol::AshConstructorProgram].width(),
        SymbolWidth::ValueDetermined { .. },
    ));
    assert!(matches!(
        table[&BundleSymbol::AshConstructorProgramVersion].width(),
        SymbolWidth::ValueDetermined { .. },
    ));
    assert_eq!(
        table[&BundleSymbol::CandidateAshBound].width(),
        SymbolWidth::Unserialized,
    );
}

#[test]
fn the_resource_formulas_reproduce_every_measurement_exactly() {
    // §13.3's exact projection, as a formula over the candidate shape
    // set. The model is recomputed at every shape it claims to cover;
    // a model that missed one would have been discarded whole rather
    // than kept as a good enough fit.
    let bundle = bundle();

    for (role, dimensions) in bundle.formulas() {
        for (dimension, formula) in dimensions {
            assert_eq!(formula.role(), *role);
            assert_eq!(formula.dimension(), *dimension);
            assert_eq!(formula.measurements().len(), 9);

            for (shape, measure) in formula.measurements() {
                match formula.model() {
                    ResourceModel::Affine { .. } => assert_eq!(
                        formula.predict(*shape),
                        Some(i64::try_from(*measure).expect("a measured figure fits")),
                        "{role:?} {dimension:?} {shape:?}",
                    ),
                    ResourceModel::ExactTableOnly => assert_eq!(formula.predict(*shape), None),
                }
            }
        }
    }

    // The dimensions §13.3 keeps separately typed and this wave does
    // not establish, each naming who owes it. An empty census here
    // would be a claim that the bundle had measured everything.
    assert_eq!(bundle.outstanding_dimensions().len(), 6);
    assert!(
        bundle
            .outstanding_dimensions()
            .contains_key(&ResourceDimension::PeakStackItems),
    );
}

#[test]
fn the_script_bytes_of_every_leaf_are_the_exact_encoded_length() {
    // §13.3: script bytes are the exact canonical encoded length, push
    // opcodes and width prefixes included, never a sum of opcode costs.
    // Recomputed here from the encoder itself.
    let target = reviewed_target();
    let bundle = bundle();
    let mut total = 0_u64;

    for leaf in bundle.constructor().leaves().values() {
        let measured = leaf
            .resources()
            .charged(ResourceDimension::ScriptBytes)
            .expect("every leaf charges script bytes");
        let encoded = leaf.program().encode(&target);

        assert_eq!(
            measured,
            u64::try_from(encoded.len()).expect("a program fits"),
            "{:?}",
            leaf.leaf(),
        );
        assert!(measured > u64::try_from(leaf.program().len()).expect("a count fits"));
        total = total.saturating_add(measured);
    }
    assert_eq!(bundle.total_script_bytes(), total);
}

#[test]
fn every_emitted_program_round_trips_through_the_supported_subset_parser() {
    // §13.2: raw script bytes appear only as the deterministic
    // serialization of a validated typed program, and the round trip is
    // what says the two are the same object.
    let target = reviewed_target();
    let bundle = bundle();

    for leaf in bundle.constructor().leaves().values() {
        let encoded = leaf.program().encode(&target);
        let parsed = crate::program::TapscriptProgram::decode(&target, &encoded)
            .expect("an emitted program parses");
        assert_eq!(&parsed, leaf.program(), "{:?}", leaf.leaf());
    }
}

#[test]
fn the_relation_placement_census_is_exactly_the_plans_carrier_census() {
    // Every carrier requirement the compiler published is placed, and
    // nothing else is. A placement that covered most of them would be
    // the dropped relation §1.3 refuses.
    let plan = compact_ash_plan();
    let bundle = bundle();

    let expected: BTreeSet<_> = plan
        .carriers()
        .map(|requirement| requirement.relation_case.clone())
        .collect();
    let placed: BTreeSet<_> = bundle.placements().keys().cloned().collect();

    assert_eq!(placed, expected);
    assert_eq!(bundle.placements().len(), 30);

    for placement in bundle.placements().values() {
        assert!(placement.realizable() <= placement.offered());
        assert!(!placement.sites().is_empty());
        for site in placement.sites() {
            assert!(matches!(
                site,
                ConcreteCarrierSite::CoordinatorLeaf
                    | ConcreteCarrierSite::MemberLeaf
                    | ConcreteCarrierSite::BundleStructure
                    | ConcreteCarrierSite::OutsideBundle
            ));
        }
    }
}

#[test]
fn the_constructor_binds_what_the_rule_lists_and_nothing_that_would_weaken_it() {
    // §11.1's binding, §11.3's internal key, and §11.4's key path. The
    // policies have one variant each, so what is checked here is that
    // the bundle carries them at all — a second variant would be a
    // change to the type, and every match would have to answer it.
    let target = reviewed_target();
    let bundle = bundle();
    let constructor = bundle.constructor();

    assert_eq!(
        constructor.leaf_version(),
        target.definition().leaf_version(),
        "the leaf version is read from the reviewed contract",
    );
    assert_eq!(constructor.contract(), target.definition().version());
    assert_eq!(
        constructor.internal_key(),
        InternalKeyPolicy::UnspendableWithResidualDiscreteLogAssumption,
    );
    assert_eq!(constructor.key_path(), KeyPathPolicy::NoAcceptedEscape);
    assert_eq!(constructor.assumptions().len(), 2);
    assert_eq!(constructor.shapes(), &demonstration_shape_set());

    // No leaf carries a key-path escape hatch by another name: nothing
    // in an emitted program verifies a signature or a timelock.
    for leaf in constructor.leaves().values() {
        for instruction in leaf.program().instructions() {
            assert!(
                !matches!(
                    instruction,
                    TapscriptInstruction::Opcode(
                        OpcodeId::CheckSig
                            | OpcodeId::CheckSigVerify
                            | OpcodeId::CheckSigFromStack
                            | OpcodeId::CheckSigFromStackVerify
                            | OpcodeId::CheckSequenceVerify
                    )
                ),
                "{:?}",
                leaf.leaf(),
            );
        }
    }
}

#[test]
fn the_bundle_is_a_candidate_and_says_so_in_its_types() {
    // §1.9 and §11.5. The status is read-only and is the weakest of the
    // three §13.5 names, the clear exit is outstanding, and the
    // outstanding set is structurally non-empty — so a bundle claiming
    // a complete lifecycle has no representation at all.
    let bundle = bundle();

    assert_eq!(bundle.status(), BackendArtifactStatus::Prototype);
    assert_eq!(bundle.outstanding_lifecycle().count(), NonZeroUsize::MIN);
    assert_eq!(bundle.outstanding_lifecycle().requirements().count(), 1);
    assert!(!bundle.plan().lifecycle().release_complete());

    // The plan is carried whole, so the bundle can be checked against
    // its own source rather than against a plan a reader supplies.
    assert_eq!(bundle.plan(), &compact_ash_plan());
    assert_eq!(bundle.plan().carriers().count(), bundle.placements().len());
}

#[test]
fn the_bundle_carries_the_assessment_it_was_gated_by_and_the_assumptions_it_rests_on() {
    // The verdict census is the one §8 produced, carried rather than
    // recomputed by a consumer; and the ABI assumptions Wave 6 stated
    // are still stated here rather than dropped at the boundary.
    let bundle = bundle();

    assert_eq!(bundle.assessment().len(), 79);
    assert_eq!(bundle.selected_patterns().len(), 8);
    assert_eq!(
        bundle.selected_patterns(),
        &demonstration_policy()
            .pattern_preference()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
    );

    let assumptions: BTreeSet<_> = bundle.abi_assumptions().iter().copied().collect();
    assert!(assumptions.contains(&crate::pattern::AbiAssumption::SymbolsResolvedAtLink));
    assert!(
        assumptions
            .contains(&crate::pattern::AbiAssumption::ExplicitFormEstablishedByPrefixEquality),
    );
    assert!(!bundle.target_evidence().is_empty());

    // The projection is the policy's own, not a second copy that could
    // drift from the one the selection was made under.
    assert_eq!(
        bundle.target_projection(),
        demonstration_policy().projection(),
    );
    assert_eq!(bundle.shapes(), &demonstration_shape_set());
}

#[test]
fn the_relocation_multiplicities_add_up_to_the_program_sites() {
    // A relocation states its multiplicity, and the figure is the site
    // count rather than a number beside it. Checked in aggregate so a
    // relocation that overstated its reach fails here.
    let bundle = bundle();
    let mut stated = 0;
    let mut counted = 0;

    for relocation in bundle.relocations() {
        stated += relocation.multiplicity().get();
        counted += match relocation.site() {
            RelocationSite::ProgramInstructions { indices, .. } => indices.len(),
            RelocationSite::ConstructorBinding => 1,
        };
    }
    assert_eq!(stated, counted);
    assert!(stated > bundle.symbols().len());
}

#[test]
fn a_sponsorless_shape_carries_no_fee_role_and_no_reserve_asset_site() {
    // §10.6 and §12.9 together: the reviewed target represents a zero
    // fee by the absence of the output, so a sponsorless shape has no
    // fee position; and with no sponsor region there is nothing for the
    // reserve asset to be compared with.
    let bundle = bundle();
    let sponsorless: Vec<_> = shapes().into_iter().filter(|s| !s.sponsored()).collect();
    assert_eq!(sponsorless.len(), 3);

    for shape in sponsorless {
        let layout = bundle.layout(shape).expect("a layout");
        assert_eq!(layout.output_position(OutputRole::TargetFee), None);
        assert_eq!(layout.output_position(OutputRole::SponsorChange), None);
        assert_eq!(layout.output_position(OutputRole::Successor), Some(0));
        assert_eq!(layout.input_run(InputRole::Sponsor), None);

        let leaf = LeafRole::Coordinator { shape };
        let reserve_sites = bundle
            .relocations_for(BundleSymbol::ReserveAsset)
            .filter(|relocation| {
                matches!(
                    relocation.site(),
                    RelocationSite::ProgramInstructions { leaf: named, .. } if *named == leaf
                )
            })
            .count();
        assert_eq!(reserve_sites, 0, "{shape:?}");
    }
}

#[test]
fn the_measured_shape_of_the_candidate_is_exactly_this() {
    // The figures a later wave links, budgets, and measures against,
    // written out so a change to any emitted program is visible as a
    // change here rather than as a quiet difference in a bundle nobody
    // recounted.
    let bundle = bundle();
    let mut per_shape: BTreeMap<(u8, u8, bool), u64> = BTreeMap::new();

    for shape in shapes() {
        let leaf = bundle
            .constructor()
            .leaf(LeafRole::Coordinator { shape })
            .expect("a coordinator leaf");
        per_shape.insert(
            (
                shape.ash_inputs(),
                shape.sponsor_inputs(),
                shape.sponsor_change() == SponsorChangePresence::Present,
            ),
            leaf.resources()
                .charged(ResourceDimension::ScriptBytes)
                .expect("script bytes"),
        );
    }

    assert_eq!(
        per_shape,
        BTreeMap::from([
            ((2, 0, false), 334),
            ((2, 1, false), 448),
            ((2, 1, true), 512),
            ((3, 0, false), 443),
            ((3, 1, false), 557),
            ((3, 1, true), 621),
            ((4, 0, false), 552),
            ((4, 1, false), 666),
            ((4, 1, true), 730),
        ]),
    );

    // The affine coefficients, written out. One further ASH source
    // costs a recognition fragment and an addition; the first sponsor
    // input costs an isolation test and the fee role it forces; the
    // change role costs its own asset and program tests.
    assert_eq!(
        bundle.formulas()[&ProgramRole::Coordinator][&ResourceDimension::ScriptBytes].model(),
        ResourceModel::Affine {
            base: 116,
            per_ash_input: 109,
            per_sponsor_input: 114,
            per_sponsor_change: 64,
        },
    );

    // The member leaf does not vary with the shape's byte count at all:
    // its only shape-dependent literal is the batch bound, carried as a
    // fixed-width signed integer, so every batch size costs the same
    // hundred and two bytes while remaining a distinct program.
    assert_eq!(
        bundle.formulas()[&ProgramRole::Member][&ResourceDimension::ScriptBytes].model(),
        ResourceModel::Affine {
            base: 102,
            per_ash_input: 0,
            per_sponsor_input: 0,
            per_sponsor_change: 0,
        },
    );
    for ash_inputs in [2, 3, 4] {
        assert_eq!(
            bundle
                .constructor()
                .leaf(LeafRole::Member { ash_inputs })
                .expect("a member leaf")
                .resources()
                .charged(ResourceDimension::ScriptBytes),
            Some(102),
        );
    }

    assert_eq!(bundle.total_script_bytes(), 5169);
    assert_eq!(bundle.relocations().len(), 103);
}
