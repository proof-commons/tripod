# Elements Target Error Vocabulary · `err:target-elements:vocabulary`

> Illustrative boundary for `tripod-target-elements`.
> **Package contract:** [target-elements.md](../target-elements.md)

```rust
pub enum TargetError {
    UnsupportedTargetSchema(u32), InvalidTargetDefinition, TargetDefinitionIdentityMismatch,
    MissingReviewProvenance(TargetClaimId), InvalidReviewProvenance(TargetClaimId),
    MissingLicenseProvenance, MissingSourceLocation(TargetClaimId),
    DuplicateOpcodeCode(u8), DuplicateOpcodeName(OpcodeName), MissingOpcodeDefinition(OpcodeName),
    MissingExecutionDomain(OpcodeName), MissingStackContract(OpcodeName),
    MissingFailureSemantics(OpcodeName), MissingResourceCost(OpcodeName),
    ConflictingOpcodeDefinition(OpcodeName), InvalidOperandContract(OpcodeName),
    InvalidResultContract(OpcodeName), InvalidFailureStackContract(OpcodeName),
    UnsupportedExecutionDomain { opcode: OpcodeName, domain: ExecutionDomain },
    DuplicateEncodingPrefix(u8), UnknownEncodingPrefix(u8), ConflictingEncodingClass(u8),
    MissingByteOrder(FieldKind), InvalidFixedWidth { field: FieldKind, width: usize },
    MissingCanonicalEncoding(FieldKind), MissingCapabilityProvenance(ElementsCapability),
    ConflictingCapabilityDeclaration(ElementsCapability), UnsupportedCapability(ElementsCapability),
    ProofPatternNotAvailable(ElementsCapability), UnsupportedSighashMode(SighashMode),
    SighashDoesNotCommitRequiredOutputs(SighashMode), InvalidTimelockDeclaration,
    IncompleteConfidentialValueCapability, ConfidentialAssetClassificationUnsupported,
    AuthenticatedOpeningPatternUnavailable, IncompleteIssuanceDeclaration,
    MissingConsensusLimit(ResourceDimension), MissingPolicyLimit(ResourceDimension),
    IncompleteCryptoBudgetRule, UnresolvedInitialWitnessPolicy,
    MissingEvidenceRequirement(ElementsCapability), UnsupportedNetworkFlavor(ElementsNetworkFlavor),
    ZeroNetworkId, ZeroGenesisId, MissingActivationBinding, ProductionEvidenceMissing,
    ProtocolPolicyLeak { field: String }, NonDeterministicTargetDefinition,
    NonDeterministicDeploymentIdentity,
}
```

Missing production evidence prevents a production deployment binding; it does
not rewrite the static target contract. Primitive availability and a complete
proof pattern remain distinct statuses.
