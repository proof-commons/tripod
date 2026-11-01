//! One confidential sponsor observation after its explicit accepted control.
//!
//! Construction material is borrowed locally; only public coordinates enter the campaign.

use super::{
    AssetField, AssetId, BranchContext, CandidateDeploymentIdentity, CandidateLinkedMaturityBundle,
    FinalizedMaturityAnnouncement, FundedOutput, MaturityAcceptanceRoute,
    MaturityAnnouncementRequest, MaturityDeployment, MaturityLead, MaturityViewStatement,
    MaturityWitnessSelection, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse,
    ObservedOutcomeLayer, OperationCaseId, OperationStep, OperationSubject, OracleStateCurve,
    Outpoint, PlanRefused, PublicMaturityStateView, RequestedForm, SponsorChangeRequest,
    TargetFundingSubject, TargetOperationPlanner, TargetSubmissionSubject, TargetTransaction,
    ValueField, asset_of, build_with_sponsor, check_coin, closure, closure_target, decode_hex,
    derive_maturity_announcement_abi, link_for, maturity_sources, outpoint_of, predecessor_program,
    sha256, sign_with_sponsor, singleton_declaration, sponsored,
};
use crate::confidential_materializer::{
    FirstPartyCommitmentCheck, ReferenceConfidentialMaterializer,
};
use crate::confidential_sponsor_reserve::{
    SPONSOR_RESERVE_AMOUNTS, frozen_sponsor_reserve_registry, sponsor_reserve_opening,
    sponsor_reserve_subject,
};
use serde_json::{Value, json};
use target_elements_conformance::confidential_fixture::{
    FrozenConfidentialFixtureRegistry, sponsor_reserve_handle,
};
use target_elements_conformance::confidential_funding::RangeproofVerifier;
use target_elements_conformance::confidential_oracles::{
    ReferenceProofMaterial, ReferenceRangeproofVerifier,
};
use target_elements_conformance::protocol::{
    ConfidentialFundedOutput, TargetSponsorFundingSubject, TargetSponsorSigningSubject,
    WireOutpoint, WireSighashProfile,
};
use transaction::bytes::NonceField;
use transaction::sponsor::{
    SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest,
};
use transaction::view::{PublicConstructionView, PublicOutputView};

/// The sole family in this profile.
pub const FAMILY: &str = "confidential-sponsor";
/// The Safety claim, whose final build-property standing requires separate archive admission.
pub const CLAIM: &str =
    "confidential-sponsor-value-where-the-selected-target-policy-claims-support";
/// Exact successful schedule; template transactions are never submitted.
pub const STEPS: [&str; 14] = [
    "issue",
    "control-predecessor",
    "fund-template-sponsor",
    "sign-control-template",
    "measure-control",
    "fund-sponsor",
    "sign-control",
    "control",
    "subject-predecessor",
    "fund-confidential-sponsor-reserve",
    "sign-confidential-template",
    "measure-confidential",
    "sign-confidential",
    CLAIM,
];
const RESERVE: u64 = SPONSOR_RESERVE_AMOUNTS[0];
const MAX_PROOF: usize = 5_134;

/// A refusal contains public coordinates, never construction material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteRefusal {
    /// First binding that did not hold.
    pub field: &'static str,
}
const fn refused(field: &'static str) -> RouteRefusal {
    RouteRefusal { field }
}
fn map<T>(result: Result<T, super::Refusal>, field: &'static str) -> Result<T, RouteRefusal> {
    result.map_err(|_| refused(field))
}
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut text = String::new();
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}
fn point(point: Outpoint) -> WireOutpoint {
    WireOutpoint {
        txid: point.txid().to_target_display(),
        vout: point.index(),
    }
}

/// Typed intent frozen before issuance; observation acceptance is not a Safety layer.
pub struct RouteSpecification {
    row: &'static crate::maturity_safety::MaturitySafetyRow,
    identity: CandidateDeploymentIdentity,
    branch: BranchContext,
    schedule: tapscript::StateWitnessSchedule,
}
impl RouteSpecification {
    fn new(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
    ) -> Result<Self, RouteRefusal> {
        use crate::maturity_safety::{MaturityRowBoundary, MaturitySafetySection};
        let row = crate::maturity_safety::rows()
            .iter()
            .find(|row| row.section() == MaturitySafetySection::SponsorFault && row.name() == CLAIM)
            .ok_or_else(|| refused("typed claim"))?;
        if !matches!(
            row.boundary(),
            MaturityRowBoundary::ConfidentialSponsorAwaitsBalancedProofRoute
                | MaturityRowBoundary::ConfidentialSponsorValueUndecidedByCandidate
        ) {
            return Err(refused("typed boundary"));
        }
        let schedule = tapscript::StateWitnessSchedule::VariableMetadata;
        let sources = map(
            maturity_sources(
                MaturityDeployment::PublishedSignerHeld,
                MaturityWitnessSelection::Retained(schedule),
            )
            .map_err(closure),
            "candidate source",
        )?;
        let schedule = sources.record().schedule();
        Ok(Self {
            row,
            identity,
            branch,
            schedule,
        })
    }
    fn row(&self) -> Value {
        json!({"claim":{"section":self.row.section().section(),"name":self.row.name()},
            "operation":"announce_maturity", "family":FAMILY,
            "candidate":{"schedule":self.schedule.name()},"abi":self.schedule.name(),
            "target":"elements-tapscript-v2",
            "deployment":{"network":hex(self.identity.network_id()),"genesis":hex(self.identity.genesis_id())},
            "branch":{"identifier":hex(self.branch.identifier()),"checkpoint":self.branch.checkpoint()},
            "policy_requirement":"node-defaults","policy":null,
            "constructor":"construct_confidential_maturity_announcement","mutation":"explicit-to-confidential-sponsor-value",
            "declared_boundary":"build-property","expected_layer":ObservedOutcomeLayer::Accepted,
            "control_id":format!("{FAMILY}:control:7"),
            "compatibility":{"kind":"equivalent-coins","same":["issued-asset","predecessor-program","singleton-amount","metadata","request","reserve-asset","sponsor-program","candidate","abi","operator","sponsor-key","policy"],"different":["outpoints","identities","signatures","value-representation","fee","change"]},
            "control":null,"observation":null,"expected_consumer":"final-build-property-reason"})
    }
}

/// Frozen campaign intent with public funding joined before submission.
#[derive(Clone)]
pub struct CampaignManifest {
    /// Exactly one row; untrusted projections must be compared with replay.
    pub rows: Vec<Value>,
    /// Public funding records in request order.
    pub funding: Vec<(String, Value)>,
    control_accepted: bool,
    canonical: Value,
}
impl CampaignManifest {
    fn new(specification: &RouteSpecification) -> Self {
        let canonical = specification.row();
        Self {
            rows: vec![canonical.clone()],
            canonical,
            funding: Vec::new(),
            control_accepted: false,
        }
    }
    fn retain(&mut self, step: &OperationStep, response: &NativeOperationResponse) {
        self.funding.extend(
            response
                .funded_outputs
                .iter()
                .map(|coin| (step.case().step.clone(), json!(coin))),
        );
        self.funding.extend(
            response
                .confidential_funded_outputs
                .iter()
                .map(|coin| (step.case().step.clone(), json!(coin))),
        );
    }
    /// Bind a submission to frozen intent and previously validated funding.
    /// # Errors
    /// Returns the first changed public coordinate.
    #[expect(
        clippy::too_many_lines,
        reason = "The closed campaign guards are checked in their refusal order."
    )]
    pub fn bind_submission(
        &mut self,
        step: &OperationStep,
        policy: Option<&sponsored::Policy>,
    ) -> Result<(), RouteRefusal> {
        let OperationSubject::Submission(subject) = step.subject() else {
            return Ok(());
        };
        let parts = step.case().step.split(':').collect::<Vec<_>>();
        let [family, label, ordinal] = parts.as_slice() else {
            return Err(refused("step shape"));
        };
        if ordinal
            .parse::<usize>()
            .ok()
            .is_none_or(|n| n.to_string() != *ordinal)
        {
            return Err(refused("step shape"));
        }
        if *family != FAMILY {
            return Err(refused("unknown family"));
        }
        let [row] = self.rows.as_mut_slice() else {
            return Err(refused("row census"));
        };
        let control = *label == "control";
        if control && (row["control_id"] != step.case().step || *ordinal != "7") {
            return Err(refused("control identity"));
        }
        if !control
            && (*label != CLAIM || *ordinal != "13" || row["claim"] != self.canonical["claim"])
        {
            return Err(refused("subject claim mapping"));
        }
        for field in [
            "candidate",
            "abi",
            "deployment",
            "branch",
            "constructor",
            "mutation",
            "expected_layer",
            "expected_consumer",
            "compatibility",
        ] {
            if row[field] != self.canonical[field] {
                return Err(refused("incompatible control"));
            }
        }
        let tx = TargetTransaction::decode(&subject.transaction_bytes)
            .map_err(|_| refused("transaction encoding"))?;
        let inputs = tx
            .inputs()
            .iter()
            .map(|input| {
                let (funding, coin) = self
                    .funding
                    .iter()
                    .find(|(_, coin)| coin["outpoint"] == json!(point(input.outpoint())))
                    .ok_or_else(|| refused("unretained funding"))?;
                Ok(json!({"funding_step":funding,"output":coin}))
            })
            .collect::<Result<Vec<_>, RouteRefusal>>()?;
        if inputs.len() != 2 {
            return Err(refused("input census"));
        }
        let policy = policy.ok_or_else(|| refused("policy"))?;
        if !row["policy"].is_null() && row["policy"] != policy.0 {
            return Err(refused("policy"));
        }
        let slot = if control { "control" } else { "observation" };
        if !row[slot].is_null() {
            return Err(refused("duplicate submission"));
        }
        if !control {
            if !self.control_accepted {
                return Err(refused("subject before accepted control"));
            }
            let old = &row["control"]["funded_outpoints"];
            for key in ["asset", "amount_satoshis", "script"] {
                if old[0]["output"][key] != inputs[0]["output"][key] {
                    return Err(refused("incompatible control"));
                }
            }
            if old[1]["output"]["asset"] != inputs[1]["output"]["explicit_asset"]
                || old[1]["output"]["script"] != inputs[1]["output"]["script"]
            {
                return Err(refused("incompatible control"));
            }
            if inputs.iter().any(|new| {
                old.as_array().is_some_and(|coins| {
                    coins
                        .iter()
                        .any(|old| old["output"]["outpoint"] == new["output"]["outpoint"])
                })
            }) {
                return Err(refused("reused control outpoints"));
            }
            let coin: ConfidentialFundedOutput =
                serde_json::from_value(inputs[1]["output"].clone())
                    .map_err(|_| refused("confidential funding fields"))?;
            if coin.value_commitment.len() != 33
                || coin.nonce.len() != 33
                || !coin.surjection_proof.is_empty()
                || coin.rangeproof.is_empty()
                || coin.rangeproof.len() > MAX_PROOF
            {
                return Err(refused("confidential funding fields"));
            }
        }
        row["policy"] = policy.0.clone();
        let Some(ValueField::Explicit(fee)) = tx
            .outputs()
            .last()
            .map(transaction::bytes::TargetOutput::value)
        else {
            return Err(refused("fee fit"));
        };
        let change = RESERVE
            .checked_sub(fee)
            .filter(|change| *change > 0)
            .ok_or_else(|| refused("fee fit"))?;
        row[slot] = json!({"step":step.case().step,"digest":hex(&sha256(&subject.transaction_bytes)),"funded_outpoints":inputs,"claims":[row["claim"].clone()],"fee":fee,"change":change,"weight":tx.weight(),"readback":null});
        Ok(())
    }
    /// Exact comparison against a replayed planner, including all closed fields.
    /// # Errors
    /// Refuses any changed campaign projection.
    pub fn validate(plan: &Planner, offered: &[Value]) -> Result<(), RouteRefusal> {
        let replay = replay_confidential_route(
            plan.specification.identity.clone(),
            plan.specification.branch,
            &plan.exchanges,
        )?;
        if replay.manifest.rows != offered {
            return Err(refused("campaign projection"));
        }
        Ok(())
    }
}

/// A public campaign coordinate that failed record binding or archive admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidentialCampaignRefusal {
    /// Family whose record was refused.
    pub family: String,
    /// First field that disagreed.
    pub field: &'static str,
}

fn campaign_refused(field: &'static str) -> ConfidentialCampaignRefusal {
    ConfidentialCampaignRefusal {
        family: FAMILY.to_owned(),
        field,
    }
}

/// Closed records shared by the diagnostic wrapper and immutable archive admission.
pub struct ConfidentialCampaignFields {
    /// Selected default policy and its provenance.
    pub policy: Value,
    /// Single family, including the complete control and subject readbacks.
    pub families: Value,
    /// Families stopped before a qualified control.
    pub stopped: Value,
    /// Deployment readings, never a pinned library identity.
    pub preflight: Value,
    /// Closed campaign projection.
    pub rows: Vec<Value>,
}

impl ConfidentialCampaignFields {
    /// Bind preflight readings without treating a self-built library as a constant.
    /// # Errors
    /// Refuses surplus keys, missing capabilities, changed profiles or unequal readings.
    pub fn validate_preflight(&self) -> Result<(), ConfidentialCampaignRefusal> {
        let p = &self.preflight;
        let keys = [
            "library_configured",
            "library_readable",
            "library_sha256",
            "probe",
            "capture",
        ];
        if !p.as_object().is_some_and(|map| {
            map.len() == keys.len() && keys.iter().all(|key| map.contains_key(*key))
        }) {
            return Err(campaign_refused("preflight keys"));
        }
        for key in ["library_configured", "library_readable"] {
            if p[key] != true {
                return Err(campaign_refused(key));
            }
        }
        if !p["library_sha256"].as_str().is_some_and(|s| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }) {
            return Err(campaign_refused("library_sha256"));
        }
        let expected = json!({
            "confidential_value_test_funding": true,
            "confidential_value_sponsor_authorization": true,
            "advertisement_matches": true,
            "profiles": crate::confidential_predecessor::selected_profiles()
        });
        if p["probe"] != p["capture"] {
            return Err(campaign_refused("probe"));
        }
        if p["capture"] != expected {
            return Err(campaign_refused("capture"));
        }
        Ok(())
    }

    /// Bind diagnostic records to the replayed planner without requiring agreement.
    /// # Errors
    /// Refuses any changed policy, family, campaign, stopped set or preflight reading.
    pub fn validate(&self, plan: &Planner) -> Result<(), ConfidentialCampaignRefusal> {
        if plan.policy().map(|p| &p.0) != Some(&self.policy) {
            return Err(campaign_refused("policy"));
        }
        let expected = plan.families_json();
        let keys = [
            "family",
            "claim",
            "control",
            "subject",
            "stopped",
            "expectation",
            "control_accepted",
            "eligible",
            "refusal",
        ];
        if self.families.as_array().is_none_or(|rows| rows.len() != 1) {
            return Err(campaign_refused("families"));
        }
        let family = &self.families[0];
        if !family.as_object().is_some_and(|map| {
            map.len() == keys.len() && keys.iter().all(|key| map.contains_key(*key))
        }) {
            return Err(campaign_refused("family keys"));
        }
        for key in keys {
            if family[key] != expected[0][key] {
                return Err(campaign_refused(key));
            }
        }
        let stopped = if expected[0]["stopped"] == true {
            json!([FAMILY])
        } else {
            json!([])
        };
        if self.stopped != stopped {
            return Err(campaign_refused("stopped set"));
        }
        self.validate_preflight()?;
        if self.rows.len() != 1 {
            return Err(campaign_refused("campaign count"));
        }
        let canonical = &plan.manifest.rows[0];
        let offered = &self.rows[0];
        if offered
            .as_object()
            .is_none_or(|map| Some(map.len()) != canonical.as_object().map(serde_json::Map::len))
        {
            return Err(campaign_refused("campaign keys"));
        }
        // These comparisons reject mutations before the independent, more expensive replay.
        for key in [
            "claim",
            "operation",
            "family",
            "candidate",
            "abi",
            "target",
            "deployment",
            "branch",
            "policy_requirement",
            "policy",
            "constructor",
            "mutation",
            "declared_boundary",
            "expected_layer",
            "control_id",
            "compatibility",
            "control",
            "observation",
            "expected_consumer",
        ] {
            if offered[key] != canonical[key] {
                return Err(campaign_refused(key));
            }
        }
        CampaignManifest::validate(plan, &self.rows).map_err(|error| campaign_refused(error.field))
    }

    /// Require the accepted-control agreement boundary for immutable archive admission.
    /// # Errors
    /// Refuses changed records or a stopped, contradicted or pending family, even if structurally eligible.
    pub fn admit(&self, plan: &Planner) -> Result<(), ConfidentialCampaignRefusal> {
        self.validate(plan)?;
        self.require_agreement()?;
        plan.admit().map_err(|error| campaign_refused(error.field))
    }

    /// Check the closed admission outcome before expensive route replay.
    /// # Errors
    /// Refuses every outcome other than eligible agreement after an accepted control.
    pub fn require_agreement(&self) -> Result<(), ConfidentialCampaignRefusal> {
        let family = &self.families[0];
        if self.stopped != json!([]) {
            return Err(campaign_refused("stopped set"));
        }
        for (key, value) in [
            ("family", json!(FAMILY)),
            ("stopped", json!(false)),
            ("expectation", json!("agreement")),
            ("control_accepted", json!(true)),
            ("eligible", json!(true)),
            ("refusal", Value::Null),
        ] {
            if family[key] != value {
                return Err(campaign_refused(key));
            }
        }
        Ok(())
    }
}

struct Sponsor {
    outpoint: Outpoint,
    program: Vec<u8>,
    fee: u64,
    answer: Option<SponsorSignature>,
}
impl SponsorCapability for Sponsor {
    fn offer(&self) -> SponsorOffer {
        SponsorOffer::new(
            [self.outpoint],
            self.fee,
            Some(ValueField::Explicit(RESERVE - self.fee)),
        )
        .expect("validated positive reserve change")
    }
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, self.program.get(2..)?.to_vec()))
    }
    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        self.answer
            .clone()
            .filter(|answer| answer.bound_to() == request.transaction() && request.input() == 1)
    }
}

/// Serial planner; the confidential arm is unreachable until the control qualifies.
pub struct Planner {
    specification: RouteSpecification,
    /// Campaign record frozen before the first request.
    pub manifest: CampaignManifest,
    pending: Option<OperationStep>,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    policy: Option<sponsored::Policy>,
    asset: Option<String>,
    bundle: Option<CandidateLinkedMaturityBundle>,
    predecessor: Option<Outpoint>,
    sponsor: Option<Sponsor>,
    reserve: Option<FundedOutput>,
    committed: Option<ConfidentialFundedOutput>,
    registry: Option<FrozenConfidentialFixtureRegistry>,
    finalized: Option<FinalizedMaturityAnnouncement>,
    signed: Vec<u8>,
    fee: u64,
    refusal: Option<RouteRefusal>,
    label: &'static str,
    eligible: bool,
}
impl Planner {
    /// Freeze the typed claim and candidate before any operation leaves the planner.
    /// # Errors
    /// Refuses an absent typed claim or source binding.
    pub fn new(
        identity: CandidateDeploymentIdentity,
        branch: BranchContext,
    ) -> Result<Self, RouteRefusal> {
        let specification = RouteSpecification::new(identity, branch)?;
        let manifest = CampaignManifest::new(&specification);
        Ok(Self {
            specification,
            manifest,
            pending: None,
            exchanges: Vec::new(),
            policy: None,
            asset: None,
            bundle: None,
            predecessor: None,
            sponsor: None,
            reserve: None,
            committed: None,
            registry: None,
            finalized: None,
            signed: Vec::new(),
            fee: RESERVE / 2,
            refusal: None,
            label: "stopped",
            eligible: false,
        })
    }
    /// Raw exchanges, including the response that stopped the family.
    #[must_use]
    pub fn exchanges(&self) -> &[(OperationStep, NativeOperationResponse)] {
        &self.exchanges
    }
    /// Validated default policy shared by both arms.
    #[must_use]
    pub const fn policy(&self) -> Option<&sponsored::Policy> {
        self.policy.as_ref()
    }
    /// Public refusal coordinate.
    #[must_use]
    pub const fn refusal(&self) -> Option<&RouteRefusal> {
        self.refusal.as_ref()
    }
    /// Structural eligibility is deliberately weaker than archive admission.
    #[must_use]
    pub const fn eligible(&self) -> bool {
        self.eligible
    }
    /// The label records agreement, contradiction, stopped control or pending subject.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        self.label
    }
    /// Closed family record; exact readbacks remain in the campaign and exchanges.
    #[must_use]
    pub fn families_json(&self) -> Value {
        json!([{"family":FAMILY,"claim":self.specification.row()["claim"],"expectation":self.label,"stopped":!self.manifest.control_accepted,"control_accepted":self.manifest.control_accepted,"control":self.manifest.rows[0]["control"],"subject":self.manifest.rows[0]["observation"],"eligible":self.eligible,"refusal":self.refusal.as_ref().map(|refusal| refusal.field)}])
    }
    /// Whole-archive admission requires agreement after the qualified control.
    /// # Errors
    /// Contradiction, stopping and incomplete readback all refuse admission.
    pub fn admit(&self) -> Result<(), RouteRefusal> {
        if self.eligible && self.label == "agreement" {
            Ok(())
        } else {
            Err(refused("admission"))
        }
    }
    fn sponsor(&self) -> Result<&Sponsor, RouteRefusal> {
        self.sponsor.as_ref().ok_or_else(|| refused("sponsor"))
    }
    #[expect(
        clippy::too_many_lines,
        reason = "Both representations share the same STATE request before materialization."
    )]
    fn build(&self, confidential: bool) -> Result<FinalizedMaturityAnnouncement, RouteRefusal> {
        let sponsor = self.sponsor()?;
        let bundle = self.bundle.as_ref().ok_or_else(|| refused("candidate"))?;
        let asset = self
            .asset
            .as_deref()
            .and_then(asset_of)
            .ok_or_else(|| refused("asset"))?;
        let predecessor = self.predecessor.ok_or_else(|| refused("predecessor"))?;
        let reserve = self.reserve.as_ref().ok_or_else(|| refused("reserve"))?;
        let reserve_asset = asset_of(&reserve.asset).ok_or_else(|| refused("reserve asset"))?;
        let value = if confidential {
            ValueField::Commitment(
                self.committed
                    .as_ref()
                    .ok_or_else(|| refused("committed coin"))?
                    .value_commitment
                    .clone()
                    .try_into()
                    .map_err(|_| refused("commitment"))?,
            )
        } else {
            ValueField::Explicit(RESERVE)
        };
        let public = PublicConstructionView::new([PublicOutputView::new(
            sponsor.outpoint,
            AssetField::Explicit(reserve_asset),
            value,
            sponsor.program.clone(),
        )])
        .map_err(|_| refused("public construction view"))?;
        let context = transaction::MaturitySponsorContext {
            reserve_asset,
            view: &public,
            capability: sponsor,
        };
        let amount = map(singleton_declaration(), "singleton")?
            .fixed_amount()
            .get();
        if !confidential {
            return map(
                build_with_sponsor(
                    bundle,
                    asset,
                    predecessor,
                    self.specification.branch,
                    amount,
                    MaturityLead::Interior,
                    Some(&context),
                ),
                "explicit construction",
            );
        }
        let target = closure_target().map_err(|_| refused("target"))?;
        let state = &bundle.instances()[0];
        let metadata = state.metadata();
        let view = PublicMaturityStateView::new([
            MaturityViewStatement::CurrentStateOutpoint(predecessor),
            MaturityViewStatement::AssetAndAmount(
                AssetField::Explicit(asset),
                ValueField::Explicit(amount),
            ),
            MaturityViewStatement::PredecessorMetadata(metadata.semantic),
            MaturityViewStatement::PredecessorRepresentationNonce(metadata.representation),
            MaturityViewStatement::CurrentRootBinding(self.specification.branch),
            MaturityViewStatement::PredecessorProgram(state.constructor().output_program()),
            MaturityViewStatement::AcceptedLinkedBundle(Box::new(bundle.clone())),
        ])
        .and_then(|view| view.validate(&target, &OracleStateCurve))
        .map_err(|_| refused("STATE view"))?;
        let abi = derive_maturity_announcement_abi(&target, &view).map_err(|_| refused("ABI"))?;
        let window = bundle
            .deployment()
            .lead_bounds()
            .bounds()
            .window(metadata.semantic.cycle)
            .map_err(|_| refused("request"))?;
        let request = MaturityAnnouncementRequest::new(
            MaturityLead::Interior.announced(window),
            RequestedForm::Sponsored,
            SponsorChangeRequest::Requested,
        )
        .map_err(|_| refused("request"))?;
        let registry = self.registry.as_ref().ok_or_else(|| refused("registry"))?;
        let digest = registry
            .registered_digest(&sponsor_reserve_handle())
            .ok_or_else(|| refused("fixture digest"))?;
        let opening =
            sponsor_reserve_opening(registry, digest).map_err(|_| refused("reserve opening"))?;
        let derive = |domain, input: &[u8; 32]| {
            let mut bytes = input.to_vec();
            bytes.extend(1_u16.to_be_bytes());
            transaction::taproot::tagged_hash(domain, &bytes)
        };
        let material = transaction::MaturityConfidentialSponsorMaterial {
            outpoint: sponsor.outpoint,
            semantic_input_amount: RESERVE,
            input_blinder_sum: opening.value_blinder,
            nonce_input: derive("STATE-change/nonce", &opening.nonce_input),
            rangeproof_seed: derive("STATE-change/rangeproof", &opening.rangeproof_seed),
        };
        transaction::construct_confidential_maturity_announcement(
            transaction::MaturityConfidentialConstructionInputs {
                target: &target,
                abi: &abi,
                validated: &view,
                request: &request,
                curve: &OracleStateCurve,
                sponsor: &context,
                material: &material,
            },
            &ReferenceConfidentialMaterializer::new(),
            &FirstPartyCommitmentCheck::new(),
        )
        .map_err(|_| refused("confidential construction"))
    }
    fn funding(&self, issue: bool) -> Result<OperationSubject, RouteRefusal> {
        let bootstrap;
        let bundle = if issue {
            bootstrap = map(
                link_for(
                    &self.specification.identity,
                    AssetId::from_internal([0x11; 32]),
                    MaturityWitnessSelection::Retained(self.specification.schedule),
                ),
                "candidate",
            )?
            .0;
            &bootstrap
        } else {
            self.bundle.as_ref().ok_or_else(|| refused("candidate"))?
        };
        Ok(OperationSubject::Funding(Box::new(TargetFundingSubject {
            issue_asset: issue,
            asset: if issue { None } else { self.asset.clone() },
            output_program: predecessor_program(bundle),
            outputs: 1,
            amount_per_output: map(singleton_declaration(), "singleton")?
                .fixed_amount()
                .get(),
        })))
    }
    fn signing(&self, bytes: Vec<u8>) -> Result<OperationSubject, RouteRefusal> {
        Ok(OperationSubject::SponsorSigning(Box::new(
            TargetSponsorSigningSubject {
                finalized_transaction: bytes,
                sponsor_input_index: 1,
                sponsor_outpoint: point(self.sponsor()?.outpoint),
                sighash_profile: WireSighashProfile::AllInputsAllOutputs,
            },
        )))
    }
    fn next(&mut self) -> Result<Option<OperationStep>, RouteRefusal> {
        let n = self.exchanges.len();
        let subject = match n {
            0 => self.funding(true)?,
            1 | 8 => self.funding(false)?,
            2 | 5 => OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                sponsor_outputs: 1,
                amount_per_sponsor_output: RESERVE,
            })),
            3 | 6 | 10 | 12 => {
                let finalized = self.build(n >= 10)?;
                let subject = self.signing(finalized.protected_bytes().to_vec())?;
                self.finalized = Some(finalized);
                subject
            }
            4 | 11 => self.signing(self.signed.clone())?,
            7 | 13 => OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: self.signed.clone(),
            })),
            9 => {
                let reserve = self.reserve.as_ref().ok_or_else(|| refused("reserve"))?;
                let program =
                    decode_hex(&reserve.script).ok_or_else(|| refused("sponsor program"))?;
                let asset = asset_of(&reserve.asset).ok_or_else(|| refused("reserve asset"))?;
                let registry = frozen_sponsor_reserve_registry(*asset.internal(), &program)
                    .map_err(|_| refused("registry"))?;
                let digest = *registry
                    .registered_digest(&sponsor_reserve_handle())
                    .ok_or_else(|| refused("fixture digest"))?;
                self.registry = Some(registry);
                OperationSubject::ConfidentialSponsorFunding(Box::new(sponsor_reserve_subject(
                    digest, &program,
                )))
            }
            _ => return Ok(None),
        };
        let step = OperationStep::new(&format!("{FAMILY}:{}:{n}", STEPS[n]), subject);
        self.manifest.bind_submission(&step, self.policy.as_ref())?;
        self.pending = Some(step.clone());
        Ok(Some(step))
    }
    #[expect(
        clippy::too_many_lines,
        reason = "The fourteen exchanges are settled in schedule order at one boundary."
    )]
    fn settle(
        &mut self,
        step: &OperationStep,
        response: &NativeOperationResponse,
    ) -> Result<bool, RouteRefusal> {
        let n = self.exchanges.len() - 1;
        let (policy, route) = map(sponsored::detail(response), "policy or route")?;
        if self.policy.as_ref().is_some_and(|prior| prior != &policy) {
            return Err(refused("policy"));
        }
        self.policy = Some(policy);
        match n {
            0 | 1 | 8 => {
                let OperationSubject::Funding(subject) = step.subject() else {
                    return Err(refused("funding request"));
                };
                let [coin] = response.funded_outputs.as_slice() else {
                    return Err(refused("funding census"));
                };
                if response.observed_layer != ObservedOutcomeLayer::Accepted {
                    return Err(refused("funding acceptance"));
                }
                let asset = if n == 0 {
                    response.issued_asset.as_deref()
                } else {
                    self.asset.as_deref()
                }
                .ok_or_else(|| refused("issued asset"))?;
                let outpoint = map(check_coin(subject, coin, asset), "predecessor funding")?;
                if n == 0 {
                    self.bundle = Some(
                        map(
                            link_for(
                                &self.specification.identity,
                                asset_of(asset).ok_or_else(|| refused("issued asset"))?,
                                MaturityWitnessSelection::Retained(self.specification.schedule),
                            ),
                            "candidate",
                        )?
                        .0,
                    );
                    self.asset = Some(asset.to_owned());
                } else {
                    self.predecessor = Some(outpoint);
                }
            }
            2 | 5 => {
                let [coin] = response.funded_outputs.as_slice() else {
                    return Err(refused("reserve census"));
                };
                let program = decode_hex(&coin.script).ok_or_else(|| refused("sponsor program"))?;
                if response.observed_layer != ObservedOutcomeLayer::Accepted
                    || coin.amount_satoshis != RESERVE
                    || program.len() != 22
                    || program[..2] != [0, 20]
                {
                    return Err(refused("reserve funding"));
                }
                if let Some(old) = &self.reserve
                    && (old.asset != coin.asset
                        || old.script != coin.script
                        || old.outpoint == coin.outpoint)
                {
                    return Err(refused("reserve compatibility"));
                }
                self.sponsor = Some(Sponsor {
                    outpoint: outpoint_of(&coin.outpoint)
                        .ok_or_else(|| refused("reserve outpoint"))?,
                    program,
                    fee: self.fee,
                    answer: None,
                });
                self.reserve = Some(coin.clone());
            }
            3 | 6 | 10 | 12 => {
                let signature = map(
                    sponsored::checked_signature_program(step, response, &self.sponsor()?.program),
                    "sponsor signature",
                )?;
                self.sponsor
                    .as_mut()
                    .ok_or_else(|| refused("sponsor"))?
                    .answer = Some(signature);
                self.signed = map(
                    sign_with_sponsor(
                        self.finalized
                            .as_ref()
                            .ok_or_else(|| refused("finalized"))?,
                        Some(self.sponsor()?),
                    ),
                    "operator signing",
                )?;
                if n == 6 || n == 12 {
                    let weight = TargetTransaction::decode(&self.signed)
                        .map_err(|_| refused("signed encoding"))?
                        .weight();
                    check_final_fee(
                        self.policy.as_ref().ok_or_else(|| refused("policy"))?,
                        weight,
                        self.fee,
                    )?;
                }
            }
            4 | 11 => {
                map(
                    sponsored::checked_signature_program(step, response, &self.sponsor()?.program),
                    "measurement signature",
                )?;
                let tx = TargetTransaction::decode(&self.signed)
                    .map_err(|_| refused("measurement encoding"))?;
                if response.resources.transaction_weight != Some(tx.weight()) {
                    return Err(refused("measurement weight"));
                }
                let proof = tx
                    .output_witnesses()
                    .get(1)
                    .ok_or_else(|| refused("proof census"))?
                    .range_proof()
                    .len();
                let growth = if n == 11 {
                    MAX_PROOF
                        .checked_sub(proof)
                        .ok_or_else(|| refused("proof bound"))?
                        + 2
                } else {
                    0
                };
                self.fee = measured_fee(
                    self.policy.as_ref().ok_or_else(|| refused("policy"))?,
                    tx.weight(),
                    growth,
                )?;
                if n == 11 {
                    let sponsor = self.sponsor.as_mut().ok_or_else(|| refused("sponsor"))?;
                    sponsor.fee = self.fee;
                    sponsor.answer = None;
                }
            }
            7 | 13 => {
                let slot = if n == 7 { "control" } else { "observation" };
                if response.observed_layer == ObservedOutcomeLayer::Accepted {
                    map(
                        sponsored::check_readback(&self.signed, response),
                        "readback mismatch",
                    )?;
                    self.manifest.rows[0][slot]["readback"] = json!(response.mined_readback);
                    if route != Some(MaturityAcceptanceRoute::RelayAndTemplate) {
                        self.label = if n == 7 { "stopped" } else { "contradiction" };
                        self.eligible = n == 13;
                        return Ok(false);
                    }
                    if n == 7 {
                        self.manifest.control_accepted = true;
                        self.label = "pending";
                        self.fee = RESERVE / 2;
                    } else {
                        self.label = "agreement";
                        self.eligible = true;
                    }
                } else if matches!(
                    response.observed_layer,
                    ObservedOutcomeLayer::ConsensusRejectionBeforeScript
                        | ObservedOutcomeLayer::RelayPolicyRejection
                        | ObservedOutcomeLayer::ScriptPathRejection
                        | ObservedOutcomeLayer::KeyPathRejection
                ) {
                    self.label = if n == 7 { "stopped" } else { "contradiction" };
                    self.eligible = n == 13;
                    return Ok(false);
                } else {
                    return Err(refused("infrastructure"));
                }
            }
            9 => self.bind_committed(response)?,
            _ => return Err(refused("schedule")),
        }
        self.manifest.retain(step, response);
        Ok(true)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "The funding binding checks one mined transaction against its complete fixture."
    )]
    fn bind_committed(&mut self, response: &NativeOperationResponse) -> Result<(), RouteRefusal> {
        if !self.manifest.control_accepted
            || response.observed_layer != ObservedOutcomeLayer::Accepted
            || response.confidential_funded_outputs.len() != 2
        {
            return Err(refused("confidential funding census"));
        }
        let reserve = self.reserve.as_ref().ok_or_else(|| refused("reserve"))?;
        let registry = self.registry.as_ref().ok_or_else(|| refused("registry"))?;
        let digest = registry
            .registered_digest(&sponsor_reserve_handle())
            .ok_or_else(|| refused("fixture digest"))?;
        let resolved = registry
            .resolve(&sponsor_reserve_handle(), digest)
            .map_err(|_| refused("fixture binding"))?;
        let commitments = resolved
            .value_commitments()
            .ok_or_else(|| refused("commitments"))?;
        let readback = response
            .mined_readback
            .as_ref()
            .ok_or_else(|| refused("funding readback"))?;
        let tx = TargetTransaction::decode(&readback.raw_transaction)
            .map_err(|_| refused("funding encoding"))?;
        let mut accepted = response.clone();
        accepted.accepted_txid = Some(readback.transaction_id.clone());
        map(
            sponsored::check_readback(&readback.raw_transaction, &accepted),
            "funding readback",
        )?;
        let verifier = ReferenceRangeproofVerifier::new();
        let materializer = ReferenceProofMaterial::new();
        let target_elements_conformance::confidential_fixture::FixtureOpenings::Derived {
            openings,
            ..
        } = resolved.openings()
        else {
            return Err(refused("reserve opening"));
        };
        for ((coin, commitment), opening) in response
            .confidential_funded_outputs
            .iter()
            .zip(commitments)
            .zip(openings)
        {
            let opening = opening.as_ref().ok_or_else(|| refused("reserve opening"))?;
            let index =
                usize::try_from(coin.outpoint.vout).map_err(|_| refused("funding index"))?;
            let output = tx
                .outputs()
                .get(index)
                .ok_or_else(|| refused("funding output"))?;
            let witness = tx
                .output_witnesses()
                .get(index)
                .ok_or_else(|| refused("funding witness"))?;
            let program = decode_hex(&coin.script).ok_or_else(|| refused("funding program"))?;
            if coin.explicit_asset != reserve.asset {
                return Err(refused("confidential funding fields"));
            }
            if materializer
                .nonce_commitment(&opening.nonce_input)
                .map(|nonce| nonce.to_vec())
                .as_ref()
                != Some(&coin.nonce)
                || materializer
                    .range_proof(
                        resolved.explicit_asset(),
                        RESERVE,
                        &opening.value_blinder,
                        &opening.rangeproof_seed,
                        &commitment,
                        &program,
                    )
                    .as_ref()
                    != Some(&coin.rangeproof)
            {
                return Err(refused("confidential funding material binding"));
            }
            if coin.script != reserve.script
                || coin.outpoint.txid != readback.transaction_id
                || coin.output_witness_index != coin.outpoint.vout
                || coin.value_commitment != commitment
                || output.asset()
                    != AssetField::Explicit(
                        asset_of(&reserve.asset).ok_or_else(|| refused("reserve asset"))?,
                    )
                || output.value() != ValueField::Commitment(commitment)
                || output.program() != program
                || output.nonce()
                    != NonceField::Commitment(
                        coin.nonce
                            .clone()
                            .try_into()
                            .map_err(|_| refused("funding nonce"))?,
                    )
                || witness.range_proof() != coin.rangeproof
                || witness.surjection_proof() != coin.surjection_proof
                || !coin.surjection_proof.is_empty()
                || coin.rangeproof.len() > MAX_PROOF
                || !verifier.verifies(
                    &coin.rangeproof,
                    &coin.value_commitment,
                    &coin.explicit_asset,
                    &program,
                )
            {
                return Err(refused("confidential funding fields"));
            }
        }
        let coin = &response.confidential_funded_outputs[0];
        if coin.outpoint == response.confidential_funded_outputs[1].outpoint {
            return Err(refused("funding distinctness"));
        }
        self.sponsor = Some(Sponsor {
            outpoint: outpoint_of(&coin.outpoint).ok_or_else(|| refused("committed outpoint"))?,
            program: decode_hex(&coin.script).ok_or_else(|| refused("sponsor program"))?,
            fee: self.fee,
            answer: None,
        });
        self.committed = Some(coin.clone());
        Ok(())
    }
    fn advance(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, RouteRefusal> {
        match (self.pending.take(), previous) {
            (Some(step), Some((case, response)))
                if step.case() == case && &response.case == case =>
            {
                self.exchanges.push((step.clone(), response.clone()));
                if response.schema != NATIVE_PROTOCOL_SCHEMA || response.validate_shape().is_err() {
                    return Err(refused("response shape"));
                }
                if !self.settle(&step, response)? {
                    return Ok(None);
                }
            }
            (None, None) if self.exchanges.is_empty() => {}
            _ => return Err(refused("response order")),
        }
        self.next()
    }
}
impl TargetOperationPlanner for Planner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if self.refusal.is_some() {
            return Err(PlanRefused);
        }
        self.advance(previous).map_err(|error| {
            self.refusal = Some(error);
            self.eligible = false;
            PlanRefused
        })
    }
}
fn check_final_fee(policy: &sponsored::Policy, weight: u64, fee: u64) -> Result<(), RouteRefusal> {
    let required = map(policy.fee(weight), "fee fit")?;
    if required > fee || fee >= RESERVE {
        return Err(refused("fee fit"));
    }
    Ok(())
}

fn measured_fee(
    policy: &sponsored::Policy,
    weight: u64,
    growth: usize,
) -> Result<u64, RouteRefusal> {
    let weight = weight
        .checked_add(8)
        .and_then(|weight| weight.checked_add(u64::try_from(growth).ok()?))
        .ok_or_else(|| refused("fee fit"))?;
    let fee = map(policy.fee(weight), "fee fit")?;
    RESERVE
        .checked_sub(fee)
        .filter(|change| *change > 0)
        .ok_or_else(|| refused("fee fit"))?;
    Ok(fee)
}

/// Rebuild every request and campaign binding before accepting a transcript.
/// # Errors
/// Refuses altered requests, funding, control identity, ordering and incomplete schedules.
pub fn replay_confidential_route(
    identity: CandidateDeploymentIdentity,
    branch: BranchContext,
    exchanges: &[(OperationStep, NativeOperationResponse)],
) -> Result<Planner, RouteRefusal> {
    let mut planner = Planner::new(identity, branch)?;
    let mut next = planner
        .next_step(None)
        .map_err(|_| refused("replay start"))?;
    for (step, response) in exchanges {
        if next.as_ref() != Some(step) {
            return Err(refused("replay request"));
        }
        next = planner
            .next_step(Some((step.case(), response)))
            .map_err(|_| {
                planner
                    .refusal
                    .clone()
                    .unwrap_or_else(|| refused("replay response"))
            })?;
    }
    if next.is_some() {
        return Err(refused("incomplete replay"));
    }
    Ok(planner)
}

#[cfg(test)]
mod tests {
    use super::{RESERVE, check_final_fee, measured_fee, sponsored};

    fn policy() -> sponsored::Policy {
        sponsored::Policy(
            serde_json::json!({"relay":"0.00000100","mempool":"0.00000100","block":"0.00000100"}),
        )
    }

    #[test]
    fn measured_fee_includes_proof_growth_and_witness_margin() {
        let policy = policy();
        assert_eq!(measured_fee(&policy, 2809, 0).expect("explicit fee"), 141);
        assert_eq!(
            measured_fee(&policy, 2809, 5134).expect("proof growth"),
            398
        );
        assert_eq!(measured_fee(&policy, 2800, 0).expect("witness margin"), 141);
        assert_eq!(
            measured_fee(&policy, u64::MAX, 5134)
                .expect_err("overflow")
                .field,
            "fee fit"
        );
    }

    #[test]
    fn final_weight_fit_refuses_underpayment_and_nonpositive_change() {
        let policy = policy();
        assert!(check_final_fee(&policy, 2809, 141).is_ok());
        assert_eq!(
            check_final_fee(&policy, 2821, 141)
                .expect_err("final growth")
                .field,
            "fee fit"
        );
        assert_eq!(
            check_final_fee(&policy, 2809, RESERVE)
                .expect_err("zero change")
                .field,
            "fee fit"
        );
    }
}
