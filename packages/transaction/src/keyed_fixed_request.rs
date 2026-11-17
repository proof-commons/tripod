//! Sealed structural SOURCE/MERGE requests. No signing or policy admission.
use crate::{
    NonceField, Outpoint,
    keyed_fixed_inputs::{BoundKeyedFixedInput, KeyedFixedInputOffer, KeyedFixedInputRefusal},
    keyed_fixed_taproot::KeyedFixedCommitment,
};
use architecture::keyed_fixed::KeyedFixedOperationId;
use linker::{
    keyed_fixed_bundle::{KeyedFixedLinkRefusal, link_keyed_fixed_bundle},
    keyed_fixed_reference::keyed_fixed_parameter_offers,
};
use realization::keyed_fixed::KeyedFixedParameters;
use std::collections::BTreeSet;
use tapscript::{
    keyed_fixed_constructor::KeyedFixedLeafRecipe,
    keyed_fixed_members::{
        InputOutpoint, KeyedFixedInputContext, KeyedFixedInputRole as MemberRole,
        KeyedFixedMemberBinding, KeyedFixedMemberReceipt, KeyedFixedMemberRefusal,
        OfflinePredecessorAssertion, bind_keyed_fixed_members,
    },
};
use target_elements::ReviewedElementsTapscriptDefinition;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedFixedRequestRefusal {
    AmountRange,
    Link(KeyedFixedLinkRefusal),
    Scope,
    StaleIndex {
        offered: u32,
        expected: u32,
    },
    DuplicateInput(Outpoint),
    Input {
        index: u32,
        refusal: KeyedFixedInputRefusal,
    },
    SourceConsumesMember,
    EmptyMerge,
    /// This constructor spends only empty-nonce member parents.
    MemberNonce {
        index: u32,
    },
    Members(KeyedFixedMemberRefusal),
}
impl KeyedFixedRequestRefusal {
    #[must_use]
    pub fn refusal_kind(&self) -> &'static str {
        crate::trace_support::TraceRefusal::tracing_kind(self)
    }
}
impl crate::trace_support::TraceRefusal for KeyedFixedRequestRefusal {
    fn tracing_kind(&self) -> &'static str {
        match self {
            Self::AmountRange => "amount_range",
            Self::Link(_) => "link",
            Self::Scope => "scope",
            Self::StaleIndex { .. } => "stale_index",
            Self::DuplicateInput(_) => "duplicate_input",
            Self::Input { .. } => "input",
            Self::SourceConsumesMember => "source_consumes_member",
            Self::EmptyMerge => "empty_merge",
            Self::MemberNonce { .. } => "member_nonce",
            Self::Members(_) => "members",
        }
    }
}
#[derive(Debug)]
struct Request<'a> {
    commitment: &'a KeyedFixedCommitment,
    inputs: Vec<BoundKeyedFixedInput<'a>>,
    members: KeyedFixedMemberReceipt,
}
/// SOURCE's checked input census, never an authorized or materialized transaction.
/// ```compile_fail,E0451
/// use transaction::keyed_fixed_request::KeyedFixedSourceRequest;
/// fn forge() -> KeyedFixedSourceRequest<'static> { KeyedFixedSourceRequest { inner: panic!() } }
/// ```
#[derive(Debug)]
pub struct KeyedFixedSourceRequest<'a> {
    inner: Request<'a>,
}
/// Nonempty MERGE's checked input census and per-member relation coverage.
/// ```compile_fail,E0451
/// use transaction::keyed_fixed_request::KeyedFixedMergeRequest;
/// fn forge() -> KeyedFixedMergeRequest<'static> { KeyedFixedMergeRequest { inner: panic!() } }
/// ```
#[derive(Debug)]
pub struct KeyedFixedMergeRequest<'a> {
    inner: Request<'a>,
}
impl<'a> KeyedFixedSourceRequest<'a> {
    #[must_use]
    pub const fn commitment(&self) -> &KeyedFixedCommitment {
        self.inner.commitment
    }
    #[must_use]
    pub fn inputs(&self) -> &[BoundKeyedFixedInput<'a>] {
        &self.inner.inputs
    }
    #[must_use]
    pub const fn members(&self) -> &KeyedFixedMemberReceipt {
        &self.inner.members
    }
    #[must_use]
    pub const fn obligations(&self) -> &[crate::keyed_fixed_abi::KeyedFixedRequestObligation] {
        &crate::keyed_fixed_abi::KeyedFixedRequestObligation::ALL
    }
}
/// Check the SOURCE request; authorization and materialization remain deferred.
/// # Errors
/// Returns the exact link, parent, role, position or member-binding refusal.
pub fn keyed_fixed_source_request<'a>(
    target: &ReviewedElementsTapscriptDefinition,
    parameters: KeyedFixedParameters,
    commitment: &'a KeyedFixedCommitment,
    inputs: &[KeyedFixedInputOffer<'_, 'a>],
) -> Result<KeyedFixedSourceRequest<'a>, KeyedFixedRequestRefusal> {
    let input_count = u64::try_from(inputs.len()).unwrap_or(u64::MAX);
    let span = tracing::debug_span!(target: "transaction::keyed_fixed_request", "keyed_fixed_source_request", family = "keyed", input_count);
    let _entered = span.enter();
    let result = check(
        target,
        parameters,
        commitment,
        inputs,
        KeyedFixedOperationId::Source,
    )
    .map(|inner| KeyedFixedSourceRequest { inner });
    match &result {
        Ok(_) => {
            tracing::debug!(target: "transaction::keyed_fixed_request", outcome = "completed", family = "keyed", "keyed source request completed");
        }
        Err(error) => {
            tracing::debug!(target: "transaction::keyed_fixed_request", outcome = "refused", family = "keyed", refusal_kind = error.refusal_kind(), "keyed source request refused");
        }
    }
    result
}
impl<'a> KeyedFixedMergeRequest<'a> {
    #[must_use]
    pub const fn commitment(&self) -> &KeyedFixedCommitment {
        self.inner.commitment
    }
    #[must_use]
    pub fn inputs(&self) -> &[BoundKeyedFixedInput<'a>] {
        &self.inner.inputs
    }
    #[must_use]
    pub const fn members(&self) -> &KeyedFixedMemberReceipt {
        &self.inner.members
    }
    #[must_use]
    pub const fn obligations(&self) -> &[crate::keyed_fixed_abi::KeyedFixedRequestObligation] {
        &crate::keyed_fixed_abi::KeyedFixedRequestObligation::ALL
    }
}
/// Check the MERGE request; authorization and materialization remain deferred.
/// # Errors
/// Returns the exact link, parent, role, position or member-binding refusal.
pub fn keyed_fixed_merge_request<'a>(
    target: &ReviewedElementsTapscriptDefinition,
    parameters: KeyedFixedParameters,
    commitment: &'a KeyedFixedCommitment,
    inputs: &[KeyedFixedInputOffer<'_, 'a>],
) -> Result<KeyedFixedMergeRequest<'a>, KeyedFixedRequestRefusal> {
    let input_count = u64::try_from(inputs.len()).unwrap_or(u64::MAX);
    let span = tracing::debug_span!(target: "transaction::keyed_fixed_request", "keyed_fixed_merge_request", family = "keyed", input_count);
    let _entered = span.enter();
    let result = check(
        target,
        parameters,
        commitment,
        inputs,
        KeyedFixedOperationId::Merge,
    )
    .map(|inner| KeyedFixedMergeRequest { inner });
    match &result {
        Ok(_) => {
            tracing::debug!(target: "transaction::keyed_fixed_request", outcome = "completed", family = "keyed", "keyed merge request completed");
        }
        Err(error) => {
            tracing::debug!(target: "transaction::keyed_fixed_request", outcome = "refused", family = "keyed", refusal_kind = error.refusal_kind(), "keyed merge request refused");
        }
    }
    result
}

fn check<'a>(
    target: &ReviewedElementsTapscriptDefinition,
    parameters: KeyedFixedParameters,
    commitment: &'a KeyedFixedCommitment,
    offers: &[KeyedFixedInputOffer<'_, 'a>],
    operation: KeyedFixedOperationId,
) -> Result<Request<'a>, KeyedFixedRequestRefusal> {
    use KeyedFixedRequestRefusal as E;
    if !target_elements::transaction_form::reviewed_stated_amount_bound()
        .admits(parameters.value.get())
    {
        return Err(E::AmountRange);
    }
    // The caller's own key creates the equality offer, never emission's recipe.
    let recipe = KeyedFixedLeafRecipe::new(parameters);
    let linked = link_keyed_fixed_bundle(
        target,
        commitment.linked().emission(),
        &keyed_fixed_parameter_offers(target, recipe),
    )
    .map_err(E::Link)?;
    let scope = &linked.project().selection.scope;
    if !scope.contains(&operation)
        || (operation == KeyedFixedOperationId::Source && !linked.project().fragments.is_empty())
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
        let bound =
            crate::keyed_fixed_inputs::bind_input(commitment, offer).map_err(|refusal| {
                E::Input {
                    index: offer.input_index,
                    refusal,
                }
            })?;
        if bound.predecessor().is_some() && bound.parent().output().nonce() != NonceField::Null {
            return Err(E::MemberNonce {
                index: offer.input_index,
            });
        }
        inputs.push(bound);
    }
    let count = inputs.iter().filter(|i| i.predecessor().is_some()).count();
    if operation == KeyedFixedOperationId::Source && count != 0 {
        return Err(E::SourceConsumesMember);
    }
    if operation == KeyedFixedOperationId::Merge && count == 0 {
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
        contexts.push(KeyedFixedInputContext {
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
            bindings.push(KeyedFixedMemberBinding {
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
        bind_keyed_fixed_members(linked.emission(), &contexts, &bindings).map_err(E::Members)?;
    Ok(Request {
        commitment,
        inputs,
        members,
    })
}
