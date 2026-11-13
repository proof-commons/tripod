//! Explicit sponsor readback binding for revision-nine funding responses.
//!
//! This conformance check reads public transaction bytes with the pinned
//! Elements decoder. It proves local content and output equality, not inclusion,
//! unspentness, retrieval or finality. The construction consumer separately uses
//! transaction's decoder when it binds its request; neither trusts reported totals.

use crate::protocol::{FundedOutput, MinedFundingReadback};

/// Maximum canonical sponsor parent bytes, below the bounded wire record budget.
pub const MAXIMUM_SPONSOR_PARENT_BYTES: usize = 512 * 1024;

/// A refusal of public readback binding, never a covenant or native verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SponsorParentRefusal {
    /// The raw parent exceeds the explicit byte budget.
    Size,
    /// The bytes do not decode as one Elements transaction.
    Decode,
    /// Defensive round-trip guard: the pinned decoder rejects known noncanonical
    /// encodings before this condition; no reachable public subject is claimed.
    NonCanonical,
    /// The recomputed stripped content address differs from the readback.
    ReadbackIdentity,
    /// The recomputed witness identity differs from the readback.
    WitnessIdentity,
    /// The funded outpoint names a different transaction.
    OutpointIdentity,
    /// The funded output index does not exist.
    OutputIndex,
    /// The selected asset is not explicit; it cannot be compared as an asset id.
    NonExplicitAsset,
    /// The selected value is not explicit; no amount is inferred from a commitment.
    NonExplicitValue,
    /// The actual explicit asset differs from the reported asset.
    Asset,
    /// The actual explicit value differs from the reported amount.
    Amount,
    /// The actual script differs from the reported script.
    Script,
}

impl SponsorParentRefusal {
    /// Stable diagnostic label for the exact refusing condition.
    #[must_use]
    pub const fn refusal_kind(self) -> &'static str {
        match self {
            Self::Size => "size",
            Self::Decode => "decode",
            Self::NonCanonical => "non_canonical",
            Self::ReadbackIdentity => "readback_identity",
            Self::WitnessIdentity => "witness_identity",
            Self::OutpointIdentity => "outpoint_identity",
            Self::OutputIndex => "output_index",
            Self::NonExplicitAsset => "non_explicit_asset",
            Self::NonExplicitValue => "non_explicit_value",
            Self::Asset => "asset",
            Self::Amount => "amount",
            Self::Script => "script",
        }
    }
}

/// Checks canonical content addressing and every reported funded-output field.
///
/// The caller receives the original bytes, never a repaired encoding. This is
/// not an inclusion assertion. The protocol shape check must precede this check.
///
/// # Errors
/// Returns the first [`SponsorParentRefusal`] in size, decoding, canonicality,
/// readback identities, outpoint and output-field order.
pub fn bind_sponsor_parent<'a>(
    readback: &'a MinedFundingReadback,
    funded: &FundedOutput,
) -> Result<&'a [u8], SponsorParentRefusal> {
    let span = tracing::info_span!(target: "target_elements_conformance::sponsor_parent", "bind_sponsor_parent");
    let _entered = span.enter();
    let result = bind_core(readback, funded);
    match &result {
        Ok(bytes) => {
            tracing::info!(target: "target_elements_conformance::sponsor_parent", parent_bytes = bytes.len(), "sponsor parent binding completed");
        }
        Err(error) => {
            tracing::debug!(target: "target_elements_conformance::sponsor_parent", refusal_kind = error.refusal_kind(), "sponsor parent binding refused");
        }
    }
    result
}

fn bind_core<'a>(
    readback: &'a MinedFundingReadback,
    funded: &FundedOutput,
) -> Result<&'a [u8], SponsorParentRefusal> {
    use SponsorParentRefusal as R;
    let raw = &readback.raw_transaction;
    if raw.len() > MAXIMUM_SPONSOR_PARENT_BYTES {
        return Err(R::Size);
    }
    let transaction: elements::Transaction =
        elements::encode::deserialize(raw).map_err(|_| R::Decode)?;
    if elements::encode::serialize(&transaction) != *raw {
        return Err(R::NonCanonical);
    }
    let txid = transaction.txid().to_string();
    if txid != readback.transaction_id {
        return Err(R::ReadbackIdentity);
    }
    if transaction.wtxid().to_string() != readback.witness_transaction_id {
        return Err(R::WitnessIdentity);
    }
    if txid != funded.outpoint.txid {
        return Err(R::OutpointIdentity);
    }
    let output = transaction
        .output
        .get(funded.outpoint.vout as usize)
        .ok_or(R::OutputIndex)?;
    let asset = output.asset.explicit().ok_or(R::NonExplicitAsset)?;
    if asset.to_string() != funded.asset {
        return Err(R::Asset);
    }
    let amount = output.value.explicit().ok_or(R::NonExplicitValue)?;
    if amount != funded.amount_satoshis {
        return Err(R::Amount);
    }
    if hex(output.script_pubkey.as_bytes()) != funded.script {
        return Err(R::Script);
    }
    Ok(raw)
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut encoded = String::new();
    for byte in bytes {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}
