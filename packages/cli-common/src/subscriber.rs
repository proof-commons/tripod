//! Subscriber-owned ADR-010 control and diagnostic stderr routes.

use std::io::Write as _;
use std::sync::OnceLock;

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Layer, Registry, reload};

use crate::{
    ControlPlaneRecord, ControlPlaneRecordKind, LockedStderr, LockedStderrWriter,
    set_json_panic_hook, tracing_filter,
};

const CONTROL_TARGET: &str = "cli_common::control";
static FILTER_HANDLE: OnceLock<reload::Handle<EnvFilter, Registry>> = OnceLock::new();

/// Install the command's subscriber and panic hook before parsing arguments.
///
/// Calling this again preserves the subscriber and replaces the hook for the
/// command name supplied by the caller.
pub fn install_command_subscriber(command_name: &str) {
    ensure_subscriber();
    set_json_panic_hook(command_name);
}

fn ensure_subscriber() {
    if FILTER_HANDLE.get().is_some() {
        return;
    }

    let initial = tracing_filter(false, Level::INFO);
    let (filter, rejected) = diagnostic_filter(&initial.directive);
    let (reload_layer, handle) = reload::Layer::new(filter);
    let diagnostics = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(LockedStderr)
        .with_filter(filter_fn(|metadata| metadata.target() != CONTROL_TARGET));
    let subscriber = Registry::default()
        .with(reload_layer)
        .with(ControlRoute)
        .with(crate::span_close::NumericCloseLayer)
        .with(diagnostics);
    if tracing::subscriber::set_global_default(subscriber).is_ok() {
        let _stored = FILTER_HANDLE.set(handle);
        if rejected {
            tracing::warn!(target: "cli_common::command", "filter directive rejected");
        }
    }
}

pub fn reload_command_filter(directive: &str) -> bool {
    let Some(handle) = FILTER_HANDLE.get() else {
        return false;
    };
    let (filter, rejected) = diagnostic_filter(directive);
    let reloaded = handle.reload(filter).is_ok();
    if rejected && reloaded {
        tracing::warn!(target: "cli_common::command", "filter directive rejected");
    }
    reloaded
}

fn diagnostic_filter(directive: &str) -> (EnvFilter, bool) {
    let (mut filter, rejected) = EnvFilter::try_new(directive)
        .map_or_else(|_| (EnvFilter::new("info"), true), |filter| (filter, false));
    if let Ok(control) = "cli_common::control=trace".parse() {
        filter = filter.add_directive(control);
    }
    (filter, rejected)
}

pub fn emit_control_record(record: &ControlPlaneRecord, panic_record: bool) -> std::io::Result<()> {
    let level = match record.kind {
        ControlPlaneRecordKind::Help
        | ControlPlaneRecordKind::Version
        | ControlPlaneRecordKind::Status => Level::INFO,
        ControlPlaneRecordKind::UsageError | ControlPlaneRecordKind::TtyRefusal => Level::WARN,
        ControlPlaneRecordKind::Panic | ControlPlaneRecordKind::Diagnostic => Level::ERROR,
    };
    let enabled = match level {
        Level::INFO => tracing::enabled!(target: "cli_common::control", Level::INFO),
        Level::WARN => tracing::enabled!(target: "cli_common::control", Level::WARN),
        _ => tracing::enabled!(target: "cli_common::control", Level::ERROR),
    };
    if !enabled {
        return Ok(());
    }
    let json = serde_json::to_string(record)?;
    match level {
        Level::INFO => {
            tracing::event!(target: "cli_common::control", Level::INFO, record_json = json.as_str(), panic_record, message = "control record");
        }
        Level::WARN => {
            tracing::event!(target: "cli_common::control", Level::WARN, record_json = json.as_str(), panic_record, message = "control record");
        }
        _ => {
            tracing::event!(target: "cli_common::control", Level::ERROR, record_json = json.as_str(), panic_record, message = "control record");
        }
    }
    Ok(())
}

struct ControlRoute;

impl<S> Layer<S> for ControlRoute
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        if event.metadata().target() != CONTROL_TARGET {
            return;
        }
        let mut fields = ControlFields::default();
        event.record(&mut fields);
        let Some(json) = fields.record_json else {
            return;
        };
        let writer = if fields.panic_record {
            LockedStderrWriter::try_new()
        } else {
            Some(LockedStderrWriter::new())
        };
        if let Some(mut writer) = writer {
            let _ignored = writer.write_all(json.as_bytes());
            let _ignored = writer.write_all(b"\n");
            let _ignored = writer.flush();
        }
    }
}

#[derive(Default)]
struct ControlFields {
    record_json: Option<String>,
    panic_record: bool,
}

impl Visit for ControlFields {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "record_json" {
            self.record_json = Some(value.to_owned());
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == "panic_record" {
            self.panic_record = value;
        }
    }

    fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
}
