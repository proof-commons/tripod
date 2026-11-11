//! Sealed structural SOURCE/MERGE requests. No signing or policy admission.
use crate::{
    Outpoint,
    keyed_inputs::{BoundKeyedInput, KeyedInputOffer, KeyedInputRefusal},
    keyed_taproot::KeyedCommitment,
};
use architecture::semantic::{KeyedOperationId, SemanticOperationId};
use linker::{
    keyed_bundle::{KeyedLinkRefusal, link_keyed_bundle},
    keyed_reference::keyed_parameter_offers,
};
use std::collections::BTreeSet;
use tapscript::{
    keyed_constructor::KeyedLeafRecipe,
    keyed_members::{
        InputOutpoint, KeyedInputContext, KeyedInputRole as MemberRole, KeyedMemberBinding,
        KeyedMemberReceipt, KeyedMemberRefusal, OfflinePredecessorAssertion, bind_keyed_members,
    },
};
use target_elements::ReviewedElementsTapscriptDefinition;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedRequestRefusal {
    Link(KeyedLinkRefusal),
    Scope,
    StaleIndex {
        offered: u32,
        expected: u32,
    },
    DuplicateInput(Outpoint),
    Input {
        index: u32,
        refusal: KeyedInputRefusal,
    },
    SourceConsumesMember,
    EmptyMerge,
    Members(KeyedMemberRefusal),
}
impl KeyedRequestRefusal {
    #[must_use]
    pub fn refusal_kind(&self) -> &'static str {
        crate::trace_support::TraceRefusal::tracing_kind(self)
    }
}
impl crate::trace_support::TraceRefusal for KeyedRequestRefusal {
    fn tracing_kind(&self) -> &'static str {
        match self {
            Self::Link(_) => "link",
            Self::Scope => "scope",
            Self::StaleIndex { .. } => "stale_index",
            Self::DuplicateInput(_) => "duplicate_input",
            Self::Input { .. } => "input",
            Self::SourceConsumesMember => "source_consumes_member",
            Self::EmptyMerge => "empty_merge",
            Self::Members(_) => "members",
        }
    }
}
#[derive(Debug)]
struct Request<'a> {
    commitment: &'a KeyedCommitment,
    inputs: Vec<BoundKeyedInput<'a>>,
    members: KeyedMemberReceipt,
}
/// SOURCE's checked input census, never an authorized or materialized transaction.
/// ```compile_fail,E0451
/// use transaction::keyed_request::KeyedSourceRequest;
/// fn forge() -> KeyedSourceRequest<'static> { KeyedSourceRequest { inner: panic!() } }
/// ```
#[derive(Debug)]
pub struct KeyedSourceRequest<'a> {
    inner: Request<'a>,
}
/// Nonempty MERGE's checked input census and per-member relation coverage.
/// ```compile_fail,E0451
/// use transaction::keyed_request::KeyedMergeRequest;
/// fn forge() -> KeyedMergeRequest<'static> { KeyedMergeRequest { inner: panic!() } }
/// ```
#[derive(Debug)]
pub struct KeyedMergeRequest<'a> {
    inner: Request<'a>,
}
impl<'a> KeyedSourceRequest<'a> {
    #[must_use]
    pub const fn commitment(&self) -> &KeyedCommitment {
        self.inner.commitment
    }
    #[must_use]
    pub fn inputs(&self) -> &[BoundKeyedInput<'a>] {
        &self.inner.inputs
    }
    #[must_use]
    pub const fn members(&self) -> &KeyedMemberReceipt {
        &self.inner.members
    }
    #[must_use]
    pub const fn obligations(&self) -> &[crate::keyed_abi::KeyedRequestObligation] {
        &crate::keyed_abi::KeyedRequestObligation::ALL
    }
}
/// Check the SOURCE request; authorization and materialization remain deferred.
/// # Errors
/// Returns the exact link, parent, role, position or member-binding refusal.
pub fn keyed_source_request<'a>(
    target: &ReviewedElementsTapscriptDefinition,
    key: [u8; 32],
    commitment: &'a KeyedCommitment,
    inputs: &[KeyedInputOffer<'_, 'a>],
) -> Result<KeyedSourceRequest<'a>, KeyedRequestRefusal> {
    let input_count = u64::try_from(inputs.len()).unwrap_or(u64::MAX);
    let span = tracing::debug_span!(target: "transaction::keyed_request", "keyed_source_request", family = "keyed", input_count);
    let _entered = span.enter();
    let result = check(target, key, commitment, inputs, KeyedOperationId::Source)
        .map(|inner| KeyedSourceRequest { inner });
    match &result {
        Ok(_) => {
            tracing::debug!(target: "transaction::keyed_request", outcome = "completed", family = "keyed", "keyed source request completed");
        }
        Err(error) => {
            tracing::debug!(target: "transaction::keyed_request", outcome = "refused", family = "keyed", refusal_kind = error.refusal_kind(), "keyed source request refused");
        }
    }
    result
}
impl<'a> KeyedMergeRequest<'a> {
    #[must_use]
    pub const fn commitment(&self) -> &KeyedCommitment {
        self.inner.commitment
    }
    #[must_use]
    pub fn inputs(&self) -> &[BoundKeyedInput<'a>] {
        &self.inner.inputs
    }
    #[must_use]
    pub const fn members(&self) -> &KeyedMemberReceipt {
        &self.inner.members
    }
    #[must_use]
    pub const fn obligations(&self) -> &[crate::keyed_abi::KeyedRequestObligation] {
        &crate::keyed_abi::KeyedRequestObligation::ALL
    }
}
/// Check the MERGE request; authorization and materialization remain deferred.
/// # Errors
/// Returns the exact link, parent, role, position or member-binding refusal.
pub fn keyed_merge_request<'a>(
    target: &ReviewedElementsTapscriptDefinition,
    key: [u8; 32],
    commitment: &'a KeyedCommitment,
    inputs: &[KeyedInputOffer<'_, 'a>],
) -> Result<KeyedMergeRequest<'a>, KeyedRequestRefusal> {
    let input_count = u64::try_from(inputs.len()).unwrap_or(u64::MAX);
    let span = tracing::debug_span!(target: "transaction::keyed_request", "keyed_merge_request", family = "keyed", input_count);
    let _entered = span.enter();
    let result = check(target, key, commitment, inputs, KeyedOperationId::Merge)
        .map(|inner| KeyedMergeRequest { inner });
    match &result {
        Ok(_) => {
            tracing::debug!(target: "transaction::keyed_request", outcome = "completed", family = "keyed", "keyed merge request completed");
        }
        Err(error) => {
            tracing::debug!(target: "transaction::keyed_request", outcome = "refused", family = "keyed", refusal_kind = error.refusal_kind(), "keyed merge request refused");
        }
    }
    result
}

fn check<'a>(
    target: &ReviewedElementsTapscriptDefinition,
    key: [u8; 32],
    commitment: &'a KeyedCommitment,
    offers: &[KeyedInputOffer<'_, 'a>],
    operation: KeyedOperationId,
) -> Result<Request<'a>, KeyedRequestRefusal> {
    use KeyedRequestRefusal as E;
    // The caller's own key creates the equality offer, never emission's recipe.
    let recipe = KeyedLeafRecipe::new(key);
    let linked = link_keyed_bundle(
        target,
        commitment.linked().emission(),
        &keyed_parameter_offers(target, recipe),
    )
    .map_err(E::Link)?;
    let scope = &linked.project().selection.scope;
    if !scope.contains(&SemanticOperationId::Keyed(operation))
        || (operation == KeyedOperationId::Source && !linked.project().fragments.is_empty())
    {
        return Err(E::Scope);
    }
    let mut seen = BTreeSet::new();
    let mut inputs = Vec::new();
    for (expected, offer) in offers.iter().enumerate() {
        if usize::try_from(offer.input_index).ok() != Some(expected) {
            return Err(E::StaleIndex {
                offered: offer.input_index,
                expected: u32::try_from(expected).unwrap_or(u32::MAX),
            });
        }
        if !seen.insert(offer.outpoint) {
            return Err(E::DuplicateInput(offer.outpoint));
        }
        inputs.push(
            crate::keyed_inputs::bind_input(commitment, offer).map_err(|refusal| E::Input {
                index: offer.input_index,
                refusal,
            })?,
        );
    }
    let count = inputs.iter().filter(|i| i.predecessor().is_some()).count();
    if operation == KeyedOperationId::Source && count != 0 {
        return Err(E::SourceConsumesMember);
    }
    if operation == KeyedOperationId::Merge && count == 0 {
        return Err(E::EmptyMerge);
    }
    let mut contexts = Vec::new();
    let mut bindings = Vec::new();
    for input in &inputs {
        let p = input.parent();
        let outpoint = InputOutpoint {
            txid: *p.outpoint().txid().internal(),
            output_index: p.outpoint().index(),
        };
        contexts.push(KeyedInputContext {
            input_index: input.input_index(),
            outpoint,
            role: if input.predecessor().is_some() {
                MemberRole::ConsumedHandle
            } else {
                MemberRole::Sponsor
            },
            predecessor_program: p.output().program().to_vec(),
        });
        if input.predecessor().is_some() {
            // Public binder input remains explicitly an assertion. Its provenance is
            // the separate sealed parent+authentication receipt retained above.
            bindings.push(KeyedMemberBinding {
                input_index: input.input_index(),
                outpoint,
                recipe,
                offline_assertion: OfflinePredecessorAssertion {
                    outpoint,
                    predecessor_program: p.output().program().to_vec(),
                    recipe,
                },
                relations: linked.project().fragments.keys().copied().collect(),
            });
        }
    }
    let members =
        bind_keyed_members(linked.emission(), &contexts, &bindings).map_err(E::Members)?;
    Ok(Request {
        commitment,
        inputs,
        members,
    })
}
#[cfg(test)]
#[path = "keyed_request/tests.rs"]
mod tests;
