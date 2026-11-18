//! Fixed-form native subject provenance before any node submission.
//! The identity is per triple and comes only from a validated current freeze.

use crate::keyed_fixed_manifest::ValidatedFreeze;
use elements::hashes::sha256d;
use transaction::keyed_fixed_sponsor::verify_keyed_fixed_sponsor;
use transaction::{InputWitness, TargetTransaction};

/// A capture-time identity supplied by the external process or packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedFreeze(pub [u8; 32]);

/// Refusal at the native subject's provenance boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeFreezeRefusal {
    /// The process supplied no fixed-form identity.
    Missing,
    /// The supplied identity does not name this validated triple and inventory.
    Unequal,
}

/// Bind a native subject to the exact current fixed-form freeze.
/// # Errors
/// Refuses a missing or unequal identity, including an identity from the old edition.
pub fn bind_native_freeze(
    expected: &ValidatedFreeze,
    observed: Option<ObservedFreeze>,
) -> Result<ObservedFreeze, NativeFreezeRefusal> {
    let _span =
        tracing::debug_span!(target: "vectors::keyed_fixed_native", "bind_native_freeze").entered();
    let observed = observed.ok_or(NativeFreezeRefusal::Missing)?;
    if observed.0 != expected.identity() {
        tracing::debug!(target: "vectors::keyed_fixed_native", outcome = "refused", "native freeze identity differs");
        return Err(NativeFreezeRefusal::Unequal);
    }
    tracing::debug!(target: "vectors::keyed_fixed_native", outcome = "completed", "native freeze identity bound");
    Ok(observed)
}

/// Node-free structural route for exact bytes outside S6's admitted request.
/// The caller supplies parents, output proofs and ordinary authorization.
#[derive(Clone, Debug)]
pub struct StructuralSubject {
    transaction: TargetTransaction,
    parents: Vec<TargetTransaction>,
    sponsor_index: usize,
}

/// Refusal before any structural byte receives native credit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructuralRefusal {
    Freeze(NativeFreezeRefusal),
    Transaction,
    ParentCount,
    Parent,
    SponsorIndex,
    SponsorAuthorization,
}

/// Bind exact transaction and parent bytes without invoking S6 admission.
/// # Errors
/// Refuses an unbound freeze, noncanonical transaction, detached parent or absent sponsor.
pub fn bind_structural_subject(
    freeze: &ValidatedFreeze,
    observed: Option<ObservedFreeze>,
    transaction: TargetTransaction,
    submitted: &[u8],
    parents: &[(TargetTransaction, Vec<u8>)],
    sponsor_index: usize,
) -> Result<StructuralSubject, StructuralRefusal> {
    let _span = tracing::debug_span!(target: "vectors::keyed_fixed_native", "bind_structural_subject", parents = parents.len()).entered();
    bind_native_freeze(freeze, observed).map_err(StructuralRefusal::Freeze)?;
    if transaction.encode() != submitted {
        return Err(StructuralRefusal::Transaction);
    }
    if parents.len() != transaction.inputs().len() {
        return Err(StructuralRefusal::ParentCount);
    }
    let parents: Vec<_> = parents
        .iter()
        .map(|(parent, bytes)| {
            if parent.encode() != *bytes {
                return Err(StructuralRefusal::Parent);
            }
            Ok(parent.clone())
        })
        .collect::<Result<_, _>>()?;
    for (input, parent) in transaction.inputs().iter().zip(&parents) {
        let outpoint = input.outpoint();
        let digest = sha256d::Hash::hash(&parent.encode_without_witness()).to_byte_array();
        if outpoint.txid().internal() != &digest
            || parent.outputs().get(outpoint.index() as usize).is_none()
        {
            return Err(StructuralRefusal::Parent);
        }
    }
    if sponsor_index >= transaction.inputs().len() {
        return Err(StructuralRefusal::SponsorIndex);
    }
    Ok(StructuralSubject {
        transaction,
        parents,
        sponsor_index,
    })
}

impl StructuralSubject {
    /// Exact bytes the native adapter must submit.
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        self.transaction.encode()
    }

    /// Check ordinary sponsor authority against its retained parent output.
    /// # Errors
    /// Refuses a malformed or invalid ordinary sponsor witness.
    pub fn verify_sponsor(&self) -> Result<(), StructuralRefusal> {
        let sponsor = &self.transaction.inputs()[self.sponsor_index];
        let spent = self.parents[self.sponsor_index]
            .outputs()
            .get(sponsor.outpoint().index() as usize)
            .ok_or(StructuralRefusal::Parent)?;
        let mut witnesses = self.transaction.witnesses().to_vec();
        let stack = witnesses[self.sponsor_index].stack().to_vec();
        witnesses[self.sponsor_index] = InputWitness::new(vec![]);
        let unsigned = TargetTransaction::with_output_witnesses(
            self.transaction.version(),
            self.transaction.inputs().to_vec(),
            self.transaction.outputs().to_vec(),
            self.transaction.lock_time(),
            witnesses,
            self.transaction.output_witnesses().to_vec(),
        )
        .map_err(|_| StructuralRefusal::Transaction)?;
        verify_keyed_fixed_sponsor(
            &unsigned.encode_without_witness(),
            self.sponsor_index,
            spent,
            &stack,
        )
        .map_err(|_| StructuralRefusal::SponsorAuthorization)
    }
}
