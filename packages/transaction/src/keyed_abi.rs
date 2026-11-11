//! Structural keyed request obligations, before materialization or policy admission.
/// Outstanding external-evidence and step-4 construction obligations.
///
/// Four external facts (`Inclusion`, `Unspentness`, `Retrieval`, `Finality`) apply
/// to every spent input's parent, including collateral, and are owned outside
/// construction. `CollateralAuthorization` and `FundingAndPolicyAdmission` belong
/// to step 4: the admitted P2WPKH sponsor's signature over frozen bytes, and
/// measured-envelope admission. Per-relation compiler evidence stays separately
/// in `members().evidence()`; this list neither replaces nor discharges it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum KeyedRequestObligation {
    Inclusion,
    Unspentness,
    Retrieval,
    Finality,
    CollateralAuthorization,
    FundingAndPolicyAdmission,
}
impl KeyedRequestObligation {
    pub const ALL: [Self; 6] = [
        Self::Inclusion,
        Self::Unspentness,
        Self::Retrieval,
        Self::Finality,
        Self::CollateralAuthorization,
        Self::FundingAndPolicyAdmission,
    ];
}
/// The structural successor position, not an assembled transaction output.
pub const KEYED_SUCCESSOR_OUTPUT: u32 = 0;
