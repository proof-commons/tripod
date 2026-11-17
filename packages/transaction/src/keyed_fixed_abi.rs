//! Structural keyed request obligations, before materialization or policy admission.
/// Outstanding external-evidence and step-4 construction obligations.
///
/// Four external facts (`Inclusion`, `Unspentness`, `Retrieval`, `Finality`) apply
/// to every spent input's parent, including collateral, and are owned outside
/// construction. `CollateralAuthorization` and `FundingAndPolicyAdmission` belong
/// to S6 admission: the P2WPKH sponsor's signature over frozen bytes and
/// declared-envelope policy. Per-relation compiler evidence stays separately
/// in `members().evidence()`; this list neither replaces nor discharges it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum KeyedFixedRequestObligation {
    Inclusion,
    Unspentness,
    Retrieval,
    Finality,
    CollateralAuthorization,
    FundingAndPolicyAdmission,
    /// S8 must repeat policy observations on the final bytes.
    FinalBytePolicyObservation,
    /// Member covenant execution and every per-relation evidence premise remain open.
    MemberExecutionAndRelationEvidence,
}
impl KeyedFixedRequestObligation {
    /// Initial request/unsigned-structure obligations. Admission refines the
    /// funding-policy entry into final-byte observation and adds the explicit
    /// member-execution reminder; this is not an enumeration of every variant.
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
pub const KEYED_FIXED_SUCCESSOR_OUTPUT: u32 = 0;
