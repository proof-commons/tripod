//! Document weld: structural extraction of the realization document's
//! attached manifest and masthead identities.
//!
//! `realization.md` claims its appendix
//! `(´[RZ-app:realization:architecture]´)` is the generated
//! `architecture.toml` attached **verbatim**, and its
//! masthead states the release identities (schema, hashes, specification
//! pin). These claims are release-integrity surfaces: the weld tests
//! in this crate compare them mechanically against the typed
//! architecture, so changing any attached manifest byte or masthead
//! identity fails deterministically.
//!
//! Extraction is structural — explicit unique markers, not broad
//! substring searches.

use anyhow::bail;

fn observe<T>(
    result: anyhow::Result<T>,
    part: &'static str,
    refusal_kind: &'static str,
) -> anyhow::Result<T> {
    if !tracing::enabled!(target: "artifacts::weld", tracing::Level::DEBUG) {
        return result;
    }
    if result.is_ok() {
        tracing::debug!(
            target: "artifacts::weld",
            outcome = "completed",
            part,
            "document weld extracted"
        );
    } else {
        tracing::debug!(
            target: "artifacts::weld",
            outcome = "refused",
            part,
            refusal_kind,
            "document weld extraction refused"
        );
    }
    result
}

fn set_refusal_kind(slot: &mut &'static str, kind: &'static str) {
    if tracing::enabled!(target: "artifacts::weld", tracing::Level::DEBUG) {
        *slot = kind;
    }
}

/// The document masthead: every line before the first thematic break
/// (`---`). This is where the document states its release identities.
///
/// # Errors
///
/// Returns an error when the document has no thematic break.
pub fn masthead(document: &str) -> anyhow::Result<&str> {
    let span = tracing::span!(
        target: "artifacts::weld",
        tracing::Level::DEBUG,
        "extract_weld",
        part = "masthead"
    );
    let _entered = span.enter();
    let result = document
        .find("\n---\n")
        .map(|end| &document[..end])
        .ok_or_else(|| {
            anyhow::anyhow!("realization document has no thematic break after the masthead")
        });
    observe(result, "masthead", "missing_thematic_break")
}

/// Extract the verbatim `architecture.toml` bytes attached under the
/// unique appendix heading
/// `(´[RZ-app:realization:architecture]´)`.
///
/// Structural rules enforced:
///
/// 1. exactly one heading line starting with `## Appendix` and citing
///    `(´[RZ-app:realization:architecture]´)`;
/// 2. exactly one fenced `toml` block after that heading;
/// 3. the fence must close.
///
/// The returned string is the block's content with each line's
/// original bytes and one trailing newline — the exact attached file.
///
/// # Errors
///
/// Returns an error when any structural rule fails.
pub fn extract_appendix_toml(document: &str) -> anyhow::Result<String> {
    let span = tracing::span!(
        target: "artifacts::weld",
        tracing::Level::DEBUG,
        "extract_weld",
        part = "appendix"
    );
    let _entered = span.enter();
    let mut refusal_kind = "missing_appendix_heading";
    let result = extract_appendix_core(document, &mut refusal_kind);
    observe(result, "appendix", refusal_kind)
}

fn extract_appendix_core(
    document: &str,
    refusal_kind: &mut &'static str,
) -> anyhow::Result<String> {
    let heading_indices = document
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            line.starts_with("## Appendix") && line.contains("`app:realization:architecture`")
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [heading_index] = heading_indices.as_slice() else {
        set_refusal_kind(refusal_kind, "appendix_heading_count");
        bail!(
            "expected exactly one app:realization:architecture appendix heading, found {}",
            heading_indices.len(),
        );
    };

    let section_lines = document.lines().skip(heading_index + 1);

    let mut block = String::new();
    let mut state = FenceState::Before;

    for line in section_lines {
        match state {
            FenceState::Before => {
                if line == "```toml" {
                    state = FenceState::Inside;
                } else if line.starts_with("```") {
                    set_refusal_kind(refusal_kind, "unexpected_fence");
                    bail!("unexpected non-toml fence in the appendix before the manifest block");
                }
            }
            FenceState::Inside => {
                if line == "```" {
                    state = FenceState::After;
                } else {
                    block.push_str(line);
                    block.push('\n');
                }
            }
            FenceState::After => {
                if line.starts_with("```") {
                    set_refusal_kind(refusal_kind, "second_fence");
                    bail!(
                        "expected exactly one fenced toml block in the appendix, found another fence"
                    );
                }
            }
        }
    }

    match state {
        FenceState::Before => {
            set_refusal_kind(refusal_kind, "missing_toml_fence");
            bail!("appendix has no fenced toml block")
        }
        FenceState::Inside => {
            set_refusal_kind(refusal_kind, "unclosed_toml_fence");
            bail!("appendix toml fence is not closed")
        }
        FenceState::After => Ok(block),
    }
}

enum FenceState {
    Before,
    Inside,
    After,
}
