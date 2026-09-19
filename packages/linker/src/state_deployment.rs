//! The typed sources one maturity announcement link is given.
//!
//! # Supplied, never minted
//!
//! Every source arrives from the caller already validated by the type
//! that owns the question it answers: the compiler's announcement plan,
//! the realization's lead window, the reviewed target's revision, the
//! candidate deployment identity and the established operator binding.
//! This crate mints none of them, because §1.10 refuses a speculative
//! identity and a link that invented a deployment value would be
//! inventing the thing a deployment exists to fix.
//!
//! # A validated type, never a value map
//!
//! The sources are not collected into a map of names to values. A map
//! erases where each value came from: two byte strings of the right
//! width are indistinguishable inside it, a fixture magnitude reads
//! exactly like a calibrated one, and the check that admitted a value
//! happened somewhere the map cannot name — so the map itself becomes
//! the authority, which is the one thing none of these values may rest
//! on. Carrying each source as the type that validated it keeps that
//! evidence attached to the value all the way to the link, and it is
//! why the numeric lead bounds travel with their origin rather than
//! alone.
//!
//! # No secret enters
//!
//! Nothing here holds, derives or accepts a private key. The operator
//! binding carries a public key whose curve validity it explicitly does
//! not claim, and an internal key is not a parameter of this generation
//! at all: the constructor's own key policy fixes it, so there is no
//! deployment-supplied key for this type to carry or to check.

use std::collections::BTreeSet;
use std::num::NonZeroU32;

use tapscript::upstream::{AnnouncementLeadBounds, ValidatedMaturityAnnouncementOperationPlan};
use tapscript::{StateAnnouncementProgram, StateExternalEvidenceRole};
use target_elements::{ReviewedElementsTapscriptDefinition, TargetContractVersion};

use crate::error::LinkRefusal;
use crate::operator_deployment::{CandidateDeploymentIdentity, OperatorDeploymentBinding};

/// The external evidence roles this link records as deployment facts.
///
/// A fixed census rather than a judgement made at each call site,
/// because which of the record's roles are facts about a deployment has
/// one answer for the family, and a call site free to answer it
/// differently would let the same record produce two different
/// bindings.
///
/// [`StateExternalEvidenceRole::CurrentStateRootFreshness`] is
/// deliberately not one of them. It is a report-layer fact about chain
/// context at the moment of spending, established by whoever assembles
/// the transaction against the chain they see. The three below are
/// properties of a deployment that already happened — what the
/// substrate conserves for every asset, what the asset declaration
/// fixed when the singleton was declared, and where a past issuance
/// placed the whole amount — and a reader chasing them goes to
/// deployment records, not to the spending context.
const DEPLOYMENT_FACTS: [StateExternalEvidenceRole; 3] = [
    StateExternalEvidenceRole::SubstrateConservation,
    StateExternalEvidenceRole::SingletonNonReissuable,
    StateExternalEvidenceRole::SingletonIssuedUnderConstructor,
];

/// Where a lead window's magnitudes came from.
///
/// The window's own type establishes that the pair is well formed; it
/// cannot establish that the numbers describe a deployment, because
/// two admissible integers look the same whatever produced them. A
/// link carrying fixture magnitudes and a link carrying calibrated
/// ones are different artifacts, and a reader who cannot tell them
/// apart has no ground on which to refuse the first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateLeadBoundOrigin {
    /// Test material standing for no deployment.
    ///
    /// The lead magnitudes 2 and 4 that the fixtures use are values of
    /// this kind. The architecture declares both lead bounds with no
    /// default and marks them as requiring deployment calibration, so
    /// no integer here is the contract's own: a fixture is a stand-in
    /// that lets the rest of the link be exercised, and naming it one
    /// is what keeps it from being read as the deployment's answer.
    Fixture,
    /// Resolved from one deployment's calibration of both magnitudes.
    DeploymentCalibrated,
}

/// A validated lead window together with the origin of its magnitudes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateLeadBounds {
    bounds: AnnouncementLeadBounds,
    origin: StateLeadBoundOrigin,
}

impl StateLeadBounds {
    /// Pair a validated window with the origin of its magnitudes.
    ///
    /// The window is not re-checked here. [`AnnouncementLeadBounds`]
    /// refuses a zero minimum and an inverted pair when it is built,
    /// and its architecture-keyed resolution refuses a magnitude the
    /// architecture does not carry; a second copy of those checks in
    /// this crate would be a second authority on one question, free to
    /// drift from the first and with no way to tell which had drifted.
    #[must_use]
    pub const fn new(bounds: AnnouncementLeadBounds, origin: StateLeadBoundOrigin) -> Self {
        Self { bounds, origin }
    }

    /// The validated lead window.
    #[must_use]
    pub const fn bounds(&self) -> AnnouncementLeadBounds {
        self.bounds
    }

    /// Where the window's magnitudes came from.
    #[must_use]
    pub const fn origin(&self) -> StateLeadBoundOrigin {
        self.origin
    }
}

/// The typed sources one maturity announcement link is given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateLinkDeploymentParameters {
    plan: ValidatedMaturityAnnouncementOperationPlan,
    lead_bounds: StateLeadBounds,
    revision: TargetContractVersion,
    identity: CandidateDeploymentIdentity,
    operator: OperatorDeploymentBinding,
    maximum_control_path_depth: NonZeroU32,
    deployment_facts: BTreeSet<StateExternalEvidenceRole>,
}

impl StateLinkDeploymentParameters {
    /// Bind the link's sources against the reviewed target.
    ///
    /// The revision is read from the reviewed target rather than
    /// supplied, so a caller cannot hold the operator's authorization
    /// to one revision and link the result against another.
    /// [`OperatorDeploymentBinding::check`] is the whole of the
    /// operator validation: it already compares key, deployment and
    /// revision in a fixed order, and a second comparison written here
    /// would be a second answer to a question already settled.
    ///
    /// # The deployment facts are read, not checked
    ///
    /// The record names the external roles its own emitted bytes rest
    /// on. What this constructor does with them is select those that
    /// are facts about a deployment and record them beside the binding,
    /// so a reader knows which deployment records the reduced leaf
    /// sends them to. It verifies none of them, and could not: neither
    /// the asset declaration that fixed the singleton as
    /// non-reissuable nor the issuance transaction that placed its
    /// whole amount under the constructor is something this process
    /// observes. Recording them is what keeps them from being silent
    /// assumptions; it is not a claim that they hold.
    ///
    /// A plan cannot arrive as bytes:
    /// ```compile_fail,E0308
    /// use std::num::NonZeroU32;
    /// use linker::{CandidateDeploymentIdentity, OperatorDeploymentBinding, StateLeadBounds,
    ///     StateLinkDeploymentParameters};
    /// use tapscript::StateAnnouncementProgram;
    /// use target_elements::ReviewedElementsTapscriptDefinition;
    /// fn raw_plan(target: &ReviewedElementsTapscriptDefinition, plan: Vec<u8>,
    ///     lead_bounds: StateLeadBounds, identity: CandidateDeploymentIdentity,
    ///     operator: OperatorDeploymentBinding, depth: NonZeroU32,
    ///     record: &StateAnnouncementProgram) {
    ///     StateLinkDeploymentParameters::bind(target, plan, lead_bounds, identity, operator,
    ///         depth, record);
    /// }
    /// ```
    /// A value map cannot stand in for the record:
    /// ```compile_fail,E0308
    /// use std::collections::BTreeMap;
    /// use std::num::NonZeroU32;
    /// use linker::{CandidateDeploymentIdentity, OperatorDeploymentBinding, StateLeadBounds,
    ///     StateLinkDeploymentParameters};
    /// use tapscript::upstream::ValidatedMaturityAnnouncementOperationPlan;
    /// use target_elements::ReviewedElementsTapscriptDefinition;
    /// fn value_map(target: &ReviewedElementsTapscriptDefinition,
    ///     plan: ValidatedMaturityAnnouncementOperationPlan, lead_bounds: StateLeadBounds,
    ///     identity: CandidateDeploymentIdentity, operator: OperatorDeploymentBinding,
    ///     depth: NonZeroU32, record: &BTreeMap<String, Vec<u8>>) {
    ///     StateLinkDeploymentParameters::bind(target, plan, lead_bounds, identity, operator,
    ///         depth, record);
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns what [`OperatorDeploymentBinding::check`] raises for the
    /// supplied identity: [`LinkRefusal::OperatorKeyMismatch`], then
    /// [`LinkRefusal::OperatorDeploymentMismatch`] for an identity
    /// other than the one the operator was bound under, then
    /// [`LinkRefusal::InvalidOperatorProfile`] for a profile pinned to
    /// another revision. The first is unreachable through this entry,
    /// because the key offered is the binding's own; the third is
    /// unreachable while exactly one reviewed definition exists,
    /// because the profile's pin and this revision are both read from
    /// it. Both are named regardless: the contract is `check`'s, and
    /// narrowing it here would make this documentation wrong on the day
    /// either premise stops holding.
    pub fn bind(
        target: &ReviewedElementsTapscriptDefinition,
        plan: ValidatedMaturityAnnouncementOperationPlan,
        lead_bounds: StateLeadBounds,
        identity: CandidateDeploymentIdentity,
        operator: OperatorDeploymentBinding,
        maximum_control_path_depth: NonZeroU32,
        record: &StateAnnouncementProgram,
    ) -> Result<Self, LinkRefusal> {
        let revision = target.definition().version();
        operator.check(operator.key(), &identity, revision)?;

        let deployment_facts = record
            .metadata()
            .external
            .iter()
            .copied()
            .filter(|role| DEPLOYMENT_FACTS.contains(role))
            .collect();

        Ok(Self {
            plan,
            lead_bounds,
            revision,
            identity,
            operator,
            maximum_control_path_depth,
            deployment_facts,
        })
    }

    /// The validated announcement plan the link is for.
    #[must_use]
    pub const fn plan(&self) -> &ValidatedMaturityAnnouncementOperationPlan {
        &self.plan
    }

    /// The lead window and the origin of its magnitudes.
    #[must_use]
    pub const fn lead_bounds(&self) -> StateLeadBounds {
        self.lead_bounds
    }

    /// The reviewed target revision this link is bound to.
    #[must_use]
    pub const fn revision(&self) -> TargetContractVersion {
        self.revision
    }

    /// The candidate deployment identity the link is for.
    #[must_use]
    pub const fn identity(&self) -> &CandidateDeploymentIdentity {
        &self.identity
    }

    /// The established operator binding checked at construction.
    #[must_use]
    pub const fn operator(&self) -> &OperatorDeploymentBinding {
        &self.operator
    }

    /// The deepest control path the deployment admits.
    #[must_use]
    pub const fn maximum_control_path_depth(&self) -> NonZeroU32 {
        self.maximum_control_path_depth
    }

    /// The record's external roles that are facts about a deployment.
    #[must_use]
    pub const fn deployment_facts(&self) -> &BTreeSet<StateExternalEvidenceRole> {
        &self.deployment_facts
    }
}
