//! First-party negative evidence for the §15.4–§15.7 fault rows.
//!
//! [`crate::live_first_party`] discharges §15.3's owner and signature
//! faults, whose boundary is one entry point. The rest of the matrix's
//! pre-target rows are spread across six more: the owner-key encoding
//! closure, the static constructor derivation, the linker's symbol
//! census, the typed protocol value, the live-transfer finalization, and
//! the offered-transaction check. This module meets §4.2 for those.
//!
//! # The same argument as §15.3's, made six more times
//!
//! §4.2 discharges a row only with a canonical malformed typed input, the
//! exact owning validator, a typed refusal naming the intended class, a
//! focused positive control, a focused negative test, and a validated
//! report. The control is half of it: a validator that refuses everything
//! refuses a malformed input too, so [`validate_live_fault`] runs the
//! owning entry point twice — once honest, once with the case's one
//! stated change — and concludes nothing if the control did not pass.
//!
//! # Every row this census owes is discharged, and three stopped being
//! owed
//!
//! Three rows stood outstanding before this wave, and none of them was
//! closed by lowering a bar. §15.4's `wrong-constructor-schema` named
//! the linker, and the leaf schema is the constructor derivation's: it
//! is discharged below, at the boundary it actually has. §15.5's
//! `amount-outside-semantic-domain` was recorded as a missing
//! request-path validator and is not one — the owner ruled the ceiling
//! blockchain-enforced, the same class as conservation — so its verdict
//! is the target's and no first-party discharge is owed of it. §15.4's
//! `mixed-operation-program` asked for an input the operation vocabulary
//! admits no value of, which
//! [`crate::live_evidence::LiveRowStanding::OperationVocabularyClosed`]
//! records rather than counting as an unanswered refusal.
//!
//! # Nothing is silently outstanding, and no list is trusted to say so
//!
//! §4.2's alternative — report a requirement whose policy cannot be met
//! rather than leave it outstanding — used to be carried by a list of
//! rows and obstacles here. The list is empty now, and it is gone rather
//! than kept empty: a vocabulary nobody carries is one a reader has to
//! check is unused. The guarantee it stood for is enforced without it
//! and unconditionally, in `crate::live_evidence`'s classification: a
//! first-party row this census does not stage is filed under
//! [`crate::live_evidence::FirstPartyGap::NoStagedCase`], which is
//! counted, rendered, and stops a report calling itself complete. A row
//! added to §15 tomorrow is outstanding and visible the day it is added
//! rather than the day somebody remembers a list.
//!
//! # Every secret here is published
//!
//! The owners are [`crate::live_plan`]'s BIP-340 appendix scalars,
//! admitted under ADR-015's test-material rule
//! `(´[ADR015-rule:security:test-material]´)` and Guide-13 §1.10. The
//! signatures are opaque bytes that authorize nothing, for the reason
//! [`crate::live_first_party`] states: none of these refusals reads one.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
use linker::{
    LinkRefusal, LiveDefinitionCensus, LiveDefinitionOrigin, LiveLinkSymbol, LiveSymbolValue,
    OwnerParameter, collect_live_definitions,
};
use tapscript::{
    LiveConstructorRefusal, LiveTransferLeafRole, OwnerKey, OwnerKeyRejection,
    demonstration_live_shape_set, derive_live_receipt_constructor, owner_key_encoding_closure,
    static_transfer_leaf_set,
};
use target_elements::EncodingClass;
use transaction::bytes::{AssetField, AssetId, Outpoint, TargetTransaction, Txid, ValueField};
use transaction::error::TransactionRefusal;
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_construct::finalize_live_transfer;
use transaction::live_private::PrivateValueCapability;
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SponsorChangeRequest,
};
use transaction::live_signing::{LiveOwnerResponse, authorize_live_transfer};
use transaction::sponsor::{
    SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest,
};
use transaction::taproot::{TAPROOT_WITNESS_VERSION, witness_program_script};
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::bundle::PINNED_PROGRAM;
use crate::error::VectorError;
use crate::live_capability::OracleFixtureValues;
use crate::live_plan::{
    FEE_PROGRAM_DIGEST, FIRST_SCALAR, PROTOCOL_ASSET, RESERVE_ASSET, SECOND_SCALAR,
    demonstration_live_abi, live_deployment_for_asset, live_transfer_plan, owner_key,
    published_owner, relocatable_live_bundles, reviewed_target,
};
use crate::live_safety::{LiveSafetyRow, required_safety_matrix};

/// Which first-party validator owns one §15.4–§15.7 row.
///
/// A name for the exact entry point a discharge drove. There is
/// deliberately no arm naming a *crate*: the linker refuses many things
/// and a row answered by one of them is not answered by the others.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LiveFaultValidator {
    /// `tapscript::OwnerKey::new`, which authenticates an offered owner
    /// key against the reviewed contract's approved encoding closure.
    OwnerKeyEncoding,
    /// `tapscript::derive_live_receipt_constructor`, the sole site that
    /// validates a static constructor's admissible leaf schema.
    ///
    /// The boundary §15.4's `wrong-constructor-schema` row actually has.
    /// It sits before backend emission and before linking, and the
    /// linker cannot substitute for it: by the time a bundle exists, the
    /// schema has already been checked and the sealed types admit no
    /// unchecked one.
    ConstructorDerivation,
    /// `linker::LiveDefinitionCensus::define`, the sole site that admits
    /// a symbol definition into a link.
    LiveSymbolDefinition,
    /// `transaction::live_request::ProtocolValue::new`, which is where a
    /// semantic amount becomes a value a request may carry.
    ProtocolValueDomain,
    /// `transaction::live_construct::finalize_live_transfer`, which
    /// settles §12.6's ten items and refuses at whichever of its ten
    /// stages the fault reaches.
    LiveTransferFinalization,
    /// `FinalizedLiveTransfer::check_offered`, which compares an offered
    /// transaction against the finalized one.
    OfferedTransactionCheck,
}

impl LiveFaultValidator {
    /// The validator's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::OwnerKeyEncoding => "owner-key-encoding",
            Self::ConstructorDerivation => "constructor-derivation",
            Self::LiveSymbolDefinition => "live-symbol-definition",
            Self::ProtocolValueDomain => "protocol-value-domain",
            Self::LiveTransferFinalization => "live-transfer-finalization",
            Self::OfferedTransactionCheck => "offered-transaction-check",
        }
    }
}

/// The one focused change a case makes to the honest input.
///
/// Each arm is a single disagreement with what the honest call was given,
/// which is what makes the refusal attributable: an input differing in
/// two places would be refused for whichever the validator reached first.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum FaultMutation {
    /// Offer a key encoding whose domain is not the key domain.
    OfferAKeyEncodingOutsideTheKeyDomain,
    /// Place a leaf of the other representation in the leaf schema.
    PlaceALeafOfTheOtherRepresentationInTheSchema,
    /// Remove an admitted shape's coordinator leaf from the schema.
    RemoveAnAdmittedShapesCoordinatorLeaf,
    /// Add a leaf no admitted shape reaches to the schema.
    AddALeafNoAdmittedShapeReaches,
    /// Define the protocol-asset symbol with a value of another kind.
    DefineTheAssetSymbolWithANonAssetValue,
    /// Define one symbol twice in one census.
    DefineOneSymbolTwice,
    /// Ask for a destination worth nothing.
    AskForAZeroValuedDestination,
    /// Name a destination owner nothing was linked for.
    NameADestinationOwnerNothingWasLinkedFor,
    /// Offer a receipt input sitting under a taptree nothing was linked
    /// for.
    ///
    /// The input-side twin of
    /// [`Self::NameADestinationOwnerNothingWasLinkedFor`], and the one
    /// change is the same kind of change: the coin's own program. A
    /// receipt CLASS is a leaf set and therefore a tree, so a receipt of
    /// another class sits under another program — which is the only
    /// thing about a class that is visible at a spend, on this side of
    /// the target and on the target's.
    OfferAReceiptInputUnderAProgramNothingWasLinkedFor,
    /// Offer a receipt input paying to an honest ASH program.
    ///
    /// Not a corrupted program but a REAL one of another family: the
    /// pinned ASH bundle's own taproot output key, wrapped in the
    /// reviewed witness-program script. That is what makes the refusal
    /// about the family rather than about malformed bytes.
    OfferAnAshProgramAsAReceiptInput,
    /// Offer owner metadata the approved closure does not fix a width
    /// for.
    ///
    /// One byte short of the approved width, with the encoding class
    /// left honest, so the refusal is the width's and not the domain's
    /// — the sibling change `unknown-key-type` already drives.
    OfferOwnerMetadataOutsideTheApprovedWidth,
    /// Offer the reserve asset under an honestly linked receipt program.
    ///
    /// The program, the value form and the outpoint all stay honest and
    /// ONLY the asset field moves, which is what puts the refusal at the
    /// asset check rather than at the program lookup after it.
    OfferTheReserveAssetUnderAReceiptShapedProgram,
    /// Make the destination total exceed the target's explicit width.
    OverflowTheDestinationTotal,
    /// Offer a commitment-valued receipt to the explicit plan.
    OfferACommitmentValuedReceiptToTheExplicitPlan,
    /// Offer a receipt census that is not homogeneous.
    OfferAHeterogeneousReceiptCensus,
    /// Offer an empty sponsor capability for a sponsored request.
    OfferAnEmptySponsorOfferForASponsoredRequest,
    /// Offer the fee role's own program as the sponsor-change
    /// destination.
    OfferTheFeeRoleProgramAsTheSponsorChangeDestination,
    /// Reorder the destination outputs after signing.
    ReorderTheDestinationOutputsAfterSigning,
    /// Offer the private plan's constructor program to an explicit
    /// request.
    OfferThePrivateProgramToTheExplicitPlan,
    /// Offer the explicit plan's constructor program to a private
    /// request.
    OfferTheExplicitProgramToThePrivatePlan,
}

/// The refusal one owning validator returned.
///
/// Four vocabularies, because the rows are spread across three crates
/// and tapscript owns two of them: the owner-key encoding closure and
/// the constructor derivation refuse different things and say so in
/// different words. Collapsing them into one enum of this crate's would
/// mean re-spelling somebody else's refusal, and a discharge would then
/// be evidence about the re-spelling.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ObservedFaultRefusal {
    /// The owner-key encoding closure refused.
    OwnerKey(OwnerKeyRejection),
    /// The static constructor derivation refused.
    Constructor(LiveConstructorRefusal),
    /// The linker refused.
    Link(LinkRefusal),
    /// The transaction layer refused.
    Transaction(TransactionRefusal),
}

/// One first-party case: a §15 row, an owning validator, and one change.
#[derive(Clone, Debug)]
pub struct LiveFaultCase {
    row: &'static str,
    validator: LiveFaultValidator,
    mutation: FaultMutation,
    expected: fn(&ObservedFaultRefusal) -> bool,
    expected_name: &'static str,
}

impl PartialEq for LiveFaultCase {
    /// Compared by what a reader can check, never by function address.
    fn eq(&self, other: &Self) -> bool {
        self.row == other.row
            && self.validator == other.validator
            && self.mutation == other.mutation
            && self.expected_name == other.expected_name
    }
}

impl Eq for LiveFaultCase {}

impl LiveFaultCase {
    /// The §15 row this case answers.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// Which entry point owns the refusal.
    #[must_use]
    pub const fn validator(&self) -> LiveFaultValidator {
        self.validator
    }

    /// The one change the case makes.
    #[must_use]
    pub const fn mutation(&self) -> FaultMutation {
        self.mutation
    }

    /// The refusal class the case expects, by name.
    #[must_use]
    pub const fn expected_class(&self) -> &'static str {
        self.expected_name
    }
}

/// Why one offered case does not discharge its row.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveFaultRefusal {
    /// §15 names no such row.
    RowNotInTheMatrix(&'static str),
    /// The row's boundary is the target's.
    RowIsNotFirstParty(&'static str),
    /// The live-transfer substrate could not be built.
    SubstrateUnavailable,
    /// The honest input could not be assembled, so there is nothing to
    /// malform.
    ControlNotConstructible,
    /// The validator refused the honest input too.
    ///
    /// Then the refusal is not attributable to the mutation: the input
    /// was unacceptable before it was malformed.
    ControlWasRefused(Box<ObservedFaultRefusal>),
    /// The validator accepted the malformed input.
    MalformedInputWasAccepted,
    /// The validator refused, naming a class other than the row's.
    RefusalNamesAnotherClass(Box<ObservedFaultRefusal>),
}

impl From<VectorError> for LiveFaultRefusal {
    fn from(_: VectorError) -> Self {
        Self::SubstrateUnavailable
    }
}

/// One §15.4–§15.7 row, discharged.
///
/// # What holding one of these establishes
///
/// That the exact entry point the case names was run twice — once on an
/// honest input and once on that input with one stated change — that it
/// accepted the first and refused the second naming the row's own class.
/// All of it is recomputed by [`validate_live_fault`], which is the only
/// constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedLiveFaultEvidence {
    row: &'static str,
    validator: LiveFaultValidator,
    mutation: FaultMutation,
    observed: ObservedFaultRefusal,
}

impl ValidatedLiveFaultEvidence {
    /// The §15 row this evidence answers.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// Which entry point refused.
    #[must_use]
    pub const fn validator(&self) -> LiveFaultValidator {
        self.validator
    }

    /// The one change the discharge made.
    #[must_use]
    pub const fn mutation(&self) -> FaultMutation {
        self.mutation
    }

    /// The refusal the validator actually returned.
    #[must_use]
    pub const fn observed(&self) -> &ObservedFaultRefusal {
        &self.observed
    }
}

/// One case constructor, spelled once.
const fn case(
    row: &'static str,
    validator: LiveFaultValidator,
    mutation: FaultMutation,
    expected: fn(&ObservedFaultRefusal) -> bool,
    expected_name: &'static str,
) -> LiveFaultCase {
    LiveFaultCase {
        row,
        validator,
        mutation,
        expected,
        expected_name,
    }
}

/// Whether an observed refusal is one linker variant.
macro_rules! link_is {
    ($pattern:pat) => {
        |observed| matches!(observed, ObservedFaultRefusal::Link($pattern))
    };
}

/// Whether an observed refusal is one transaction variant.
macro_rules! transaction_is {
    ($pattern:pat) => {
        |observed| matches!(observed, ObservedFaultRefusal::Transaction($pattern))
    };
}

/// Whether an observed refusal is one constructor variant.
macro_rules! constructor_is {
    ($pattern:pat) => {
        |observed| matches!(observed, ObservedFaultRefusal::Constructor($pattern))
    };
}

/// The complete census of first-party cases for §15.4–§15.7.
///
/// Fifteen cases over six owning entry points, and no §15.4–§15.7 row
/// whose verdict a first-party layer owns is missing from it.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "one entry per §15 row, each with the reason it is filed at \
              that validator; splitting the census would put the matrix's \
              rows in two lists and let one of them be forgotten"
)]
pub fn live_fault_cases() -> Vec<LiveFaultCase> {
    use FaultMutation as M;
    use LiveFaultValidator as V;

    vec![
        // §15.3's last first-party row. §1.8 closes unknown-key forward
        // compatibility, and the constructor authenticates the offered
        // encoding against the approved closure before a constructor
        // exists to place the key in.
        case(
            "unknown-key-type",
            V::OwnerKeyEncoding,
            M::OfferAKeyEncodingOutsideTheKeyDomain,
            |observed| {
                matches!(
                    observed,
                    ObservedFaultRefusal::OwnerKey(OwnerKeyRejection::NotAKeyEncoding { .. })
                )
            },
            "NotAKeyEncoding",
        ),
        // §15.4's constructor-schema row, at the boundary the erratum
        // moved it to. One canonical malformed leaf set, as §4.2 asks
        // for; the other two malformations the schema admits are driven
        // by this module's own focused test through the same staging.
        case(
            "wrong-constructor-schema",
            V::ConstructorDerivation,
            M::PlaceALeafOfTheOtherRepresentationInTheSchema,
            constructor_is!(LiveConstructorRefusal::LeafOfAnotherRepresentation { .. }),
            "LeafOfAnotherRepresentation",
        ),
        // §15.4's constructor and object rows the ABI owns.
        //
        // The input row is here because it was RETYPED, and the retyping
        // is a correction of a boundary rather than a lowering of a bar.
        // It was typed target-side, asking that a
        // time-locked predecessor offered to a live leaf be refused by
        // the target's own introspection, and no such introspection
        // exists: the live class is carried by CONSTRUCTOR TYPING, which
        // emits zero instructions, no live fragment introspects an
        // input's program, and the two receipt classes carry the same
        // asset — so a live leaf that ran would compare nothing that
        // separates them.
        //
        // What separates them is structural, at two sites, and the row
        // is discharged against the first of them because the second can
        // never be observed as being about the lock.
        //
        // The first is the linked table this case drives. A class cannot
        // be constructed into a live transfer at all:
        // `compiler::live_transfer_plan::derive_class` admits only
        // `ObjectId::ReceiptLive` as the protocol object on both sides
        // and refuses any analyzed program that names another, so no
        // time-locked constructor can enter the destination table the
        // input recognition searches, and no request can name one — the
        // request states outpoints, and the class of a spent coin is its
        // program.
        //
        // The second is the leaf commitment, on a chain: a leaf runs
        // only from a taptree the spent program commits to, so a
        // candidate revealing a live-transfer leaf against a coin of
        // another class fails `VerifyTaprootCommitment` before a single
        // opcode executes (the pinned target source, at
        // `src/script/interpreter.cpp:3286-3290`, where a failed
        // commitment is `SCRIPT_ERR_WITNESS_PROGRAM_MISMATCH`). THAT
        // REFUSAL IS PROGRAM-GENERIC AND CAN NEVER NAME THE LOCK: what
        // it attributes to is a leaf the spent program does not commit
        // to, which is true of every foreign taptree, so an observation
        // of it says the commitment rule holds and says nothing about
        // receipt classes. No later wave is owed that observation, and
        // producing it would not answer this row.
        //
        // The maturity reading the old typing rested on is refused by
        // §10.4 besides: the time-locked class is a receipt CLASS whose
        // maturity is committed cycle arithmetic, not a consensus
        // timelock, so there is no "after maturity" control to accept
        // beside a refusal and no attributable pair was ever available.
        //
        // What the case below observes is therefore the same fact its
        // sibling observes, from the consumed side: the recognition
        // admits exactly the programs the linked live constructors emit
        // and refuses everything else, naming the class it required. It
        // does not single out the time-locked class, and it does not
        // claim to — no time-locked program exists to offer, which is
        // the structural protection rather than a gap in the evidence.
        case(
            "time-locked-input",
            V::LiveTransferFinalization,
            M::OfferAReceiptInputUnderAProgramNothingWasLinkedFor,
            transaction_is!(TransactionRefusal::ReceiptInputIsNotALiveReceipt(_)),
            "ReceiptInputIsNotALiveReceipt",
        ),
        // §15.4's three retyped program-generic rows. Each is discharged
        // by its own validator driven twice, with its own mutant and its
        // own declared field.
        //
        // Two of them draw the same refusal class as `time-locked-input`
        // above, and that is the established discipline rather than a
        // collision: the recognition has ONE answer for an input it
        // cannot find in the linked table, so the LAYER cannot separate
        // these rows and the FIELD does. Each case below changes exactly
        // one field and a different one — a whole program of another
        // family here, the asset alone below — so a reader can tell
        // which fact each discharge established.
        case(
            "ash-input-or-output",
            V::LiveTransferFinalization,
            M::OfferAnAshProgramAsAReceiptInput,
            transaction_is!(TransactionRefusal::ReceiptInputIsNotALiveReceipt(_)),
            "ReceiptInputIsNotALiveReceipt",
        ),
        // The one that is not program-generic in any form, and the only
        // one of the three answered before a program exists at all.
        case(
            "malformed-live-metadata",
            V::OwnerKeyEncoding,
            M::OfferOwnerMetadataOutsideTheApprovedWidth,
            |observed| {
                matches!(
                    observed,
                    ObservedFaultRefusal::OwnerKey(OwnerKeyRejection::WrongWidth { .. })
                )
            },
            "WrongWidth",
        ),
        // The asset check runs BEFORE the program lookup, so this row's
        // refusal names the asset and separates from the two above by
        // class as well as by field.
        case(
            "foreign-asset-under-receipt-shaped-program",
            V::LiveTransferFinalization,
            M::OfferTheReserveAssetUnderAReceiptShapedProgram,
            transaction_is!(TransactionRefusal::ReceiptInputCarriesForeignAsset(_)),
            "ReceiptInputCarriesForeignAsset",
        ),
        case(
            "time-locked-output",
            V::LiveTransferFinalization,
            M::NameADestinationOwnerNothingWasLinkedFor,
            transaction_is!(TransactionRefusal::DestinationOwnerHasNoConstructor { .. }),
            "DestinationOwnerHasNoConstructor",
        ),
        // §15.5's value and representation rows.
        case(
            "zero-receipt-output",
            V::ProtocolValueDomain,
            M::AskForAZeroValuedDestination,
            transaction_is!(TransactionRefusal::DestinationValueIsZero),
            "DestinationValueIsZero",
        ),
        case(
            "explicit-arithmetic-overflow",
            V::LiveTransferFinalization,
            M::OverflowTheDestinationTotal,
            transaction_is!(TransactionRefusal::DestinationTotalOutOfRange),
            "DestinationTotalOutOfRange",
        ),
        case(
            "representation-mismatch",
            V::LiveTransferFinalization,
            M::OfferACommitmentValuedReceiptToTheExplicitPlan,
            transaction_is!(TransactionRefusal::ReceiptInputValueFormRefused(_)),
            "ReceiptInputValueFormRefused",
        ),
        case(
            "mixed-representation-under-homogeneous-only-abi",
            V::LiveTransferFinalization,
            M::OfferAHeterogeneousReceiptCensus,
            transaction_is!(TransactionRefusal::ReceiptInputValueFormRefused(_)),
            "ReceiptInputValueFormRefused",
        ),
        // §15.6's two construction-time sponsor rows.
        case(
            "empty-sponsor-offer-for-a-sponsored-request",
            V::LiveTransferFinalization,
            M::OfferAnEmptySponsorOfferForASponsoredRequest,
            transaction_is!(TransactionRefusal::EmptyLiveSponsorOffer),
            "EmptyLiveSponsorOffer",
        ),
        case(
            "fee-change-substitution",
            V::LiveTransferFinalization,
            M::OfferTheFeeRoleProgramAsTheSponsorChangeDestination,
            transaction_is!(TransactionRefusal::MalformedLiveDeploymentSymbol { .. }),
            "MalformedLiveDeploymentSymbol",
        ),
        // §15.7's linker and ABI rows.
        case(
            "unresolved-or-duplicate-relocation",
            V::LiveSymbolDefinition,
            M::DefineOneSymbolTwice,
            link_is!(LinkRefusal::DuplicateLiveSymbolDefinition(_)),
            "DuplicateLiveSymbolDefinition",
        ),
        case(
            "wrong-u-or-owner-schema-relocation",
            V::LiveSymbolDefinition,
            M::DefineTheAssetSymbolWithANonAssetValue,
            link_is!(LinkRefusal::IncompatibleLiveSymbolType { .. }),
            "IncompatibleLiveSymbolType",
        ),
        case(
            "destination-order-changed-after-signing",
            V::OfferedTransactionCheck,
            M::ReorderTheDestinationOutputsAfterSigning,
            transaction_is!(TransactionRefusal::OutputMutatedAfterSigning { .. }),
            "OutputMutatedAfterSigning",
        ),
        case(
            "explicit-plan-paired-with-private-abi",
            V::LiveTransferFinalization,
            M::OfferThePrivateProgramToTheExplicitPlan,
            transaction_is!(TransactionRefusal::ReceiptInputIsNotALiveReceipt(_)),
            "ReceiptInputIsNotALiveReceipt",
        ),
        case(
            "private-plan-paired-with-explicit-program",
            V::LiveTransferFinalization,
            M::OfferTheExplicitProgramToThePrivatePlan,
            transaction_is!(TransactionRefusal::ReceiptInputIsNotALiveReceipt(_)),
            "ReceiptInputIsNotALiveReceipt",
        ),
    ]
}

/// The §15 row one case names, if the matrix has it.
fn matrix_row(name: &str) -> Option<&'static LiveSafetyRow> {
    required_safety_matrix()
        .into_iter()
        .find(|row| row.name() == name)
}

/// A published randomness for the private half of two cases.
const FAULT_RANDOMNESS: [u8; 32] = [0x64; 32];

/// A fixed opaque signature.
///
/// Opaque bytes, and deliberately not a signature anybody took: the
/// offered-transaction check compares censuses and reads no signature.
const OPAQUE_SIGNATURE: [u8; 64] = [0x5c; 64];

/// The outpoint of `index` of a transaction whose identifier is `byte`
/// repeated.
fn outpoint(byte: u8, index: u32) -> Result<Outpoint, VectorError> {
    Outpoint::new(Txid::from_internal([byte; 32]), index)
        .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// One representation's linked destination program for one owner.
fn program(
    abi: &CandidateLiveTransferAbi,
    scalar: &[u8; 32],
    representation: LiveTransferRepresentationPlan,
) -> Result<Vec<u8>, VectorError> {
    Ok(abi
        .destinations()
        .get(
            &OwnerParameter::new(published_owner(scalar)?),
            representation,
        )
        .ok_or(VectorError::LiveSubstrateUnavailable)?
        .instance()
        .program()
        .to_vec())
}

/// A confidential value field for one amount at one position.
fn commitment(
    abi: &CandidateLiveTransferAbi,
    amount: u64,
    position: u16,
) -> Result<ValueField, VectorError> {
    let value = ProtocolValue::new(amount).map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    Ok(ValueField::Commitment(
        OracleFixtureValues
            .value_commitment(
                abi.symbols().protocol_asset(),
                value,
                &PublicTestRandomness::from_published_bytes(FAULT_RANDOMNESS),
                position,
            )
            .ok_or(VectorError::LiveSubstrateUnavailable)?,
    ))
}

/// One receipt view at one outpoint.
const fn view_of(
    abi: &CandidateLiveTransferAbi,
    point: Outpoint,
    program_bytes: Vec<u8>,
    value: ValueField,
) -> PublicOutputView {
    PublicOutputView::new(
        point,
        AssetField::Explicit(abi.symbols().protocol_asset()),
        value,
        program_bytes,
    )
}

/// One destination for a published owner at `amount`.
fn destination(scalar: &[u8; 32], amount: u64) -> Result<LiveReceiptDestination, VectorError> {
    Ok(LiveReceiptDestination::new(
        OwnerParameter::new(published_owner(scalar)?),
        ProtocolValue::new(amount).map_err(|_| VectorError::LiveSubstrateUnavailable)?,
    ))
}

/// The two honest outpoints every finalization case consumes.
fn honest_points() -> Result<[Outpoint; 2], VectorError> {
    Ok([outpoint(0xe1, 0)?, outpoint(0xe2, 1)?])
}

/// The sponsor envelope the two §15.6 cases are staged with.
///
/// Its `sign` declines, which is honest rather than a stub: no case here
/// reaches completion, and an envelope that returned bytes would be
/// modelling the signer Wave 10 recorded as absent.
struct StagedSponsor {
    offer: SponsorOffer,
    change: Option<(u8, Vec<u8>)>,
}

impl SponsorCapability for StagedSponsor {
    fn offer(&self) -> SponsorOffer {
        self.offer.clone()
    }

    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        self.change.clone()
    }

    fn sign(&self, _request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        None
    }
}

/// The honest explicit finalization every finalization case starts from.
///
/// Two receipts of two owners, two destinations, sponsorless. Every
/// mutation below is one change of exactly this.
fn explicit_control(
    abi: &CandidateLiveTransferAbi,
) -> Result<(LiveTransferRequest, PublicConstructionView), VectorError> {
    let [first, second] = honest_points()?;
    let view = PublicConstructionView::new(vec![
        view_of(
            abi,
            first,
            program(abi, &FIRST_SCALAR, LiveTransferRepresentationPlan::Explicit)?,
            ValueField::Explicit(400),
        ),
        view_of(
            abi,
            second,
            program(
                abi,
                &SECOND_SCALAR,
                LiveTransferRepresentationPlan::Explicit,
            )?,
            ValueField::Explicit(600),
        ),
    ])
    .map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    let request = LiveTransferRequest::new(
        [first, second],
        [
            destination(&SECOND_SCALAR, 250)?,
            destination(&FIRST_SCALAR, 750)?,
        ],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        None,
    )
    .map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    Ok((request, view))
}

/// The honest private finalization the private-pairing case starts from.
fn private_control(
    abi: &CandidateLiveTransferAbi,
) -> Result<(LiveTransferRequest, PublicConstructionView), VectorError> {
    use LiveTransferRepresentationPlan::PrivateCommitted as Private;

    let [first, second] = honest_points()?;
    let view = PublicConstructionView::new(vec![
        view_of(
            abi,
            first,
            program(abi, &FIRST_SCALAR, Private)?,
            commitment(abi, 400, 0)?,
        ),
        view_of(
            abi,
            second,
            program(abi, &SECOND_SCALAR, Private)?,
            commitment(abi, 600, 1)?,
        ),
    ])
    .map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    let request = LiveTransferRequest::new(
        [first, second],
        [
            destination(&SECOND_SCALAR, 250)?,
            destination(&FIRST_SCALAR, 750)?,
        ],
        Private,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
        Some(PublicTestRandomness::from_published_bytes(FAULT_RANDOMNESS)),
    )
    .map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    Ok((request, view))
}

/// The honest sponsored finalization the two §15.6 cases start from.
///
/// `change` selects whether the sponsor asks for the change role, which
/// is the difference between the two cases' controls.
fn sponsored_control(
    abi: &CandidateLiveTransferAbi,
    change: bool,
) -> Result<(LiveTransferRequest, PublicConstructionView, StagedSponsor), VectorError> {
    let (_, receipts) = explicit_control(abi)?;
    let [first, second] = honest_points()?;
    let sponsor_point = outpoint(0xe3, 0)?;

    // The sponsor coin joins the view the receipts came from. Both §15.6
    // cases are about a fault somewhere else entirely — an empty offer,
    // and a change destination naming the fee role's program — so the
    // sponsor input has to be one construction has no complaint about,
    // or the mutation under study would not be the reason either half
    // was refused.
    let mut views: Vec<PublicOutputView> = receipts.outputs().values().cloned().collect();
    views.push(PublicOutputView::new(
        sponsor_point,
        AssetField::Explicit(abi.symbols().reserve_asset()),
        ValueField::Explicit(if change { 375 } else { 250 }),
        abi.symbols().sponsor_change_program().to_vec(),
    ));
    let view =
        PublicConstructionView::new(views).map_err(|_| VectorError::LiveSubstrateUnavailable)?;

    let request = LiveTransferRequest::new(
        [first, second],
        [
            destination(&SECOND_SCALAR, 250)?,
            destination(&FIRST_SCALAR, 750)?,
        ],
        LiveTransferRepresentationPlan::Explicit,
        RequestedForm::Sponsored,
        if change {
            SponsorChangeRequest::Requested
        } else {
            SponsorChangeRequest::NotRequested
        },
        None,
    )
    .map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    let sponsor = StagedSponsor {
        offer: SponsorOffer::new(
            [sponsor_point],
            250,
            change.then_some(ValueField::Explicit(125)),
        )
        .map_err(|_| VectorError::LiveSubstrateUnavailable)?,
        change: None,
    };
    Ok((request, view, sponsor))
}

/// Run one finalization, honest or malformed, and report what happened.
fn finalize_outcome(
    abi: &CandidateLiveTransferAbi,
    request: &LiveTransferRequest,
    view: &PublicConstructionView,
    sponsor: Option<&dyn SponsorCapability>,
) -> Result<Option<ObservedFaultRefusal>, LiveFaultRefusal> {
    let target = reviewed_target()?;
    let private: Option<&dyn PrivateValueCapability> = match request.representation() {
        LiveTransferRepresentationPlan::Explicit => None,
        LiveTransferRepresentationPlan::PrivateCommitted => Some(&OracleFixtureValues),
    };
    Ok(
        finalize_live_transfer(&target, abi, request, view, sponsor, private)
            .err()
            .map(ObservedFaultRefusal::Transaction),
    )
}

/// A control that must pass, and a malformed input that must not.
struct Staged {
    control: Option<ObservedFaultRefusal>,
    malformed: Option<ObservedFaultRefusal>,
}

/// Stage one case: run the owning validator twice.
#[expect(
    clippy::too_many_lines,
    reason = "one arm per §15 row, and splitting the match would separate \
              each row's control from the malformation it is the control for"
)]
fn stage(mutation: FaultMutation) -> Result<Staged, LiveFaultRefusal> {
    use FaultMutation as M;
    use LiveTransferRepresentationPlan::Explicit;
    use LiveTransferRepresentationPlan::PrivateCommitted as Private;

    let abi = demonstration_live_abi()?;
    match mutation {
        M::OfferAKeyEncodingOutsideTheKeyDomain => {
            let target = reviewed_target()?;
            let closure = owner_key_encoding_closure(target.definition().authorization());
            let bytes = vec![0x11_u8; 32];
            Ok(Staged {
                control: OwnerKey::new(&closure, closure.approved(), bytes.clone())
                    .err()
                    .map(ObservedFaultRefusal::OwnerKey),
                // One change: the encoding class, whose domain is now the
                // scalar domain rather than the key domain.
                malformed: OwnerKey::new(&closure, EncodingClass::EcScalar, bytes)
                    .err()
                    .map(ObservedFaultRefusal::OwnerKey),
            })
        }
        M::PlaceALeafOfTheOtherRepresentationInTheSchema
        | M::RemoveAnAdmittedShapesCoordinatorLeaf
        | M::AddALeafNoAdmittedShapeReaches => {
            let target = reviewed_target()?;
            let plan = live_transfer_plan()?;
            let shapes = demonstration_live_shape_set();
            let honest = static_transfer_leaf_set(Explicit, &shapes);
            // One change to that exact set, and only one. The empty set
            // is deliberately not among them: §7.5 classifies it as the
            // key-path escape, and §15.4 has its own row for that.
            let mut malformed = honest.clone();
            match mutation {
                M::PlaceALeafOfTheOtherRepresentationInTheSchema => {
                    malformed.insert(LiveTransferLeafRole::Member {
                        representation: Private,
                        receipt_inputs: 2,
                    });
                }
                M::RemoveAnAdmittedShapesCoordinatorLeaf => {
                    let coordinator = *honest
                        .iter()
                        .find(|leaf| matches!(leaf, LiveTransferLeafRole::Coordinator { .. }))
                        .ok_or(LiveFaultRefusal::ControlNotConstructible)?;
                    malformed.remove(&coordinator);
                }
                M::AddALeafNoAdmittedShapeReaches => {
                    // A receipt-input count no admitted shape reaches,
                    // so the leaf serves nothing rather than serving the
                    // wrong thing.
                    malformed.insert(LiveTransferLeafRole::Member {
                        representation: Explicit,
                        receipt_inputs: u8::MAX,
                    });
                }
                _ => return Err(LiveFaultRefusal::ControlNotConstructible),
            }
            if malformed == honest {
                return Err(LiveFaultRefusal::ControlNotConstructible);
            }
            let derive = |leaves: BTreeSet<LiveTransferLeafRole>| -> Result<
                Option<ObservedFaultRefusal>,
                LiveFaultRefusal,
            > {
                Ok(derive_live_receipt_constructor(
                    &target,
                    &plan,
                    Explicit,
                    published_owner(&FIRST_SCALAR)?,
                    shapes.clone(),
                    leaves,
                )
                .err()
                .map(ObservedFaultRefusal::Constructor))
            };
            Ok(Staged {
                control: derive(honest)?,
                malformed: derive(malformed)?,
            })
        }
        M::DefineTheAssetSymbolWithANonAssetValue | M::DefineOneSymbolTwice => {
            let (symbol, value) = honest_asset_definition()?;
            let origin = LiveDefinitionOrigin::DeploymentParameters;
            if mutation == M::DefineOneSymbolTwice {
                let mut census = LiveDefinitionCensus::default();
                let control = census
                    .define(symbol.clone(), value.clone(), origin)
                    .err()
                    .map(ObservedFaultRefusal::Link);
                // One change: the symbol is already defined in this
                // census, which is what the second call is.
                let malformed = census
                    .define(symbol, value, origin)
                    .err()
                    .map(ObservedFaultRefusal::Link);
                return Ok(Staged { control, malformed });
            }
            let mut honest = LiveDefinitionCensus::default();
            let control = honest
                .define(symbol.clone(), value, origin)
                .err()
                .map(ObservedFaultRefusal::Link);
            // One change: the value's kind. A script number where the
            // symbol's declared type is an asset.
            let mut malformed_census = LiveDefinitionCensus::default();
            let malformed = malformed_census
                .define(symbol, LiveSymbolValue::ScriptNumber(0), origin)
                .err()
                .map(ObservedFaultRefusal::Link);
            Ok(Staged { control, malformed })
        }
        M::AskForAZeroValuedDestination => Ok(Staged {
            control: ProtocolValue::new(500)
                .err()
                .map(ObservedFaultRefusal::Transaction),
            // One change: the amount.
            malformed: ProtocolValue::new(0)
                .err()
                .map(ObservedFaultRefusal::Transaction),
        }),
        M::NameADestinationOwnerNothingWasLinkedFor => {
            let (control_request, view) = explicit_control(&abi)?;
            let [first, second] = honest_points()?;
            // One change: the destination owner, which is a well-formed
            // owner key the deployment linked no constructor for.
            let stranger = owner_key(&[0x11_u8; 32])?;
            let request = LiveTransferRequest::new(
                [first, second],
                [
                    LiveReceiptDestination::new(
                        OwnerParameter::new(stranger),
                        ProtocolValue::new(250)
                            .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?,
                    ),
                    destination(&FIRST_SCALAR, 750)?,
                ],
                Explicit,
                RequestedForm::Sponsorless,
                SponsorChangeRequest::NotRequested,
                None,
            )
            .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            Ok(Staged {
                control: finalize_outcome(&abi, &control_request, &view, None)?,
                malformed: finalize_outcome(&abi, &request, &view, None)?,
            })
        }
        M::OfferAReceiptInputUnderAProgramNothingWasLinkedFor => {
            let (request, control_view) = explicit_control(&abi)?;
            let [first, second] = honest_points()?;
            // One change: the tree the first receipt's coin sits under.
            // The linked program with one byte of the output key it
            // commits to moved, which is a coin of a taptree this
            // deployment did not build — what a receipt of another class
            // is, a class being a leaf set and therefore a tree. The
            // asset and the value form stay honest, so the refusal is
            // the program lookup's rather than either check before it.
            let mut foreign = program(&abi, &FIRST_SCALAR, Explicit)?;
            let key_byte = foreign
                .last_mut()
                .ok_or(LiveFaultRefusal::ControlNotConstructible)?;
            *key_byte ^= 0x01;
            let view = PublicConstructionView::new(vec![
                view_of(&abi, first, foreign, ValueField::Explicit(400)),
                view_of(
                    &abi,
                    second,
                    program(&abi, &SECOND_SCALAR, Explicit)?,
                    ValueField::Explicit(600),
                ),
            ])
            .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            Ok(Staged {
                control: finalize_outcome(&abi, &request, &control_view, None)?,
                malformed: finalize_outcome(&abi, &request, &view, None)?,
            })
        }
        M::OfferAnAshProgramAsAReceiptInput => {
            let (request, control_view) = explicit_control(&abi)?;
            let target = reviewed_target()?;
            let [first, second] = honest_points()?;
            // One change: the program the first receipt's coin pays to,
            // which is now the pinned ASH bundle's own taproot output
            // key under the reviewed witness-program script. An HONEST
            // program of another family rather than corrupted bytes —
            // the row is about a family, so a malformed program would
            // answer a different question. The asset and the value form
            // stay honest, so the refusal is the program lookup's rather
            // than the asset check before it.
            let ash = witness_program_script(&target, TAPROOT_WITNESS_VERSION, &PINNED_PROGRAM)
                .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            let view = PublicConstructionView::new(vec![
                view_of(&abi, first, ash, ValueField::Explicit(400)),
                view_of(
                    &abi,
                    second,
                    program(&abi, &SECOND_SCALAR, Explicit)?,
                    ValueField::Explicit(600),
                ),
            ])
            .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            Ok(Staged {
                control: finalize_outcome(&abi, &request, &control_view, None)?,
                malformed: finalize_outcome(&abi, &request, &view, None)?,
            })
        }
        M::OfferOwnerMetadataOutsideTheApprovedWidth => {
            let target = reviewed_target()?;
            let closure = owner_key_encoding_closure(target.definition().authorization());
            Ok(Staged {
                control: OwnerKey::new(&closure, closure.approved(), vec![0x11_u8; 32])
                    .err()
                    .map(ObservedFaultRefusal::OwnerKey),
                // One change: the width. The encoding class stays the
                // approved one, so what refuses is the fixed width and
                // not the domain — which is the sibling case's change
                // and would answer the sibling's row.
                malformed: OwnerKey::new(&closure, closure.approved(), vec![0x11_u8; 31])
                    .err()
                    .map(ObservedFaultRefusal::OwnerKey),
            })
        }
        M::OfferTheReserveAssetUnderAReceiptShapedProgram => {
            let (request, control_view) = explicit_control(&abi)?;
            let [first, second] = honest_points()?;
            // One change: the asset the first receipt's coin carries,
            // which is now the deployment's own reserve asset. The
            // program stays the honestly linked receipt program and the
            // value form stays explicit, so the recognition reaches its
            // ASSET check — which runs before the program lookup — and
            // refuses naming the outpoint whose asset was foreign.
            let view = PublicConstructionView::new(vec![
                PublicOutputView::new(
                    first,
                    AssetField::Explicit(AssetId::from_internal(RESERVE_ASSET)),
                    ValueField::Explicit(400),
                    program(&abi, &FIRST_SCALAR, Explicit)?,
                ),
                view_of(
                    &abi,
                    second,
                    program(&abi, &SECOND_SCALAR, Explicit)?,
                    ValueField::Explicit(600),
                ),
            ])
            .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            Ok(Staged {
                control: finalize_outcome(&abi, &request, &control_view, None)?,
                malformed: finalize_outcome(&abi, &request, &view, None)?,
            })
        }
        M::OverflowTheDestinationTotal => {
            let (control_request, view) = explicit_control(&abi)?;
            let [first, second] = honest_points()?;
            // One change: the destination amounts, whose sum no longer
            // fits the target's explicit width.
            let request = LiveTransferRequest::new(
                [first, second],
                [
                    destination(&SECOND_SCALAR, u64::MAX)?,
                    destination(&FIRST_SCALAR, 750)?,
                ],
                Explicit,
                RequestedForm::Sponsorless,
                SponsorChangeRequest::NotRequested,
                None,
            )
            .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            Ok(Staged {
                control: finalize_outcome(&abi, &control_request, &view, None)?,
                malformed: finalize_outcome(&abi, &request, &view, None)?,
            })
        }
        M::OfferACommitmentValuedReceiptToTheExplicitPlan | M::OfferAHeterogeneousReceiptCensus => {
            let (request, control_view) = explicit_control(&abi)?;
            let [first, second] = honest_points()?;
            // One change: the value field's form on one receipt, or on
            // both. The heterogeneous census is the one the ABI admits no
            // mixed plan for; the homogeneous confidential one is a
            // private census offered to the explicit plan.
            let first_value = if mutation == M::OfferAHeterogeneousReceiptCensus {
                ValueField::Explicit(400)
            } else {
                commitment(&abi, 400, 0)?
            };
            let view = PublicConstructionView::new(vec![
                view_of(
                    &abi,
                    first,
                    program(&abi, &FIRST_SCALAR, Explicit)?,
                    first_value,
                ),
                view_of(
                    &abi,
                    second,
                    program(&abi, &SECOND_SCALAR, Explicit)?,
                    commitment(&abi, 600, 1)?,
                ),
            ])
            .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            Ok(Staged {
                control: finalize_outcome(&abi, &request, &control_view, None)?,
                malformed: finalize_outcome(&abi, &request, &view, None)?,
            })
        }
        M::OfferAnEmptySponsorOfferForASponsoredRequest => {
            let (request, view, sponsor) = sponsored_control(&abi, false)?;
            // One change: the offer's input census, which is now empty.
            let empty = StagedSponsor {
                offer: SponsorOffer::new([], 250, None)
                    .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?,
                change: None,
            };
            Ok(Staged {
                control: finalize_outcome(&abi, &request, &view, Some(&sponsor))?,
                malformed: finalize_outcome(&abi, &request, &view, Some(&empty))?,
            })
        }
        M::OfferTheFeeRoleProgramAsTheSponsorChangeDestination => {
            let (request, view, sponsor) = sponsored_control(&abi, true)?;
            // One change: the change destination, which is now the fee
            // role's own empty program rather than the deployment's
            // sponsor-change one.
            let substituted = StagedSponsor {
                offer: sponsor.offer.clone(),
                change: Some((abi.symbols().sponsor_change_version(), Vec::new())),
            };
            Ok(Staged {
                control: finalize_outcome(&abi, &request, &view, Some(&sponsor))?,
                malformed: finalize_outcome(&abi, &request, &view, Some(&substituted))?,
            })
        }
        M::ReorderTheDestinationOutputsAfterSigning => {
            let (request, view) = explicit_control(&abi)?;
            let target = reviewed_target()?;
            let finalized = finalize_live_transfer(&target, &abi, &request, &view, None, None)
                .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?
                .into_finalized();
            let responses: Vec<_> = finalized
                .signing_requests()
                .iter()
                .map(|signing| {
                    (
                        signing.input(),
                        LiveOwnerResponse::to(signing, OPAQUE_SIGNATURE.to_vec()),
                    )
                })
                .collect();
            let authorized = authorize_live_transfer(finalized.clone(), responses)
                .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            let honest = TargetTransaction::decode(finalized.protected_bytes())
                .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            // One change: the destination outputs, swapped. The census is
            // the same size and holds the same members, so nothing but
            // the order differs.
            let mut outputs = honest.outputs().to_vec();
            if outputs.len() < 2 {
                return Err(LiveFaultRefusal::ControlNotConstructible);
            }
            outputs.swap(0, 1);
            let reordered = TargetTransaction::new(
                honest.version(),
                honest.inputs().to_vec(),
                outputs,
                honest.lock_time(),
                honest.witnesses().to_vec(),
            )
            .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            if reordered == honest {
                return Err(LiveFaultRefusal::ControlNotConstructible);
            }
            Ok(Staged {
                control: authorized
                    .check_offered(&honest)
                    .err()
                    .map(ObservedFaultRefusal::Transaction),
                malformed: authorized
                    .check_offered(&reordered)
                    .err()
                    .map(ObservedFaultRefusal::Transaction),
            })
        }
        M::OfferThePrivateProgramToTheExplicitPlan => {
            let (request, control_view) = explicit_control(&abi)?;
            let [first, second] = honest_points()?;
            // One change: the program the first receipt pays to, which is
            // now the private plan's constructor. The value stays
            // explicit, so the refusal is attributable to the plan
            // pairing and not to the value form.
            let view = PublicConstructionView::new(vec![
                view_of(
                    &abi,
                    first,
                    program(&abi, &FIRST_SCALAR, Private)?,
                    ValueField::Explicit(400),
                ),
                view_of(
                    &abi,
                    second,
                    program(&abi, &SECOND_SCALAR, Explicit)?,
                    ValueField::Explicit(600),
                ),
            ])
            .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            Ok(Staged {
                control: finalize_outcome(&abi, &request, &control_view, None)?,
                malformed: finalize_outcome(&abi, &request, &view, None)?,
            })
        }
        M::OfferTheExplicitProgramToThePrivatePlan => {
            let (request, control_view) = private_control(&abi)?;
            let [first, second] = honest_points()?;
            // The mirror. The value stays a commitment so that the plan
            // pairing is what the refusal is about.
            let view = PublicConstructionView::new(vec![
                view_of(
                    &abi,
                    first,
                    program(&abi, &FIRST_SCALAR, Explicit)?,
                    commitment(&abi, 400, 0)?,
                ),
                view_of(
                    &abi,
                    second,
                    program(&abi, &SECOND_SCALAR, Private)?,
                    commitment(&abi, 600, 1)?,
                ),
            ])
            .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
            Ok(Staged {
                control: finalize_outcome(&abi, &request, &control_view, None)?,
                malformed: finalize_outcome(&abi, &request, &view, None)?,
            })
        }
    }
}

/// The protocol-asset symbol and the value the deployment defines it
/// with.
///
/// Taken out of the census the deployment's own link collects, rather
/// than invented here: a control built from a value nothing else uses
/// would establish that the census admits *that* value.
fn honest_asset_definition() -> Result<(LiveLinkSymbol, LiveSymbolValue), LiveFaultRefusal> {
    let target = reviewed_target()?;
    let bundles = relocatable_live_bundles()?;
    let bundle = bundles
        .first()
        .ok_or(LiveFaultRefusal::ControlNotConstructible)?;
    let deployment = live_deployment_for_asset(PROTOCOL_ASSET, RESERVE_ASSET, FEE_PROGRAM_DIGEST)?;
    let census = collect_live_definitions(&target, bundle, &deployment)
        .map_err(|_| LiveFaultRefusal::ControlNotConstructible)?;
    census
        .definitions()
        .iter()
        .find(|(symbol, _)| matches!(symbol, LiveLinkSymbol::ProtocolAsset))
        .map(|(symbol, definition)| (symbol.clone(), definition.value().clone()))
        .ok_or(LiveFaultRefusal::ControlNotConstructible)
}

/// Discharge one first-party negative row, per §4.2.
///
/// The owning entry point is run twice: once on the honest input, which
/// must be accepted, and once on the input with the case's one stated
/// change, which must be refused naming the case's own class.
///
/// # Errors
///
/// [`LiveFaultRefusal`], naming which of §4.2's conditions the offered
/// case does not meet.
pub fn validate_live_fault(
    case: &LiveFaultCase,
) -> Result<ValidatedLiveFaultEvidence, LiveFaultRefusal> {
    let row = matrix_row(case.row).ok_or(LiveFaultRefusal::RowNotInTheMatrix(case.row))?;
    if !row.is_first_party() {
        return Err(LiveFaultRefusal::RowIsNotFirstParty(case.row));
    }

    let staged = stage(case.mutation)?;
    // The control first: a refusal of the honest input would make every
    // later conclusion about the wrong thing.
    if let Some(refused) = staged.control {
        return Err(LiveFaultRefusal::ControlWasRefused(Box::new(refused)));
    }
    let observed = staged
        .malformed
        .ok_or(LiveFaultRefusal::MalformedInputWasAccepted)?;
    if !(case.expected)(&observed) {
        return Err(LiveFaultRefusal::RefusalNamesAnotherClass(Box::new(
            observed,
        )));
    }
    Ok(ValidatedLiveFaultEvidence {
        row: case.row,
        validator: case.validator,
        mutation: case.mutation,
        observed,
    })
}

/// Every §15.4–§15.7 row this census discharges.
///
/// # Errors
///
/// Whatever [`validate_live_fault`] refuses, on the first case that does
/// not meet §4.2.
pub fn discharge_live_faults() -> Result<Vec<ValidatedLiveFaultEvidence>, LiveFaultRefusal> {
    static CACHED: OnceLock<Result<Vec<ValidatedLiveFaultEvidence>, LiveFaultRefusal>> =
        OnceLock::new();
    CACHED.get_or_init(run_every_fault_case).clone()
}

/// The census proper, run once behind the cache.
///
/// # The memoization changes nothing about what is established
///
/// Each case is a pure function of constants in this file and of the
/// substrate [`crate::live_plan`] already caches, so two runs cannot
/// differ; driving thirteen entry points twice each cost about a minute
/// per caller, and [`crate::live_evidence`] asks for the census once per
/// evidence plan. The cache holds the *result*, refusals included, so a
/// census that failed §4.2 keeps failing rather than being retried into a
/// different answer.
fn run_every_fault_case() -> Result<Vec<ValidatedLiveFaultEvidence>, LiveFaultRefusal> {
    live_fault_cases().iter().map(validate_live_fault).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        LiveFaultCase, discharge_live_faults, live_fault_cases, matrix_row, validate_live_fault,
    };
    use std::collections::BTreeSet;

    #[test]
    fn every_case_names_a_first_party_row_of_the_matrix() {
        // A case naming a row §15 does not have, or naming one whose
        // boundary is the target's, would be evidence filed against
        // nothing.
        for case in live_fault_cases() {
            let row =
                matrix_row(case.row()).unwrap_or_else(|| panic!("{} is not a §15 row", case.row()));
            assert!(row.is_first_party(), "{row} is not answered first-party");
        }
    }

    #[test]
    fn the_census_stages_each_row_once_and_leaves_none_of_them_out() {
        // §4.2's partition, and this census now takes the whole of its
        // side of it: every §15.4–§15.7 row a first-party layer owns has
        // a staged case, and each is staged once. A row missing from
        // here would be silently outstanding, which is exactly what
        // §4.2's last sentence forbids.
        let staged: BTreeSet<_> = live_fault_cases().iter().map(LiveFaultCase::row).collect();
        assert_eq!(
            staged.len(),
            live_fault_cases().len(),
            "a row is staged twice"
        );
        for row in &staged {
            assert_ne!(matrix_row(row), None, "{row} is not in the matrix");
        }

        // The rows this census owes, recomputed from the matrix rather
        // than listed: the pre-target rows of the four fault tables.
        // §15.3's own table is `crate::live_first_party`'s, except for
        // the one row whose validator lives in this vocabulary.
        let owed: BTreeSet<_> = crate::live_safety::required_safety_matrix()
            .into_iter()
            .filter(|row| {
                row.is_first_party()
                    && !matches!(
                        row.section(),
                        crate::live_safety::LiveSafetySection::PositiveExplicit
                            | crate::live_safety::LiveSafetySection::PositivePrivate
                            | crate::live_safety::LiveSafetySection::OwnerSignatureFault
                    )
            })
            .map(crate::live_safety::LiveSafetyRow::name)
            .collect();
        assert_eq!(
            owed.difference(&staged).count(),
            0,
            "a §15.4–§15.7 first-party row has no staged case",
        );
    }

    #[test]
    fn every_case_discharges_its_row_against_its_own_control() {
        // §4.2 performed, over the whole census. Each case runs its
        // owning entry point twice and must be refused naming its own
        // class while the honest input is accepted; a case whose refusal
        // came from somewhere else fails here rather than being filed as
        // coverage.
        let discharged: Vec<_> = live_fault_cases()
            .iter()
            .map(|case| {
                validate_live_fault(case)
                    .unwrap_or_else(|refusal| panic!("{case:?} did not meet §4.2: {refusal:?}"))
            })
            .collect();
        assert_eq!(discharged.len(), live_fault_cases().len());
        assert_eq!(
            discharge_live_faults().expect("the census discharges"),
            discharged
        );

        // Every owning entry point the census names was really driven,
        // so a census that quietly lost one would fail rather than
        // report a smaller matrix.
        let validators: BTreeSet<_> = discharged
            .iter()
            .map(super::ValidatedLiveFaultEvidence::validator)
            .collect();
        assert_eq!(validators.len(), 6, "six owning entry points");
    }

    #[test]
    fn every_malformed_constructor_schema_is_refused_by_its_own_name() {
        // §4.2 asks for one canonical malformed input and the census
        // stages one. The leaf schema admits three malformations, and a
        // discharge of one of them says nothing about the other two, so
        // all three are driven here against the same accepted control.
        //
        // The empty leaf set is absent on purpose: the constructor
        // refuses it as `KeyPathWouldBeTheOnlySpendingRoute`, which is
        // §15.4's key-path-escape row rather than this one.
        use super::{FaultMutation as M, ObservedFaultRefusal, stage};
        use tapscript::LiveConstructorRefusal as R;

        /// One malformation and the refusal class it must meet.
        type SchemaCase = (M, fn(&R) -> bool);

        let expected: &[SchemaCase] = &[
            (
                M::PlaceALeafOfTheOtherRepresentationInTheSchema,
                |refusal| matches!(refusal, R::LeafOfAnotherRepresentation { .. }),
            ),
            (M::RemoveAnAdmittedShapesCoordinatorLeaf, |refusal| {
                matches!(refusal, R::LeafSetIncomplete { .. })
            }),
            (M::AddALeafNoAdmittedShapeReaches, |refusal| {
                matches!(refusal, R::LeafServesNoAdmittedShape { .. })
            }),
        ];

        for (mutation, names_it) in expected {
            let staged = stage(*mutation).expect("the schema stages");
            assert_eq!(
                staged.control, None,
                "{mutation:?} refused the canonical leaf set too",
            );
            let ObservedFaultRefusal::Constructor(refusal) =
                staged.malformed.expect("the malformed schema is refused")
            else {
                panic!("{mutation:?} was refused by another vocabulary");
            };
            assert!(names_it(&refusal), "{mutation:?} met {refusal:?}");
        }
    }
}
