//! Import, declaration and acquisition boundaries for public observation.

use serde_json::{Value, json};
use vectors::maturity_observer::{
    ObserverManifest, ObserverNodeReader, encode_hex, observe_maturity, render_maturity_observation,
};

#[test]
fn observer_imports_and_input_surface_are_pinned() {
    let source = include_str!("maturity_observer.rs");
    let imports: Vec<_> = source
        .lines()
        .filter(|line| line.starts_with("use "))
        .collect();
    assert_eq!(
        imports,
        [
            "use std::io::{self, IsTerminal, Read, Write};",
            "use vectors::maturity_observer::ObserverManifest;",
            "use vectors::maturity_observer::ObserverNodeReader;",
            "use vectors::maturity_observer::observe_maturity;",
            "use vectors::maturity_observer::render_maturity_observation;",
        ]
    );
    assert_eq!(
        source.matches("vectors::").count(),
        imports
            .iter()
            .filter(|line| line.starts_with("use vectors::"))
            .count()
    );
    for line in source
        .lines()
        .filter(|line| line.trim_start().starts_with("use "))
    {
        assert!(imports.contains(&line), "unpinned import: {line}");
    }
    for forbidden in [
        "std::fs",
        "std::path",
        "std::env",
        "std::process",
        "include_bytes!",
        "include_str!",
        "maturity_corpus",
        "run_of_record",
        "linker::",
        "CandidateDeploymentIdentity",
        "TargetOperationPlanner",
        "MaturityAnnouncementPlanner",
        "accepted_public_handoff",
    ] {
        assert!(
            !source.contains(forbidden),
            "forbidden import or input: {forbidden}"
        );
    }
    let implementation = include_str!("../src/maturity_observer.rs");
    pin_implementation_imports(implementation);
    refuse_grouped_process_imports(implementation);
    for forbidden in [
        "std::fs",
        "std::path",
        "std::env",
        "std::process",
        "include_bytes!",
        "include_str!",
        "maturity_corpus",
        "run_of_record",
        "linker::",
        "CandidateDeploymentIdentity",
        "TargetOperationPlanner",
        "MaturityAnnouncementPlanner",
        "accepted_public_handoff",
    ] {
        assert!(
            !implementation.contains(forbidden),
            "forbidden acquisition dependency: {forbidden}"
        );
    }
    scan_observer_call_path();
}

fn scan_observer_call_path() {
    // History and recovery call continuity and closure; their reports also call
    // safety::rows_of through collect_rows and report::MaturitySafetyCompleteness::name.
    for (name, module) in [
        ("history", include_str!("../src/maturity_history.rs")),
        (
            "history_report",
            include_str!("../src/maturity_history_report.rs"),
        ),
        ("recovery", include_str!("../src/maturity_recovery.rs")),
        (
            "recovery_report",
            include_str!("../src/maturity_recovery_report.rs"),
        ),
        ("continuity", include_str!("../src/maturity_continuity.rs")),
        ("closure", include_str!("../src/maturity_closure.rs")),
        ("safety", include_str!("../src/maturity_safety.rs")),
        ("report", include_str!("../src/maturity_report.rs")),
    ] {
        scan_non_test_module(name, module);
    }
}

fn pin_implementation_imports(source: &str) {
    let imports: Vec<_> = source
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("use "))
        .collect();
    assert_eq!(
        imports,
        [
            "use std::fmt::Write as _;",
            "use realization::{AnnouncementLeadBounds, Cycle};",
            "use serde_json::{Value, json};",
            "use tapscript::{",
            "use target_elements::TargetContractVersion;",
            "use transaction::bytes::{TargetTransaction, Txid};",
            "use crate::maturity_closure::{OracleStateCurve, closure_target};",
            "use crate::maturity_continuity::{MaturityByteSource, submitted_transaction_identities};",
            "use crate::maturity_history::{StateRootEdge, validate_state_root_history};",
            "use crate::maturity_history_report::validate_observed_root_history;",
            "use crate::maturity_recovery::{PublicAnnouncementHandoff, PublicAnnouncementLocator};",
            "use crate::maturity_recovery_report::validate_observed_public_recovery;",
            "use ObserverFactProvenance::{",
            "use target_elements_conformance::constructor::tagged::sha256;",
        ]
    );
}

fn refuse_grouped_process_imports(source: &str) {
    for statement in source.split(';') {
        let compact: String = statement.chars().filter(|c| !c.is_whitespace()).collect();
        if let Some((_, group)) = compact.split_once("usestd::{") {
            let names: Vec<_> = group
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .collect();
            for forbidden in ["fs", "env", "process", "path"] {
                assert!(
                    !names.contains(&forbidden),
                    "grouped process import: {forbidden}"
                );
            }
        }
    }
}

fn scan_non_test_module(name: &str, module: &str) {
    // Rustfmt leaves the enclosing module's closing brace at column zero.
    let (before, tests) = module.split_once("#[cfg(test)]").expect("test module");
    assert!(matches!(
        tests.trim_start().lines().next(),
        Some("mod tests {" | "pub(crate) mod tests {")
    ));
    let (_, after) = tests
        .split_once("\n}\n")
        .expect("test module closing brace");
    refuse_grouped_process_imports(before);
    refuse_grouped_process_imports(after);
    for forbidden in [
        "maturity_corpus",
        "run_of_record",
        "include_bytes!",
        "include_str!",
        "std::fs",
        "std::path",
        "std::env",
        "std::process",
    ] {
        assert!(
            !before.contains(forbidden) && !after.contains(forbidden),
            "{name}: forbidden acquisition dependency {forbidden}"
        );
    }
}

fn public_operands() -> (Value, Value) {
    use vectors::maturity_corpus::accepted_public_handoff;
    use vectors::maturity_corpus::maturity_variable_run_of_record;
    use vectors::maturity_native::MaturityAcceptanceObligation;
    let corpus = maturity_variable_run_of_record().expect("archive for boundary tests only");
    let handoff = accepted_public_handoff(corpus).expect("public handoff");
    let MaturityAcceptanceObligation::Established { readback, .. } =
        corpus.evidence().acceptance_obligation()
    else {
        panic!("accepted test subject")
    };
    let candidate = json!({"witness_schedule":"variable-metadata","minimum_lead":4,"maximum_lead":6,"static_root":encode_hex(handoff.static_subtree().root()),"internal_policy":"state-nums","contract_revision":"elements-tapscript-v2"});
    let mut manifest = candidate.clone();
    let object = manifest.as_object_mut().expect("object");
    object.insert("schema".to_owned(), json!(1));
    object.insert("chain_name".to_owned(), json!("elementsregtest"));
    object.insert("chain_id".to_owned(), json!(corpus.report().network_id()));
    object.insert("genesis_id".to_owned(), json!(corpus.report().genesis_id()));
    object.insert(
        "transaction_id".to_owned(),
        json!(readback.identity().to_target_display()),
    );
    object.insert("state_output_index".to_owned(), json!(0));
    object.insert("candidate_declaration".to_owned(), candidate);
    object.insert(
        "abi_declaration".to_owned(),
        json!({"witness_schedule":"variable-metadata","contract_revision":"elements-tapscript-v2"}),
    );
    let block = encode_hex(readback.block_hash());
    let reading = json!({"chain_name":"elementsregtest","genesis_id":corpus.report().genesis_id(),"block_hash":block,"block_height":readback.block_height(),"active_before":block,"active_after":block,"transaction_id":readback.identity().to_target_display(),"witness_identity":readback.witness_identity().to_target_display(),"raw_transaction":encode_hex(readback.bytes()),"members":[readback.identity().to_target_display()],"relay_floor":"0.00000100","mempool_floor":"0.00000100","block_floor":"0.00000100","block_provenance":"pinned-source-no-override","fee_overrides":[]});
    (manifest, reading)
}

#[test]
fn closed_manifest_refuses_every_missing_and_private_field() {
    let (manifest, _) = public_operands();
    ObserverManifest::from_value(manifest.clone()).expect("control");
    for key in manifest.as_object().expect("object").keys() {
        let mut missing = manifest.clone();
        missing.as_object_mut().expect("object").remove(key);
        let refusal = ObserverManifest::from_value(missing).expect_err("missing field");
        assert_eq!(refusal, format!("missing-public-input:{key}"));
    }
    for key in [
        "retained_instance",
        "descriptor",
        "operator_key",
        "datadir",
        "path",
        "wallet",
        "unknown",
    ] {
        let mut changed = manifest.clone();
        changed[key] = json!("forbidden");
        assert_eq!(
            ObserverManifest::from_value(changed).expect_err("extra field"),
            "unknown-publication-field",
            "extra {key}"
        );
    }
}

#[test]
fn boundary_replay_checks_seven_outcomes_and_separate_report_roles() {
    let (manifest, reading) = public_operands();
    let m = ObserverManifest::from_value(manifest).expect("manifest");
    let r = ObserverNodeReader::from_value(reading).expect("reading");
    let result = observe_maturity(&m, &r).expect("boundary replay, not fresh acquisition");
    let rendered: Value =
        serde_json::from_str(&render_maturity_observation(&result)).expect("JSON");
    assert_eq!(rendered["experiment"].as_array().expect("rows").len(), 7);
    assert_eq!(rendered["experiment"][5]["equal_final_cursor"], true);
    for (field, role) in [
        ("history_report", "state-root-history"),
        ("recovery_report", "state-public-recovery"),
    ] {
        let report = rendered[field].as_str().expect("report");
        assert!(report.contains(role));
        assert!(report.contains("residual no-root-cursor-freshness\n"));
        assert!(report.contains("residual no-archive-origin-beyond-admission\n"));
        if field == "recovery_report" {
            assert!(report.contains("residual no-chain-observation\n"));
            assert!(report.contains(
                "acquisition_residual single-node-acquisition-only replaces no-chain-observation\n"
            ));
        } else {
            assert!(!report.contains("residual no-chain-observation\n"));
        }
        assert!(report.contains("transaction_locator node-interface") || field == "history_report");
    }
}

#[test]
fn source_tag_cannot_grant_observation_credit() {
    use vectors::maturity_continuity::MaturityByteSource;
    use vectors::maturity_corpus::{accepted_public_handoff, maturity_variable_run_of_record};
    use vectors::maturity_recovery::{PublicAnnouncementHandoff, PublicAnnouncementLocator};
    use vectors::maturity_recovery_report::{
        assemble_maturity_public_recovery_report, render_maturity_public_recovery_report,
        validate_maturity_public_recovery_report,
    };
    let handoff = accepted_public_handoff(maturity_variable_run_of_record().expect("archive"))
        .expect("handoff");
    let tagged = PublicAnnouncementHandoff::new(
        PublicAnnouncementLocator::new(
            MaturityByteSource::NodeInterface {
                binding: "unobserved".to_owned(),
            },
            handoff.locator().identity(),
        ),
        handoff.bytes().to_vec(),
        handoff.state_output_index(),
        handoff.schedule(),
        handoff.bounds(),
        handoff.static_subtree().clone(),
        handoff.internal_key(),
        handoff.contract(),
    );
    let structural =
        assemble_maturity_public_recovery_report(tagged.clone()).expect("structural assembly");
    let validated = validate_maturity_public_recovery_report(&structural, &tagged)
        .expect("structural validation");
    assert!(
        render_maturity_public_recovery_report(&validated)
            .contains("residual no-chain-observation\n")
    );
    let (manifest, reading) = public_operands();
    let m = ObserverManifest::from_value(manifest).expect("manifest");
    for (field, changed) in [
        ("active_after", json!("de".repeat(32))),
        ("members", json!([])),
        ("block_height", json!(0)),
        ("raw_transaction", json!("00")),
        ("fee_overrides", json!(["override"])),
    ] {
        let mut bad = reading.clone();
        bad[field] = changed;
        let r = ObserverNodeReader::from_value(bad).expect("closed shape");
        assert!(observe_maturity(&m, &r).is_err(), "unbound {field}");
    }
}
