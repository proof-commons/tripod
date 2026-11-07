//! Numeric close records for spans accepted by the command subscriber.

use std::fmt::Write as _;
use std::io::Write as _;
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};
use tracing::Subscriber;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id};
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

use crate::LockedStderrWriter;

pub struct NumericCloseLayer;

impl<S> Layer<S> for NumericCloseLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        if let Some(span) = context.span(id) {
            let mut fields = EntryFields::default();
            attributes.record(&mut fields);
            span.extensions_mut().insert(SpanTiming {
                start: Instant::now(),
                entered: None,
                busy: Duration::ZERO,
                fields: fields.0,
            });
        }
    }

    fn on_enter(&self, id: &Id, context: Context<'_, S>) {
        if let Some(span) = context.span(id)
            && let Some(timing) = span.extensions_mut().get_mut::<SpanTiming>()
            && timing.entered.is_none()
        {
            timing.entered = Some(Instant::now());
        }
    }

    fn on_exit(&self, id: &Id, context: Context<'_, S>) {
        if let Some(span) = context.span(id)
            && let Some(timing) = span.extensions_mut().get_mut::<SpanTiming>()
            && let Some(entered) = timing.entered.take()
        {
            timing.busy = timing.busy.saturating_add(entered.elapsed());
        }
    }

    fn on_close(&self, id: Id, context: Context<'_, S>) {
        let Some(span) = context.span(&id) else {
            return;
        };
        let Some(timing) = span.extensions_mut().remove::<SpanTiming>() else {
            return;
        };
        let total = timing.start.elapsed();
        let busy = timing
            .busy
            .saturating_add(timing.entered.map_or(Duration::ZERO, |at| at.elapsed()));
        let busy_ns = saturating_nanos(busy);
        let idle_ns = saturating_nanos(total.saturating_sub(busy));
        let metadata = span.metadata();
        let mut span_fields = timing.fields;
        span_fields.insert("name".to_owned(), Value::String(metadata.name().to_owned()));
        let mut fields = Map::new();
        fields.insert(
            "message".to_owned(),
            Value::String("span closed".to_owned()),
        );
        fields.insert("busy_ns".to_owned(), json!(busy_ns));
        fields.insert("idle_ns".to_owned(), json!(idle_ns));
        let mut timestamp = String::new();
        let _ignored =
            tracing_subscriber::fmt::time::SystemTime.format_time(&mut Writer::new(&mut timestamp));
        let record = json!({
            "timestamp": timestamp,
            "level": metadata.level().as_str(),
            "target": metadata.target(),
            "fields": fields,
            "span": span_fields,
            "spans": [span_fields],
        });
        if let Ok(line) = serde_json::to_string(&record) {
            let _ignored = writeln!(LockedStderrWriter::new(), "{line}");
        }
    }
}

struct SpanTiming {
    start: Instant,
    entered: Option<Instant>,
    busy: Duration,
    fields: Map<String, Value>,
}

fn saturating_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[derive(Default)]
struct EntryFields(Map<String, Value>);

impl Visit for EntryFields {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_owned(), json!(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_owned(), json!(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_owned(), json!(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_owned(), json!(value));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let mut text = String::new();
        let _ignored = write!(text, "{value:?}");
        self.0.insert(field.name().to_owned(), json!(text));
    }
}
