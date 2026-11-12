//! New public conformance boundaries each emit one completed/refused event.
use super::{bounded, child, fixture, one_test, reader, reading};
use vectors::keyed_evidence as e;

#[test]
fn reading_and_report_boundaries_have_distinct_single_events() {
    one_test(&bounded(
        &mut child("keyed_history::reading_trace::trace_child", "10"),
        "evidence trace timeout=10s",
    ));
}

#[test]
#[ignore = "bounded parent launches exactly this trace child"]
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
    let fixture = fixture::build(fixture::graph("source"));
    let input = reader::input(fixture.reader_inputs());
    let request = reading::request(&input, 10);
    captured.lock().unwrap().clear();
    let mut report = e::reading_report(&input, &request).unwrap();
    assert_eq!(e::validate_report(&input, &request, &report), Ok(()));
    report.header.schema += 1;
    assert_eq!(
        e::validate_report(&input, &request, &report),
        Err(e::Refusal::SchemaMismatch)
    );
    let mut required = request;
    required.finality = e::FinalityRequest::Required;
    assert_eq!(
        e::reading_report(&input, &required),
        Err(e::Refusal::FinalityMissing)
    );
    let mut corrupt = input;
    corrupt.bytes.get_mut(&10).unwrap()[0] ^= 1;
    let inner = vectors::keyed_history_reader::read(&corrupt).unwrap_err();
    assert_eq!(
        e::reading_report(&corrupt, &required),
        Err(e::Refusal::Reader(inner.clone()))
    );
    let output = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
    for message in [
        "keyed evidence reading completed",
        "keyed evidence reading refused",
        "keyed evidence report validated",
        "keyed evidence report refused",
    ] {
        assert_eq!(
            output.lines().filter(|line| line.contains(message)).count(),
            if message == "keyed evidence reading refused" {
                2
            } else {
                1
            },
            "{output}"
        );
    }
    assert!(
        output.contains(&format!("inner_refusal_kind=\"{}\"", inner.refusal_kind())),
        "{output}"
    );
    assert!(output.contains("read_keyed_evidence"), "{output}");
    assert!(output.contains("validate_keyed_evidence"), "{output}");
    assert!(
        output.contains("refusal_kind=\"finality_missing\""),
        "{output}"
    );
    assert!(
        output.contains("refusal_kind=\"schema_mismatch\""),
        "{output}"
    );
}
