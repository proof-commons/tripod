//! Stable semantic identifiers used by the architecture manifest.
//!
//! Explicit discriminants are part of the canonical semantic encoding.
//! Do not reorder or renumber existing variants after publication; add
//! new variants only under a new architecture schema version.

use core::fmt;

macro_rules! simple_id {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $(
                $variant:ident = $code:expr => $text:literal
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr(u16)]
        #[derive(
            Clone,
            Copy,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
        )]
        pub enum $name {
            $(
                $variant = $code,
            )+
        }

        impl $name {
            pub const ALL: &'static [Self] = &[
                $(
                    Self::$variant,
                )+
            ];

            pub const fn code(self) -> u16 {
                self as u16
            }

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $text,
                    )+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

simple_id! {
    /// Canonical asset identifiers.
    pub enum AssetId {
        Lbtc     = 1 => "L-BTC",
        U        = 2 => "U",
        Ent      = 3 => "ENT",
        DistCtl  = 4 => "DIST_CTL",
        Pid      = 5 => "PID",
        Pace     = 6 => "PACE",
        EntAuth  = 7 => "ENT_AUTH",
        DistAuth = 8 => "DIST_AUTH",
    }
}

simple_id! {
    /// Monetary receipt classes.
    pub enum ReceiptClassId {
        Live       = 1 => "live",
        TimeLocked = 2 => "time-locked",
    }
}

simple_id! {
    /// Constant-cardinality roots.
    pub enum RootId {
        State    = 1 => "STATE",
        Resv     = 2 => "RESV",
        Pace     = 3 => "PACE",
        EntAuth  = 4 => "ENT_AUTH",
        DistAuth = 5 => "DIST_AUTH",
    }
}

simple_id! {
    /// Stable operation and branch-family identifiers.
    pub enum OperationId {
        CreateRequest      = 1  => "create-request",
        CancelRequest      = 2  => "cancel-request",
        AdmitDeposits      = 3  => "admit-deposits",
        Cycle              = 4  => "cycle",
        SettleDistribution = 5  => "settle-distribution",
        TransferLive       = 6  => "transfer-live-receipts",
        TransferTimeLocked = 7  => "transfer-time-locked-receipts",
        Redeem             = 8  => "redeem",
        ReceiptRelabel     = 9  => "receipt-relabel",
        Burn               = 10 => "burn",
        CompactAsh         = 11 => "compact-ash",
        Clear              = 12 => "clear",
        AnnounceMaturity   = 13 => "announce-maturity",
    }
}

simple_id! {
    /// Stable protocol object identifiers.
    pub enum ObjectId {
        State                 = 1  => "STATE",
        Resv                  = 2  => "RESV",
        Pace                  = 3  => "PACE",
        EntitlementAuthority  = 4  => "ENTITLEMENT_AUTHORITY",
        DistributionAuthority = 5  => "DISTRIBUTION_AUTHORITY",
        ReceiptLive           = 6  => "RECEIPT_L",
        ReceiptTimeLocked     = 7  => "RECEIPT_T",
        DepositRequest        = 8  => "DEPOSIT_REQUEST",
        DepositEntitlement    = 9  => "DEPOSIT_ENTITLEMENT",
        DistributionControl   = 10 => "DISTRIBUTION_CONTROL",
        DistributionVault     = 11 => "DISTRIBUTION_VAULT",
        Ash                   = 12 => "ASH",
        PlainLbtc             = 13 => "PLAIN_LBTC",
        CpfpAnchor            = 14 => "CPFP_ANCHOR",
    }
}

simple_id! {
    /// The realization document's invariant state clauses (𝗜₁–𝗜₁₁).
    ///
    /// Codes are the clause ordinals and freeze at publication. The
    /// `as_str` text **is** the document label, so the doc-conformance
    /// test covers clauses and witnesses uniformly.
    pub enum InvariantClauseId {
        Identity       = 1  => "inv:invariant:identity",
        Domains        = 2  => "inv:invariant:domains",
        RateFloor      = 3  => "inv:invariant:rate-floor",
        NoTrap         = 4  => "inv:invariant:no-trap",
        Backing        = 5  => "inv:invariant:backing",
        NoStarve       = 6  => "inv:invariant:no-starve",
        EscrowReceipts = 7  => "inv:invariant:escrow-receipts",
        Accounting     = 8  => "inv:invariant:accounting",
        ConsensusValue = 9  => "inv:invariant:consensus-value",
        Succession     = 10 => "inv:invariant:succession",
        Maturity       = 11 => "inv:invariant:maturity",
    }
}

simple_id! {
    /// Named witness and proof families.
    pub enum WitnessId {
        CanonicalDelta            = 1  => "canonical-delta",
        AshLineage                = 2  => "ash-lineage",
        StateSuccession           = 3  => "state-succession",
        ResvSuccession            = 4  => "resv-succession",
        ReceiptOwnerRouting       = 5  => "receipt-owner-routing",
        ValueFlowClosure          = 6  => "value-flow-closure",
        RequestClosure            = 7  => "request-closure",
        EntitlementClosure        = 8  => "entitlement-closure",
        DistributionClosure       = 9  => "distribution-closure",
        ReceiptClassClosure       = 10 => "receipt-class-closure",
        AttestationAuthentication = 11 => "attestation-authentication",
        NativeFeeAuction          = 12 => "native-fee-auction",
        UtxoLifecycle             = 13 => "utxo-lifecycle",
        FloorNondecrease          = 14 => "floor-nondecrease",
    }
}

simple_id! {
    /// Named quantities and derived computations.
    pub enum QuantityId {
        Floor                       = 1 => "floor-phi",
        RedemptionPayout            = 2 => "redemption-payout",
        CycleIssuance               = 3 => "cycle-issuance",
        AttestationDelta            = 4 => "attestation-delta",
        AttestationMap              = 5 => "attestation-map",
        HistoricalLiveResidue       = 6 => "historical-live-residue",
        HistoricalTimeLockedResidue = 7 => "historical-time-locked-residue",
        ReceiptAccountingAudit      = 8 => "receipt-accounting-audit",
    }
}

simple_id! {
    /// Primitive data sources used by quantity declarations.
    pub enum DataId {
        StateOmega                  = 1  => "state.omega",
        StateYLive                  = 2  => "state.y-live",
        StateYTimeLocked            = 3  => "state.y-time-locked",
        StateQ                      = 4  => "state.q",
        ClearOmega                  = 5  => "clear.omega",
        ClearY                      = 6  => "clear.y",
        BurnRecordAmount            = 7  => "burn-record.amount",
        HistoricalLiveResidue       = 8  => "history.residue-live",
        HistoricalTimeLockedResidue = 9  => "history.residue-time-locked",
        AttestationTerms            = 10 => "attestation.terms",
    }
}

simple_id! {
    /// Realization and deployment dependencies.
    ///
    /// Schema 17 splits the former `explicit-values` dependency (code
    /// 3) into its four proof-method components: code 3 is renarrowed
    /// to explicit-value introspection, and codes 10–12 name the
    /// confidential-transaction discharge routes.
    pub enum DependencyId {
        NativeAssetConservation       = 1  => "native-asset-conservation",
        IssuanceIntrospection         = 2  => "issuance-introspection",
        ExplicitValueIntrospection    = 3  => "explicit-value-introspection",
        SighashProfile                = 4  => "sighash-profile",
        PackageRelay                  = 5  => "package-relay",
        UnspendableUtxoExclusion      = 6  => "unspendable-utxo-exclusion",
        WeldEnforcement               = 7  => "weld-enforcement",
        LbtcSettlement                = 8  => "lbtc-settlement",
        ScriptEmissionFidelity        = 9  => "script-emission-fidelity",
        ConfidentialValueConservation = 10 => "confidential-value-conservation",
        ValueCommitmentEquality       = 11 => "value-commitment-equality",
        ValueCommitmentOpening        = 12 => "value-commitment-opening",
    }
}

simple_id! {
    /// Closed architecture decisions.
    pub enum DecisionId {
        NoOnchainAttestationAccumulator  = 1 => "no-onchain-attestation-accumulator",
        FullChainAttestationVerification = 2 => "full-chain-attestation-verification",
        HistoricalResidueAuditOnly       = 3 => "historical-residue-audit-only",
        OneEntitlementPerRequest         = 4 => "one-entitlement-per-request",
        SettlementPermissionless         = 5 => "settlement-permissionless",
        AtomicMaturityCycle              = 6 => "atomic-maturity-cycle",
        ActiveBackingCap                 = 7 => "active-backing-cap",
    }
}

simple_id! {
    /// Fixed protocol amount limits.
    pub enum AmountLimitId {
        ActiveBackingMax = 1 => "ACTIVE_BACKING_MAX",
    }
}

simple_id! {
    /// Finite branch and script bounds.
    pub enum BoundId {
        AdmissionBatchMax  = 1  => "ADMISSION_BATCH_MAX",
        SettlementBatchMax = 2  => "SETTLEMENT_BATCH_MAX",
        RelabelBatchMax    = 3  => "RELABEL_BATCH_MAX",
        AshBatchMax        = 4  => "ASH_BATCH_MAX",
        BurnInputMax       = 5  => "BURN_INPUT_MAX",
        BurnChangeMax      = 6  => "BURN_CHANGE_MAX",
        BurnRecordMax      = 7  => "BURN_RECORD_MAX",
        TransferInputMax   = 8  => "TRANSFER_INPUT_MAX",
        TransferOutputMax  = 9  => "TRANSFER_OUTPUT_MAX",
        FeeSponsorInputMax = 10 => "FEE_SPONSOR_INPUT_MAX",
    }
}

simple_id! {
    /// Derived history projections.
    pub enum ProjectionId {
        TransitionCertificate = 1 => "transition-certificate",
        BurnEvent             = 2 => "burn-event",
        ClearEvent            = 3 => "clear-event",
        DistributionResidue   = 4 => "distribution-residue",
    }
}

simple_id! {
    /// Domain-separated data-output tags.
    pub enum TagId {
        Burn                     = 1 => "tag-burn",
        Recon                    = 2 => "tag-recon",
        Redeem                   = 3 => "tag-redeem",
        Entitlement              = 4 => "tag-entitlement",
        DistributionControlClose = 5 => "tag-distribution-control-close",
        DistributionResidue      = 6 => "tag-distribution-residue",
    }
}

simple_id! {
    /// Value-flow authorization classes.
    pub enum ValueFlowClass {
        OwnerConsented             = 1 => "owner-consented",
        ImmutableDestination       = 2 => "immutable-destination",
        FormulaBoundPayout         = 3 => "formula-bound-payout",
        PreauthorizedServiceBudget = 4 => "preauthorized-service-budget",
        OwnerlessTerminalSink      = 5 => "ownerless-terminal-sink",
        SponsorEnvelope            = 6 => "sponsor-envelope",
        OwnerlessBoundSink         = 7 => "ownerless-bound-sink",
        OwnerlessBoundMovement     = 8 => "ownerless-bound-movement",
    }
}

simple_id! {
    /// Open L-BTC flow roles.
    pub enum OpenFlowKind {
        RequestCreation  = 1 => "request-creation",
        RequestRefund    = 2 => "request-refund",
        DepositAdmission = 3 => "deposit-admission",
        ReserveCarry     = 4 => "reserve-carry",
        Redemption       = 5 => "redemption",
        FeeSponsor       = 6 => "fee-sponsor",
    }
}

simple_id! {
    /// Canonical asset-flow categories.
    pub enum DeltaKind {
        Issuance         = 1 => "issuance",
        Destruction      = 2 => "destruction",
        Lateral          = 3 => "lateral",
        OwnerlessLateral = 4 => "ownerless-lateral",
    }
}

simple_id! {
    /// Authorization applied to one operation input family.
    ///
    /// `CovenantCompanion` marks a root input that carries no
    /// independent owner signature: its participation is authorized by
    /// the enclosing covenant branch shape, root-use policy, co-spend
    /// requirements, and transaction-shape predicates. Who authorizes
    /// the *operation* is the separate `PermissionClass`.
    /// `InputOwner` is the owner committed by the consumed object;
    /// `RefundKey` is the request metadata's refund key;
    /// `SponsorOwner` is an ordinary L-BTC owner; and
    /// `Permissionless` requires no signature or secret for the input.
    pub enum InputAuthorization {
        CovenantCompanion = 1 => "covenant-companion",
        InputOwner        = 2 => "input-owner",
        RefundKey         = 3 => "refund-key",
        SponsorOwner      = 4 => "sponsor-owner",
        Permissionless    = 5 => "permissionless",
    }
}

simple_id! {
    /// Canonical-delta and data-output activation conditions.
    pub enum DeltaCondition {
        Always                      = 1 => "always",
        PositiveAdmittedPrincipal   = 2 => "positive-admitted-principal",
        PositiveCycleIssuance       = 3 => "positive-cycle-issuance",
        PositiveCyclePrincipal      = 4 => "positive-cycle-principal",
        DistributionContinues       = 5 => "distribution-continues",
        DistributionTerminates      = 6 => "distribution-terminates",
        PositiveSettlementUOutput   = 7 => "positive-settlement-u-output",
        PositiveDistributionResidue = 8 => "positive-distribution-residue",
        PositiveAshResidual         = 9 => "positive-ash-residual",
    }
}

simple_id! {
    /// Data-output families.
    pub enum DataOutputKind {
        BurnRecord  = 1 => "burn-record",
        Destruction = 2 => "destruction",
    }
}

simple_id! {
    /// Evidence classes backing an authorization claim.
    ///
    /// A green executable-model test proves model-level evidence only;
    /// exact opcodes, covenant predicates, and sighash/CSV behavior
    /// remain compiler and deployment obligations.
    pub enum AuthorizationEvidenceKind {
        ModelSignerSet            = 1  => "model-signer-set",
        CompilerChecksig          = 2  => "compiler-checksig",
        DeploymentSighash         = 3  => "deployment-sighash",
        NonePermissionless        = 4  => "none-permissionless",
        ModelBranchShape          = 5  => "model-branch-shape",
        ModelCadenceBand          = 6  => "model-cadence-band",
        CompilerCovenantPredicate = 7  => "compiler-covenant-predicate",
        CompilerCadenceLeaves     = 8  => "compiler-cadence-leaves",
        DeploymentScriptSemantics = 9  => "deployment-script-semantics",
        DeploymentCsvSemantics    = 10 => "deployment-csv-semantics",
    }
}

/// Evidence triple backing one authorization mode: model-level,
/// compiler-level, and deployment-level evidence.
pub struct AuthorizationEvidence {
    pub model: AuthorizationEvidenceKind,
    pub compiler: AuthorizationEvidenceKind,
    pub deployment: AuthorizationEvidenceKind,
}

const SIGNER_BACKED_EVIDENCE: AuthorizationEvidence = AuthorizationEvidence {
    model: AuthorizationEvidenceKind::ModelSignerSet,
    compiler: AuthorizationEvidenceKind::CompilerChecksig,
    deployment: AuthorizationEvidenceKind::DeploymentSighash,
};

const PERMISSIONLESS_EVIDENCE: AuthorizationEvidence = AuthorizationEvidence {
    model: AuthorizationEvidenceKind::NonePermissionless,
    compiler: AuthorizationEvidenceKind::NonePermissionless,
    deployment: AuthorizationEvidenceKind::NonePermissionless,
};

impl InputAuthorization {
    /// The evidence classes backing this input-authorization mode at
    /// each assurance layer. A covenant companion carries no
    /// independent signature: its evidence is branch shape, covenant
    /// predicate, and emitted-script semantics.
    pub const fn evidence(self) -> AuthorizationEvidence {
        match self {
            Self::CovenantCompanion => AuthorizationEvidence {
                model: AuthorizationEvidenceKind::ModelBranchShape,
                compiler: AuthorizationEvidenceKind::CompilerCovenantPredicate,
                deployment: AuthorizationEvidenceKind::DeploymentScriptSemantics,
            },

            Self::InputOwner | Self::RefundKey | Self::SponsorOwner => SIGNER_BACKED_EVIDENCE,

            Self::Permissionless => PERMISSIONLESS_EVIDENCE,
        }
    }
}

simple_id! {
    /// Root-use policy.
    pub enum RootUse {
        Forbidden               = 1 => "forbidden",
        Succession              = 2 => "succession",
        SuccessionOrTermination = 3 => "succession-or-termination",
    }
}

simple_id! {
    /// Operation semantic kind.
    pub enum OperationKind {
        ClientProtocol = 1 => "client-protocol",
        CovenantBranch = 2 => "covenant-branch",
    }
}

simple_id! {
    /// Primary operation authorization.
    pub enum PermissionClass {
        ClientAuthorized = 1 => "client-authorized",
        RefundKey        = 2 => "refund-key",
        Permissionless   = 3 => "permissionless",
        CadenceBand      = 4 => "cadence-band",
        ReceiptOwners    = 5 => "receipt-owners",
        Operator         = 6 => "operator",
    }
}

impl PermissionClass {
    /// The evidence classes backing this operation-level authorization
    /// at each assurance layer. Cadence-band authorization is a
    /// composite policy — before the minimum cadence the branch is
    /// invalid, within the band it requires an operator signature, and
    /// after the maximum it is permissionless — so its evidence is the
    /// cadence band itself, the emitted cadence leaves, and deployment
    /// CSV semantics, not an ordinary signature check.
    pub const fn evidence(self) -> AuthorizationEvidence {
        match self {
            Self::ClientAuthorized | Self::RefundKey | Self::ReceiptOwners | Self::Operator => {
                SIGNER_BACKED_EVIDENCE
            }

            Self::Permissionless => PERMISSIONLESS_EVIDENCE,

            Self::CadenceBand => AuthorizationEvidence {
                model: AuthorizationEvidenceKind::ModelCadenceBand,
                compiler: AuthorizationEvidenceKind::CompilerCadenceLeaves,
                deployment: AuthorizationEvidenceKind::DeploymentCsvSemantics,
            },
        }
    }
}

simple_id! {
    /// Asset openness.
    pub enum AssetClass {
        Open   = 1 => "open",
        Closed = 2 => "closed",
    }
}

simple_id! {
    /// Functional asset role.
    pub enum AssetRole {
        ReserveAsset        = 1 => "reserve-asset",
        MonetaryReceipt     = 2 => "monetary-receipt",
        DepositEntitlement  = 3 => "deposit-entitlement",
        DistributionControl = 4 => "distribution-control",
        Identity            = 5 => "identity",
        IssuanceAuthority   = 6 => "issuance-authority",
    }
}

simple_id! {
    /// Root semantics.
    ///
    /// The no-accumulator decision is stronger because an
    /// attestation-root role cannot be represented at all: the
    /// declared root set is exactly the five expected roots, no
    /// operation touches an undeclared root, and
    /// `DecisionId::NoOnchainAttestationAccumulator` is closed.
    pub enum RootRole {
        StateIdentity         = 1 => "state-identity",
        ActiveReserve         = 2 => "active-reserve",
        CadenceAndIssuance    = 3 => "cadence-and-issuance",
        EntitlementAuthority  = 4 => "entitlement-authority",
        DistributionAuthority = 5 => "distribution-authority",
    }
}

simple_id! {
    /// Quantity category.
    pub enum QuantityKind {
        Monetary  = 1 => "monetary",
        Interface = 2 => "interface",
        AuditOnly = 3 => "audit-only",
        Derived   = 4 => "derived",
    }
}

simple_id! {
    /// Decision status.
    pub enum DecisionStatus {
        Open   = 1 => "open",
        Closed = 2 => "closed",
    }
}

simple_id! {
    /// Publication status.
    pub enum PublicationStatus {
        Draft = 1 => "draft",
        Final = 2 => "final",
    }
}

simple_id! {
    /// Event/projection availability.
    pub enum ProjectionRule {
        Forbidden = 1 => "forbidden",
        Optional  = 2 => "optional",
        Required  = 3 => "required",
    }
}

simple_id! {
    /// Cardinality maximum encoding.
    pub enum LimitKind {
        Exact = 1 => "exact",
        Bound = 2 => "bound",
    }
}

simple_id! {
    /// Issuance guard conditions.
    pub enum IssuanceCondition {
        PositiveAdmittedPrincipal = 1 => "positive-admitted-principal",
        PositiveCycleIssuance     = 2 => "positive-cycle-issuance",
        PositiveCyclePrincipal    = 3 => "positive-cycle-principal",
    }
}

simple_id! {
    /// Protocol-object lifecycle class.
    pub enum LifecycleClass {
        ConstantRoot    = 1 => "constant-root",
        OpenOffer       = 2 => "open-offer",
        UserPosition    = 3 => "user-position",
        CycleQueue      = 4 => "cycle-queue",
        Backlog         = 5 => "backlog",
        BoundedRetained = 6 => "bounded-retained",
        ExternalWallet  = 7 => "external-wallet",
    }
}

simple_id! {
    /// Accounting domain of a recognized object.
    pub enum AccountingDomain {
        None        = 1 => "none",
        Backing     = 2 => "backing",
        Receipt     = 3 => "receipt",
        Entitlement = 4 => "entitlement",
        Fee         = 5 => "fee",
    }
}

/// Object allocation provenance. Requires a parameterized operation
/// variant, so it is defined outside the simple-id macro.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AllocatorId {
    Genesis,
    External,
    Operation(OperationId),
}

impl AllocatorId {
    pub const fn code(self) -> (u16, u16) {
        match self {
            Self::Genesis => (1, 0),
            Self::External => (2, 0),
            Self::Operation(operation) => (3, operation.code()),
        }
    }

    pub fn display_name(self) -> String {
        match self {
            Self::Genesis => "genesis".to_owned(),
            Self::External => "external".to_owned(),
            Self::Operation(operation) => {
                format!("operation:{}", operation.as_str())
            }
        }
    }
}

/// Object deallocation provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeallocatorId {
    None,
    ExternalSpend,
    Operation(OperationId),
}

impl DeallocatorId {
    pub const fn code(self) -> (u16, u16) {
        match self {
            Self::None => (1, 0),
            Self::ExternalSpend => (2, 0),
            Self::Operation(operation) => (3, operation.code()),
        }
    }

    pub fn display_name(self) -> String {
        match self {
            Self::None => "none".to_owned(),
            Self::ExternalSpend => "external-spend".to_owned(),
            Self::Operation(operation) => {
                format!("operation:{}", operation.as_str())
            }
        }
    }
}

/// Quantity reader identities require a parameterized operation variant,
/// so this identifier is defined outside the simple-id macro.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReaderId {
    Operation(OperationId),
    AttestationIndexer,
    InvariantChecker,
    ExternalAuditor,
    ConsumerFormula,
}

impl ReaderId {
    pub const fn code(self) -> (u16, u16) {
        match self {
            Self::Operation(operation) => (1, operation.code()),
            Self::AttestationIndexer => (2, 0),
            Self::InvariantChecker => (3, 0),
            Self::ExternalAuditor => (4, 0),
            Self::ConsumerFormula => (5, 0),
        }
    }
}

impl fmt::Display for ReaderId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Operation(operation) => {
                formatter.write_str("operation:")?;
                formatter.write_str(operation.as_str())
            }
            Self::AttestationIndexer => formatter.write_str("attestation-indexer"),
            Self::InvariantChecker => formatter.write_str("invariant-checker"),
            Self::ExternalAuditor => formatter.write_str("external-auditor"),
            Self::ConsumerFormula => formatter.write_str("consumer-formula"),
        }
    }
}
