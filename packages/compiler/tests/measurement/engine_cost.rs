//! Measurement-only harness; the node runner installs these identical bytes on both revisions.
use std::{collections::BTreeMap, error::Error, fmt::Debug, fs, num::NonZeroU64, sync::{Arc, Mutex}, time::Instant};
use architecture::{ARCHITECTURE, OperationId};
use realization::RealizationScope;
use sha2::{Digest, Sha256};
use crate::{AnalysisPolicy, CompilationScope, ProofSearchLimits, analyzed::analyze_scoped_program, bind_input, placement::PlacementSearchLimits};
use tracing_subscriber::{layer::SubscriberExt, Layer};
const PROOF_STATES: u64 = 1_000_000;
const PROOF_CANDIDATES: u64 = 10_000;
const PLACEMENT_STATES: u64 = 10_000_000;
const PLACEMENT_CANDIDATES: u64 = 1_000_000;
const C: &[OperationId] = &[OperationId::CompactAsh];
const T: &[OperationId] = &[OperationId::TransferLive];
const A: &[OperationId] = &[OperationId::AnnounceMaturity];
const CT: &[OperationId] = &[OperationId::CompactAsh, OperationId::TransferLive];
const CTA: &[OperationId] = &[
    OperationId::CompactAsh,
    OperationId::TransferLive,
    OperationId::AnnounceMaturity,
];

struct Subject {
    name: &'static str,
    realization: &'static [OperationId],
    compilation: Option<&'static [OperationId]>,
    constructor: &'static str,
}

const SUBJECTS: &[Subject] = &[
    Subject {
        name: "realization-C",
        realization: C,
        compilation: None,
        constructor: "derive(C).project()",
    },
    Subject {
        name: "realization-T",
        realization: T,
        compilation: None,
        constructor: "derive(T).project()",
    },
    Subject {
        name: "realization-CT",
        realization: CT,
        compilation: None,
        constructor: "derive(C,T).project()",
    },
    Subject {
        name: "realization-A",
        realization: A,
        compilation: None,
        constructor: "derive(A).project()",
    },
    Subject {
        name: "realization-CTA",
        realization: CTA,
        compilation: None,
        constructor: "derive(C,T,A).project()",
    },
    Subject {
        name: "compiler-C",
        realization: CT,
        compilation: Some(C),
        constructor: "fallible bound_input(C): derive(C,T); bind_input(C); analyze; project",
    },
    Subject {
        name: "compiler-T",
        realization: CT,
        compilation: Some(T),
        constructor: "fallible bound_input(T): derive(C,T); bind_input(T); analyze; project",
    },
    Subject {
        name: "compiler-CT",
        realization: CT,
        compilation: Some(CT),
        constructor: "fallible bound_input(C,T): derive(C,T); bind_input(C,T); analyze; project",
    },
    Subject {
        name: "compiler-A",
        realization: A,
        compilation: Some(A),
        constructor: "fallible announcement_input(): derive(A); bind_input(A); analyze; project",
    },
    Subject {
        name: "compiler-CTA",
        realization: CTA,
        compilation: Some(CTA),
        constructor: "explicit derive(C,T,A); bind_input(C,T,A); analyze; project",
    },
];


#[derive(Clone)]
struct Counters(BTreeMap<String, u64>);
impl Counters {
    fn read() -> Result<Self, Box<dyn Error>> {
        let mut values = BTreeMap::new();
        let root = std::env::var("TRIPOD_COST_CGROUP").unwrap_or_else(|_| "/sys/fs/cgroup".into());
        for line in fs::read_to_string(format!("{root}/cpu.stat"))?.lines() {
            let (name, value) = line.split_once(' ').ok_or("cpu counter")?;
            values.insert(format!("cpu.{name}"), value.parse()?);
        }
        for resource in ["cpu", "memory", "io"] {
            for line in fs::read_to_string(format!("{root}/{resource}.pressure"))?.lines() {
                let class = line.split_whitespace().next().ok_or("pressure class")?;
                let value = line.split_whitespace().find_map(|v| v.strip_prefix("total=")).ok_or("pressure total")?;
                values.insert(format!("{resource}.{class}.stall_us"), value.parse()?);
            }
        }
        Ok(Self(values))
    }
}
struct Start { counters: Counters, instant: Instant }
struct Sample { phase: String, wall_ns: u128, start: Counters, end: Counters }
impl Start {
    fn new() -> Self { Self { counters: Counters::read().expect("own cgroup readable"), instant: Instant::now() } }
    fn end(self, phase: &str) -> Sample {
        let wall_ns = self.instant.elapsed().as_nanos();
        Sample { phase: phase.into(), wall_ns, start: self.counters, end: Counters::read().expect("own cgroup readable") }
    }
}
#[derive(Clone)]
struct Publications(Arc<Mutex<Vec<Sample>>>);
impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>> Layer<S> for Publications {
    fn on_enter(&self, id: &tracing::span::Id, context: tracing_subscriber::layer::Context<'_, S>) {
        let span = context.span(id).expect("registered span");
        if span.metadata().target() == "compiler::engine_keyed" && span.metadata().name() == "publish" {
            span.extensions_mut().insert(Start::new());
        }
    }
    fn on_exit(&self, id: &tracing::span::Id, context: tracing_subscriber::layer::Context<'_, S>) {
        let span = context.span(id).expect("registered span");
        if let Some(start) = span.extensions_mut().remove::<Start>() {
            self.0.lock().expect("publication samples").push(start.end("projection_nested"));
        }
    }
}
fn measured<T>(phase: &str, samples: &mut Vec<Sample>, f: impl FnOnce() -> T) -> T {
    let start = Start::new(); let result = f(); samples.push(start.end(phase)); result
}
fn finish(name: &str, result: &str, stage: &str, value: &impl Debug, samples: &mut Vec<Sample>) -> Result<(), Box<dyn Error>> {
    let bytes = measured("format", samples, || format!("result: {result}\nstage: {stage}\n{value:#?}\n"));
    let digest = measured("digest", samples, || format!("{:x}", Sha256::digest(bytes.as_bytes())));
    if !name.starts_with("keyed-") {
        let manifest = include_str!("../../tests/fixtures/engine_base/MANIFEST");
        let fields = manifest.split("\n\n").find_map(|entry| {
            let fields = entry.lines().filter_map(|line| line.split_once('=')).collect::<BTreeMap<_, _>>();
            (fields.get("subject").copied() == Some(name)).then_some(fields)
        }).ok_or("missing subject")?;
        assert_eq!(fields["bytes"].parse::<usize>()?, bytes.len(), "length {name}");
        assert_eq!(fields["sha256"], digest, "digest {name}");
        assert_eq!(fields["result"], result); assert_eq!(fields["stage"], stage);
    }
    let path = std::env::var("TRIPOD_COST_RESULT")?;
    fs::write(format!("{path}.result"), format!("subject={name}\nresult={result}\nstage={stage}\nbytes={}\nsha256={digest}\n", bytes.len()))?;
    Ok(())
}
fn attest(subject: &Subject, samples: &mut Vec<Sample>) -> Result<(), Box<dyn Error>> {
    let start = Start::new();
    let scope = RealizationScope::from_operations(subject.realization.iter().copied())?;
    let realized = realization::derive(&ARCHITECTURE, scope)?;
    let Some(operations) = subject.compilation else {
        samples.push(start.end("derivation_inclusive"));
        let projection = measured("projection", samples, || realized.project());
        return finish(subject.name, "projection", "realization_projection", &projection, samples);
    };
    let input = bind_input(&ARCHITECTURE, realized, CompilationScope::from_operations(operations.iter().copied())?, AnalysisPolicy::strict(ProofSearchLimits::new(NonZeroU64::new(PROOF_STATES).unwrap(), NonZeroU64::new(PROOF_CANDIDATES).unwrap())))?;
    let program = analyze_scoped_program(&input, PlacementSearchLimits::new(NonZeroU64::new(PLACEMENT_STATES).unwrap(), NonZeroU64::new(PLACEMENT_CANDIDATES).unwrap()));
    samples.push(start.end("derivation_inclusive"));
    match program {
        Ok(program) => {
            let projection = measured("projection", samples, || program.project());
            finish(subject.name, "projection", "compiler_projection", &projection, samples)
        }
        Err(error) => finish(subject.name, "typed-refusal", "analyze_scoped_program", &error, samples),
    }
}
#[cfg(feature = "cost-keyed")]
fn keyed(name: &str, samples: &mut Vec<Sample>) -> Result<(), Box<dyn Error>> {
    use architecture::semantic::{KeyedOperationId as K, SemanticOperationId as O};
    let operations = match name { "keyed-SOURCE" => vec![K::Source], "keyed-MERGE" => vec![K::Merge], "keyed-joint" => vec![K::Source, K::Merge], _ => return Err("unknown keyed subject".into()) };
    let start = Start::new();
    let manifest = architecture::keyed::KeyedManifest::contract().map_err(|e| format!("{e:?}"))?;
    let realized = realization::keyed::KeyedRealizationDraft::derive(&manifest).map_err(|e| format!("{e:?}"))?.validate(&manifest).map_err(|e| format!("{e:?}"))?;
    let input = bind_input(&manifest, realized, operations.into_iter().map(O::Keyed).collect(), AnalysisPolicy::strict(ProofSearchLimits::new(NonZeroU64::new(PROOF_STATES).unwrap(), NonZeroU64::new(PROOF_CANDIDATES).unwrap()))).map_err(|e| format!("{e:?}"))?;
    let plan = crate::keyed_plan::plan(&input, PlacementSearchLimits::new(NonZeroU64::new(PLACEMENT_STATES).unwrap(), NonZeroU64::new(PLACEMENT_CANDIDATES).unwrap()))?;
    samples.push(start.end("derivation_inclusive"));
    // Publication is inside plan twice; the layer measures both nested publish spans.
    // This separate phase is the consumer-owned projection snapshot, not a third derivation.
    let projection = measured("projection_snapshot", samples, || plan.project().clone());
    finish(name, "projection", "keyed_projection", &projection, samples)
}

#[test]
#[ignore = "node measurement runner only; not a performance claim"]
fn measure_engine_subject() -> Result<(), Box<dyn Error>> {
    let Ok(name) = std::env::var("TRIPOD_COST_SUBJECT") else { return Ok(()) };
    let nested = Publications(Arc::new(Mutex::new(Vec::new())));
    let subscriber = tracing_subscriber::registry().with(nested.clone().with_filter(tracing_subscriber::filter::filter_fn(|m| m.target() == "compiler::engine_keyed" && m.name() == "publish")));
    let mut samples = Vec::new();
    let result = tracing::subscriber::with_default(subscriber, || {
        if let Some(subject) = SUBJECTS.iter().find(|subject| subject.name == name) {
            let _recipe = subject.constructor;
            attest(subject, &mut samples)
        } else {
            #[cfg(feature = "cost-keyed")]
            { keyed(&name, &mut samples) }
            #[cfg(not(feature = "cost-keyed"))]
            { Err("keyed production scopes absent on baseline".into()) }
        }
    });
    samples.append(&mut nested.0.lock().expect("nested samples"));
    let mut output = String::from("phase\twall_ns\tstart\tend\n");
    use std::fmt::Write;
    for sample in samples { writeln!(output, "{}\t{}\t{:?}\t{:?}", sample.phase, sample.wall_ns, sample.start.0, sample.end.0)?; }
    fs::write(std::env::var("TRIPOD_COST_RESULT")?, output)?;
    result
}
