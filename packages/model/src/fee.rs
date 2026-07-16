//! Fee envelope.
//!
//! Implements `(´def:auction:fee-envelope´)` and
//! `(´rule:auction:fee-envelope-validation´)`.

use std::collections::BTreeSet;

use crate::asset::Asset;
use crate::guard::Guard;
use crate::object::Meta;
use crate::scalar::{OutPoint, OwnerKey, Sat};
use crate::signer::{SignerSet, require_signer};
use crate::world::World;

// ´def:auction:fee-envelope´

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FeeEnvelope {
    pub inputs: Vec<OutPoint>,
    pub signers: SignerSet,

    pub chain_fee: Sat,

    pub change: Option<FeeChange>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeeChange {
    pub owner: OwnerKey,
    pub value: Sat,
}

// ´rule:auction:fee-envelope-validation´

pub fn validate_fee_envelope(world: &World, envelope: &FeeEnvelope) -> Result<Sat, Guard> {
    if envelope.inputs.len() > world.constants.fee_sponsor_input_max {
        return Err(Guard::Domain);
    }

    let mut total = Sat::ZERO;

    let mut seen = BTreeSet::new();

    for input in &envelope.inputs {
        if !seen.insert(*input) {
            return Err(Guard::DuplicateInput);
        }

        let utxo = world.utxo(*input)?;

        let owner = match (utxo.asset, utxo.meta) {
            (Asset::Lbtc, Meta::PlainLbtc { owner }) => owner,

            _ => return Err(Guard::SponsorMismatch),
        };

        require_signer(&envelope.signers, owner)?;

        total = total.checked_add(utxo.value)?;
    }

    let change = envelope
        .change
        .map(|change| change.value)
        .unwrap_or(Sat::ZERO);

    let required = envelope.chain_fee.checked_add(change)?;

    if total != required {
        return Err(Guard::FeeMismatch);
    }

    Ok(total)
}
