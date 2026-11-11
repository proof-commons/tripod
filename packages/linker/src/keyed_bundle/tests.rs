use super::*;
use crate::keyed_reference::KeyedParameterValue;
use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId as O},
};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_plan::{PlacementSearchLimits, plan},
};
use tapscript::keyed_bundle::{KeyedBundleSelection, KeyedEmissionBundle, emit_keyed_bundle};
use tapscript::keyed_constructor::KeyedLeafRecipe;
use target_elements::reviewed_elements_tapscript;
use tracing_subscriber::prelude::*;

fn fixture(scope: Vec<K>, key: [u8; 32]) -> KeyedEmissionBundle {
    let manifest = KeyedManifest::contract().expect("manifest");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("derive")
        .validate(&manifest)
        .expect("validate");
    let states = std::num::NonZeroU64::new(1_000_000).expect("positive");
    let candidates = std::num::NonZeroU64::new(100_000).expect("positive");
    let input = bind_input(
        &manifest,
        source,
        scope.into_iter().map(O::Keyed).collect(),
        AnalysisPolicy::strict(ProofSearchLimits::new(states, candidates)),
    )
    .expect("bind");
    let plan = plan(&input, PlacementSearchLimits::new(states, candidates)).expect("plan");
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
    emit_keyed_bundle(
        &reviewed_elements_tapscript().expect("target"),
        &plan,
        selection,
        KeyedLeafRecipe::new(key),
    )
    .expect("emit")
}

#[test]
fn constructor_offers_refuse_each_coordinate_beside_control() {
    let target = reviewed_elements_tapscript().expect("target");
    let emission = fixture(vec![K::Source, K::Merge], [42; 32]);
    let control = keyed_parameter_offers(&target, KeyedLeafRecipe::new([42; 32]));
    let check = |subject: Vec<KeyedParameterOffer>, refusal| {
        assert!(link_keyed_bundle(&target, &emission, &control).is_ok());
        assert_eq!(
            link_keyed_bundle(&target, &emission, &subject),
            Err(refusal)
        );
    };
    for (i, row) in control.iter().enumerate() {
        let KeyedParameter::Constructor(field) = row.parameter else {
            panic!("constructor")
        };
        let mut subject = control.clone();
        subject.remove(i);
        check(subject, KeyedLinkRefusal::MissingConstructor(field));
        let mut subject = control.clone();
        subject.push(row.clone());
        check(subject, KeyedLinkRefusal::DuplicateConstructor(field));
        let mut subject = control.clone();
        subject[i].value = match field {
            KeyedConstructorField::Recipe => {
                KeyedParameterValue::Recipe(KeyedLeafRecipe::new([43; 32]))
            }
            KeyedConstructorField::TargetVersion => KeyedParameterValue::TargetVersion(0),
        };
        let refusal = match field {
            KeyedConstructorField::Recipe => KeyedLinkRefusal::RecipeMismatch,
            KeyedConstructorField::TargetVersion => KeyedLinkRefusal::TargetMismatch,
        };
        check(subject, refusal);
    }
    let mut subject = control.clone();
    subject.push(KeyedParameterOffer {
        parameter: KeyedParameter::Extension(17),
        value: KeyedParameterValue::TargetVersion(0),
    });
    check(subject, KeyedLinkRefusal::UnknownParameter(17));
    for reference in [
        KeyedReference::BirthCurrentInput,
        KeyedReference::ConsumedOutpoint,
        KeyedReference::ContinuationCurrentInput,
        KeyedReference::CurrentProgram,
        KeyedReference::SuccessorProgram,
    ] {
        let mut subject = control.clone();
        subject.push(KeyedParameterOffer {
            parameter: KeyedParameter::Introspection(reference),
            value: KeyedParameterValue::TargetVersion(0),
        });
        check(
            subject,
            KeyedLinkRefusal::IntrospectionSubstitution(reference),
        );
    }
}

#[test]
fn instruction_mutants_refuse_the_complete_leaf_grammar() {
    let emission = fixture(vec![K::Source], [42; 32]);
    let material = &emission.project().successor;
    let instructions = material.program().instructions();
    let key = material.recipe().key();
    for i in 0..instructions.len() {
        let mut subject = instructions.to_vec();
        subject[i] = tapscript::TapscriptInstruction::Opcode(
            if instructions[i] == tapscript::TapscriptInstruction::Opcode(OpcodeId::Drop) {
                OpcodeId::EqualVerify
            } else {
                OpcodeId::Drop
            },
        );
        assert!(inventory(instructions, key).is_ok());
        assert_eq!(
            inventory(&subject, key),
            Err(KeyedLinkRefusal::InstructionMismatch { instruction: i })
        );
    }
    let mut subject = instructions.to_vec();
    subject.push(tapscript::TapscriptInstruction::Opcode(OpcodeId::Drop));
    assert_eq!(
        inventory(&subject, key),
        Err(KeyedLinkRefusal::InstructionMismatch { instruction: 16 })
    );
    assert!(inventory(instructions, key).is_ok());
    assert_eq!(
        inventory(&instructions[..15], key),
        Err(KeyedLinkRefusal::InstructionMismatch { instruction: 15 })
    );
}

#[derive(Clone, Default)]
struct Capture(std::sync::Arc<std::sync::Mutex<Vec<String>>>);
impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Capture {
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        struct Fields(String);
        impl tracing::field::Visit for Fields {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                use std::fmt::Write as _;
                write!(&mut self.0, " {}={value:?}", field.name()).expect("string");
            }
        }
        let mut fields = Fields(format!(
            "{} {}",
            event.metadata().target(),
            event.metadata().level()
        ));
        event.record(&mut fields);
        self.0.lock().expect("capture").push(fields.0);
    }
    fn on_new_span(
        &self,
        attributes: &tracing::span::Attributes<'_>,
        _: &tracing::span::Id,
        _: tracing_subscriber::layer::Context<'_, S>,
    ) {
        self.0
            .lock()
            .expect("capture")
            .push(format!("span {}", attributes.metadata().name()));
    }
}

#[test]
fn keyed_link_has_one_completion_per_return() {
    const CHILD: &str = "TRIPOD_KEYED_LINK_CAPTURE";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "keyed_bundle::tests::keyed_link_has_one_completion_per_return",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .expect("child");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line.starts_with("test result: ok. 1 passed; 0 failed;"))
        );
        return;
    }
    let target = reviewed_elements_tapscript().expect("target");
    let emission = fixture(vec![K::Merge], [42; 32]);
    let offers = keyed_parameter_offers(&target, KeyedLeafRecipe::new([42; 32]));
    for subject in [offers, Vec::new()] {
        let expected = link_keyed_bundle(&target, &emission, &subject);
        let capture = Capture::default();
        let subscriber = tracing_subscriber::registry().with(capture.clone());
        let actual = tracing::subscriber::with_default(subscriber, || {
            tracing::callsite::rebuild_interest_cache();
            link_keyed_bundle(&target, &emission, &subject)
        });
        assert_eq!(actual, expected);
        let records = capture.0.lock().expect("capture");
        assert_eq!(records.len(), 2, "{records:?}");
        assert_eq!(records[0], "span link_keyed_bundle");
        assert!(records[1].contains("linker::keyed_bundle DEBUG"));
        assert!(records[1].contains("family=\"keyed\""));
        match expected {
            Ok(_) => {
                assert!(records[1].contains("keyed link completed"));
                assert!(records[1].contains("outcome=\"completed\""));
                for field in [
                    "constructor_count=2",
                    "leaf_count=1",
                    "literal_count=4",
                    "reference_count=5",
                    "relocation_count=0",
                    "outstanding_obligation_count=3",
                ] {
                    assert!(records[1].contains(field), "{records:?}");
                }
            }
            Err(error) => {
                assert!(records[1].contains("keyed link refused"));
                assert!(records[1].contains("outcome=\"refused\""));
                assert!(records[1].contains(error.refusal_kind()));
            }
        }
        drop(records);
    }
}
