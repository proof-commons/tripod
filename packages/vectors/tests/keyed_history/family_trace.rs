//! Exactly one event at each public family boundary, under a bounded child.
use super::{bounded, child, fixture, one_test, reader, reading};
use vectors::keyed_evidence as e;
#[test]
fn family_boundaries_emit_one_completion_or_refusal() {
    one_test(&bounded(
        &mut child("keyed_history::family_trace::trace_child", "10"),
        "family trace timeout=10s",
    ));
}
#[test]
#[ignore = "bounded parent launches exactly this child"]
fn trace_child() {
    use std::sync::{Arc, Mutex};
    #[derive(Clone)]
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Capture {
        type Writer = Self;
        fn make_writer(&'a self) -> Self {
            self.clone()
        }
    }
    let captured = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .without_time()
        .with_ansi(false)
        .with_writer(Capture(captured.clone()))
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);
    let fixture = fixture::build(fixture::graph("multi"));
    let input = reader::input(fixture.reader_inputs());
    let request = reading::request(&input, 20);
    captured.lock().unwrap().clear();
    let context = e::prepare_families(&input).unwrap();
    assert_event(&captured, "completed");
    for family in [
        e::Family::Birth,
        e::Family::Continuation,
        e::Family::Recurrence,
        e::Family::Reading,
        e::Family::Acceptance,
        e::Family::Resources,
    ] {
        captured.lock().unwrap().clear();
        let mut report = e::family_report(&context, family, &request).unwrap();
        assert_event(&captured, "completed");
        captured.lock().unwrap().clear();
        super::families::validator(family)(&context, &request, &report).unwrap();
        assert_event(&captured, "completed");
        captured.lock().unwrap().clear();
        report.header.schema += 1;
        assert!(super::families::validator(family)(&context, &request, &report).is_err());
        assert_event(&captured, "refused");
    }
    captured.lock().unwrap().clear();
    let mut bad = input;
    bad.bytes.get_mut(&10).unwrap()[0] ^= 1;
    assert!(e::prepare_families(&bad).is_err());
    assert_event(&captured, "refused");
}
fn assert_event(captured: &std::sync::Arc<std::sync::Mutex<Vec<u8>>>, outcome: &str) {
    let text = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
    let lines: Vec<_> = text
        .lines()
        .filter(|line| line.contains("vectors::keyed_evidence:"))
        .collect();
    assert_eq!(lines.len(), 1, "{text}");
    assert!(
        lines[0].contains(&format!("keyed family {outcome}")),
        "{text}"
    );
}
