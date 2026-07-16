# Realization Error Vocabulary · `err:realization:vocabulary`

> Illustrative boundary for `tripod-realization`.
> **Package contract:** [realization.md](../realization.md)

```rust
pub enum RealizationError {
    UnsupportedArchitectureSchema(u32), ArchitectureValidationFailed,
    ArchitectureIdentityMismatch, EmptyScope, UnsupportedScope,
    IncompleteScope { operation: architecture::OperationId },
    OperationOutsideScope { operation: architecture::OperationId },
    MissingObjectRealization(architecture::ObjectId),
    UnexpectedObjectRealization(architecture::ObjectId),
    MissingOperationRealization(architecture::OperationId),
    UnexpectedOperationRealization(architecture::OperationId),
    DuplicateFact(FactId), DuplicateExpression(ExprId), DuplicateRelation(RelationId),
    DuplicateObservable(ObservableId), DuplicateProofAlternative(ProofAlternativeId),
    UnknownFact(FactId), UnknownExpression(ExprId), UnknownRelation(RelationId),
    TypeMismatch { expression: ExprId, expected: SemanticType, actual: SemanticType },
    InvalidConstant { expression: ExprId, domain: SemanticType },
    ArithmeticDomainMismatch(ExprId), DependencyCycle { nodes: Vec<ExprId> },
    InputFamilyMismatch { operation: architecture::OperationId },
    OutputFamilyMismatch { operation: architecture::OperationId },
    AuthorizationMismatch { operation: architecture::OperationId },
    ProjectionMismatch { operation: architecture::OperationId },
    ValueFlowMismatch { operation: architecture::OperationId },
    MissingRelation { operation: architecture::OperationId, kind: RequiredRelationKind },
    MissingStateAssignment(StateFieldId), ConflictingStateAssignment(StateFieldId),
    MissingWitnessAvailability(FactId),
    PermissionlessPrivateWitness { operation: architecture::OperationId, fact: FactId },
    MissingLifecycleExit { object: architecture::ObjectId, exit: architecture::OperationId },
    MissingDisclosureReason(FactId), UndeclaredDisclosure(FactId),
    DeclassificationCycle(Vec<FactId>), StructuralIdentityCollision { id: StructuralId },
    NonDeterministicDerivation,
}
```

Architecture family mismatches belong here because realization validates its
coverage. Target-capability and model-conformance failures remain downstream. A
pilot that lacks full scope reports `IncompleteScope`, not malformed input.
