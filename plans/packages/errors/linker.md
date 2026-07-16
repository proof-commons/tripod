# Linker Error Vocabulary · `err:linker:vocabulary`

> Illustrative boundary for `tripod-linker`.
> **Package contract:** [linker.md](../linker.md)

```rust
pub enum LinkError {
    UnsupportedRelocatableSchema(u32), UnsupportedLinkedBundleSchema(u32),
    ArchitectureIdentityMismatch, RealizationIdentityMismatch, AnalyzedProgramIdentityMismatch,
    TargetIdentityMismatch, BackendConfigurationMismatch, LinkerConfigurationMismatch,
    InvalidDeploymentParameters, ZeroNetworkId, ZeroGenesisId,
    MissingDeploymentKey(DeploymentKeyRole), InvalidDeploymentKey(DeploymentKeyRole),
    MissingAssetId(architecture::AssetId), InvalidAssetId(architecture::AssetId),
    MissingBound(architecture::BoundId), DuplicateBound(architecture::BoundId),
    UnexpectedBound(architecture::BoundId), ZeroBound(architecture::BoundId),
    BoundBelowMinimum(architecture::BoundId), BoundNotRepresentable(architecture::BoundId),
    BoundIncompatibleWithLayout(architecture::BoundId), DuplicateSymbol(SymbolId),
    MissingSymbol(SymbolId), AmbiguousSymbol(SymbolId), IncompatibleSymbolType { symbol: SymbolId, expected: SymbolType, actual: SymbolType },
    UnknownReferenceTarget(ReferenceEdgeId), InvalidReferenceGraph,
    ReferenceGraphCycle(Vec<ReferenceNodeId>), UnsupportedReferenceCycle(SccId),
    MissingReferenceStrategy(ReferenceEdgeId), ImpossibleStaticFixedPoint(SccId),
    ConstructorContinuityUnresolved(architecture::ObjectId), DuplicateRelocation(RelocationId),
    UnknownRelocation(RelocationId), RelocationAlreadyApplied(RelocationId),
    RelocationTypeMismatch(RelocationId), RelocationEncodingMismatch(RelocationId),
    RelocationWidthMismatch(RelocationId), RelocationPlaceholderMismatch(RelocationId),
    UnresolvedRelocation(RelocationId), UntrackedProgramMutation(ProgramId),
    MissingConstructor(architecture::ObjectId), InvalidConstructor(architecture::ObjectId),
    MissingProgram(ProgramId), DuplicateProgram(ProgramId), InvalidLinkedProgram(ProgramId),
    MetadataPathSpendable(architecture::ObjectId), MissingLeafWeight(ProgramId),
    InvalidLeafVersion(ProgramId), DuplicateLeaf(ProgramId), MissingLeaf(ProgramId),
    TreeDepthExceeded(ProgramId), NonDeterministicTree, MissingRelationCarrier(realization::RelationId),
    DuplicateIncompatibleCarrier(realization::RelationId), UnreachableRelationCarrier(realization::RelationId),
    CarrierCensusMismatch, MissingResourceFormula(ProgramId), ResourceFormulaMismatch(ProgramId),
    StaticTargetLimitExceeded(ProgramId), CandidateBundleUsedAsFinal,
    CalibrationIdentityMismatch, CalibrationIncomplete, CalibrationStale,
    CalibrationBundleMismatch, CalibrationAbiMismatch, TargetLimitExceeded(ResourceDimension),
    NonDeterministicGraph, NonDeterministicLink, LinkedBundleIdentityMismatch,
}
```

The linker owns graph and relocation failures, not semantic formulas already
owned by realization/compiler. Candidate-to-final confusion remains typed.
