//! Source/dependency control and isolated compilation, not semantic proof.
use super::{bounded, fixture, one_test};
use std::process::Command;

const FILES: &[&str] = &[
    "src/keyed_history_reader.rs",
    "src/keyed_history_reader/evidence.rs",
    "src/keyed_history_reader/refusal.rs",
    "src/keyed_history_reader/identity.rs",
    "src/keyed_history_reader/authentication.rs",
    "src/keyed_history_reader/fold.rs",
];
fn permitted(source: &str, root: bool) -> bool {
    let code = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .filter(|line| {
            !root
                || ![
                    "mod authentication;",
                    "mod evidence;",
                    "mod fold;",
                    "mod identity;",
                    "mod refusal;",
                ]
                .contains(&line.trim())
        })
        .collect::<Vec<_>>()
        .join("\n");
    allowed_paths(&code)
        && !code
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .any(|token| {
                matches!(
                    token,
                    "cfg"
                        | "cfg_attr"
                        | "mod"
                        | "path"
                        | "include"
                        | "include_str"
                        | "include_bytes"
                        | "macro_rules"
                        | "keyed_request"
                        | "keyed_construct"
                        | "bind_keyed_members"
                        | "keyed_plan"
                        | "compiler"
                        | "realization"
                        | "keyed_history_model"
                        | "model_checks"
                        | "Packet"
                        | "extern"
                )
            })
}
fn allowed_paths(code: &str) -> bool {
    const ALLOWED: &[&str] = &[
        "transaction::TargetTransaction",
        "transaction::TargetOutput",
        "transaction::InputWitness",
        "transaction::TransactionRefusal",
        "target_elements_conformance::constructor::tagged::sha256",
        "target_elements::reviewed_elements_tapscript",
        "tapscript::keyed_constructor::KeyedSemanticKey",
        "crate::keyed_constructor::construct_keyed",
        "crate::keyed_constructor::PublicKeyedCurve",
        "crate::keyed_constructor::KeyedConstructorDescriptor",
        "crate::keyed_constructor::KeyedConstructorRefusal",
        "crate::keyed_stack_core",
        "tracing::info_span",
        "tracing::info",
        "tracing::debug",
    ];
    // String literals are not Rust paths (the tracing target is a string).
    let mut quoted = false;
    let mut escaped = false;
    let code: String = code
        .chars()
        .map(|c| {
            if quoted {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    quoted = false;
                }
                ' '
            } else if c == '"' {
                quoted = true;
                ' '
            } else {
                c
            }
        })
        .collect();
    code.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':'))
        .all(|token| {
            if !token.contains("::") {
                return true;
            }
            let token = token.trim_matches(':');
            let first = token.split("::").next().unwrap_or_default();
            if [
                "transaction",
                "target_elements",
                "target_elements_conformance",
                "tapscript",
                "crate",
                "tracing",
            ]
            .contains(&first)
            {
                return ALLOWED.iter().any(|allowed| {
                    token == *allowed
                        || token
                            .strip_prefix(allowed)
                            .is_some_and(|rest| rest.starts_with("::"))
                });
            }
            !token.contains("::")
                || first.starts_with(char::is_uppercase)
                || [
                    "std",
                    "u8",
                    "u32",
                    "usize",
                    "super",
                    "self",
                    "authentication",
                    "evidence",
                    "fold",
                    "identity",
                    "refusal",
                    "core",
                ]
                .contains(&first)
        })
}
#[test]
fn keyed_reader_import_boundary_and_isolated_compile() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for (index, file) in FILES.iter().enumerate() {
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        assert!(permitted(&source, index == 0), "{file}");
    }
    for mutant in [
        "#[cfg(test)] use elements::Transaction;",
        "use compiler as hidden;",
        "let text = \"//\"; keyed_construct();",
        "mod hidden;",
        "use crate::keyed_history_model;",
        "bind_keyed_members();",
        "use transaction::keyed_inputs::KeyedInputRole;",
        "use transaction::{keyed_inputs};",
        "use another_crate::Answer;",
    ] {
        assert!(!permitted(mutant, false), "{mutant}");
    }
    let temporary = tempfile::tempdir().unwrap();
    let harness = temporary.path().join("reader-alone.rs");
    std::fs::create_dir(temporary.path().join("keyed_history_reader")).unwrap();
    for file in FILES
        .iter()
        .copied()
        .chain(["src/keyed_constructor.rs", "src/keyed_stack_core.rs"])
    {
        let relative = file.strip_prefix("src/").unwrap();
        std::fs::copy(root.join(file), temporary.path().join(relative)).unwrap();
    }
    std::fs::write(&harness, "#![allow(dead_code)]\nmod keyed_constructor;\nmod keyed_stack_core;\npub mod keyed_history_reader;\nfn main() {}\n").unwrap();
    let executable = std::env::current_exe().unwrap();
    let (units, directories, fingerprint) = artifact_layout(&executable);
    let dependencies: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fingerprint).unwrap()).unwrap();
    let mut command = Command::new("/usr/bin/timeout");
    command
        .args([
            "--kill-after=1",
            "60",
            "rustc",
            "--edition=2024",
            "-Dwarnings",
            "--emit=metadata",
        ])
        .arg(&harness)
        .arg("-o")
        .arg(temporary.path().join("reader-alone.rmeta"));
    for directory in &directories {
        command
            .arg("-L")
            .arg(format!("dependency={}", directory.display()));
    }
    for name in [
        "transaction",
        "tapscript",
        "target_elements",
        "target_elements_conformance",
        "tracing",
        "serde",
    ] {
        let fingerprint = dependencies["deps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row[1] == name)
            .unwrap()[3]
            .as_u64()
            .unwrap();
        let expected = format!("{:016x}", fingerprint.swap_bytes());
        let library = units
            .iter()
            .find_map(|(fingerprints, output, hash)| {
                let value =
                    std::fs::read_to_string(fingerprints.join(format!("lib-{name}"))).ok()?;
                (value.trim() == expected).then(|| output.join(format!("lib{name}-{hash}.rmeta")))
            })
            .expect("the current test's exact dependency fingerprint has a retained artifact");
        assert!(library.is_file(), "{}", library.display());
        command
            .arg("--extern")
            .arg(format!("{name}={}", library.display()));
    }
    bounded(
        &mut command,
        "reader isolated compile timeout=60s explicit substrate externs only",
    );
}

#[test]
fn keyed_reader_trace_has_one_completion_or_refusal() {
    one_test(&bounded(
        &mut super::child("keyed_history::reader_boundary::reader_trace_child", "10"),
        "reader trace timeout=10s",
    ));
}
#[test]
#[ignore = "bounded capture parent launches exactly this child"]
fn reader_trace_child() {
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
    super::reader::compare(&fixture);
    captured.lock().unwrap().clear();
    super::reader::trace_calls(&fixture);
    let output = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
    assert_eq!(
        output
            .lines()
            .filter(|line| line.contains("keyed history read completed"))
            .count(),
        1,
        "{output}"
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.contains("keyed history read refused"))
            .count(),
        1,
        "{output}"
    );
    assert!(
        output.contains("refusal_kind=\"duplicate_event\""),
        "{output}"
    );
    assert!(output.contains("read_keyed_history"), "{output}");
    assert!(!output.contains("evaluate_keyed_model"), "{output}");
    assert!(!output.contains("keyed model evaluated"), "{output}");
}

type Unit = (std::path::PathBuf, std::path::PathBuf, String);
fn artifact_layout(
    executable: &std::path::Path,
) -> (Vec<Unit>, Vec<std::path::PathBuf>, std::path::PathBuf) {
    let directory = executable.parent().unwrap();
    let mut units = Vec::new();
    let mut directories = Vec::new();
    if directory.file_name().unwrap() == "out" {
        let current = directory.parent().unwrap();
        let build = current.parent().unwrap().parent().unwrap();
        for package in std::fs::read_dir(build).unwrap() {
            let package = package.unwrap().path();
            if !package.is_dir() {
                continue;
            }
            for unit in std::fs::read_dir(package).unwrap() {
                let unit = unit.unwrap().path();
                let output = unit.join("out");
                if output.is_dir() {
                    directories.push(output.clone());
                    units.push((
                        unit.join("fingerprint"),
                        output,
                        unit.file_name().unwrap().to_str().unwrap().to_owned(),
                    ));
                }
            }
        }
        (
            units,
            directories,
            current.join("fingerprint/test-integration-test-maturity_native.json"),
        )
    } else {
        assert_eq!(directory.file_name().unwrap(), "deps");
        let fingerprints = directory.parent().unwrap().join(".fingerprint");
        for unit in std::fs::read_dir(&fingerprints).unwrap() {
            let unit = unit.unwrap().path();
            let name = unit.file_name().unwrap().to_str().unwrap();
            let Some((_, hash)) = name.rsplit_once('-') else {
                continue;
            };
            units.push((unit.clone(), directory.to_owned(), hash.to_owned()));
        }
        directories.push(directory.to_owned());
        let name = executable.file_name().unwrap().to_str().unwrap();
        let hash = name.rsplit_once('-').unwrap().1;
        (
            units,
            directories,
            fingerprints.join(format!(
                "tripod-vectors-{hash}/test-integration-test-maturity_native.json"
            )),
        )
    }
}
