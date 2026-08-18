use std::{fs, path::Path};

use crate::{
    RepositoryCensus,
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    label::{Label, LabelShape},
    owner::LabelOwner,
    registry::{LabelMint, LabelRegistry},
    source::{SourceLocation, relative_to},
};

pub fn harvest_attestation(paths: &RepositoryCensus) -> (LabelRegistry, Vec<LabelDiagnostic>) {
    let mut files = vec![paths.attestation_main.clone()];
    let mut sections = paths.attestation_sections.clone();
    sections.sort();
    files.extend(sections);
    let mut registry = LabelRegistry::default();
    let mut diagnostics = Vec::new();
    for path in files {
        let relative = relative_to(&paths.root, &path);
        let Ok(source) = fs::read_to_string(&path) else {
            diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::Io,
                &SourceLocation::new(relative, 1, 1),
                "cannot read attestation source",
            ));
            continue;
        };
        let stripped = crate::participation::latex_participating(&source);
        harvest(
            &relative,
            &stripped,
            "\\label{",
            "",
            &mut registry,
            &mut diagnostics,
        );
        macro_labels(
            &relative,
            &stripped,
            "\\OpenProblem",
            "open:attestation",
            &mut registry,
            &mut diagnostics,
        );
        open_subproblem_labels(&relative, &stripped, &mut registry, &mut diagnostics);
        macro_labels(
            &relative,
            &stripped,
            "\\ProposedInvestigation",
            "invest:attestation",
            &mut registry,
            &mut diagnostics,
        );
    }
    (registry, diagnostics)
}

fn harvest(
    path: &Path,
    source: &str,
    marker: &str,
    prefix: &str,
    registry: &mut LabelRegistry,
    diagnostics: &mut Vec<LabelDiagnostic>,
) {
    let mut remainder = source;
    let mut offset = 0;
    while let Some(found) = remainder.find(marker) {
        let start = offset + found;
        let body = start + marker.len();
        if let Some(close) = source[body..].find('}') {
            let value = format!("{prefix}{}", &source[body..body + close]);
            insert(path, source, start, &value, registry, diagnostics);
            offset = body + close + 1;
            remainder = &source[offset..];
        } else {
            diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidLabel,
                &SourceLocation::new(path, source[..start].lines().count(), 1),
                "unclosed LaTeX label",
            ));
            break;
        }
    }
}
fn macro_labels(
    path: &Path,
    source: &str,
    command: &str,
    prefix: &str,
    registry: &mut LabelRegistry,
    diagnostics: &mut Vec<LabelDiagnostic>,
) {
    let mut offset = 0;
    while let Some(found) = source[offset..].find(command) {
        let start = offset + found;
        let after = start + command.len();
        let Some((_, next)) = braced(source, after) else {
            offset = after;
            continue;
        };
        let Some((name, end)) = braced(source, next) else {
            offset = next;
            continue;
        };
        let value = format!("{prefix}:{}", name.trim());
        insert(path, source, start, &value, registry, diagnostics);
        offset = end;
    }
}
fn open_subproblem_labels(
    path: &Path,
    source: &str,
    registry: &mut LabelRegistry,
    diagnostics: &mut Vec<LabelDiagnostic>,
) {
    let command = "\\OpenSubProblem";
    let mut offset = 0;
    while let Some(found) = source[offset..].find(command) {
        let start = offset + found;
        let after = start + command.len();
        let cursor = after
            + source[after..]
                .chars()
                .take_while(|character| character.is_whitespace())
                .map(char::len_utf8)
                .sum::<usize>();
        if source.as_bytes().get(cursor) != Some(&b'[') {
            offset = after;
            continue;
        }
        let Some(close) = source[cursor + 1..].find(']') else {
            diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::InvalidLabel,
                &SourceLocation::new(path, source[..start].lines().count() + 1, 1),
                "unclosed OpenSubProblem label",
            ));
            break;
        };
        let name = source[cursor + 1..cursor + 1 + close].trim();
        if !name.is_empty() {
            let value = format!("open:attestation:{name}");
            insert(path, source, start, &value, registry, diagnostics);
        }
        offset = cursor + close + 2;
    }
}
fn braced(source: &str, mut cursor: usize) -> Option<(String, usize)> {
    cursor += source[cursor..]
        .chars()
        .take_while(|character| character.is_whitespace())
        .map(char::len_utf8)
        .sum::<usize>();
    if source.as_bytes().get(cursor) != Some(&b'{') {
        return None;
    }
    let begin = cursor + 1;
    let mut depth = 1;
    for (relative, character) in source[begin..].char_indices() {
        if character == '{' {
            depth += 1;
        }
        if character == '}' {
            depth -= 1;
            if depth == 0 {
                let end = begin + relative;
                return Some((source[begin..end].to_owned(), end + 1));
            }
        }
    }
    None
}
fn insert(
    path: &Path,
    source: &str,
    offset: usize,
    value: &str,
    registry: &mut LabelRegistry,
    diagnostics: &mut Vec<LabelDiagnostic>,
) {
    if value.contains('#') {
        return;
    }
    let location = SourceLocation::new(path, source[..offset].lines().count() + 1, 1);
    match Label::parse(value, LabelShape::Attestation) {
        Ok(label) => {
            let mint = LabelMint {
                owner: LabelOwner::Attestation,
                label,
                location: location.clone(),
                home: None,
            };
            if let Err(duplicate) = registry.insert(mint) {
                // ADR-019: a duplicate-mint diagnostic identifies both
                // locations — the duplicate occurrence and the first
                // mint — matching the shared insert_or_diagnose path.
                let original = registry
                    .get(&duplicate.label)
                    .map(|first| {
                        format!("{}:{}", first.location.display_path(), first.location.line)
                    })
                    .unwrap_or_default();
                diagnostics.push(LabelDiagnostic::error(
                    LabelErrorCode::DuplicateLatexLabel,
                    &location,
                    format!("duplicate attestation label {value}; first minted at {original}"),
                ));
            }
        }
        Err(error) => diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::InvalidLabel,
            &location,
            error.to_string(),
        )),
    }
}
