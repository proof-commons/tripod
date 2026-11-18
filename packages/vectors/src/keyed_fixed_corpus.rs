//! Capture-time provenance admission for the future fixed-form corpus.
//! Retained rows keep the independently reconstructed historical identity.

use crate::keyed_fixed_manifest::ValidatedCaptureHistory;

/// Refusal at retained corpus acquisition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorpusFreezeRefusal {
    /// The retained row has no capture-time identity.
    Missing,
    /// The retained identity differs from its verified capture-time history.
    Unequal,
}

/// Bind retained bytes to the independently verified capture-time freeze.
/// # Errors
/// Refuses a missing or unequal identity, including a current or old-edition substitute.
pub fn bind_corpus_freeze(
    capture_time: &ValidatedCaptureHistory,
    retained: Option<[u8; 32]>,
) -> Result<[u8; 32], CorpusFreezeRefusal> {
    let _span =
        tracing::debug_span!(target: "vectors::keyed_fixed_corpus", "bind_corpus_freeze").entered();
    let retained = retained.ok_or(CorpusFreezeRefusal::Missing)?;
    if retained != capture_time.identity() {
        tracing::debug!(target: "vectors::keyed_fixed_corpus", outcome = "refused", "retained freeze identity differs");
        return Err(CorpusFreezeRefusal::Unequal);
    }
    tracing::debug!(target: "vectors::keyed_fixed_corpus", outcome = "completed", "retained freeze identity bound");
    Ok(retained)
}
