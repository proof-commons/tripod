//! Scoped tracing assertions for target definition and binding operations.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;

use crate::definition::{
    TargetContractVersion, TargetDefinition, TargetDefinitionParts, reviewed_definition_with,
    reviewed_elements_tapscript, validate_as_reviewed_elements, validate_target_definition,
};
use crate::deployment::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding,
    validate_development_binding, validate_reviewed_development_binding,
};
use crate::error::TargetError;
use crate::opcode::LeafVersion;
use crate::resource::{PolicyResourceLimits, ResourceContract};

static CAPTURE_INSTALL_LOCK: Mutex<()> = Mutex::new(());
const CAPTURE_CHILD: &str = "TRIPOD_TARGET_ELEMENTS_CAPTURE_CHILD";

fn in_isolated_child(test_name: &str) -> bool {
    let full_name = format!("tests::tracing_tests::{test_name}");
    if std::env::var(CAPTURE_CHILD).as_deref() == Ok(full_name.as_str()) {
        return true;
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg(&full_name)
        .env(CAPTURE_CHILD, &full_name)
        .output()
        .expect("isolated capture test");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success()
            && stdout.lines().any(|line| line
                .trim_start()
                .starts_with("test result: ok. 1 passed; 0 failed;")),
        "isolated {full_name} failed with {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    );
    false
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Number(u64),
    Text(String),
}

#[derive(Default)]
struct Fields(BTreeMap<String, Value>);

impl Visit for Fields {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.0.insert(
            field.name().to_owned(),
            Value::Text(format!("{value:?}").trim_matches('"').to_owned()),
        );
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_owned(), Value::Number(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0
            .insert(field.name().to_owned(), Value::Text(value.to_owned()));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind {
    Span,
    Event,
    Close,
}

#[derive(Clone, Debug)]
struct Seen {
    kind: Kind,
    target: &'static str,
    level: Level,
    name: &'static str,
    fields: BTreeMap<String, Value>,
}

#[derive(Clone)]
struct Recorder(Arc<Mutex<Vec<Seen>>>);

impl<S> Layer<S> for Recorder
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        let metadata = attrs.metadata();
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        self.0.lock().expect("capture lock").push(Seen {
            kind: Kind::Span,
            target: metadata.target(),
            level: *metadata.level(),
            name: metadata.name(),
            fields: fields.0,
        });
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let mut fields = Fields::default();
        event.record(&mut fields);
        self.0.lock().expect("capture lock").push(Seen {
            kind: Kind::Event,
            target: metadata.target(),
            level: *metadata.level(),
            name: metadata.name(),
            fields: fields.0,
        });
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).expect("closing span");
        let metadata = span.metadata();
        self.0.lock().expect("capture lock").push(Seen {
            kind: Kind::Close,
            target: metadata.target(),
            level: *metadata.level(),
            name: metadata.name(),
            fields: BTreeMap::new(),
        });
    }
}

fn captured<T>(call: impl FnOnce() -> T) -> (T, Vec<Seen>) {
    let _install_guard = CAPTURE_INSTALL_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let records = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(Recorder(Arc::clone(&records)));
    let value = tracing::subscriber::with_default(subscriber, || {
        tracing::callsite::rebuild_interest_cache();
        call()
    });
    let seen = records.lock().expect("capture lock").clone();
    (value, seen)
}

fn assert_boundary(
    seen: &[Seen],
    target: &str,
    span_name: &str,
    message: &str,
    outcome: &str,
    refusal_kind: Option<&str>,
    defect_count: Option<u64>,
) {
    let spans = seen
        .iter()
        .filter(|entry| entry.kind == Kind::Span)
        .collect::<Vec<_>>();
    let closes = seen
        .iter()
        .filter(|entry| entry.kind == Kind::Close)
        .collect::<Vec<_>>();
    let events = seen
        .iter()
        .filter(|entry| entry.kind == Kind::Event)
        .collect::<Vec<_>>();
    assert_eq!(spans.len(), 1, "{seen:?}");
    assert_eq!(closes.len(), 1, "{seen:?}");
    assert_eq!(events.len(), 1, "{seen:?}");
    assert_eq!(spans[0].target, target);
    assert_eq!(spans[0].level, Level::DEBUG);
    assert_eq!(spans[0].name, span_name);
    assert_eq!(closes[0].name, span_name);
    assert_eq!(events[0].target, target);
    assert_eq!(events[0].level, Level::DEBUG);
    assert_eq!(
        events[0].fields.get("message"),
        Some(&Value::Text(message.to_owned()))
    );
    assert_eq!(
        events[0].fields.get("outcome"),
        Some(&Value::Text(outcome.to_owned()))
    );
    let expected_kind = refusal_kind.map(|kind| Value::Text(kind.to_owned()));
    assert_eq!(events[0].fields.get("refusal_kind"), expected_kind.as_ref());
    let expected_count = defect_count.map(Value::Number);
    assert_eq!(
        events[0].fields.get("defect_count"),
        expected_count.as_ref()
    );
    assert!(seen.iter().all(|entry| entry.level == Level::DEBUG));
}

fn reviewed_parts() -> TargetDefinitionParts {
    let reviewed = reviewed_elements_tapscript().expect("reviewed declaration");
    let source = reviewed.definition();
    TargetDefinitionParts {
        version: source.version(),
        execution_domain: source.execution_domain(),
        leaf_version: source.leaf_version(),
        opcodes: source.opcodes().clone(),
        encodings: source.encodings().clone(),
        pushes: source.pushes().clone(),
        authorization: source.authorization().clone(),
        confidential_values: source.confidential_values().clone(),
        issuance: source.issuance().clone(),
        resources: source.resources().clone(),
        capabilities: source.capabilities().clone(),
        evidence_requirements: source.evidence_requirements().clone(),
    }
}

#[test]
fn refusal_kind_vocabulary_is_snake_case() {
    let names = crate::error::REFUSAL_KIND_NAMES;
    for (index, &name) in names.iter().enumerate() {
        let mut segment_start = true;
        let valid = name.bytes().all(|byte| match byte {
            b'a'..=b'z' => {
                segment_start = false;
                true
            }
            b'0'..=b'9' if !segment_start => true,
            b'_' if !segment_start => {
                segment_start = true;
                true
            }
            _ => false,
        });
        assert!(valid && !segment_start, "invalid refusal kind: {name}");
        assert!(
            !names[..index].contains(&name),
            "duplicate refusal kind: {name}"
        );
    }
    assert!(names.contains(&"opcode_id_mismatch"));
    assert_eq!(
        TargetError::ReviewedDefinitionMismatch.refusal_kind(),
        "reviewed_definition_mismatch"
    );
}

#[test]
fn capture_rebuilds_interest_after_uncaptured_call_on_two_threads() {
    if !in_isolated_child("capture_rebuilds_interest_after_uncaptured_call_on_two_threads") {
        return;
    }
    let definition = TargetDefinition::new(reviewed_parts());
    let expected = validate_target_definition(definition.clone());
    assert!(expected.is_ok());

    let handles = [definition.clone(), definition].map(|definition| {
        std::thread::spawn(move || captured(|| validate_target_definition(definition)))
    });
    for handle in handles {
        let (actual, seen) = handle.join().expect("capture thread");
        assert_eq!(actual, expected);
        assert_boundary(
            &seen,
            "target_elements::definition",
            "validate_definition",
            "target definition checked",
            "completed",
            None,
            Some(0),
        );
    }
}

#[test]
fn definition_control_and_one_version_mutation_have_scoped_events() {
    if !in_isolated_child("definition_control_and_one_version_mutation_have_scoped_events") {
        return;
    }
    let control = TargetDefinition::new(reviewed_parts());
    let subject = TargetDefinition::new(TargetDefinitionParts {
        version: TargetContractVersion::V1,
        ..reviewed_parts()
    });
    let expected_control = validate_target_definition(control.clone());
    let expected_subject = validate_target_definition(subject.clone());
    assert_eq!(
        expected_subject,
        Err(vec![TargetError::UnsupportedTargetContractVersion {
            offered: 1
        }])
    );

    let (actual_control, control_seen) = captured(|| validate_target_definition(control));
    let (actual_subject, subject_seen) = captured(|| validate_target_definition(subject));
    assert_eq!(actual_control, expected_control);
    assert_eq!(actual_subject, expected_subject);
    assert_boundary(
        &control_seen,
        "target_elements::definition",
        "validate_definition",
        "target definition checked",
        "completed",
        None,
        Some(0),
    );
    assert_boundary(
        &subject_seen,
        "target_elements::definition",
        "validate_definition",
        "target definition checked",
        "refused",
        Some("unsupported_target_contract_version"),
        Some(1),
    );
    assert_eq!(
        control_seen[0].fields.get("contract_version"),
        Some(&Value::Number(2))
    );
    assert_eq!(
        subject_seen[0].fields.get("contract_version"),
        Some(&Value::Number(1))
    );
}

#[test]
fn reviewed_declaration_control_and_mutation_emit_only_the_outer_event() {
    if !in_isolated_child("reviewed_declaration_control_and_mutation_emit_only_the_outer_event") {
        return;
    }
    let expected_control = reviewed_elements_tapscript();
    let (actual_control, control_seen) = captured(reviewed_elements_tapscript);
    assert_eq!(actual_control, expected_control);
    assert_boundary(
        &control_seen,
        "target_elements::definition",
        "reviewed_definition",
        "reviewed definition built",
        "completed",
        None,
        Some(0),
    );

    let subject = TargetDefinition::new(TargetDefinitionParts {
        version: TargetContractVersion::V1,
        ..reviewed_parts()
    });
    let expected_subject = reviewed_definition_with(|| subject.clone());
    let (actual_subject, subject_seen) = captured(|| reviewed_definition_with(|| subject));
    assert_eq!(actual_subject, expected_subject);
    assert_eq!(
        actual_subject,
        Err(vec![TargetError::UnsupportedTargetContractVersion {
            offered: 1
        }])
    );
    assert_boundary(
        &subject_seen,
        "target_elements::definition",
        "reviewed_definition",
        "reviewed definition built",
        "refused",
        Some("unsupported_target_contract_version"),
        Some(1),
    );
}

fn development_binding(environment: DeploymentEnvironment) -> DevelopmentDeploymentBinding {
    DevelopmentDeploymentBinding::new(
        TargetContractVersion::V2,
        environment,
        [0x11; 32],
        [0x22; 32],
        ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
        None,
    )
}

#[test]
fn promotion_control_and_one_resource_mutation_have_scoped_events() {
    if !in_isolated_child("promotion_control_and_one_resource_mutation_have_scoped_events") {
        return;
    }
    let reviewed = reviewed_elements_tapscript().expect("reviewed definition");
    let control = reviewed.validated().clone();
    let source = reviewed.definition();
    let subject = validate_target_definition(TargetDefinition::new(TargetDefinitionParts {
        resources: ResourceContract::new(
            source.resources().consensus().clone(),
            PolicyResourceLimits::new([]),
        ),
        ..reviewed_parts()
    }))
    .expect("narrowed policy remains coherent");
    let expected_control = validate_as_reviewed_elements(control.clone());
    let expected_subject = validate_as_reviewed_elements(subject.clone());
    assert_eq!(
        expected_subject,
        Err(TargetError::ReviewedDefinitionMismatch)
    );

    let (actual_control, control_seen) = captured(|| validate_as_reviewed_elements(control));
    let (actual_subject, subject_seen) = captured(|| validate_as_reviewed_elements(subject));
    assert_eq!(actual_control, expected_control);
    assert_eq!(actual_subject, expected_subject);
    assert_boundary(
        &control_seen,
        "target_elements::definition",
        "promote_reviewed_definition",
        "reviewed definition promotion checked",
        "completed",
        None,
        None,
    );
    assert_boundary(
        &subject_seen,
        "target_elements::definition",
        "promote_reviewed_definition",
        "reviewed definition promotion checked",
        "refused",
        Some("reviewed_definition_mismatch"),
        None,
    );
}

#[test]
fn development_binding_control_and_one_environment_mutation_have_scoped_events() {
    if !in_isolated_child(
        "development_binding_control_and_one_environment_mutation_have_scoped_events",
    ) {
        return;
    }
    let reviewed = reviewed_elements_tapscript().expect("reviewed definition");
    let definition = reviewed.validated();
    let control = development_binding(DeploymentEnvironment::Development);
    let subject = development_binding(DeploymentEnvironment::Production);
    let expected_control = validate_development_binding(definition, control.clone());
    let expected_subject = validate_development_binding(definition, subject.clone());
    assert_eq!(
        expected_subject,
        Err(TargetError::ProductionBindingUnsupported)
    );

    let (actual_control, control_seen) =
        captured(|| validate_development_binding(definition, control));
    let (actual_subject, subject_seen) =
        captured(|| validate_development_binding(definition, subject));
    assert_eq!(actual_control, expected_control);
    assert_eq!(actual_subject, expected_subject);
    assert_boundary(
        &control_seen,
        "target_elements::deployment",
        "validate_development_binding",
        "development binding checked",
        "completed",
        None,
        None,
    );
    assert_boundary(
        &subject_seen,
        "target_elements::deployment",
        "validate_development_binding",
        "development binding checked",
        "refused",
        Some("production_binding_unsupported"),
        None,
    );
    assert_eq!(
        control_seen[0].fields.get("required_capability_count"),
        Some(&Value::Number(0))
    );
    assert_eq!(
        control_seen[0].fields.get("override_count"),
        Some(&Value::Number(0))
    );
}

#[test]
fn reviewed_binding_control_and_one_environment_mutation_emit_only_outer_event() {
    if !in_isolated_child(
        "reviewed_binding_control_and_one_environment_mutation_emit_only_outer_event",
    ) {
        return;
    }
    let reviewed = reviewed_elements_tapscript().expect("reviewed definition");
    let control = development_binding(DeploymentEnvironment::Development);
    let subject = development_binding(DeploymentEnvironment::Production);
    let expected_control = validate_reviewed_development_binding(&reviewed, control.clone());
    let expected_subject = validate_reviewed_development_binding(&reviewed, subject.clone());
    assert_eq!(
        expected_subject,
        Err(TargetError::ProductionBindingUnsupported)
    );

    let (actual_control, control_seen) =
        captured(|| validate_reviewed_development_binding(&reviewed, control));
    let (actual_subject, subject_seen) =
        captured(|| validate_reviewed_development_binding(&reviewed, subject));
    assert_eq!(actual_control, expected_control);
    assert_eq!(actual_subject, expected_subject);
    assert_boundary(
        &control_seen,
        "target_elements::deployment",
        "validate_reviewed_development_binding",
        "reviewed development binding checked",
        "completed",
        None,
        None,
    );
    assert_boundary(
        &subject_seen,
        "target_elements::deployment",
        "validate_reviewed_development_binding",
        "reviewed development binding checked",
        "refused",
        Some("production_binding_unsupported"),
        None,
    );
    assert_eq!(
        actual_control.expect("reviewed binding").target(),
        &reviewed.projection()
    );
}
