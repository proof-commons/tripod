# Release Error Vocabulary · `err:release:vocabulary`

> Illustrative boundary for `tripod-release`.
> **Package contract:** [release.md](../release.md)

```rust
pub enum ReleaseError {
    UnsupportedReleasePolicy(ReleasePolicyId), UnsupportedReleaseManifestSchema(u32),
    UnsupportedDeploymentProfileSchema(u32), MissingReleaseDate, MissingSourceRevision,
    SourceRevisionMismatch, SourceTreeDirty, CandidateArtifactNotFinal(ArtifactIdentity), NoWaiverPolicy,
    AttestationPinMismatch, ArchitectureReleaseFailed(Vec<architecture::ManifestError>),
    ArchitectureEnvelopeInvalid, ArchitectureSemanticHashMismatch,
    ArchitectureBehaviouralHashMismatch, BehaviouralVersionGateFailed,
    ArchitecturePublicationMissing(PublicationAssetKind), ArchitecturePublicationMismatch(PublicationAssetKind),
    RealizationDocumentWeldFailed, RealizationIdentityMismatch, RealizationSchemaUnsupported(u32),
    RealizationScopeIncomplete(architecture::OperationId), RealizationValidationFailed,
    ModelConformanceReportMissing, ModelConformanceReportFailed, ModelConformanceIdentityMismatch,
    UnitTestReportMissing, UnitTestReportFailed, PropertyTestReportMissing, PropertyTestReportFailed,
    CompilerIdentityMismatch, CompilerConfigurationMissing, CompilerScopeMismatch,
    CompilerRelationCensusMismatch, CompilerAnalysisIncomplete, CompilerReportMissing,
    CompilerReportFailed, TargetPlanIncomplete, TargetDefinitionIdentityMismatch,
    DeploymentInstanceMismatch, NetworkIdMismatch, GenesisIdMismatch, ActivationBindingMissing,
    ActivationEvidenceFailed, DevelopmentTargetUsedForProduction, ProductionTargetEvidenceMissing,
    BackendConfigurationMismatch, RelocatableBundleIdentityMismatch, LinkedBundleNotFinal,
    LinkedBundleIdentityMismatch, LinkedBundleScopeIncomplete, UnresolvedLinkedReference,
    UnresolvedRelocation, ConstructorCensusIncomplete, RelationCarrierCensusMismatch,
    UnreachableRelationCarrier(realization::RelationId), TransactionAbiIdentityMismatch,
    TransactionAbiBundleMismatch, TransactionAbiTargetMismatch, TransactionAbiBoundsMismatch,
    TransactionAbiScopeIncomplete, WitnessSchemaIncomplete, MetadataSchemaIncomplete,
    UnsupportedRepresentationClaim(RepresentationModeId),
    PermissionlessConstructibilityFailed(architecture::OperationId),
    LifecycleIncomplete { object: architecture::ObjectId, exit: architecture::OperationId },
    MissingBoundCalibration(architecture::BoundId), DuplicateBoundCalibration(architecture::BoundId),
    UnexpectedBoundCalibration(architecture::BoundId), InvalidBoundCalibration(architecture::BoundId),
    BoundBelowManifestMinimum(architecture::BoundId), CalibrationBundleMismatch,
    CalibrationAbiMismatch, CalibrationReportStale,
    SharedBoundOperationMissing { bound: architecture::BoundId, operation: architecture::OperationId },
    WorstCaseFixtureInvalid(architecture::OperationId), ResourcePredictionMismatch(architecture::OperationId),
    TargetResourceLimitExceeded(architecture::OperationId), FinalRemeasurementMissing,
    MissingDependencyEvidence(architecture::DependencyId), DuplicateDependencyEvidence(architecture::DependencyId),
    DependencyEvidenceFailed(architecture::DependencyId), DependencyEvidenceIncomplete(architecture::DependencyId),
    DependencyEvidenceIdentityMismatch(architecture::DependencyId), ZeroDependencyEvidenceHash(architecture::DependencyId),
    MissingEvidenceReport(EvidenceKind), DuplicateEvidenceReport(EvidenceKind),
    EvidenceReportFailed(EvidenceKind), EvidenceReportIncomplete(EvidenceKind),
    EvidenceReportUnsupported(EvidenceKind), EvidenceReportSkipped(EvidenceKind),
    EvidenceInfrastructureError(EvidenceKind), EvidenceIdentityMismatch(EvidenceKind),
    EvidenceSchemaUnsupported(EvidenceKind), ZeroEvidenceHash(EvidenceKind), EvidenceReportStale(EvidenceKind),
    RelationCoverageIncomplete, MissingPositiveCoverage(realization::RelationId),
    MissingNegativeCoverage(realization::RelationId), VerdictMismatch(TargetVectorId),
    AcceptedProjectionMismatch(TargetVectorId), RepresentationSafetyReportFailed,
    RepresentationMinimalityReportMissing, RepresentationMinimalityReportFailed,
    ScriptIntegrationReportMissing, ScriptIntegrationReportFailed,
    IndependentEventReportMissing, IndependentEventReportFailed,
    IndependentQueryReportMissing, IndependentQueryReportFailed,
    IndependentAccountingReportMissing, IndependentAccountingReportFailed,
    IndependentObserverIdentityMismatch(EvidenceKind), IndependenceDeclarationMissing(EvidenceKind),
    IndependenceRequirementNotMet(EvidenceKind), MissingPublicationAsset(PublicationAssetKind),
    DuplicatePublicationAsset(PublicationAssetKind), UnexpectedPublicationAsset(String),
    DuplicatePublicationPath(String), UnsafePublicationPath(String),
    GeneratedArtifactStale(PublicationAssetKind), GeneratedArtifactMismatch(PublicationAssetKind),
    MissingHashRecipe(PublicationAssetKind), ArtifactHashMismatch(PublicationAssetKind),
    SourceTreeHashFailure, ArchiveHashMismatch, NonCanonicalArchive,
    NonReproducibleArtifact(PublicationAssetKind), DeploymentProfileConstructionFailed,
    DeploymentReleaseValidationFailed(Vec<architecture::DeploymentError>),
    DeploymentProfileHashMismatch, DeploymentProfilePublicationMismatch,
    ReleaseManifestIdentityMismatch, ReleaseAssetCensusMismatch, DestinationExists(PathBuf),
    PublicationStagingFailed, PublicationVerificationFailed, AtomicPublicationFailed,
    PartialPublicationDetected, NonDeterministicReleaseAssembly, NonDeterministicReleasePublication,
}
```

## Release states · `sec:release-errors:states`

Preserve `ReleaseInputs`, `ReleaseCandidate`, `ValidatedRelease`, and
`PublishedReleaseReceipt` as distinct states. Candidate-to-final misuse is a
specific release-state error, not generic validation failure.

Release never repairs missing semantics, programs, reports, or hashes. A
required skipped lane is not success; a passed report with the wrong bundle or
ABI is an identity mismatch; independent event, query, and accounting failures
remain separate. No catch-all waiver converts an error into final status.
