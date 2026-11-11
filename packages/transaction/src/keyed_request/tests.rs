use super::*;
use crate::keyed_inputs::{KeyedInputRole, UnsupportedParentForm};
use crate::keyed_taproot::{KeyedAuthenticationRefusal, commit_keyed_bundle};
use crate::tests::trace_tests::{capture, run_isolated_capture_test};
use crate::{
    AssetField, AssetId, InputWitness, NonceField, TargetInput, TargetOutput, TargetTransaction,
    Txid, ValueField,
};
use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId as O},
};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_plan::{PlacementSearchLimits, plan},
};
use linker::keyed_bundle::LinkedKeyedBundle;
use sha2::{Digest, Sha256};
use tapscript::keyed_bundle::{KeyedBundleSelection, emit_keyed_bundle};
use target_elements::reviewed_elements_tapscript;
fn linked(key: [u8; 32], source: bool) -> LinkedKeyedBundle {
    linked_scope(
        key,
        if source {
            vec![O::Keyed(K::Source)]
        } else {
            vec![O::Keyed(K::Merge)]
        },
    )
}
fn linked_scope(key: [u8; 32], scope: Vec<O>) -> LinkedKeyedBundle {
    let target = reviewed_elements_tapscript().expect("target");
    let manifest = KeyedManifest::contract().expect("manifest");
    let declaration = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("derive")
        .validate(&manifest)
        .expect("validate");
    let limit = std::num::NonZeroU64::new(100_000).expect("positive");
    let input = bind_input(
        &manifest,
        declaration,
        scope,
        AnalysisPolicy::strict(ProofSearchLimits::new(limit, limit)),
    )
    .expect("bind");
    let plan = plan(&input, PlacementSearchLimits::new(limit, limit)).expect("plan");
    let selection = KeyedBundleSelection {
        binding: plan.project().binding.clone(),
        scope: plan.project().scope.clone(),
        proof: 0,
        placements: plan.project().plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    };
    let recipe = KeyedLeafRecipe::new(key);
    let emission = emit_keyed_bundle(&target, &plan, selection, recipe).expect("emit");
    link_keyed_bundle(&target, &emission, &keyed_parameter_offers(&target, recipe)).expect("link")
}

fn parent(program: &[u8], salt: u8) -> Vec<u8> {
    let output = TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal([3; 32])),
        ValueField::Explicit(1000),
        NonceField::Null,
        program.to_vec(),
    );
    TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([salt; 32]), 0).unwrap(),
            123,
        )],
        vec![output.clone(), output],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode()
}
fn offer<'capability>(
    parent: &[u8],
    input_index: u32,
    role: KeyedInputRole,
) -> KeyedInputOffer<'_, 'capability> {
    let tx = TargetTransaction::decode(parent).unwrap();
    let txid =
        Txid::from_internal(Sha256::digest(Sha256::digest(tx.encode_without_witness())).into());
    KeyedInputOffer {
        input_index,
        parent,
        outpoint: Outpoint::new(txid, 0).unwrap(),
        sequence: 123,
        flags: 0,
        role,
        capability: None,
    }
}
#[test]
fn serialized_keyed_requests_preserve_members_collateral_and_permutations() {
    let target = reviewed_elements_tapscript().unwrap();
    let merge = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let source = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let a = parent(merge.output_program(), 1);
    let b = parent(merge.output_program(), 2);
    let collateral = parent(&[0x51], 3);
    let mut offers = vec![
        offer(&collateral, 0, KeyedInputRole::Collateral),
        offer(&a, 1, KeyedInputRole::Member),
        offer(&b, 2, KeyedInputRole::Member),
    ];
    let request = keyed_merge_request(&target, [42; 32], &merge, &offers).unwrap();
    assert_eq!(request.members().members().len(), 2);
    assert_eq!(request.members().coverage().len(), 2);
    assert!(
        request
            .members()
            .coverage()
            .values()
            .all(|members| members.len() == 2)
    );
    assert_eq!(request.inputs()[0].flags(), 0);
    assert_eq!(request.inputs()[0].sequence(), 123);
    assert_eq!(request.inputs()[0].role(), KeyedInputRole::Collateral);
    assert_eq!(request.inputs()[0].parent().output().program(), &[0x51]);
    assert_eq!(request.inputs()[1].parent().parent_bytes(), a);
    assert!(request.inputs()[1].predecessor().is_some());
    assert_eq!(request.inputs()[1].role(), KeyedInputRole::Member);

    offers.swap(1, 2);
    offers[1].input_index = 1;
    offers[2].input_index = 2;
    let permuted = keyed_merge_request(&target, [42; 32], &merge, &offers).unwrap();
    let expected = offers
        .iter()
        .skip(1)
        .map(|o| {
            (
                o.input_index,
                InputOutpoint {
                    txid: *o.outpoint.txid().internal(),
                    output_index: o.outpoint.index(),
                },
            )
        })
        .collect::<BTreeSet<_>>();
    assert!(permuted.members().coverage().values().all(|members| {
        members
            .iter()
            .map(|m| (m.input_index, m.outpoint))
            .collect::<BTreeSet<_>>()
            == expected
    }));
    let source_request = keyed_source_request(&target, [42; 32], &source, &offers[..1]).unwrap();
    assert!(source_request.members().members().is_empty());
    assert_eq!(source_request.obligations().len(), 6);
    assert_eq!(
        source_request.commitment().output_program(),
        merge.output_program()
    );
}
#[test]
fn serialized_request_negatives_keep_their_boundaries() {
    let target = reviewed_elements_tapscript().unwrap();
    let merge = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let source = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let a = parent(merge.output_program(), 1);
    let other = commit_keyed_bundle(&target, &linked([43; 32], false)).unwrap();
    let b = parent(other.output_program(), 2);
    let good = || offer(&a, 0, KeyedInputRole::Member);
    assert!(keyed_merge_request(&target, [42; 32], &merge, &[good()]).is_ok());
    let fail = |subject: KeyedInputOffer<'_, '_>, expected| {
        assert!(keyed_merge_request(&target, [42; 32], &merge, &[good()]).is_ok());
        assert_eq!(
            keyed_merge_request(&target, [42; 32], &merge, &[subject]).unwrap_err(),
            expected
        );
    };
    let input = |refusal| KeyedRequestRefusal::Input { index: 0, refusal };
    fail(
        offer(&a, 0, KeyedInputRole::Collateral),
        input(KeyedInputRefusal::FalseCollateralRole),
    );
    fail(
        offer(&b, 0, KeyedInputRole::Member),
        input(KeyedInputRefusal::Authentication(
            KeyedAuthenticationRefusal::WrongProgram,
        )),
    );
    let mut later = good();
    later.outpoint = Outpoint::new(later.outpoint.txid(), 1).unwrap();
    fail(later, input(KeyedInputRefusal::LaterBornMember(1)));
    let mut absent = good();
    absent.outpoint = Outpoint::new(absent.outpoint.txid(), 2).unwrap();
    fail(absent, input(KeyedInputRefusal::MissingOutput(2)));
    for flag in [0x80, 0x40, 0xc0] {
        let mut flagged = good();
        flagged.flags = flag;
        fail(
            flagged,
            input(KeyedInputRefusal::UnsupportedSpendingInputForm(flag)),
        );
    }
    let mut concealed = good();
    concealed.role = KeyedInputRole::Collateral;
    concealed.outpoint = Outpoint::new(concealed.outpoint.txid(), 1).unwrap();
    fail(concealed, input(KeyedInputRefusal::FalseCollateralRole));
    let mut stale = good();
    stale.input_index = 1;
    fail(
        stale,
        KeyedRequestRefusal::StaleIndex {
            offered: 1,
            expected: 0,
        },
    );
    let mut hash = good();
    let computed = hash.outpoint.txid();
    let offered = Txid::from_internal([9; 32]);
    hash.outpoint = Outpoint::new(offered, 0).unwrap();
    fail(
        hash,
        input(KeyedInputRefusal::ParentHash { offered, computed }),
    );
    assert_eq!(
        keyed_merge_request(&target, [42; 32], &merge, &[]).unwrap_err(),
        KeyedRequestRefusal::EmptyMerge
    );
    let collateral = parent(&[0x51], 9);
    let source_control = || offer(&collateral, 0, KeyedInputRole::Collateral);
    assert!(keyed_source_request(&target, [42; 32], &source, &[source_control()]).is_ok());
    assert_eq!(
        keyed_merge_request(&target, [42; 32], &merge, &[source_control()]).unwrap_err(),
        KeyedRequestRefusal::EmptyMerge
    );
    assert_eq!(
        keyed_source_request(&target, [42; 32], &source, &[good()]).unwrap_err(),
        KeyedRequestRefusal::SourceConsumesMember
    );
    assert!(keyed_source_request(&target, [42; 32], &source, &[source_control()]).is_ok());
    assert_eq!(
        keyed_source_request(
            &target,
            [42; 32],
            &source,
            &[offer(&a, 0, KeyedInputRole::Collateral)]
        )
        .unwrap_err(),
        input(KeyedInputRefusal::FalseCollateralRole)
    );
    assert_eq!(
        keyed_merge_request(&target, [43; 32], &merge, &[good()]).unwrap_err(),
        KeyedRequestRefusal::Link(KeyedLinkRefusal::RecipeMismatch)
    );
    let second = offer(&a, 1, KeyedInputRole::Member);
    assert_eq!(
        keyed_merge_request(&target, [42; 32], &merge, &[good(), second]).unwrap_err(),
        KeyedRequestRefusal::DuplicateInput(good().outpoint)
    );
}
#[test]
fn second_other_key_member_refuses_while_first_still_passes() {
    let target = reviewed_elements_tapscript().unwrap();
    let merge = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let other = commit_keyed_bundle(&target, &linked([43; 32], false)).unwrap();
    let a = parent(merge.output_program(), 1);
    let b = parent(other.output_program(), 2);
    let own = parent(merge.output_program(), 2);
    let good = || offer(&a, 0, KeyedInputRole::Member);
    assert!(
        keyed_merge_request(
            &target,
            [42; 32],
            &merge,
            &[good(), offer(&own, 1, KeyedInputRole::Member)]
        )
        .is_ok()
    );
    // The second member alone changes; the original first member remains accepted.
    assert_eq!(
        keyed_merge_request(
            &target,
            [42; 32],
            &merge,
            &[good(), offer(&b, 1, KeyedInputRole::Member)]
        )
        .unwrap_err(),
        KeyedRequestRefusal::Input {
            index: 1,
            refusal: KeyedInputRefusal::Authentication(KeyedAuthenticationRefusal::WrongProgram)
        }
    );
    assert!(keyed_merge_request(&target, [42; 32], &merge, &[good()]).is_ok());
}
#[test]
fn unsupported_parent_flags_are_not_covenant_invalidity() {
    let target = reviewed_elements_tapscript().unwrap();
    let merge = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let a = parent(merge.output_program(), 1);
    assert!(
        keyed_merge_request(
            &target,
            [42; 32],
            &merge,
            &[offer(&a, 0, KeyedInputRole::Member)]
        )
        .is_ok()
    );
    // version(4), witness flag(1), count(1), txid(32), vout(4).
    for (flag, form) in [
        (0x80, UnsupportedParentForm::Issuance),
        (0x40, UnsupportedParentForm::PegIn),
    ] {
        let mut bytes = a.clone();
        bytes[41] |= flag;
        let mut subject = offer(&a, 0, KeyedInputRole::Member);
        subject.parent = &bytes;
        assert_eq!(
            keyed_merge_request(&target, [42; 32], &merge, &[subject]).unwrap_err(),
            KeyedRequestRefusal::Input {
                index: 0,
                refusal: KeyedInputRefusal::UnsupportedForm(form)
            }
        );
    }
}
#[test]
fn keyed_request_boundaries_trace_one_completion_or_refusal() {
    if run_isolated_capture_test(
        module_path!(),
        "keyed_request_boundaries_trace_one_completion_or_refusal",
    ) {
        return;
    }
    let target = reviewed_elements_tapscript().unwrap();
    for source in [true, false] {
        let c = commit_keyed_bundle(&target, &linked([42; 32], source)).unwrap();
        let a = parent(if source { &[0x51] } else { c.output_program() }, 1);
        for refused in [false, true] {
            let mut input = offer(
                &a,
                0,
                if source {
                    KeyedInputRole::Collateral
                } else {
                    KeyedInputRole::Member
                },
            );
            if refused {
                input.input_index = 2;
            }
            let (result, rows) = capture(|| {
                if source {
                    keyed_source_request(&target, [42; 32], &c, &[input]).map(|_| ())
                } else {
                    keyed_merge_request(&target, [42; 32], &c, &[input]).map(|_| ())
                }
            });
            assert_eq!(result.is_err(), refused);
            let events = rows
                .iter()
                .filter(|r| r.kind == "event" && r.target == "transaction::keyed_request")
                .collect::<Vec<_>>();
            assert_eq!(events.len(), 1);
            let action = if source { "source" } else { "merge" };
            let outcome = if refused { "refused" } else { "completed" };
            assert_eq!(
                events[0].fields["message"].trim_matches('"'),
                format!("keyed {action} request {outcome}")
            );
            assert_eq!(events[0].fields["outcome"], outcome);
            assert_eq!(events[0].level, tracing::Level::DEBUG);
            if refused {
                assert_eq!(events[0].fields["refusal_kind"], "stale_index");
            } else {
                assert!(!events[0].fields.contains_key("refusal_kind"));
            }
            let spans = rows
                .iter()
                .filter(|r| r.kind == "span" && r.target == "transaction::keyed_request")
                .collect::<Vec<_>>();
            assert_eq!(spans.len(), 1);
            assert_eq!(spans[0].name, format!("keyed_{action}_request"));
            assert_eq!(spans[0].level, tracing::Level::DEBUG);
            assert_eq!(spans[0].fields["input_count"], "1");
            assert_eq!(spans[0].fields["family"], "keyed");
        }
    }
}

fn sealed_surface(source: &str, names: &[&str]) -> bool {
    for name in names {
        let marker = format!("pub struct {name}");
        let Some(start) = source.find(&marker) else {
            return false;
        };
        let Some(end) = source[start..].find("\n}") else {
            return false;
        };
        let body = &source[start..start + end];
        if body
            .lines()
            .skip(1)
            .any(|line| line.trim_start().starts_with("pub "))
        {
            return false;
        }
    }
    !source.contains("pub fn new(")
        && !source.contains("pub const fn new(")
        && !source.contains("&mut self")
        && !source.contains("DerefMut for")
        && !source.contains("AsMut<")
        && !source.contains("impl Default for")
}

#[test]
fn keyed_request_and_parent_receipt_surfaces_remain_sealed() {
    let inputs = include_str!("../keyed_inputs.rs");
    let requests = include_str!("../keyed_request.rs");
    assert!(sealed_surface(
        inputs,
        &["BoundParentOutput", "BoundKeyedInput"]
    ));
    assert!(sealed_surface(
        requests,
        &["KeyedSourceRequest", "KeyedMergeRequest"]
    ));
    assert!(!sealed_surface(
        &inputs.replace("    parent: Vec<u8>,", "    pub parent: Vec<u8>,"),
        &["BoundParentOutput", "BoundKeyedInput"]
    ));
    assert!(!sealed_surface(
        &format!("{requests}\npub fn new(bytes: [u8; 32]) {{}}"),
        &["KeyedSourceRequest", "KeyedMergeRequest"]
    ));
    for source in [inputs, requests] {
        assert!(!source.contains("pub(crate) struct Bound"));
        assert!(!source.contains("pub fn revalidate"));
    }
}

#[test]
fn keyed_request_refusal_names_are_snake_case() {
    for source in [
        include_str!("../keyed_inputs.rs"),
        include_str!("../keyed_request.rs"),
    ] {
        let table = crate::tests::trace_tests::trace_kind_table(source);
        assert!(!table.is_empty());
        for name in table.values() {
            assert!(name.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'));
        }
    }
}

#[test]
fn a_changed_script_and_second_later_born_member_refuse_beside_the_first_member() {
    let target = reviewed_elements_tapscript().unwrap();
    let c = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let a = parent(c.output_program(), 1);
    let b = parent(c.output_program(), 2);
    let good = || {
        vec![
            offer(&a, 0, KeyedInputRole::Member),
            offer(&b, 1, KeyedInputRole::Member),
        ]
    };
    assert!(keyed_merge_request(&target, [42; 32], &c, &good()).is_ok());
    let mut later = good();
    later[1].outpoint = Outpoint::new(later[1].outpoint.txid(), 1).unwrap();
    assert_eq!(
        keyed_merge_request(&target, [42; 32], &c, &later).unwrap_err(),
        KeyedRequestRefusal::Input {
            index: 1,
            refusal: KeyedInputRefusal::LaterBornMember(1)
        }
    );
    assert!(keyed_merge_request(&target, [42; 32], &c, &good()[..1]).is_ok());
    let mut script = c.output_program().to_vec();
    script[2] ^= 1;
    let wrong = parent(&script, 2);
    assert_eq!(
        keyed_merge_request(
            &target,
            [42; 32],
            &c,
            &[
                offer(&a, 0, KeyedInputRole::Member),
                offer(&wrong, 1, KeyedInputRole::Member)
            ]
        )
        .unwrap_err(),
        KeyedRequestRefusal::Input {
            index: 1,
            refusal: KeyedInputRefusal::Authentication(KeyedAuthenticationRefusal::WrongProgram)
        }
    );
    assert!(keyed_merge_request(&target, [42; 32], &c, &good()).is_ok());
}

#[test]
fn request_scope_refusals_and_joint_merge_have_accepted_twins() {
    let target = reviewed_elements_tapscript().unwrap();
    let source = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let merge = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let joint = commit_keyed_bundle(
        &target,
        &linked_scope([42; 32], vec![O::Keyed(K::Source), O::Keyed(K::Merge)]),
    )
    .unwrap();
    let bytes = parent(merge.output_program(), 1);
    let member = || offer(&bytes, 0, KeyedInputRole::Member);
    assert!(keyed_merge_request(&target, [42; 32], &merge, &[member()]).is_ok());
    assert_eq!(
        keyed_merge_request(&target, [42; 32], &source, &[member()]).unwrap_err(),
        KeyedRequestRefusal::Scope
    );
    assert!(keyed_source_request(&target, [42; 32], &source, &[]).is_ok());
    assert_eq!(
        keyed_source_request(&target, [42; 32], &merge, &[]).unwrap_err(),
        KeyedRequestRefusal::Scope
    );
    let accepted = keyed_merge_request(&target, [42; 32], &joint, &[member()]).unwrap();
    assert_eq!(accepted.members().coverage().len(), 2);
    assert_eq!(accepted.members().evidence().len(), 7);
    assert!(keyed_source_request(&target, [42; 32], &source, &[]).is_ok());
    assert_eq!(
        keyed_source_request(&target, [42; 32], &joint, &[]).unwrap_err(),
        KeyedRequestRefusal::Scope
    );
    // The ABI position is pinned to the closed instruction-10 minimal zero push.
    let literal = joint
        .linked()
        .closure()
        .literals()
        .iter()
        .find(|l| l.kind == linker::keyed_reference::KeyedLiteralKind::SuccessorOutputIndex)
        .unwrap();
    assert_eq!(literal.instruction, 10);
    assert_eq!(literal.bytes, [] as [u8; 0]);
    let decoded = literal.bytes.first().copied().map_or(0, u32::from);
    assert_eq!(crate::keyed_abi::KEYED_SUCCESSOR_OUTPUT, decoded);
}

#[test]
fn every_nonzero_spending_flag_is_refused_for_both_roles() {
    let target = reviewed_elements_tapscript().unwrap();
    let c = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let member = parent(c.output_program(), 1);
    let collateral = parent(&[0x51], 2);
    for (position, role) in [(0, KeyedInputRole::Member), (1, KeyedInputRole::Collateral)] {
        for flag in 1..=u8::MAX {
            let mut inputs = vec![
                offer(&member, 0, KeyedInputRole::Member),
                offer(&collateral, 1, KeyedInputRole::Collateral),
            ];
            assert!(keyed_merge_request(&target, [42; 32], &c, &inputs).is_ok());
            assert_eq!(inputs[position].role, role);
            inputs[position].flags = flag;
            let refusal = if flag & !0xc0 != 0 {
                KeyedInputRefusal::InvalidOutpointFlags(flag)
            } else {
                KeyedInputRefusal::UnsupportedSpendingInputForm(flag)
            };
            assert_eq!(
                keyed_merge_request(&target, [42; 32], &c, &inputs).unwrap_err(),
                KeyedRequestRefusal::Input {
                    index: u32::try_from(position).unwrap(),
                    refusal
                }
            );
        }
    }
}

#[test]
fn collateral_parent_issuance_is_an_unsupported_form_too() {
    let target = reviewed_elements_tapscript().unwrap();
    let c = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let bytes = parent(&[0x51], 1);
    assert!(
        keyed_source_request(
            &target,
            [42; 32],
            &c,
            &[offer(&bytes, 0, KeyedInputRole::Collateral)]
        )
        .is_ok()
    );
    let mut unsupported = bytes.clone();
    unsupported[41] |= 0x80;
    let mut input = offer(&bytes, 0, KeyedInputRole::Collateral);
    input.parent = &unsupported;
    assert_eq!(
        keyed_source_request(&target, [42; 32], &c, &[input]).unwrap_err(),
        KeyedRequestRefusal::Input {
            index: 0,
            refusal: KeyedInputRefusal::UnsupportedForm(UnsupportedParentForm::Issuance)
        }
    );
}
