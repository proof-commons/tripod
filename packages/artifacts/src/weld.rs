//! Document weld: structural extraction of the realization document's
//! attached manifest and masthead identities.
//!
//! `realization.md` claims its `app:realization:architecture` appendix is the
//! generated `architecture.toml` attached **verbatim**, and its
//! masthead states the release identities (schema, hashes, specification
//! pin). These claims are release-integrity surfaces: the weld tests
//! in this crate compare them mechanically against the typed
//! architecture, so changing any attached manifest byte or masthead
//! identity fails deterministically.
//!
//! Extraction is structural — explicit unique markers, not broad
//! substring searches.

use anyhow::{bail, ensure};

/// The document masthead: every line before the first thematic break
/// (`---`). This is where the document states its release identities.
///
/// # Errors
///
/// Returns an error when the document has no thematic break.
pub fn masthead(document: &str) -> anyhow::Result<&str> {
    match document.find("\n---\n") {
        Some(end) => Ok(&document[..end]),
        None => bail!("realization document has no thematic break after the masthead"),
    }
}

/// Extract the verbatim `architecture.toml` bytes attached under the
/// unique `app:realization:architecture` appendix heading.
///
/// Structural rules enforced:
///
/// 1. exactly one heading line starting with `## Appendix` and citing
///    `app:realization:architecture`;
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
    let heading_indices = document
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            line.starts_with("## Appendix") && line.contains("`app:realization:architecture`")
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [heading_index] = heading_indices.as_slice() else {
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
                } else {
                    ensure!(
                        !line.starts_with("```"),
                        "unexpected non-toml fence in the appendix before the manifest block",
                    );
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
                ensure!(
                    !line.starts_with("```"),
                    "expected exactly one fenced toml block in the appendix, found another fence",
                );
            }
        }
    }

    match state {
        FenceState::Before => bail!("appendix has no fenced toml block"),
        FenceState::Inside => bail!("appendix toml fence is not closed"),
        FenceState::After => Ok(block),
    }
}

enum FenceState {
    Before,
    Inside,
    After,
}
