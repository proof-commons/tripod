use super::*;
use crate::tests::trace_tests::{capture, run_isolated_capture_test, trace_kind_table};
use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId as O},
};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_plan::{PlacementSearchLimits, plan},
};
use linker::{keyed_bundle::link_keyed_bundle, keyed_reference::keyed_parameter_offers};
use tapscript::keyed_bundle::{KeyedBundleSelection, emit_keyed_bundle};
use target_elements::reviewed_elements_tapscript;

fn linked(key: [u8; 32]) -> LinkedKeyedBundle {
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
        vec![O::Keyed(K::Source), O::Keyed(K::Merge)],
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

#[test]
fn keyed_commitment_and_authentication_refuse_one_coordinate_beside_controls() {
    let target = reviewed_elements_tapscript().expect("target");
    let bundle = linked([42; 32]);
    let c = commit_keyed_bundle(&target, &bundle).expect("commit");
    assert_eq!(c.linked(), &bundle);
    assert_eq!(c.leaf(), bundle.bytes());
    assert_eq!(
        c.paths(),
        [
            KeyedSpendPath::ScriptWithBothClauses,
            KeyedSpendPath::ConditionalNumsKeyPath
        ]
    );
    let check = |leaf: &[u8], control: &[u8], error| {
        assert_eq!(authenticate_keyed_path(&c, c.leaf(), c.control()), Ok(()));
        assert_eq!(authenticate_keyed_path(&c, leaf, control), Err(error));
    };
    let mut leaf = c.leaf().to_vec();
    leaf[1] ^= 1;
    check(&leaf, c.control(), KeyedAuthenticationRefusal::WrongLeaf);
    let mut control = *c.control();
    control[0] ^= 2;
    check(c.leaf(), &control, KeyedAuthenticationRefusal::WrongVersion);
    let mut control = *c.control();
    control[0] ^= 1;
    check(c.leaf(), &control, KeyedAuthenticationRefusal::WrongParity);
    let mut control = *c.control();
    control[1] ^= 1;
    check(c.leaf(), &control, KeyedAuthenticationRefusal::WrongControl);
    check(
        c.leaf(),
        &c.control()[..32],
        KeyedAuthenticationRefusal::WrongControl,
    );
    assert_eq!(
        authenticate_keyed_predecessor(&c, c.output_program()),
        Ok(())
    );
    let accepted =
        authenticate_keyed_predecessor_core(&c, c.output_program()).expect("internal equality");
    assert!(std::ptr::eq(
        std::ptr::from_ref(accepted.commitment()),
        std::ptr::from_ref(&c)
    ));
    assert_eq!(
        crate::keyed_curve::apply_tweak(c.internal_key(), *c.tweak()),
        KeyedTweakOutcome::OutputKey {
            key: *c.output_key(),
            parity: c.parity()
        }
    );
    let mut wrong_version = c.output_program().to_vec();
    wrong_version[0] = 0;
    let mut trailing = c.output_program().to_vec();
    trailing.push(0);
    for subject in [wrong_version, c.output_key().to_vec(), trailing] {
        assert_eq!(
            authenticate_keyed_predecessor(&c, c.output_program()),
            Ok(())
        );
        assert_eq!(
            authenticate_keyed_predecessor(&c, &subject),
            Err(KeyedAuthenticationRefusal::WrongProgram)
        );
    }
    let other = commit_keyed_bundle(&target, &linked([43; 32])).expect("other key valid");
    assert!(authenticate_keyed_predecessor(&other, other.output_program()).is_ok());
    assert_eq!(
        authenticate_keyed_predecessor(&c, other.output_program()),
        Err(KeyedAuthenticationRefusal::WrongProgram)
    );
    // A changed root produces another commitment program, not a mutated sealed value.
    let mut root = *c.root();
    root[0] ^= 1;
    let KeyedTweakOutcome::OutputKey { key, .. } = KeyedCurve.output_key(c.internal_key(), &root)
    else {
        panic!("finite test point")
    };
    let wrong_root_program = witness_program_script(&target, 1, &key).expect("program");
    assert!(authenticate_keyed_predecessor(&c, c.output_program()).is_ok());
    assert_eq!(
        authenticate_keyed_predecessor(&c, &wrong_root_program),
        Err(KeyedAuthenticationRefusal::WrongProgram)
    );
}

struct CurveSubject {
    lifts: bool,
    outcome: KeyedTweakOutcome,
}
impl KeyedCurveCapability for CurveSubject {
    fn internal_key_is_a_point(&self, _: &[u8; 32]) -> bool {
        self.lifts
    }
    fn output_key(&self, _: &[u8; 32], _: &[u8; 32]) -> KeyedTweakOutcome {
        self.outcome
    }
}

#[test]
fn constructor_curve_refusals_are_not_authentication_or_execution() {
    let target = reviewed_elements_tapscript().expect("target");
    let bundle = linked([42; 32]);
    for (lifts, outcome, error) in [
        (
            false,
            KeyedTweakOutcome::InternalKeyNotAPoint,
            KeyedCommitmentRefusal::InternalKeyNotAPoint,
        ),
        (
            true,
            KeyedTweakOutcome::InternalKeyNotAPoint,
            KeyedCommitmentRefusal::InternalKeyNotAPoint,
        ),
        (
            true,
            KeyedTweakOutcome::TweakAboveGroupOrder,
            KeyedCommitmentRefusal::TweakAboveGroupOrder,
        ),
        (
            true,
            KeyedTweakOutcome::TweakedPointIsIdentity,
            KeyedCommitmentRefusal::TweakedPointIsIdentity,
        ),
    ] {
        assert!(commit_core(&target, &bundle, &KeyedCurve).is_ok());
        assert_eq!(
            commit_core(&target, &bundle, &CurveSubject { lifts, outcome }),
            Err(error)
        );
    }
}

fn captured<T>(name: &str, expected_kind: Option<&str>, call: impl FnOnce() -> T) -> T {
    let (result, rows) = capture(call);
    assert_eq!(rows.len(), 3, "{rows:?}");
    for (row, kind) in rows.iter().zip(["span", "event", "close"]) {
        assert_eq!(row.kind, kind);
        assert_eq!(row.target, "transaction::keyed_taproot");
        assert_eq!(row.level, tracing::Level::DEBUG);
    }
    assert_eq!(rows[0].name, name);
    assert_eq!(rows[2].name, name);
    assert_eq!(
        rows[0].fields.get("family").map(String::as_str),
        Some("keyed")
    );
    let fields = &rows[1].fields;
    assert_eq!(fields.get("family").map(String::as_str), Some("keyed"));
    assert_eq!(
        fields.get("refusal_kind").map(String::as_str),
        expected_kind
    );
    assert_eq!(
        fields.get("outcome").map(String::as_str),
        Some(if expected_kind.is_some() {
            "refused"
        } else {
            "completed"
        })
    );
    let message = match (name, expected_kind.is_some()) {
        ("commit_keyed_bundle", false) => "keyed commitment completed",
        ("commit_keyed_bundle", true) => "keyed commitment refused",
        ("authenticate_keyed_predecessor", false) => "keyed predecessor authenticated",
        ("authenticate_keyed_predecessor", true) => "keyed predecessor refused",
        ("authenticate_keyed_path", false) => "keyed path authenticated",
        ("authenticate_keyed_path", true) => "keyed path refused",
        _ => panic!("unknown boundary"),
    };
    assert_eq!(
        fields.get("message").map(|s| s.trim_matches('"')),
        Some(message)
    );
    for count in ["script_path_count", "key_path_count"] {
        assert_eq!(
            fields.get(count).map(String::as_str),
            if name == "commit_keyed_bundle" && expected_kind.is_none() {
                Some("1")
            } else {
                None
            }
        );
    }
    result
}

#[test]
fn keyed_refusal_trace_kinds_are_snake_case() {
    let table = trace_kind_table(include_str!("../keyed_taproot.rs"));
    assert_eq!(table.len(), 11);
}

#[test]
fn keyed_public_boundaries_emit_once_and_capture_runs_exactly_one_test() {
    if run_isolated_capture_test(
        module_path!(),
        "keyed_public_boundaries_emit_once_and_capture_runs_exactly_one_test",
    ) {
        return;
    }
    let target = reviewed_elements_tapscript().expect("target");
    let bundle = linked([42; 32]);
    let c = commit_keyed_bundle(&target, &bundle).expect("commit");
    assert_eq!(
        captured("commit_keyed_bundle", None, || commit_keyed_bundle(
            &target, &bundle
        )),
        Ok(c.clone())
    );
    let subject = CurveSubject {
        lifts: true,
        outcome: KeyedTweakOutcome::TweakAboveGroupOrder,
    };
    assert_eq!(
        captured(
            "commit_keyed_bundle",
            Some("tweak_above_group_order"),
            || commit_with_curve(&target, &bundle, &subject)
        ),
        Err(KeyedCommitmentRefusal::TweakAboveGroupOrder)
    );
    for program in [c.output_program(), &[][..]] {
        let expected = authenticate_keyed_predecessor(&c, program);
        let kind = expected
            .as_ref()
            .err()
            .map(KeyedAuthenticationRefusal::refusal_kind);
        assert_eq!(
            captured("authenticate_keyed_predecessor", kind, || {
                authenticate_keyed_predecessor(&c, program)
            }),
            expected
        );
    }
    for leaf in [c.leaf(), &[][..]] {
        let expected = authenticate_keyed_path(&c, leaf, c.control());
        let kind = expected
            .as_ref()
            .err()
            .map(KeyedAuthenticationRefusal::refusal_kind);
        assert_eq!(
            captured("authenticate_keyed_path", kind, || authenticate_keyed_path(
                &c,
                leaf,
                c.control()
            )),
            expected
        );
    }
}

fn immutable_artifact(source: &str, name: &str, producer: &str) -> bool {
    let text = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let flat = text.split_whitespace().collect::<String>();
    let marker = format!("pubstruct{name}");
    let Some(body) = flat
        .split(&marker)
        .nth(1)
        .and_then(|s| s.split_once('{'))
        .and_then(|(_, s)| s.split('}').next())
    else {
        return false;
    };
    if body.contains("pub") || flat.contains("engine_") || flat.contains("Default") {
        return false;
    }
    for implementation in flat.split("impl").skip(1) {
        let head = implementation.split('{').next().unwrap_or(implementation);
        if head.contains(&format!("for{name}"))
            && ["From<", "TryFrom<", "DerefMut", "AsMut<", "BorrowMut<"]
                .iter()
                .any(|word| head.contains(word))
        {
            return false;
        }
    }
    let mut own_impl = false;
    let mut depth = 0_i32;
    for section in text.split_inclusive(['{', '}']) {
        let head = section.trim();
        if head.contains(&format!("impl {name}")) {
            own_impl = true;
            depth = 0;
        }
        if let Some(start) = head.find("pub ") {
            let signature = head[start..].split_whitespace().collect::<String>();
            if signature.contains("fn") {
                let returned = signature.split("->").nth(1).unwrap_or("");
                let creates = (returned.contains(name) && !returned.starts_with('&'))
                    || (own_impl && returned.contains("Self") && !returned.starts_with('&'));
                if signature.contains("&mut")
                    || signature.contains("_mut(")
                    || (creates && !signature.contains(&format!("fn{producer}(")))
                {
                    return false;
                }
            }
        }
        if section.ends_with('{') {
            depth += 1;
        }
        if section.ends_with('}') {
            depth -= 1;
        }
        if depth == 0 {
            own_impl = false;
        }
    }
    true
}

#[test]
fn keyed_commitment_values_are_sealed_and_read_only() {
    let source = include_str!("../keyed_taproot.rs");
    for (name, producer, field) in [
        (
            "KeyedCommitment",
            "commit_keyed_bundle",
            "linked: LinkedKeyedBundle,",
        ),
        (
            "AuthenticatedKeyedPredecessor",
            "authenticate_keyed_predecessor",
            "commitment: &'a KeyedCommitment,",
        ),
    ] {
        assert!(immutable_artifact(source, name, producer));
        for addition in [
            format!("impl {name} {{ pub fn new(x: [u8; 32]) -> Self {{ todo!() }} }}"),
            format!("impl {name} {{ pub fn mutate(&mut self) {{}} }}"),
            format!("impl Default for {name} {{}}"),
            format!("impl From<()> for {name} {{}}"),
            format!("impl DerefMut for {name} {{}}"),
        ] {
            assert!(!immutable_artifact(
                &format!("{source}\n{addition}"),
                name,
                producer
            ));
        }
        assert!(!immutable_artifact(
            &source.replace(field, &format!("pub {field}")),
            name,
            producer
        ));
    }
}
