# Tapscript Error Vocabulary · `err:tapscript:vocabulary`

> Illustrative boundary for `tripod-tapscript`.
> **Package contract:** [tapscript.md](../tapscript.md)

```rust
pub enum TapscriptError {
    UnsupportedAnalyzedProgramSchema(u32), AnalyzedProgramIdentityMismatch,
    TargetPlanIdentityMismatch, TargetIdentityMismatch, UnsupportedBackendConfiguration(u32),
    MissingRelation(realization::RelationId), MissingSelectedProof(realization::RelationId),
    UnsupportedProofAlternative(ProofAlternativeId), WeakenedProofAttempt(realization::RelationId),
    MissingTargetCapability(target_elements::ElementsCapability), MissingFactSource(realization::FactId),
    UnsupportedFactEncoding(realization::FactId), UnauthenticatedFactSource(realization::FactId),
    UnavailableWitness(realization::FactId), PermissionlessSecretDependency(realization::FactId),
    MissingPattern(ProofPatternId), DuplicatePattern(ProofPatternId), PatternCapabilityMismatch(ProofPatternId),
    PatternStackMismatch(ProofPatternId), PatternFailureContractMismatch(ProofPatternId),
    UnsupportedInstruction(target_elements::OpcodeName), InstructionEncodingFailure(InstructionId),
    StackUnderflow(InstructionId), StackTypeMismatch { instruction: InstructionId, expected: StackValueType, actual: StackValueType },
    AltstackTypeMismatch(InstructionId), BranchStackMismatch(ProgramId), NonCanonicalFinalTruth(ProgramId),
    StackLimitExceeded(ProgramId), InitialStackLimitExceeded(ProgramId), StackElementSizeExceeded(ProgramId),
    ScriptSizeLimitExceeded(ProgramId), CryptoBudgetExceeded(ProgramId),
    ConfidentialClosedAssetForbidden, UnauthenticatedPublicOpening(realization::FactId),
    MissingPublicOpening(realization::FactId), UnsupportedSignaturePattern(AuthorizationId),
    OutputCommitmentNotEstablished(AuthorizationId),
    ForbiddenSignatureOnPermissionlessPath { operation: architecture::OperationId },
    ArithmeticBoundNotEstablished(realization::RelationId), NarrowArithmeticOverflowPossible(realization::RelationId),
    WideArithmeticPatternUnavailable(realization::RelationId), ArithmeticSuccessFlagUnchecked(ProgramId),
    MissingConstructor(architecture::ObjectId), ConstructorContinuityUnresolved(architecture::ObjectId),
    ConstructorPrototypeRequired(architecture::ObjectId), MetadataPathSpendable(architecture::ObjectId),
    InternalKeyPolicyMissing(architecture::ObjectId), MissingPlacement(realization::RelationId),
    InvalidPlacement(realization::RelationId), UnreachableCarrier(realization::RelationId),
    MissingCoordinator(architecture::OperationId), AmbiguousCoordinator(architecture::OperationId),
    DuplicateSymbol(SymbolId), UnknownSymbol(SymbolId), DuplicateRelocation(RelocationId),
    InvalidRelocation(RelocationId), MissingRelocationValue(RelocationId),
    StackScheduleFailure(ProgramId), PeepholePreconditionFailed(PeepholeRuleId),
    RewriteChangedStackContract(PeepholeRuleId), AmbiguousPatternSelection(realization::RelationId),
    NonDeterministicPatternSelection, NonDeterministicEmission,
}
```

A compiler relation without a target proof is an emission failure, never a
silent weakening. Final symbol resolution belongs to linker; concrete witness
failures belong to transaction; target-node verdict mismatches belong to vectors.
