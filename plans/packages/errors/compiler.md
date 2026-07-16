# Compiler Error Vocabulary · `err:compiler:vocabulary`

> Illustrative boundary for `tripod-compiler`.
> **Package contract:** [compiler.md](../compiler.md)

```rust
pub enum CompileError {
    UnsupportedRealizationSchema(u32), InvalidRealization, RealizationIdentityMismatch,
    IncompleteRealizationScope { operation: architecture::OperationId },
    DuplicateAnalysisNode(AnalysisNodeId), DuplicateProofPlanNode(ProofPlanNodeId),
    DuplicateDisclosureRequirement(DisclosureRequirementId),
    UnknownFact(realization::FactId), UnknownExpression(realization::ExprId),
    UnknownRelation(realization::RelationId), AnalysisDependencyCycle(Vec<AnalysisNodeId>),
    RelationDropped(realization::RelationId), RelationOutsideScope(realization::RelationId),
    ConstantFoldTypeMismatch(AnalysisNodeId), ConstantFoldOverflow(AnalysisNodeId),
    ConstantFoldChangedFailureSemantics(AnalysisNodeId),
    MissingProofAlternative(realization::RelationId),
    UnsupportedProofAlternative(ProofAlternativeId),
    WeakenedProofAlternative(realization::RelationId),
    NoSupportedProofPlan(realization::RelationId),
    AmbiguousProofSelection { relation: realization::RelationId, candidates: Vec<ProofAlternativeId> },
    MissingRequiredDisclosure(realization::FactId), MissingDisclosureReason(realization::FactId),
    UnsupportedMinimalityClaim(realization::FactId), PolicyDisclosureNotDeclared(realization::FactId),
    MissingFactSource(realization::FactId), UnauthenticatedFactSource(realization::FactId),
    ConflictingFactSources(realization::FactId), UnavailableWitness(realization::FactId),
    PermissionlessSecretDependency { operation: architecture::OperationId, fact: realization::FactId },
    MissingLifecyclePath { object: architecture::ObjectId, exit: architecture::OperationId },
    UnsupportedRepresentation { operation: architecture::OperationId, representation: RepresentationModeId },
    MissingPlacement(realization::RelationId), UnconditionalRelationOnOptionalCarrier(realization::RelationId),
    GlobalRelationHasOnlyLocalCarrier(realization::RelationId), AmbiguousCarrierSelection(realization::RelationId),
    UnboundedTargetFamily { operation: architecture::OperationId, family: FamilyId },
    MissingLayoutRequirement(realization::RelationId), MissingSponsorIsolation { operation: architecture::OperationId },
    UnsupportedTargetCapability(RequiredCapability), IncompleteTargetRequirement(RequiredCapability),
    MissingCoverageRequirement(realization::RelationId), MissingPositiveCase(realization::RelationId),
    MissingNegativeCase(realization::RelationId), StructuralIdentityCollision(AnalysisNodeId),
    NonDeterministicAnalysis, NonDeterministicProofSelection,
}
```

Unsupported target capability means no valid target plan exists; it never
permits dropping a relation. Backend-pattern absence and target execution
evidence belong to their downstream owners.
