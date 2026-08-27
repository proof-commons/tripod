//! The versioning gates `(´[RZ-pin:pins:denotation]´)` /
//! `(´[RZ-rule:versioning:decision]´)`.
//!
//! Two gates enforce the versioning law mechanically.
//!
//! The **binding gate**: `realization_version` binds the document to
//! the compiler line — `major.minor` copied from the workspace
//! version, patch zeroed, the prerelease marker carried verbatim. The
//! expected value is derived from `CARGO_PKG_VERSION` at compile time,
//! so a compiler minor bump fails the build until the recorded binding
//! is bumped with it: every version movement is a deliberate, recorded
//! event even when no content changed. The version signals nothing
//! about content.
//!
//! The **denotation gate**: the behavioural hash covers the
//! behavioural arrays only — the finite presentation of the abstract
//! system's denotation, with calibrated draft bound defaults projected
//! out. Content stability has exactly one witness: this hash. Moving
//! it requires editing the pinned value below *and* the realization
//! document's masthead (the document weld enforces the latter), plus a
//! denotation record naming the deciding test — a moved hash
//! discovered rather than declared fails the build here.
//!
//! ## Algorithm migration record
//!
//! The gate is pinned per behavioural-hash *algorithm*. Replacing the
//! algorithm is a measurement change, not a denotation change, and is
//! recorded here explicitly rather than by silently redefining an
//! already-published identifier:
//!
//! - `sha256-canonical-json-behavioural-v1` (retired): hashed the
//!   full `BoundExport` rows, so a calibrated bound's draft
//!   `default_value` moved the hash, contradicting
//!   `(´[RZ-def:versioning:denotation-law]´)`. Pinned release value on
//!   the schema-17 tree:
//!   `fef2149d90c05a2790b41f73bba72ab4a088bf9912db03b0b497aa2ed34fff18`.
//! - `sha256-canonical-json-behavioural-v2` (retired): projected
//!   calibrated draft defaults out of the bound rows, but still hashed
//!   full witness and clause rows, so a document-label rename (a
//!   witness `semantic_tag` or the clause registry's citation label)
//!   moved the hash — contradicting the same law, which lists labels
//!   as presentation. Pinned release value on the schema-17 tree:
//!   `2044e02acb727323678732455915cc04679a819eb2564441a85c460edecfac63`.
//! - `sha256-canonical-json-behavioural-v3` (current): additionally
//!   projects document citation labels out of the witness and clause
//!   rows; witness semantic identity (code, id) and clause identity
//!   (code) remain hash inputs. The pinned value below was minted over
//!   the same denotation the v1 and v2 values pinned — all three
//!   identify the same unchanged denotation under their respective
//!   algorithms.
//!
use crate::*;

const PINNED_BEHAVIOURAL_HASH: &str =
    "756ea65ce3dc370e76ec70dc001231693facd58e53ebca315d549967f03cf206";

/// Retired identifiers must stay retired: reusing one for new
/// artifacts would silently redefine a published identifier.
#[test]
fn retired_behavioural_algorithms_stay_retired() {
    assert!(RETIRED_BEHAVIOURAL_HASH_ALGORITHMS.contains(&"sha256-canonical-json-behavioural-v1"));
    assert!(RETIRED_BEHAVIOURAL_HASH_ALGORITHMS.contains(&"sha256-canonical-json-behavioural-v2"));
    assert!(!RETIRED_BEHAVIOURAL_HASH_ALGORITHMS.contains(&BEHAVIOURAL_HASH_ALGORITHM));
    assert_eq!(
        BEHAVIOURAL_HASH_ALGORITHM,
        "sha256-canonical-json-behavioural-v3",
    );
}

/// The binding gate: the recorded realization version equals the value
/// the workspace version implies. A compiler minor bump without the
/// recorded binding moving is caught here; so is a hand-authored patch
/// or a drifted prerelease marker.
#[test]
fn realization_version_tracks_the_compiler_line() {
    let expected = spec::tracked_realization_version(env!("CARGO_PKG_VERSION"));
    assert_eq!(
        ARCHITECTURE.document.realization_version, expected,
        "realization_version is a binding to the compiler line: \
         re-record it as {expected:?} (derived from the workspace \
         version) in the same change that moves the workspace minor",
    );
    assert!(spec::realization_version_well_formed(
        ARCHITECTURE.document.realization_version
    ));
}

/// The derivation itself: patch-blind, prerelease carried verbatim.
#[test]
fn tracked_version_derivation_is_patch_blind() {
    assert_eq!(spec::tracked_realization_version("0.6.0-dev"), "0.6.0-dev");
    assert_eq!(spec::tracked_realization_version("0.6.0-dev"), "0.6.0-dev");
    assert_eq!(spec::tracked_realization_version("1.2.3"), "1.2.0");
    assert!(spec::realization_version_well_formed("0.6.0-dev"));
    assert!(spec::realization_version_well_formed("1.2.0"));
    assert!(!spec::realization_version_well_formed("42"));
    assert!(!spec::realization_version_well_formed("0.6.1-dev"));
    assert!(!spec::realization_version_well_formed("0.6.0-"));
    assert!(!spec::realization_version_well_formed("0.6.0-DEV"));
}

/// The denotation gate: if the behavioural hash moves away from the
/// pinned value, the denotation moved, and the build fails until the
/// move is declared — here, in the masthead the weld checks, and in a
/// denotation record.
#[test]
fn behavioural_hash_gate() {
    assert_eq!(
        behavioural_hash_hex(&super::validated(&ARCHITECTURE)).unwrap(),
        PINNED_BEHAVIOURAL_HASH,
        "the behavioural hash moved: the denotation changed, and the \
         change must be declared — re-pin here, update the realization \
         document's masthead, and record the deciding test",
    );
}

#[test]
fn behavioural_hash_is_stable_within_one_build() {
    assert_eq!(
        behavioural_hash(&super::validated(&ARCHITECTURE)).unwrap(),
        behavioural_hash(&super::validated(&ARCHITECTURE)).unwrap(),
    );
}
