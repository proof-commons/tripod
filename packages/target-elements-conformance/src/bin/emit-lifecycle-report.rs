//! `emit-lifecycle-report`: the §14.1 `FreshProcessLifecycle` report.
//!
//! # The judgement happens here, and the runner is transport
//!
//! The same division `emit-normalization-report` rests on, and for a
//! sharper reason. This lane's runner is the process that CORRUPTS the
//! public record: it builds the wrong txid, the copied bytes, and the
//! foreign genesis. A runner that also decided whether each corruption
//! was caught would be marking its own work, and the one row it would
//! never fail is the one it got wrong.
//!
//! So the runner records what each reading process observed, verbatim,
//! and this command rebuilds every expectation from
//! [`target_elements_conformance::lifecycle::canonical_lifecycle_matrix`]
//! before comparing. A row the run did not answer is a missing row, not
//! a passing one.

use std::io::{Read as _, Write as _};
use std::process::ExitCode;

use target_elements_conformance::lifecycle::{
    LifecycleOutcome, LifecycleRow, PublicHandoff, canonical_lifecycle_matrix,
};
use target_elements_conformance::lifecycle_report::{
    CheckOutcome, DestroyedFile, DestructionRecord, FreshProcessLifecycleReport,
    LifecycleReportRole, LifecycleRowOutcome, MINIMUM_COMPLETE_PASSES, ReadingPass, UnbuiltRow,
};
use target_elements_conformance::normalization::AuthorizationProfile;
use target_elements_conformance::protocol::LifecycleCheck;

fn main() -> ExitCode {
    match run() {
        Ok(rendered) => {
            let mut out = std::io::stdout();
            match out
                .write_all(rendered.as_bytes())
                .and_then(|()| out.write_all(b"\n"))
            {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => fail(&format!("stdout refused the report: {error}")),
            }
        }
        Err(note) => fail(&note),
    }
}

/// Writes one refusal and reports the failing status.
fn fail(note: &str) -> ExitCode {
    drop(writeln!(std::io::stderr(), "emit-lifecycle-report: {note}"));
    ExitCode::FAILURE
}

fn run() -> Result<String, String> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| "usage: emit-lifecycle-report RUN-RECORD".to_owned())?;

    let mut text = String::new();
    std::fs::File::open(&path)
        .and_then(|mut file| file.read_to_string(&mut text))
        .map_err(|error| format!("the run record at {path} could not be read: {error}"))?;
    let record: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| format!("the run record is not JSON: {error}"))?;

    // Parsed into the typed record rather than read field by field. An
    // extra field here is exactly the covert channel §13 excludes, and
    // `deny_unknown_fields` is what refuses it — so the report cannot be
    // built at all from a record that carried one.
    let handoff: PublicHandoff =
        serde_json::from_value(record["process_a"]["subject"]["handoff"].clone())
            .map_err(|error| format!("the published record is not a public handoff: {error}"))?;
    if !handoff.schema_is_current() {
        return Err(format!(
            "the published record declares the schema {}",
            handoff.schema
        ));
    }

    let matrix = canonical_lifecycle_matrix();
    let mut passes = Vec::new();
    let empty = Vec::new();
    let runs = record["process_b_runs"].as_array().unwrap_or(&empty);
    if runs.is_empty() {
        return Err("the run record carries no reading pass".to_owned());
    }
    for run in runs {
        let mut rows = Vec::new();
        for answer in run["rows"].as_array().unwrap_or(&empty) {
            let name = answer["row"].as_str().unwrap_or_default();
            let row = row_of(name)?;
            let observed: LifecycleOutcome =
                serde_json::from_value(answer["observed_outcome"].clone()).map_err(|error| {
                    format!(
                        "the pass reported an outcome for {name} this report cannot read: {error}"
                    )
                })?;
            // From source, never from the record. The runner knows which
            // record it corrupted, and this is the line that stops that
            // knowledge from becoming the answer.
            let expected = matrix
                .iter()
                .find(|expectation| expectation.row == row)
                .ok_or_else(|| format!("the matrix states no row named {name}"))?
                .expected;
            rows.push(LifecycleRowOutcome::new(
                row,
                expected,
                observed,
                checks_of(&answer["checks"], name)?,
            ));
        }
        passes.push(ReadingPass {
            attempt: narrow(&run["attempt"], "a pass ordinal")?,
            // Refused rather than truncated. This identifier is the
            // evidence that two passes were two processes, and a
            // truncated one could collide with another pass's — which
            // would report a shared process as distinct ones, the exact
            // claim §13.5's closing sentence forbids.
            pid: narrow(&run["pid"], "a process identifier")?,
            wallet_name: run["wallet_name"].as_str().unwrap_or_default().to_owned(),
            rows,
        });
    }

    let mut unbuilt = Vec::new();
    if let Some(reason) = record["stale_row"]["unbuilt_reason"].as_str() {
        unbuilt.push(UnbuiltRow {
            row: LifecycleRow::StaleEvidence,
            reason: reason.to_owned(),
        });
    }

    let report = FreshProcessLifecycleReport {
        role: LifecycleReportRole::Experimental,
        handoff,
        authorization_profile: serde_json::from_value::<Option<AuthorizationProfile>>(
            record["process_a"]["subject"]["authorization_profile"].clone(),
        )
        .unwrap_or(None),
        destruction: destruction_of(&record["destruction"])?,
        passes,
        unbuilt_rows: unbuilt,
        observed_genesis: record["environment"]["genesis_id"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        observed_network: record["environment"]["network_id"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        declared_tip: record["environment"]["handshake"]["intended_executed_tip"]
            .as_str()
            .map(ToOwned::to_owned),
        binary_reported_revision: record["environment"]["handshake"]["binary_reported_revision"]
            .as_str()
            .map(ToOwned::to_owned),
    };

    gates_hold(&report)?;

    serde_json::to_string_pretty(&report)
        .map_err(|error| format!("the report does not serialize: {error}"))
}

/// Every condition the report must meet before it is emitted at all.
///
/// Checked here rather than left for a reader. Every other fact in this
/// report is worthless if the creator was still running, or its wallet
/// still on disk, while the chain was read.
fn gates_hold(report: &FreshProcessLifecycleReport) -> Result<(), String> {
    if !report.boundary_holds() {
        return Err(
            "the run's process boundary does not hold: the creator's wallet or process \
             survived it, or two reading passes shared one process or one attempt ordinal"
                .to_owned(),
        );
    }
    if !report.matrix_is_complete() {
        return Err(
            "a reading pass did not answer the matrix exactly: a row is missing, answered \
             twice, or not one the matrix states"
                .to_owned(),
        );
    }
    // The whole claim, not the two parts the gate used to consult. A
    // report that reached here with one pass, or with passes that
    // disagreed, would be offering a reading nothing shows was
    // uncached.
    if !report.cache_independence_established() {
        return Err(format!(
            "the run does not establish cache independence: it needs at least \
             {MINIMUM_COMPLETE_PASSES} complete passes, in distinct processes, agreeing row \
             for row"
        ));
    }
    Ok(())
}

/// One unsigned field of the run record, refused where it does not fit.
fn narrow(value: &serde_json::Value, what: &str) -> Result<u32, String> {
    let wide = value
        .as_u64()
        .ok_or_else(|| format!("the run record states {what} that is not a whole number"))?;
    u32::try_from(wide).map_err(|_ignored| format!("the run record states {what} of {wide}"))
}

fn row_of(name: &str) -> Result<LifecycleRow, String> {
    serde_json::from_value(serde_json::Value::String(name.to_owned()))
        .map_err(|error| format!("the run names a row this report does not have: {name}: {error}"))
}

/// The checks one pass recorded, read as the protocol record they are.
///
/// Parsed into [`LifecycleCheck`] rather than picked apart member by
/// member. The previous reading defaulted: a check whose name was absent
/// became the empty string and one whose agreement was absent became
/// `false`, so a malformed record produced a report of well-formed
/// checks that nobody had made. A check is evidence, and a defaulted one
/// is evidence of nothing — under revision 4 the record either states
/// the check or the report is not built `(´[PLAN-rule:guide12-exec:protocol-revision]´)`.
fn checks_of(value: &serde_json::Value, row: &str) -> Result<Vec<CheckOutcome>, String> {
    let checks: Vec<LifecycleCheck> = serde_json::from_value(value.clone()).map_err(|error| {
        format!("the pass recorded a check for {row} this report cannot read: {error}")
    })?;
    Ok(checks
        .into_iter()
        .map(|check| CheckOutcome {
            check: check.check,
            expected: check.expected,
            observed: check.observed,
            agrees: check.agrees,
        })
        .collect())
}

fn destruction_of(value: &serde_json::Value) -> Result<DestructionRecord, String> {
    let empty = Vec::new();
    let files = value["destroyed_files"]
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .map(|entry| DestroyedFile {
            path: entry["path"].as_str().unwrap_or_default().to_owned(),
            bytes: entry["bytes"].as_u64().unwrap_or_default(),
        })
        .collect();
    Ok(DestructionRecord {
        scope: value["scope"]
            .as_str()
            .ok_or_else(|| "the destruction record states no scope".to_owned())?
            .to_owned(),
        wallet_directory: value["wallet_directory"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        destroyed_files: files,
        destroyed_bytes: value["destroyed_bytes"].as_u64().unwrap_or_default(),
        wallet_directory_present_after: value["wallet_directory_present_after"]
            .as_bool()
            .unwrap_or(true),
        process_a_exit_status: value["process_a_exit_status"]
            .as_i64()
            .and_then(|status| i32::try_from(status).ok()),
        // Absent means "the runner did not say", and a boundary nobody
        // checked is not a boundary that held.
        process_a_running_after: value["process_a_running_after"].as_bool().unwrap_or(true),
    })
}
