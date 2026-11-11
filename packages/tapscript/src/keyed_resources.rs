//! Resource formulas for the fixed public keyed grammar, not policy admission.
//!
//! A leaf is `push32(key) DROP birth continuation TRUE`: 52 script bytes,
//! twelve primitive calls, four literals and no signature checks. The scripts
//! have no execution arguments. Script and depth-zero control are two witness
//! items removed before execution. No annex is included. Other transaction
//! base fields, output proofs and non-covenant inputs belong to the caller.
//! The target flag is included in stripped base bytes, not witness bytes.

use crate::{
    AbstractLimits, AbstractStackState, TapscriptError, TapscriptProgram, program_stack_profile,
    resource_projection,
};
use std::collections::BTreeMap;
use target_elements::{ResourceBound, ResourceDimension, ReviewedElementsTapscriptDefinition};

/// Which fixed grammar is described. Fragments omit final truth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedSchedule {
    /// Empty-argument outpoint-position fragment, without final truth.
    Birth,
    /// Empty-argument program-equality fragment, without final truth.
    Continuation,
    /// Key commitment, both fragments and final truth.
    Leaf,
}

/// A known quantity or an explicit outstanding obligation, never implicit zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedResourceTerm {
    /// Exact per-script usage in the dimension's own units.
    Exact(u64),
    /// Maximum width over the reviewed introspection domain, including failing paths.
    ElementWidthBound(u64),
    /// The full leaf has an 88-byte script-witness stack; a complete target
    /// input witness is 91 bytes with empty issuance proofs and peg-in stack.
    LeafWitnessFormula,
    /// A fragment must first be composed into a leaf before witness accounting.
    CompositionRequired,
    /// Supply complete base and non-covenant witness bytes to the weight formula.
    TransactionContextRequired,
    /// Package and deployment policy must be supplied and checked separately.
    DeploymentRequired,
}

/// Per-script analysis with target bounds kept separate from actual usage.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct KeyedResourceProjection {
    /// Every reviewed resource dimension, including outstanding obligations.
    pub dimensions: BTreeMap<ResourceDimension, KeyedResourceTerm>,
    /// Primitive calls, not operation-budget or signature-validation charges.
    pub primitive_count: u64,
    /// Consensus ceilings supplied by the reviewed target, not a verdict.
    pub consensus_bounds: BTreeMap<ResourceDimension, ResourceBound>,
    /// Reviewed policy defaults; concrete deployment and package checks remain.
    pub policy_bounds: BTreeMap<ResourceDimension, ResourceBound>,
}

/// Derive the fixed schedule's quantities through the unchanged projectors.
///
/// The 40-byte maximum covers the full reviewed introspection payload domain;
/// it is a conservative tight domain bound, not a claim that a P2TR payload is
/// 40 bytes (that payload is 32). Alternate stack usage is always zero.
/// # Errors
/// Returns the existing checked program/literal construction refusal.
pub fn keyed_resource_projection(
    target: &ReviewedElementsTapscriptDefinition,
    schedule: KeyedSchedule,
) -> Result<KeyedResourceProjection, TapscriptError> {
    let span =
        tracing::debug_span!(target: "tapscript::keyed_resources", "keyed_resource_projection");
    let _entered = span.enter();
    let result = project(target, schedule);
    match &result {
        Ok(_) => {
            tracing::debug!(target: "tapscript::keyed_resources", outcome = "completed", "keyed resources projected");
        }
        Err(error) => {
            tracing::debug!(target: "tapscript::keyed_resources", outcome = "refused", refusal_kind = error.refusal_kind(), "keyed resources projected");
        }
    }
    result
}

pub(crate) fn project(
    target: &ReviewedElementsTapscriptDefinition,
    schedule: KeyedSchedule,
) -> Result<KeyedResourceProjection, TapscriptError> {
    use KeyedResourceTerm as T;
    use ResourceDimension as D;
    let program: TapscriptProgram = match schedule {
        KeyedSchedule::Birth => crate::keyed_pattern::born_first_fragment_untraced(target)?,
        KeyedSchedule::Continuation => {
            crate::keyed_pattern::continue_first_fragment_untraced(target)?
        }
        KeyedSchedule::Leaf => crate::keyed_pattern::compose_keyed_leaf(target, &[0; 32])?,
    };
    let stack = program_stack_profile(
        target,
        &program,
        &AbstractStackState::from_main(vec![]),
        AbstractLimits::for_target(target),
    );
    let mut dimensions: BTreeMap<_, _> = resource_projection(target, &program)
        .into_iter()
        .map(|(d, n)| (d, T::Exact(n)))
        .collect();
    dimensions.extend([
        (D::InitialStackItems, T::Exact(0)),
        (D::InitialWitnessItemBytes, T::Exact(0)),
        (
            D::PeakStackItems,
            T::Exact(stack.peak_main() + stack.peak_alternate()),
        ),
        (
            D::StackElementBytes,
            T::ElementWidthBound(if schedule == KeyedSchedule::Birth {
                32
            } else {
                40
            }),
        ),
        (
            D::ControlPathDepth,
            if schedule == KeyedSchedule::Leaf {
                T::Exact(0)
            } else {
                T::CompositionRequired
            },
        ),
        (
            D::WitnessBytes,
            if schedule == KeyedSchedule::Leaf {
                T::LeafWitnessFormula
            } else {
                T::CompositionRequired
            },
        ),
        (D::TransactionWeight, T::TransactionContextRequired),
        (D::PackageLimit, T::DeploymentRequired),
    ]);
    Ok(KeyedResourceProjection {
        dimensions,
        primitive_count: program
            .instructions()
            .iter()
            .filter(|instruction| matches!(instruction, crate::TapscriptInstruction::Opcode(_)))
            .count() as u64,
        consensus_bounds: target.definition().resources().consensus().bounds().clone(),
        policy_bounds: target.definition().resources().policy().bounds().clone(),
    })
}

/// Arithmetic cannot represent the requested serialization size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyedResourceOverflow;

impl KeyedResourceOverflow {
    /// Stable diagnostic classification, independent of display wording.
    #[must_use]
    pub const fn refusal_kind(self) -> &'static str {
        "arithmetic_overflow"
    }
}
impl std::fmt::Display for KeyedResourceOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("keyed resource arithmetic overflow")
    }
}
impl std::error::Error for KeyedResourceOverflow {}

/// `CompactSize`'s encoded width, including its discriminant.
#[must_use]
const fn compact_size_width(value: u64) -> u64 {
    match value {
        0..=252 => 1,
        253..=65_535 => 3,
        65_536..=4_294_967_295 => 5,
        _ => 9,
    }
}

/// One CompactSize-framed byte string. Useful at framing transition boundaries.
/// # Errors
/// Returns overflow rather than saturating or wrapping the frame length.
fn framed_bytes(length: u64) -> Result<u64, KeyedResourceOverflow> {
    length
        .checked_add(compact_size_width(length))
        .ok_or(KeyedResourceOverflow)
}

/// Exact covenant script witnesses for any member count, with no finite cap.
///
/// Each member contributes CS(2) + CS(52) + 52 + CS(33) + 33 = 88 bytes.
/// The control block is one parity/version byte plus a 32-byte internal key;
/// the depth-zero path contributes no hashes. There is no signature or annex.
/// This excludes the three empty-field prefixes which the transaction formula
/// adds per member (two issuance proofs and the peg-in stack). A zero member count is arithmetic, not a valid MERGE.
/// # Errors
/// Returns overflow when the sum is not representable.
pub fn keyed_witness_bytes(member_count: u64) -> Result<u64, KeyedResourceOverflow> {
    let per_member = compact_size_width(2) + framed_bytes(52)? + framed_bytes(33)?;
    member_count
        .checked_mul(per_member)
        .ok_or(KeyedResourceOverflow)
}

/// Weight = target scale * complete stripped bytes + other witness bytes
/// plus 91 bytes per keyed member (88 script-stack bytes and three empty-field
/// prefixes: two issuance proofs and the peg-in stack).
///
/// Include every serialization field exactly once: input/output counts and
/// the target's flag byte are in `base_bytes`; output proofs and non-covenant
/// input witnesses are in `other_witness_bytes`. Keyed members here have empty
/// issuance and peg-in fields. No policy admission follows from this formula.
/// # Errors
/// Returns overflow on any product or sum.
pub fn keyed_transaction_weight(
    target: &ReviewedElementsTapscriptDefinition,
    member_count: u64,
    base_bytes: u64,
    other_witness_bytes: u64,
) -> Result<u64, KeyedResourceOverflow> {
    let span = tracing::debug_span!(target: "tapscript::keyed_resources", "keyed_transaction_weight", member_count);
    let _entered = span.enter();
    let result = keyed_witness_bytes(member_count).and_then(|script_stack| {
        let witness = member_count
            .checked_mul(3)
            .and_then(|empty_fields| script_stack.checked_add(empty_fields))
            .ok_or(KeyedResourceOverflow)?;
        base_bytes
            .checked_mul(
                target
                    .definition()
                    .resources()
                    .consensus()
                    .witness_scale_factor(),
            )
            .and_then(|base| base.checked_add(other_witness_bytes))
            .and_then(|other| other.checked_add(witness))
            .ok_or(KeyedResourceOverflow)
    });
    match &result {
        Ok(_) => {
            tracing::debug!(target: "tapscript::keyed_resources", outcome = "completed", "keyed weight evaluated");
        }
        Err(error) => {
            tracing::debug!(target: "tapscript::keyed_resources", outcome = "refused", refusal_kind = error.refusal_kind(), "keyed weight evaluated");
        }
    }
    result
}

#[cfg(test)]
mod tests;
