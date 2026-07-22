//! Forbidden-text repository audit.
//!
//! Rejects a specific angle-bracket placeholder token that must never
//! appear in tracked sources (the reviewer-left generic placeholder the
//! hint steers away from `Vec<_>` toward). The token is assembled at
//! runtime from safe pieces so this module's own source never contains
//! it verbatim and the audit does not self-trip.

use serde::Serialize;

/// Stable policy identifier for the forbidden angle-bracket token.
///
/// The raw token is never serialized (ADR-010 keeps result data free of
/// values that a downstream pipeline would have to re-scrub); a policy
/// identifier is enough to locate and explain a match.
pub const FORBIDDEN_TOKEN_POLICY: &str = "rust-generic-angle-token";

/// Report schema version for the forbidden-text audit.
pub const FORBIDDEN_TEXT_SCHEMA: u32 = 1;

/// One tracked-file match of the forbidden token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForbiddenTextMatch {
    /// Repository-relative path of the matching file.
    pub path: String,
    /// One-based line number of the match.
    pub line: u32,
    /// Safe policy identifier (never the raw token).
    pub policy: String,
}

/// The forbidden-text audit result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForbiddenTextReport {
    /// Report schema version.
    pub schema: u32,
    /// Tracked-file matches (empty on a clean tree).
    pub tracked_matches: Vec<ForbiddenTextMatch>,
    /// True only when no tracked file contains the forbidden token.
    pub valid: bool,
}

/// The forbidden literal, assembled from safe pieces so it never appears
/// verbatim in tracked source.
#[must_use]
pub fn forbidden_needle() -> String {
    format!("<{}>", "char")
}

/// Parse `git grep -n -F` output — one `path:line:content` record per
/// line — into typed matches. The matched content is discarded; only the
/// location and the safe policy identifier are retained.
#[must_use]
pub fn parse_grep_matches(output: &str) -> Vec<ForbiddenTextMatch> {
    output.lines().filter_map(parse_grep_line).collect()
}

fn parse_grep_line(line: &str) -> Option<ForbiddenTextMatch> {
    // Format: "<path>:<line>:<content>". Repository paths carry no
    // colon, so a left-anchored three-way split is unambiguous.
    let mut parts = line.splitn(3, ':');
    let path = parts.next()?;
    let number = parts.next()?;
    // A third field (the matched content) must exist for a real match.
    parts.next()?;

    if path.is_empty() {
        return None;
    }

    Some(ForbiddenTextMatch {
        path: path.to_owned(),
        line: number.parse().ok()?,
        policy: FORBIDDEN_TOKEN_POLICY.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_git_grep_records_into_located_matches() {
        let output = "src/a.rs:12:some content\nsrc/b.rs:3:more\n";
        let matches = parse_grep_matches(output);
        assert_eq!(
            matches,
            vec![
                ForbiddenTextMatch {
                    path: "src/a.rs".to_owned(),
                    line: 12,
                    policy: FORBIDDEN_TOKEN_POLICY.to_owned(),
                },
                ForbiddenTextMatch {
                    path: "src/b.rs".to_owned(),
                    line: 3,
                    policy: FORBIDDEN_TOKEN_POLICY.to_owned(),
                },
            ],
        );
    }

    #[test]
    fn ignores_blank_and_malformed_lines() {
        let output = "\nnot-a-record\nsrc/c.rs:not-a-number:x\nsrc/d.rs:7:hit\n";
        let matches = parse_grep_matches(output);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "src/d.rs");
        assert_eq!(matches[0].line, 7);
    }

    #[test]
    fn the_needle_is_never_present_verbatim_here() {
        // The assembled needle must not be findable in this source file,
        // or the audit would flag its own implementation.
        let needle = forbidden_needle();
        let this_file = include_str!("forbidden.rs");
        assert!(!this_file.contains(&needle));
    }
}
