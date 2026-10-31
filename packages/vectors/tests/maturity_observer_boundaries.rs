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
    // History and recovery call continuity and closure. Only recovery's report
    // calls safety::rows_of through collect_rows; both reports call completeness.name().
    let modules = [
        ("observer", include_str!("../src/maturity_observer.rs")),
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
        (
            "negative_half",
            include_str!("../src/maturity_negative_half.rs"),
        ),
    ];
    for (name, module) in &modules {
        let (before, after) = non_test_regions(module);
        if *name == "negative_half" {
            // The named archive-bearing import is checked by the allowlist;
            // only outstanding_rows may use it in this scanned module.
            let mut source = without_comments(before);
            let bodies = callable_bodies(&source, "outstanding_rows");
            assert_eq!(bodies.len(), 1);
            let body = bodies[0].to_owned();
            source = source.replacen(&body, "{}", 1);
            source = source.replacen("pub fn outstanding_rows()", "pub fn excluded_helper()", 1);
            source = source.replacen("    derive_maturity_evidence_plan_with,", "", 1);
            scan_acquisition_names(name, &source);
        } else {
            scan_acquisition_names(name, before);
        }
        scan_acquisition_names(name, after);
    }
    pin_callable_imports(&modules);
}

#[derive(Clone, Copy, Debug)]
enum ImportKind {
    TypeOnly,
    ArchiveFree,
    ArchiveBearing,
}

fn non_test_regions(module: &str) -> (&str, &str) {
    let Some((before, tests)) = module
        .split_once("#[cfg(test)]\nmod tests {")
        .or_else(|| module.split_once("#[cfg(test)]\npub(crate) mod tests {"))
    else {
        return (module, "");
    };
    let (_, after) = tests
        .split_once("\n}\n")
        .expect("test module closing brace");
    (before, after)
}

fn scan_acquisition_names(name: &str, source: &str) {
    let source = without_comments(source);
    refuse_grouped_process_imports(&source);
    for forbidden in [
        "super::",
        "maturity_corpus",
        "run_of_record",
        "include_bytes!",
        "include_str!",
        "std::fs",
        "std::path",
        "std::env",
        "std::process",
        "outstanding_rows",
        "derive_maturity_evidence_",
    ] {
        assert!(
            !source.contains(forbidden),
            "{name}: forbidden acquisition dependency {forbidden}"
        );
    }
}

fn callable_bodies<'a>(source: &'a str, item: &str) -> Vec<&'a str> {
    let mut bodies = Vec::new();
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let names_item = line
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .any(|word| word == item);
        if (line.trim_start().starts_with("impl") && names_item)
            || line.contains(&format!("fn {item}("))
        {
            let rest = &source[offset..];
            let start = rest.find('{').expect("callable body");
            let mut depth = 0;
            let mut end = None;
            for (position, c) in rest[start..].char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(start + position + 1);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            bodies.push(&rest[start..end.expect("closed callable body")]);
        }
        offset += line.len();
    }
    bodies
}

#[expect(
    clippy::too_many_lines,
    reason = "the import exceptions retain one reason per named item"
)]
fn pin_callable_imports(modules: &[(&str, &str)]) {
    use std::collections::BTreeSet;
    let scanned: BTreeSet<_> = modules
        .iter()
        .map(|(name, _)| format!("maturity_{name}"))
        .collect();
    let mut found = BTreeSet::new();
    for (_, module) in modules {
        for region in <[&str; 2]>::from(non_test_regions(module)) {
            let source = without_comments(region);
            for statement in source.split("use crate::").skip(1) {
                let statement = statement.split_once(';').expect("import terminator").0;
                let (module, items) = statement.split_once("::").expect("qualified import");
                if scanned.contains(module) && module != "maturity_negative_half" {
                    continue;
                }
                for item in items.trim().trim_matches(['{', '}']).split(',') {
                    let item = item.trim();
                    if !item.is_empty() {
                        assert!(
                            !item.contains("::"),
                            "nested import needs an explicit scan: {item}"
                        );
                        found.insert((module.to_owned(), item.to_owned()));
                    }
                }
            }
            for path in source.split("crate::").skip(1) {
                let Some((module, rest)) = path.split_once("::") else {
                    continue;
                };
                if scanned.contains(module) && module != "maturity_negative_half" {
                    continue;
                }
                let item: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if !item.is_empty() {
                    found.insert((module.to_owned(), item));
                }
            }
        }
    }
    let allowlist = [
        (
            "maturity_negative_half",
            "MaturityNegativeHalfGap",
            ImportKind::TypeOnly,
            "The gap enum imports no callable.",
        ),
        (
            "maturity_negative_half",
            "MaturityNegativeHalfLink",
            ImportKind::TypeOnly,
            "The link enum imports no callable.",
        ),
        (
            "maturity_negative_half",
            "STILL_REQUIRED",
            ImportKind::TypeOnly,
            "The register constant imports no callable.",
        ),
        (
            "maturity_evidence",
            "MaturityConstructorMaterial",
            ImportKind::ArchiveFree,
            "Inline material construction retains supplied evidence without archive acquisition.",
        ),
        (
            "maturity_evidence",
            "MaturityConstructorMaterialAbsence",
            ImportKind::TypeOnly,
            "The inline absence variant imports no callable.",
        ),
        (
            "maturity_negative_half",
            "gap_census",
            ImportKind::ArchiveFree,
            "report::validate_register reads only the fixed register census; gap_census does not call outstanding_rows or archive derivation.",
        ),
        (
            "maturity_fixture",
            "MaturitySemanticCase",
            ImportKind::ArchiveFree,
            "Semantic case accessors read the admitted predecessor and request.",
        ),
        (
            "error",
            "VectorError",
            ImportKind::ArchiveFree,
            "Error formatting carries no acquisition input.",
        ),
        (
            "matrix",
            "EvidenceBoundary",
            ImportKind::ArchiveFree,
            "Boundary names classify observations without acquiring bytes.",
        ),
        (
            "matrix",
            "MutationLayer",
            ImportKind::ArchiveFree,
            "Mutation names classify coordinates without acquisition.",
        ),
        (
            "maturity_evidence",
            "MaturityEvidenceCensus",
            ImportKind::ArchiveFree,
            "Census accessors read retained counts.",
        ),
        (
            "maturity_evidence",
            "MaturityGuaranteeQuantifier",
            ImportKind::ArchiveFree,
            "Quantifier accessors and formatting read typed classes.",
        ),
        (
            "maturity_evidence",
            "MaturityObservationClass",
            ImportKind::ArchiveFree,
            "Observation class naming reads an enum.",
        ),
        (
            "maturity_evidence",
            "MaturityExecutorProvenanceExpectation",
            ImportKind::ArchiveFree,
            "Provenance accessors read the supplied expectation.",
        ),
        (
            "maturity_evidence",
            "MaturityMutationRegistry",
            ImportKind::ArchiveFree,
            "Registry constructors enumerate typed mutations.",
        ),
        (
            "maturity_evidence",
            "MaturityMutationRegistryKind",
            ImportKind::ArchiveFree,
            "Registry kind naming reads an enum.",
        ),
        (
            "maturity_evidence",
            "MaturityTargetBinding",
            ImportKind::ArchiveFree,
            "Binding accessors read supplied target and deployment.",
        ),
        (
            "maturity_evidence",
            "MaturityAnnouncementEvidencePlan",
            ImportKind::ArchiveFree,
            "Plan accessors read retained fields; derivation is a separate function.",
        ),
        (
            "maturity_evidence",
            "MaturityCarrierExecution",
            ImportKind::TypeOnly,
            "Execution disposition imports no callable.",
        ),
        (
            "maturity_evidence",
            "MaturityConstructorMaterialPresence",
            ImportKind::ArchiveFree,
            "Presence naming reads an enum.",
        ),
        (
            "maturity_evidence",
            "MaturityContinuityObservation",
            ImportKind::ArchiveFree,
            "Observation accessors read retained comparisons.",
        ),
        (
            "maturity_evidence",
            "MaturityContinuityRecord",
            ImportKind::ArchiveFree,
            "Record accessors read retained observations.",
        ),
        (
            "maturity_evidence",
            "MaturityRowStanding",
            ImportKind::ArchiveFree,
            "Standing classification reads the carried enum.",
        ),
        (
            "maturity_evidence",
            "MaturityEvidenceRefusal",
            ImportKind::TypeOnly,
            "The refusal imports no acquisition callable.",
        ),
        (
            "maturity_evidence",
            "derive_maturity_evidence_plan_with",
            ImportKind::ArchiveBearing,
            "negative_half::outstanding_rows calls archive derivation; observer entry points, both observed validators and the binder never call outstanding_rows.",
        ),
        (
            "maturity_native",
            "MaturityAcceptanceObligation",
            ImportKind::ArchiveFree,
            "Obligation accessors inspect a supplied acceptance.",
        ),
        (
            "maturity_native",
            "MaturityAcceptanceRoute",
            ImportKind::TypeOnly,
            "The route enum imports no callable.",
        ),
        (
            "maturity_native",
            "MaturityAcceptedReadback",
            ImportKind::ArchiveFree,
            "Readback accessors inspect already admitted bytes and facts.",
        ),
        (
            "maturity_first_party",
            "MaturityFirstPartyValidator",
            ImportKind::ArchiveFree,
            "Validator naming reads an enum, not the discharge campaign.",
        ),
        (
            "subject",
            "ExperimentalSubject",
            ImportKind::ArchiveFree,
            "Subject wrappers retain supplied values without acquisition.",
        ),
    ];
    let expected: BTreeSet<_> = allowlist
        .iter()
        .map(|(module, item, _, _)| (module.to_string(), item.to_string()))
        .collect();
    assert_eq!(
        expected.len(),
        allowlist.len(),
        "duplicate import exception"
    );
    assert_eq!(
        found, expected,
        "callable imports must match the named allowlist"
    );
    for (module, item, kind, reason) in allowlist {
        assert_ne!(reason, "");
        let source = match module {
            "error" => include_str!("../src/error.rs"),
            "matrix" => include_str!("../src/matrix.rs"),
            "maturity_evidence" => include_str!("../src/maturity_evidence.rs"),
            "maturity_native" => include_str!("../src/maturity_native.rs"),
            "maturity_first_party" => include_str!("../src/maturity_first_party.rs"),
            "maturity_fixture" => include_str!("../src/maturity_fixture.rs"),
            "maturity_negative_half" => include_str!("../src/maturity_negative_half.rs"),
            "subject" => include_str!("../src/subject.rs"),
            _ => panic!("unregistered source module"),
        };
        let (before, after) = non_test_regions(source);
        let source = without_comments(&format!("{before}\n{after}"));
        let bodies = callable_bodies(&source, item);
        match kind {
            ImportKind::TypeOnly => {
                assert!(bodies.is_empty(), "{module}::{item} now has callables");
            }
            ImportKind::ArchiveFree => {
                // Some enum types have only derived methods. Pin their definition
                // when there is no explicit implementation to scan.
                for body in &bodies {
                    scan_acquisition_names(item, body);
                }
                assert!(source.contains(item), "missing imported item");
            }
            ImportKind::ArchiveBearing => {
                assert!(!bodies.is_empty(), "missing archive-bearing callable");
                assert!(
                    bodies.iter().any(|body| body.contains("run_of_record")
                        || (body.contains("derive_maturity_evidence_source")
                            && callable_bodies(&source, "derive_maturity_evidence_source")
                                .iter()
                                .any(|source_body| source_body.contains("run_of_record")))),
                    "exception no longer names archive access"
                );
            }
        }
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
    let source = without_comments(source);
    for statement in source.split(';') {
        let has_alias = statement
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .any(|word| word == "as");
        let compact: String = statement.chars().filter(|c| !c.is_whitespace()).collect();
        let compact = compact.replace("use::std", "usestd");
        assert!(!compact.contains("usestdas"), "standard library alias");
        assert!(
            !compact.contains("externcratestd"),
            "standard library extern alias"
        );
        if let Some((_, group)) = compact.split_once("usestd::{") {
            assert!(!has_alias, "standard library group alias");
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

fn without_comments(source: &str) -> String {
    let mut result = String::new();
    let mut chars = source.chars().peekable();
    let mut depth = 0;
    let mut line = false;
    let mut quoted = false;
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if line {
            if c == '\n' {
                line = false;
                result.push(c);
            }
        } else if depth > 0 {
            if c == '/' && chars.peek() == Some(&'*') {
                chars.next();
                depth += 1;
            } else if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                depth -= 1;
            }
        } else if quoted {
            result.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
            }
        } else if c == '/' && chars.peek() == Some(&'/') {
            chars.next();
            line = true;
            result.push(' ');
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            depth = 1;
            result.push(' ');
        } else {
            quoted = c == '"';
            result.push(c);
        }
    }
    result
}

#[test]
fn archive_helpers_cannot_enter_scanned_regions_or_bodies() {
    for subject in [
        "fn injected() { super::sibling::helper(); }",
        "use super::sibling::helper;",
        "fn injected() { outstanding_rows(); }",
        "fn injected() { derive_maturity_evidence_plan_with(); }",
        "fn injected() { derive_maturity_evidence_source(); }",
    ] {
        assert!(std::panic::catch_unwind(|| scan_acquisition_names("subject", subject)).is_err());
    }
}

#[test]
fn a_test_only_import_does_not_hide_following_code() {
    let source = "#[cfg(test)]\nuse crate::fixture::Control;\nfn retained() { outstanding_rows(); }\n#[cfg(test)]\nmod tests {\n}\nfn after() {}\n";
    let (before, after) = non_test_regions(source);
    assert!(before.contains("fn retained()"));
    assert!(after.contains("fn after()"));
    assert!(std::panic::catch_unwind(|| scan_acquisition_names("subject", before)).is_err());
    let crate_visible = source.replace("mod tests {", "pub(crate) mod tests {");
    assert_eq!(non_test_regions(&crate_visible), (before, after));
}

#[test]
fn process_import_spellings_cannot_escape_the_scan() {
    refuse_grouped_process_imports("// use std::{fs};\nuse std::{fmt};");
    for subject in [
        "use std::{/* ; */ fs};",
        "use ::std::{process};",
        "use std as host;",
        "use ::std as host;",
        "extern crate std as host;",
        "extern crate std;",
        "use std::{env /* nested /* ; */ comment */};",
        "use std::{path};",
        "use std::{fs as f};",
        "use std::{self as s};",
    ] {
        assert!(
            std::panic::catch_unwind(|| refuse_grouped_process_imports(subject)).is_err(),
            "{subject}"
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
