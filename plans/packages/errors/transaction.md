# Transaction Error Vocabulary · `err:transaction:vocabulary`

> Illustrative boundary for `tripod-transaction`.
> **Package contract:** [transaction.md](../transaction.md)

```rust
pub enum TransactionError {
    UnsupportedAbiSchema(u32), CandidateBundleUsedAsFinal, CandidateAbiUsedAsFinal,
    TargetIdentityMismatch, BundleIdentityMismatch, AbiIdentityMismatch, BoundAssignmentMismatch,
    UnsupportedOperation(architecture::OperationId), MissingOperationAbi(architecture::OperationId),
    WrongEnforcementClass { operation: architecture::OperationId, expected: OperationEnforcementClass, actual: OperationEnforcementClass },
    MissingInput(OutPoint), DuplicateInput(OutPoint), InputAssignedTwice(OutPoint),
    InputFamilyMismatch { outpoint: OutPoint, expected: InputFamilyId },
    WrongInputObject(OutPoint), WrongInputAsset(OutPoint), WrongInputRepresentation(OutPoint),
    StaleInput(OutPoint), CheckpointMismatch, TooFewInputs(InputFamilyId), TooManyInputs(InputFamilyId),
    TooFewOutputs(OutputFamilyId), TooManyOutputs(OutputFamilyId), InvalidFamilyOrder,
    InvalidFamilyRange(FamilyId), FamilyRangeOverlap { first: FamilyId, second: FamilyId },
    SponsorRegionOverlap, WrongCoordinator, InvalidOperationRequest(architecture::OperationId),
    UnauthorizedRecipientChoice, UnsupportedRepresentation,
    MissingLifecyclePath { object: architecture::ObjectId, exit: architecture::OperationId },
    FormulaEvaluationFailure(realization::ExprId), ValueConservationFailure(realization::RelationId),
    StateAssignmentFailure(StateFieldId), MissingPublicFact(realization::FactId),
    MissingPublicOpening(OutPoint), InvalidPublicOpening(OutPoint),
    PermissionlessPrivateWitness(realization::FactId), MissingOwnerWitness(OwnerKey),
    MissingOperatorWitness, MissingSponsorWitness(OutPoint),
    ConstructorNotFound(architecture::ObjectId), ConstructorIdentityMismatch(architecture::ObjectId),
    MetadataSchemaMismatch(architecture::ObjectId), MetadataEncodingFailure(architecture::ObjectId),
    TargetProgramMismatch(ProgramId), ControlPathMismatch(ProgramId), InvalidLeafSelection(ProgramId),
    ConfidentialClosedAssetForbidden, MissingBlindingData(OutPoint), RandomnessUnavailable(RandomnessPurpose),
    BlindingBalanceFailure, RangeproofConstructionFailed(OutputFamilyId),
    SurjectionProofConstructionFailed(OutputFamilyId), ResidualBlindingUnrouted,
    UnsupportedDataOutput(DataOutputFamilyId), NonCanonicalRecordOrdinal, DuplicateRecordOrdinal(u32),
    DataOutputOrderMismatch, TransactionVersionMismatch, LocktimeMismatch, SequenceMismatch(OutPoint), FeeMismatch,
    SigningRequestFailure(SigningRoleId), SigningRoleMismatch(SigningRoleId),
    SignatureTransactionMismatch(SigningRoleId), MissingSignature(SigningRoleId),
    DuplicateSignature(SigningRoleId), UnexpectedSignature(SigningRoleId), InvalidSignature(SigningRoleId),
    OutputChangedAfterSigning, MissingWitness(WitnessItemId), UnexpectedWitness(WitnessItemId),
    WitnessEncodingFailure(WitnessItemId), WitnessOrderMismatch(ProgramId), WitnessSizeExceeded(WitnessItemId),
    ResourcePredictionMissing(architecture::OperationId), PredictedTargetLimitExceeded(ResourceDimension),
    ConstructionIdentityMismatch, FinalTransactionIdentityMismatch, NonDeterministicAbi, NonDeterministicConstruction,
}
```

Construction rejection is distinct from target-script rejection. Secret-related
variants identify roles and public outpoints only, never secret material.
