# Simplicity Error Vocabulary · `err:simplicity:vocabulary`

> Illustrative parked boundary for `tripod-simplicity`.
> **Package contract:** [simplicity.md](../simplicity.md)

```rust
pub enum SimplicityError {
    BackendParked, UnsupportedTarget, TargetIdentityMismatch,
    UnsupportedAnalyzedProgramSchema(u32), MissingRelation(realization::RelationId),
    MissingSelectedProof(realization::RelationId), MissingTargetCapability(SimplicityCapability),
    UnsupportedProofAlternative(ProofAlternativeId), ProgramTypeFailure(ProgramId),
    JetUnavailable(JetId), JetContractMismatch(JetId),
    MissingRelationCarrier(realization::RelationId),
    ConstructorStrategyUnavailable(architecture::ObjectId),
    UnsupportedRepresentation(RepresentationRequirementId),
    PermissionlessSecretDependency(realization::FactId),
    LayoutUnavailable(architecture::OperationId), WitnessAbiUnavailable(ProgramId),
    ResourceFormulaUnavailable(ProgramId), TargetResourceLimitExceeded(ProgramId),
    DuplicateSymbol(SymbolId), UnresolvedTargetReference(TargetReferenceId),
    NonDeterministicEmission,
}
```

The package is parked. This sketch preserves semantic failure classes without
pretending tapscript stacks, tapleaves, control blocks, or taptrees are universal.
