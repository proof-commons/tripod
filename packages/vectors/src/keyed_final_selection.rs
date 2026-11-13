//! Reviewed, key-free selection rule; no fixture funding or signing inputs.

use compiler::keyed_plan::KeyedPlanProjection;
use tapscript::keyed_bundle::KeyedBundleSelection;

/// Offer proof zero and placement zero for every selected operation.
///
/// Returns `None` for an empty proof census. The emitter checks this untrusted
/// offer against the sealed plan; this helper authenticates no selection.
#[must_use]
pub fn first_selection(plan: &KeyedPlanProjection) -> Option<KeyedBundleSelection> {
    Some(KeyedBundleSelection {
        binding: plan.binding.clone(),
        scope: plan.scope.clone(),
        proof: 0,
        placements: plan
            .plans
            .first()?
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    })
}
