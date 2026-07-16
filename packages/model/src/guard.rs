//! Guard and invariant errors.
//!
//! Implements `(´def:verification:guard´)` and
//! `(´def:verification:invariant-error´)`.

// One declaration produces the enum, `ALL`, and `name()`, so no
// hand-maintained variant list can silently omit a new variant.
macro_rules! failure_vocabulary {
    ( $(#[$meta:meta])* pub enum $name:ident { $($variant:ident),+ $(,)? } ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name { $($variant),+ }

        impl $name {
            pub const ALL: &'static [$name] = &[ $(Self::$variant),+ ];

            pub const fn name(self) -> &'static str {
                match self { $(Self::$variant => stringify!($variant)),+ }
            }
        }
    };
}

// ´def:verification:guard´

failure_vocabulary! {
    pub enum Guard {
        NoSuch,
        WrongAsset,
        WrongShape,
        WrongPool,
        WrongTarget,
        WrongClass,

        Domain,
        BadConstant,
        Overflow,
        Underflow,
        CycleOverflow,

        DuplicateInput,
        DuplicateOutputIndex,
        MissingOutputIndex,

        BadSignature,
        BadAuthorization,

        Sealed,
        NoTrap,
        ZeroProgress,

        OverDraw,
        ValuePin,
        PartitionPin,
        RecipientPin,
        ClassCross,

        MissingAuthority,
        BadIssuance,
        BadDestruction,
        CanonicalDeltaMismatch,

        // Declared for the emitted-script vocabulary (duplicate root
        // outputs are unrepresentable in the model's map-keyed world);
        // never produced at model level.
        RootMultiplicity,
        RootSuccession,
        ResvWeld,
        ControlVaultWeld,

        CadenceTooEarly,
        CadenceOperatorOnly,

        MaturityAlreadyAnnounced,
        MaturityLeadTooShort,
        MaturityLeadTooLong,
        MaturityNotComplete,

        FeeMismatch,
        SponsorMismatch,
        OpenFlowMismatch,

        WrongCheckpoint,
        UnsupportedSchema,
        HistoryOrder,
        DuplicateEvent,

        ActiveBackingCapExceeded,

        InvariantFailure,
    }
}

// ´def:verification:invariant-error´

failure_vocabulary! {
    pub enum InvariantError {
        IdentityAuthority,
        Domains,
        Floor,
        SealedTerminal,
        Backing,
        DistributionPayability,
        EntitlementLifecycle,
        ReceiptAccountingPreMaturity,
        ReceiptAccountingPostMaturity,
        MaturityCoherence,
        ConsensusValueAuthority,
        StateSuccession,
        ResvSuccession,
        CanonicalClosure,
        ActiveBackingCap,
        HistoryProjection,
    }
}
