//! The candidate construction pipeline (§15.8), and what it returns.
//!
//! # Sixteen stages, in order, each refusing rather than degrading
//!
//! §15.8 lists the stages and [`construct`] runs them in that order.
//! Nothing is returned when a stage refuses: there is no partially
//! constructed transaction, which is §1.11 applied to a pipeline.
//!
//! The ordering constraint that matters most is stage eleven before
//! stage twelve — every protected output final before any signing
//! request is issued. It holds structurally here: the outputs are
//! assembled and the transaction is built before a
//! [`SponsorSigningRequest`] can exist, because building one needs the
//! finalized bytes.
//!
//! # What the report says, and what it cannot say
//!
//! The report carries public role data and the resource figures this
//! wave settles. It carries no individual sponsor amount — not because
//! the builder forgot, but because §1.6 forbids it and the type has no
//! field for one. The sponsor's own values reached the builder,
//! balanced the transaction, and stopped there.

use std::collections::{BTreeMap, BTreeSet};

use linker::backend::{CompactAshShape, OutputRole};
use target_elements::{
    ResourceDimension, ReviewedElementsTapscriptDefinition, SponsorProgramClass, TransactionForm,
};

use crate::abi::{CandidateTransactionAbi, ShapeAbi, TargetTransactionVersion, WitnessItem};
use crate::bytes::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, ValueField,
};
use crate::error::TransactionRefusal;
use crate::request::CompactAshRequest;
use crate::sponsor::{
    SighashProfile, SignerRole, SponsorCapability, SponsorSigningRequest,
    WITNESS_V0_KEYHASH_PROGRAM_BYTES, WITNESS_V0_KEYHASH_STACK_ITEMS,
};
use crate::synthetic::SyntheticDisclaimer;
use crate::taproot::witness_program_script;
use crate::view::PublicConstructionView;

/// The public role census of one constructed transaction.
///
/// Positions and roles, and no values. This is the projection §1.6
/// calls for: what a canonical report may retain about a sponsor is
/// where it sat, not what it held.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleCensus {
    ash_inputs: Vec<u16>,
    sponsor_inputs: Vec<u16>,
    outputs: BTreeMap<u16, OutputRole>,
}

impl RoleCensus {
    /// The positions the ASH family occupies.
    #[must_use]
    pub fn ash_inputs(&self) -> &[u16] {
        &self.ash_inputs
    }

    /// The positions the sponsor suffix occupies.
    #[must_use]
    pub fn sponsor_inputs(&self) -> &[u16] {
        &self.sponsor_inputs
    }

    /// Which role each output position holds.
    #[must_use]
    pub const fn outputs(&self) -> &BTreeMap<u16, OutputRole> {
        &self.outputs
    }
}

/// The exact resource figures one constructed transaction settles.
///
/// The three whole-transaction dimensions this wave owes. Each is
/// measured from the exact bytes rather than predicted from a formula:
/// a formula is a claim about a family, and these are facts about one
/// transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SettledResources {
    witness_bytes: u64,
    weight: u64,
    virtual_size: u64,
    package_transactions: u32,
    package_virtual_size: u64,
}

impl SettledResources {
    /// The serialized witness bytes, settling `WitnessBytes`.
    #[must_use]
    pub const fn witness_bytes(self) -> u64 {
        self.witness_bytes
    }

    /// The exact weight, settling `TransactionWeight`.
    #[must_use]
    pub const fn weight(self) -> u64 {
        self.weight
    }

    /// The virtual size the relay path measures the transaction in.
    #[must_use]
    pub const fn virtual_size(self) -> u64 {
        self.virtual_size
    }

    /// How many transactions the package this one travels in may hold,
    /// settling half of `PackageLimit`.
    #[must_use]
    pub const fn package_transactions(self) -> u32 {
        self.package_transactions
    }

    /// The virtual-size ceiling this transaction is held to inside that
    /// package, settling the other half.
    #[must_use]
    pub const fn package_virtual_size(self) -> u64 {
        self.package_virtual_size
    }

    /// Every dimension this settles, keyed the way the target names it.
    #[must_use]
    pub fn dimensions(self) -> BTreeMap<ResourceDimension, u64> {
        BTreeMap::from([
            (ResourceDimension::WitnessBytes, self.witness_bytes),
            (ResourceDimension::TransactionWeight, self.weight),
            (ResourceDimension::PackageLimit, self.package_virtual_size),
        ])
    }
}

/// What a construction reports about itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructionReport {
    shape: CompactAshShape,
    form: TransactionForm,
    version: TargetTransactionVersion,
    roles: RoleCensus,
    resources: SettledResources,
    successor_amount: u64,
    disclaimers: BTreeSet<SyntheticDisclaimer>,
}

impl ConstructionReport {
    /// The shape the request selected.
    #[must_use]
    pub const fn shape(&self) -> CompactAshShape {
        self.shape
    }

    /// Which reviewed transaction form was built.
    #[must_use]
    pub const fn form(&self) -> TransactionForm {
        self.form
    }

    /// The version the transaction was built at.
    #[must_use]
    pub const fn version(&self) -> TargetTransactionVersion {
        self.version
    }

    /// The public role census.
    #[must_use]
    pub const fn roles(&self) -> &RoleCensus {
        &self.roles
    }

    /// The settled resource figures.
    #[must_use]
    pub const fn resources(&self) -> SettledResources {
        self.resources
    }

    /// The successor ASH amount.
    ///
    /// Public, and not a sponsor value: the closed protocol asset is
    /// explicit under the fixed representation, so this figure is
    /// visible to anyone reading the transaction and reporting it
    /// reveals nothing the target does not.
    #[must_use]
    pub const fn successor_amount(&self) -> u64 {
        self.successor_amount
    }

    /// Every disclaimer the origin of the spent instance carries.
    #[must_use]
    pub const fn disclaimers(&self) -> &BTreeSet<SyntheticDisclaimer> {
        &self.disclaimers
    }
}

/// One constructed candidate transaction and its report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateCompleteTransaction {
    transaction: TargetTransaction,
    report: ConstructionReport,
}

impl CandidateCompleteTransaction {
    /// The typed transaction.
    #[must_use]
    pub const fn transaction(&self) -> &TargetTransaction {
        &self.transaction
    }

    /// The exact target bytes.
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        self.transaction.encode()
    }

    /// The construction report.
    #[must_use]
    pub const fn report(&self) -> &ConstructionReport {
        &self.report
    }
}

/// Run the candidate construction pipeline (§15.8).
///
/// # Errors
///
/// Every refusal the stages raise, and no partial transaction on any
/// path. The stage each refusal belongs to is named in its own
/// documentation.
pub fn construct(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateTransactionAbi,
    request: &CompactAshRequest,
    view: &PublicConstructionView,
    sponsor: Option<&dyn SponsorCapability>,
) -> Result<CandidateCompleteTransaction, TransactionRefusal> {
    // Stages 1 and 2: validate the request and the public view, and
    // reject duplicate and overlapping outpoints. The request's own
    // constructor already rejected duplicates inside the ASH selection;
    // what is left is the overlap between the two regions, which no
    // single set can rule out.
    let offer = sponsor.map(SponsorCapability::offer);
    let sponsor_inputs: BTreeSet<Outpoint> = offer
        .as_ref()
        .map(|offer| offer.inputs().clone())
        .unwrap_or_default();
    for outpoint in &sponsor_inputs {
        if request.ash().contains(outpoint) {
            return Err(TransactionRefusal::OverlappingOutpoint(*outpoint));
        }
    }

    // Stage 3: sort ASH inputs canonically. The set is already in the
    // ABI's declared order, so this stage is a read rather than a sort,
    // which is the point of building the request out of a set.
    let ash: Vec<Outpoint> = request.ash().iter().copied().collect();
    let sponsors: Vec<Outpoint> = sponsor_inputs.iter().copied().collect();

    // Stage 4: select the exact supported shape.
    let change_wanted = offer
        .as_ref()
        .and_then(crate::sponsor::SponsorOffer::change)
        .is_some_and(|value| !is_known_zero(value));
    let shape_abi = select_shape(abi, ash.len(), sponsors.len(), change_wanted)?;

    // Stage 5: derive the coordinator. Nobody selects it: it is
    // whichever outpoint the canonical order puts at the ABI's fixed
    // index, and that index has to fall inside the family it anchors.
    let coordinator_index = usize::from(abi.coordinator().index());
    if coordinator_index >= ash.len() {
        return Err(TransactionRefusal::CoordinatorNotAtAnchor {
            placed: abi.coordinator().index(),
        });
    }

    // Stages 6 and 7: validate the public object facts and compute the
    // exact successor amount.
    let pinned_program = abi.pin().output_script(target)?;
    let successor_amount =
        validate_public_facts(target, abi, view, &ash, &sponsors, &pinned_program)?;

    // Stages 8, 9 and 10: instantiate the successor, derive the sponsor
    // suffix and the optional change role, and assemble the roles.
    let mut inputs = Vec::with_capacity(ash.len() + sponsors.len());
    for outpoint in ash.iter().chain(sponsors.iter()) {
        inputs.push(TargetInput::new(*outpoint, abi.sequence().sequence()));
    }

    let outputs = assemble_outputs(
        target,
        abi,
        shape_abi,
        successor_amount,
        &pinned_program,
        offer.as_ref(),
        sponsor,
    )?;

    // Stage 11: the outputs are now final. Everything after this point
    // changes witnesses only.
    let witnesses = ash_witnesses(abi, shape_abi, ash.len(), sponsors.len())?;
    let mut transaction = TargetTransaction::new(
        shape_abi.version().version(),
        inputs,
        outputs,
        abi.lock_time(),
        witnesses,
    )?;

    // Stages 12 and 13: issue signing requests and validate what comes
    // back. The request binds the finalized transaction, and the
    // returned signature is compared against those exact bytes.
    if let Some(capability) = sponsor {
        let witnesses = collect_sponsor_signatures(capability, &transaction, &ash, &sponsors)?;
        transaction = TargetTransaction::new(
            transaction.version(),
            transaction.inputs().to_vec(),
            transaction.outputs().to_vec(),
            transaction.lock_time(),
            witnesses,
        )?;
    }

    // Stage 15: the ABI-local preflight.
    let contributed = explicit_sponsor_contribution(abi, view, &sponsors);
    preflight(
        target,
        abi,
        shape_abi,
        &transaction,
        successor_amount,
        contributed,
    )?;

    // Stage 16: exact bytes and a typed report.
    let roles = role_census(shape_abi, ash.len(), sponsors.len(), &transaction);
    let resources = settled_resources(abi, shape_abi, &transaction);

    Ok(CandidateCompleteTransaction {
        report: ConstructionReport {
            shape: shape_abi.shape(),
            form: shape_abi.form(),
            version: shape_abi.version(),
            roles,
            resources,
            successor_amount,
            disclaimers: SyntheticDisclaimer::for_origin(abi.pin().origin()),
        },
        transaction,
    })
}

/// Validate the public facts of stage six, and total stage seven.
///
/// Two loops with one thing in common: every field either of them reads
/// is one the target hands to anybody. The sponsor loop is conspicuously
/// shorter — it reads an asset and a program shape and stops, because
/// the sponsor's value is exactly what §1.6 forbids the protocol
/// relation to depend on.
fn validate_public_facts(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateTransactionAbi,
    view: &PublicConstructionView,
    ash: &[Outpoint],
    sponsors: &[Outpoint],
    pinned_program: &[u8],
) -> Result<u64, TransactionRefusal> {
    let mut successor_amount: u64 = 0;
    for outpoint in ash {
        let public = view
            .get(*outpoint)
            .ok_or(TransactionRefusal::AshInputCarriesForeignProgram(*outpoint))?;
        let AssetField::Explicit(asset) = public.asset() else {
            return Err(TransactionRefusal::AshInputCarriesForeignAsset(*outpoint));
        };
        if asset != abi.symbols().closed_asset() {
            return Err(TransactionRefusal::AshInputCarriesForeignAsset(*outpoint));
        }
        if public.program() != pinned_program {
            return Err(TransactionRefusal::AshInputCarriesForeignProgram(*outpoint));
        }
        let ValueField::Explicit(amount) = public.value() else {
            return Err(TransactionRefusal::AshInputAmountNotExplicit(*outpoint));
        };
        // A checked add: §1.11 asks for checked bounded integers, and a
        // wrapping consolidation would silently mint or destroy the
        // closed asset.
        successor_amount = successor_amount
            .checked_add(amount)
            .ok_or(TransactionRefusal::SuccessorAmountOutOfRange)?;
    }

    for outpoint in sponsors {
        let public =
            view.get(*outpoint)
                .ok_or(TransactionRefusal::SponsorInputCarriesForeignAsset(
                    *outpoint,
                ))?;
        let AssetField::Explicit(asset) = public.asset() else {
            return Err(TransactionRefusal::SponsorInputAssetNotExplicit(*outpoint));
        };
        if asset != abi.symbols().reserve_asset() {
            return Err(TransactionRefusal::SponsorInputCarriesForeignAsset(
                *outpoint,
            ));
        }
        if !admitted_sponsor_program(target, public.program()) {
            return Err(TransactionRefusal::SponsorProgramClassNotAdmitted(
                *outpoint,
            ));
        }
    }

    Ok(successor_amount)
}

/// Issue every signing request and validate what comes back (stages
/// twelve to fourteen).
///
/// The transaction handed in is already finalized, which is what makes
/// this function's existence the enforcement of §15.8's ordering rather
/// than a note about it: there is no way to reach a
/// [`SponsorSigningRequest`] except through bytes that already exist.
fn collect_sponsor_signatures(
    capability: &dyn SponsorCapability,
    transaction: &TargetTransaction,
    ash: &[Outpoint],
    sponsors: &[Outpoint],
) -> Result<Vec<InputWitness>, TransactionRefusal> {
    let finalized = transaction.encode();
    let protected = transaction.outputs().to_vec();
    let mut witnesses = transaction.witnesses().to_vec();

    for (offset, outpoint) in sponsors.iter().enumerate() {
        let index = u16::try_from(ash.len().saturating_add(offset)).unwrap_or(u16::MAX);
        let signing = SponsorSigningRequest::new(
            finalized.clone(),
            index,
            SignerRole::SponsorSuffixMember,
            SighashProfile::AllInputsAllOutputs,
            protected.clone(),
        );
        let signature = capability
            .sign(&signing)
            .ok_or(TransactionRefusal::SponsorSignatureMissing(*outpoint))?;
        if signature.bound_to() != finalized.as_slice() {
            return Err(TransactionRefusal::SponsorSignatureBindingMismatch(
                *outpoint,
            ));
        }
        if signature.stack().len() != WITNESS_V0_KEYHASH_STACK_ITEMS {
            return Err(TransactionRefusal::SponsorWitnessShapeRefused {
                outpoint: *outpoint,
                offered: signature.stack().len(),
                expected: WITNESS_V0_KEYHASH_STACK_ITEMS,
            });
        }
        // The witness is the stack the capability returned, unchanged.
        // This crate does not reorder, pad, or reinterpret a signer's
        // stack.
        let slot = witnesses
            .get_mut(usize::from(index))
            .ok_or(TransactionRefusal::SponsorSignatureMissing(*outpoint))?;
        *slot = InputWitness::new(signature.stack().to_vec());
    }

    Ok(witnesses)
}

/// The sponsors' total explicit contribution, if every one of them is
/// explicit.
///
/// `None` the moment one sponsor's value is a commitment, and `None` is
/// the honest answer rather than a partial sum: a total over some of
/// them would be a number that looks like the sponsors' contribution
/// and is not.
fn explicit_sponsor_contribution(
    abi: &CandidateTransactionAbi,
    view: &PublicConstructionView,
    sponsors: &[Outpoint],
) -> Option<u64> {
    let reserve = AssetField::Explicit(abi.symbols().reserve_asset());
    let mut total: u64 = 0;
    for outpoint in sponsors {
        let public = view.get(*outpoint)?;
        if public.asset() != reserve {
            return None;
        }
        let ValueField::Explicit(amount) = public.value() else {
            return None;
        };
        total = total.checked_add(amount)?;
    }
    Some(total)
}

/// Whether a change value is known to be zero.
///
/// A commitment is never known to be zero here, and that is the erasure
/// law rather than a limitation: comparing a sponsor's value with zero
/// is precisely what §1.6 forbids, so a confidential residual is
/// carried as an output whatever it holds.
const fn is_known_zero(value: ValueField) -> bool {
    matches!(value, ValueField::Explicit(0))
}

/// Whether a program is one the reviewed sponsor profile admits.
fn admitted_sponsor_program(target: &ReviewedElementsTapscriptDefinition, program: &[u8]) -> bool {
    SponsorProgramClass::ALL.iter().any(|class| match class {
        SponsorProgramClass::WitnessV0KeyHash => {
            program.len() == WITNESS_V0_KEYHASH_PROGRAM_BYTES + 2
                && witness_program_script(target, 0, &program[2..])
                    .is_ok_and(|expected| expected == program)
        }
        _ => false,
    })
}

/// Select the one admitted shape these counts name.
fn select_shape(
    abi: &CandidateTransactionAbi,
    ash_inputs: usize,
    sponsor_inputs: usize,
    sponsor_change: bool,
) -> Result<&ShapeAbi, TransactionRefusal> {
    if sponsor_change && sponsor_inputs == 0 {
        return Err(TransactionRefusal::SponsorChangeWithoutSponsor);
    }
    abi.shapes()
        .values()
        .find(|candidate| {
            let shape = candidate.shape();
            usize::from(shape.ash_inputs()) == ash_inputs
                && usize::from(shape.sponsor_inputs()) == sponsor_inputs
                && candidate.sponsor_change_position().is_some() == sponsor_change
        })
        .ok_or(TransactionRefusal::UnsupportedShape {
            ash_inputs,
            sponsor_inputs,
            sponsor_change,
        })
}

/// Assemble the output roles in the layout's own position order.
fn assemble_outputs(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateTransactionAbi,
    shape: &ShapeAbi,
    successor_amount: u64,
    pinned_program: &[u8],
    offer: Option<&crate::sponsor::SponsorOffer>,
    sponsor: Option<&dyn SponsorCapability>,
) -> Result<Vec<TargetOutput>, TransactionRefusal> {
    let mut placed: BTreeMap<u16, TargetOutput> = BTreeMap::new();

    placed.insert(
        shape.successor_position(),
        TargetOutput::new(
            AssetField::Explicit(abi.symbols().closed_asset()),
            ValueField::Explicit(successor_amount),
            NonceField::Null,
            pinned_program.to_vec(),
        ),
    );

    if let (Some(position), Some(offer)) = (shape.sponsor_change_position(), offer) {
        let value = offer
            .change()
            .ok_or(TransactionRefusal::SponsorChangeWithoutSponsor)?;
        let destination = sponsor.and_then(SponsorCapability::change_destination);
        let (version, payload) = destination.unwrap_or_else(|| {
            (
                abi.symbols().sponsor_change_version(),
                abi.symbols().sponsor_change_program().to_vec(),
            )
        });
        if version != abi.symbols().sponsor_change_version()
            || payload != abi.symbols().sponsor_change_program()
        {
            // The emitted coordinator compares the change output's
            // program against a deployment constant, so a destination
            // that differs is not a sponsor preference this builder can
            // honour — it is a transaction the family would reject.
            return Err(TransactionRefusal::MalformedDeploymentSymbol {
                symbol: "sponsor change program",
            });
        }
        placed.insert(
            position,
            TargetOutput::new(
                AssetField::Explicit(abi.symbols().reserve_asset()),
                value,
                NonceField::Null,
                witness_program_script(target, version, &payload)?,
            ),
        );
    }

    if let (Some(position), Some(offer)) = (shape.fee_position(), offer) {
        placed.insert(
            position,
            TargetOutput::new(
                AssetField::Explicit(abi.symbols().reserve_asset()),
                ValueField::Explicit(offer.fee()),
                NonceField::Null,
                // The empty program: the fee role's whole identity is
                // target-structural, and this is the structure.
                Vec::new(),
            ),
        );
    }

    let expected: BTreeSet<OutputRole> = shape
        .layout()
        .outputs()
        .iter()
        .map(|placement| placement.role())
        .collect();
    let built: BTreeSet<OutputRole> = shape
        .layout()
        .outputs()
        .iter()
        .filter(|placement| placed.contains_key(&placement.position()))
        .map(|placement| placement.role())
        .collect();
    if expected != built {
        return Err(TransactionRefusal::LayoutCensusMismatch {
            shape: shape.shape(),
            missing: expected.difference(&built).copied().collect(),
        });
    }

    Ok(placed.into_values().collect())
}

/// The script-path witnesses of the ASH family, and empty sponsor ones.
fn ash_witnesses(
    abi: &CandidateTransactionAbi,
    shape: &ShapeAbi,
    ash_inputs: usize,
    sponsor_inputs: usize,
) -> Result<Vec<InputWitness>, TransactionRefusal> {
    let mut witnesses = Vec::with_capacity(ash_inputs + sponsor_inputs);
    for position in 0..ash_inputs {
        let role = if position == usize::from(abi.coordinator().index()) {
            linker::backend::InputRole::Coordinator
        } else {
            linker::backend::InputRole::Member
        };
        let leaf =
            *shape
                .tapleaf()
                .get(&role)
                .ok_or_else(|| TransactionRefusal::MissingInputRole {
                    shape: shape.shape(),
                    role,
                })?;
        let script = abi
            .tree()
            .leaf_programs()
            .get(&leaf)
            .cloned()
            .ok_or(TransactionRefusal::MissingLeaf(leaf))?;
        let control = abi.tree().control_block(leaf, abi.pin())?;

        // The ABI's declared witness order, read rather than restated.
        let mut stack = Vec::with_capacity(abi.witness_order().len());
        for item in abi.witness_order() {
            stack.push(match item {
                WitnessItem::LeafScript => script.clone(),
                WitnessItem::ControlBlock => control.clone(),
            });
        }
        witnesses.push(InputWitness::new(stack));
    }
    witnesses.resize(ash_inputs + sponsor_inputs, InputWitness::default());
    Ok(witnesses)
}

/// The public role census of one assembled transaction.
fn role_census(
    shape: &ShapeAbi,
    ash_inputs: usize,
    sponsor_inputs: usize,
    transaction: &TargetTransaction,
) -> RoleCensus {
    let ash = (0..ash_inputs)
        .map(|position| u16::try_from(position).unwrap_or(u16::MAX))
        .collect();
    let sponsor = (ash_inputs..ash_inputs.saturating_add(sponsor_inputs))
        .map(|position| u16::try_from(position).unwrap_or(u16::MAX))
        .collect();
    let mut outputs = BTreeMap::new();
    for placement in shape.layout().outputs() {
        if usize::from(placement.position()) < transaction.outputs().len() {
            outputs.insert(placement.position(), placement.role());
        }
    }
    RoleCensus {
        ash_inputs: ash,
        sponsor_inputs: sponsor,
        outputs,
    }
}

/// The three whole-transaction dimensions this wave settles.
fn settled_resources(
    abi: &CandidateTransactionAbi,
    shape: &ShapeAbi,
    transaction: &TargetTransaction,
) -> SettledResources {
    let limits = abi.package_limits();
    // A sponsored transaction travels alone, so the package ceiling it
    // is held to is the child one only when it is the child. The
    // sponsorless form is the parent of the topology-restricted package
    // by construction, which is why it takes the parent ceiling.
    let package_virtual_size = match shape.form() {
        TransactionForm::Sponsorless => limits.parent_virtual_size(),
        _ => limits.child_virtual_size(),
    };
    SettledResources {
        witness_bytes: transaction.witness_bytes(),
        weight: transaction.weight(),
        virtual_size: transaction.virtual_size(),
        package_transactions: limits.transactions(),
        package_virtual_size,
    }
}

/// The ABI-local preflight (§15.8 stage 15).
///
/// Conservation is checked only where every field of an asset is
/// explicit. A sponsor whose value stays committed is not compared with
/// anything, which is the erasure law holding rather than a check being
/// skipped.
fn preflight(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateTransactionAbi,
    shape: &ShapeAbi,
    transaction: &TargetTransaction,
    successor_amount: u64,
    contributed: Option<u64>,
) -> Result<(), TransactionRefusal> {
    let closed: AssetId = abi.symbols().closed_asset();
    let mut paid_out: u64 = 0;
    for output in transaction.outputs() {
        if output.asset() == AssetField::Explicit(closed) {
            let ValueField::Explicit(amount) = output.value() else {
                return Err(TransactionRefusal::ConservationFailed {
                    asset: "closed protocol asset".to_owned(),
                });
            };
            paid_out = paid_out
                .checked_add(amount)
                .ok_or(TransactionRefusal::SuccessorAmountOutOfRange)?;
        }
    }
    if paid_out != successor_amount {
        return Err(TransactionRefusal::ConservationFailed {
            asset: "closed protocol asset".to_owned(),
        });
    }

    if let Some(position) = shape.fee_position() {
        let fee = transaction
            .outputs()
            .get(usize::from(position))
            .ok_or_else(|| TransactionRefusal::LayoutCensusMismatch {
                shape: shape.shape(),
                missing: BTreeSet::from([OutputRole::TargetFee]),
            })?;
        if !fee.is_fee() {
            return Err(TransactionRefusal::LayoutCensusMismatch {
                shape: shape.shape(),
                missing: BTreeSet::from([OutputRole::TargetFee]),
            });
        }
    }

    check_reserve_cover(abi, transaction, contributed)?;
    check_weight(target, transaction)
}

/// Require the sponsor's explicit contribution to cover what it spends.
///
/// A construction check, not a protocol one, and the distinction is the
/// whole reason it lives here rather than in a program. §1.6 forbids the
/// protocol *relation* from depending on a sponsor's amount; it says in
/// as many words that the constructor may use sponsor-private state to
/// build a balanced transaction, which is exactly this.
///
/// The check runs only where every reserve-asset field in play is
/// explicit. One committed sponsor value and it does not run at all —
/// not as a concession, but because the alternative is opening the
/// commitment, and there is no branch here that could.
fn check_reserve_cover(
    abi: &CandidateTransactionAbi,
    transaction: &TargetTransaction,
    contributed: Option<u64>,
) -> Result<(), TransactionRefusal> {
    let Some(contributed) = contributed else {
        return Ok(());
    };
    let reserve = AssetField::Explicit(abi.symbols().reserve_asset());
    let mut spent: u64 = 0;
    for output in transaction.outputs() {
        if output.asset() != reserve {
            continue;
        }
        let ValueField::Explicit(amount) = output.value() else {
            return Ok(());
        };
        spent = spent
            .checked_add(amount)
            .ok_or(TransactionRefusal::SponsorValueDoesNotCoverFee)?;
    }
    if spent > contributed {
        return Err(TransactionRefusal::SponsorValueDoesNotCoverFee);
    }
    Ok(())
}

/// Require the assembled transaction to fit the reviewed weight bound.
///
/// The one whole-transaction dimension the reviewed contract bounds by
/// consensus. Checking it turns a transaction no node could include
/// into a construction refusal, which is §1.5's distinction applied in
/// the direction that is actually useful: the builder knows before it
/// asks anybody to sign.
///
/// The *consensus* bound and not the policy one. Exceeding the policy
/// bound makes a transaction non-standard, which is a deployment fact
/// and a relay outcome; refusing to construct it here would be the
/// backend deciding a relay question, which is the confusion the
/// two-verdict rule exists to prevent.
///
/// Public because the evidence packages need the same preflight, and
/// because a check nobody outside this module can reach is a check
/// whose failing branch could only be reasoned about.
///
/// # Errors
///
/// [`TransactionRefusal::ResourceBoundExceeded`] when the transaction's
/// weight is above the reviewed consensus maximum.
pub fn check_weight(
    target: &ReviewedElementsTapscriptDefinition,
    transaction: &TargetTransaction,
) -> Result<(), TransactionRefusal> {
    let bound = target
        .definition()
        .resources()
        .consensus()
        .bounds()
        .get(&ResourceDimension::TransactionWeight)
        .and_then(|bound| bound.maximum());
    let Some(bound) = bound else {
        return Ok(());
    };
    let reached = transaction.weight();
    if reached > bound {
        return Err(TransactionRefusal::ResourceBoundExceeded {
            dimension: ResourceDimension::TransactionWeight,
            reached,
            bound,
        });
    }
    Ok(())
}
