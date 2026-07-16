# Vectors Error Vocabulary · `err:vectors:vocabulary`

> Illustrative boundary for `tripod-vectors`.
> **Package contract:** [vectors.md](../vectors.md)

```rust
pub enum VectorError {
    UnsupportedSemanticVectorSchema(u32), UnsupportedTargetVectorSchema(u32),
    UnsupportedReportSchema { kind: EvidenceKind, schema: u32 },
    ArchitectureIdentityMismatch, RealizationIdentityMismatch, AnalyzedProgramIdentityMismatch,
    TargetIdentityMismatch, BundleIdentityMismatch, AbiIdentityMismatch, VectorSetIdentityMismatch,
    MissingRelation(realization::RelationId), UnexpectedRelation(realization::RelationId),
    RelationCensusMismatch, MissingCarrier(realization::RelationId),
    UnreachableCarrier(realization::RelationId), DuplicateIncompatibleCarrier(realization::RelationId),
    InvalidSemanticFixture(SemanticFixtureId), ModelExecutionFailed(SemanticFixtureId),
    ModelInvariantFailure(SemanticFixtureId), FactProjectionFailed(SemanticFixtureId),
    RelationActivationFailed(realization::RelationId), RelationEvaluationFailed(realization::RelationId),
    DuplicateVector(VectorId), DuplicateMutation(MutationId), InvalidSemanticVector(VectorId),
    InvalidMutation(MutationId), MutationDidNotChangeIntendedRelation(MutationId),
    MutationChangedUnexpectedFacts(MutationId), MutationCollateralSetMismatch(MutationId),
    StrictIndependenceNotEstablished(MutationId), TargetMaterializationFailed(TargetVectorId),
    ConstructorMutationRejectedBeforeTarget(TargetVectorId), SigningFixtureFailed(TargetVectorId),
    ProofFixtureFailed(TargetVectorId), MutationNotMaterializable(MutationId),
    TargetExecutorUnavailable, TargetEnvironmentIdentityMismatch, TargetActivationMismatch,
    TargetInfrastructureFailure(TargetVectorId), ConsensusExecutionFailure(TargetVectorId),
    PolicyExecutionFailure(TargetVectorId), ExpectedAcceptActualReject(TargetVectorId),
    ExpectedRejectActualAccept(TargetVectorId), ConstructionFailureMisclassified(TargetVectorId),
    TargetFailureMisclassified(TargetVectorId), AcceptedProjectionMismatch(TargetVectorId),
    RejectedStateMutation(TargetVectorId), MissingPositiveCoverage(realization::RelationId),
    MissingNegativeCoverage(realization::RelationId), MissingInactiveConditionalCase(realization::RelationId),
    MissingActiveConditionalCase(realization::RelationId), MissingInvalidConditionalCase(realization::RelationId),
    CarrierNotExecuted(realization::RelationId), RepresentationSafetyFailure(TargetVectorId),
    RepresentationMinimalityFailure(TargetVectorId), ConfidentialClosedAssetAccepted(TargetVectorId),
    PermissionlessPrivateWitness(TargetVectorId), ResourcePredictionMissing(TargetVectorId),
    ResourceMeasurementFailed(TargetVectorId), ResourcePredictionMismatch(TargetVectorId),
    TargetResourceLimitExceeded(TargetVectorId), PolicyLimitExceeded(TargetVectorId),
    ExternalReportMalformed(EvidenceKind), ExternalReportIdentityMismatch(EvidenceKind),
    ExternalReportFailed(EvidenceKind), IndependentImplementationRequirementNotMet(EvidenceKind),
    EventProjectionMismatch, AttestationQueryMismatch, ReceiptAccountingMismatch,
    ShrinkFailed(TargetVectorId), ShrinkLostFailure(TargetVectorId),
    NonDeterministicVectorGeneration, NonDeterministicMutation, NonDeterministicReport,
}

pub enum VectorOutcome {
    ExpectedAcceptActualAccept, ExpectedRejectActualReject,
    ExpectedAcceptActualReject, ExpectedRejectActualAccept,
    PlanningUnsupported, ConstructionRejected, SigningFailed, ProofConstructionFailed,
    TargetInfrastructureFailed, TargetExecutionFailed, ResourceLimitFailed, ProjectionMismatch,
}
```

Only the first two outcomes are successful verdict matches. Construction
rejection counts as target-negative evidence only when construction is the claim.
