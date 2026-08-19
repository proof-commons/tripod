//! The documents the shipped `emit-*` commands publish.
//!
//! # Why these live in the library
//!
//! Each of the four emitters used to build its document inside `main`,
//! so the only way to exercise one was to run the binary and read its
//! stdout — and nothing in this crate could do that, which is why
//! `G12-R03` and `G12-R05` both carried source-read dispositions rather
//! than reproductions. A command whose behaviour can only be observed
//! through a process boundary is a command whose behaviour is not
//! actually being checked.
//!
//! So the document is built here and the binary is the part that cannot
//! be tested any other way: argument parsing, the panic hook, the
//! terminal refusal, and the write to stdout. That division is what
//! ADR-010 asks of a shipped command, and it is the seam the Wave-2
//! subprocess harness consumes — that harness checks the *contract*
//! (exit classes, the JSON-only stderr, the terminal refusal), while the
//! *content* is checked here, in ordinary tests, without spawning
//! anything.
//!
//! # Refusals, not defaults
//!
//! Every function here returns a refusal rather than a partial
//! document. A report assembled from a record it could not fully read
//! would be a report about a run that did not happen the way it says.

use crate::conservation::canonical_conservation_matrix;
use crate::conservation_report::ConservationReportRole;
use crate::declassification::normalization_declassifications;
use crate::lifecycle::{LifecycleOutcome, LifecycleRow, PublicHandoff, canonical_lifecycle_matrix};
use crate::lifecycle_report::{
    CheckOutcome, DestroyedFile, DestructionRecord, FreshProcessLifecycleReport,
    LifecycleReportRole, LifecycleRowOutcome, MINIMUM_COMPLETE_PASSES, ReadingPass, UnbuiltRow,
};
use crate::normalization::{
    AuthorizationProfile, NormalizationClaim, NormalizationMutation, NormalizationSubject,
    canonical_mutation_matrix,
};
use crate::normalization_report::{
    NormalizationReport, ingest_normalization_responses, mutation_wire_spelling, outcome_of,
};
use crate::protocol::{
    LifecycleCheck, NATIVE_PROTOCOL_SCHEMA, NativeNormalizationResponse, NormalizationCaseId,
};

/// The canonical §8.4 conservation matrix, as the emitted document.
///
/// `rows` carries the whole matrix, expectations included, for a reader
/// and for the report. `requests` carries only what may be sent to an
/// executor: the row identity and its subject, and no expected layer
/// `(´[PLAN-rule:guide11-exec:request-subject]´)`. Two shapes rather
/// than one, so the boundary is a property of the data rather than a
/// discipline a runner is trusted to observe.
///
/// Deferred rows appear in `rows` and never in `requests`: a row this
/// wave established it cannot materialize is not asked of a target.
#[must_use]
pub fn conservation_matrix_document() -> serde_json::Value {
    let matrix = canonical_conservation_matrix();
    let requests: Vec<_> = matrix
        .iter()
        .filter(|row| row.is_executed())
        .map(|row| {
            serde_json::json!({
                "schema": NATIVE_PROTOCOL_SCHEMA,
                "case": row.id,
                "subject": row.subject(),
            })
        })
        .collect();
    serde_json::json!({
        "schema": NATIVE_PROTOCOL_SCHEMA,
        "rows": matrix,
        "requests": requests,
    })
}

/// The canonical §10.4 normalization matrix, as the emitted document.
///
/// The same two shapes the conservation matrix carries, for the same
/// reason: a request states the claim and the mutation and no expected
/// outcome, so an executor cannot report a disagreement it never
/// observed `(´[PLAN-rule:guide11-exec:request-subject]´)`.
///
/// # Errors
///
/// A note where the canonical claim does not conserve the value it
/// consumes. That was an assertion inside the command, which is to say
/// a crash; here it is a refusal a caller can observe, which is what
/// makes it testable at all.
pub fn normalization_matrix_document() -> Result<serde_json::Value, String> {
    let matrix = canonical_mutation_matrix();
    let claim = NormalizationClaim::canonical();
    if !claim.conserves() {
        return Err(
            "the canonical claim does not conserve the value it consumes, so the matrix \
             it would state is not the one §10.4 defines"
                .to_owned(),
        );
    }

    let mut requests = Vec::new();
    for row in &matrix {
        let spelling = mutation_wire_spelling(row.mutation);
        requests.push(serde_json::json!({
            "schema": NATIVE_PROTOCOL_SCHEMA,
            "case": NormalizationCaseId { normalization: spelling },
            "subject": NormalizationSubject {
                claim: claim.clone(),
                mutation: row.mutation,
            },
        }));
    }

    Ok(serde_json::json!({
        "schema": NATIVE_PROTOCOL_SCHEMA,
        "claim": claim,
        "rows": matrix,
        "requests": requests,
    }))
}

/// The §10.4 normalization report, built from one run record.
///
/// # Errors
///
/// A note naming what the record does not state, where it is not a
/// census of the canonical matrix, or where the report does not
/// serialize.
pub fn normalization_report_document(
    record: &serde_json::Value,
) -> Result<NormalizationReport, String> {
    let responses: Vec<NativeNormalizationResponse> =
        serde_json::from_value(record["responses"].clone()).map_err(|error| {
            format!("the run record's responses are not this protocol's: {error}")
        })?;

    // Indexed by the mutation each response answers, with the census
    // checked exactly in both directions: the report is a statement
    // about the canonical matrix, so a duplicated row, an unanswered
    // row, and a row the matrix does not carry are each a refusal
    // rather than something this command quietly resolves.
    let answered = ingest_normalization_responses(responses)
        .map_err(|defect| format!("the run record is not a census of the matrix: {defect}"))?;

    let mut rows = Vec::new();
    let mut profile = None;
    for row in canonical_mutation_matrix() {
        let spelling = mutation_wire_spelling(row.mutation);
        let response = answered
            .get(&spelling)
            .ok_or_else(|| format!("the run answered no row for {spelling}"))?;
        let outcome = outcome_of(row.mutation, row.expected, response);
        // The profile the run observed, taken from the unmutated row: it
        // is the one row whose signature was never disturbed, so it is
        // the only one whose witness reports the profile the owner
        // actually authorized under.
        if row.mutation == NormalizationMutation::None {
            profile = outcome.authorization_profile;
        }
        rows.push(outcome);
    }

    Ok(NormalizationReport {
        role: ConservationReportRole::Experimental,
        claim: NormalizationClaim::canonical(),
        disclosures: normalization_declassifications(),
        authorization_profile: profile,
        observed_genesis: string_at(record, "environment", "genesis_id"),
        observed_network: string_at(record, "environment", "network_id"),
        declared_tip: optional_string_at(record, "handshake", "intended_executed_tip"),
        binary_reported_revision: optional_string_at(
            record,
            "handshake",
            "binary_reported_revision",
        ),
        rows,
    })
}

/// The §14.1 fresh-process lifecycle report, built from one run record.
///
/// Every expectation is rebuilt from [`canonical_lifecycle_matrix`]
/// rather than read from the record. The runner is the process that
/// corrupts the public record — it builds the wrong txid, the copied
/// bytes, the foreign genesis — so a report that took its expectations
/// from the runner would be letting the runner mark its own work, and
/// the one row it would never fail is the one it got wrong.
///
/// The gates are checked here rather than left to a reader: every other
/// fact in the report is worthless if the creator was still running, or
/// its wallet still on disk, while the chain was read.
///
/// # Errors
///
/// A note naming what the record does not state, or which gate the run
/// does not meet.
pub fn lifecycle_report_document(
    record: &serde_json::Value,
) -> Result<FreshProcessLifecycleReport, String> {
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

    lifecycle_gates_hold(&report)?;
    Ok(report)
}

/// Every condition the lifecycle report must meet before it is emitted.
fn lifecycle_gates_hold(report: &FreshProcessLifecycleReport) -> Result<(), String> {
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
/// the check or the report is not built
/// `(´[PLAN-rule:guide12-exec:protocol-revision]´)`.
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

fn string_at(record: &serde_json::Value, section: &str, field: &str) -> String {
    record[section][field]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn optional_string_at(record: &serde_json::Value, section: &str, field: &str) -> Option<String> {
    record[section][field].as_str().map(ToOwned::to_owned)
}
