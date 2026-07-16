# Labels Error Vocabulary · `err:labels:vocabulary`

> Illustrative boundary for `tripod-labels`.
> **Package contract:** [labels.md](../labels.md)

```rust
pub enum LabelErrorCode {
    ReadSource, InvalidUtf8, SourceOutsideRepository,
    UnclosedMarkdownFence, UnclosedInlineCode, AsymmetricCitation,
    MalformedMarkdownLabel, DuplicateMint, MissingMint,
    InvalidOwnerLocalLabel, UnknownImportedOwner, UnknownImportedLabel,
    InvalidImportedCitationForm, ImportedLabelMintedLocally,
    InvalidLatexLabel, DuplicateLatexLabel, MalformedAttestationMacroInvocation,
    UnclosedAcuteDelimiter, UnknownModelLabelType, MissingRealizationCitation,
    ManifestLabelMissing, AttestationAnchorSetMismatch,
    GeneratedRegisterMissing, GeneratedRegisterStale,
    ModelLabelPublicationMissing, ModelLabelPublicationStale,
}

pub enum LabelToolError {
    InvalidRepositoryRoot(PathBuf),
    Read { path: PathBuf, source: std::io::Error },
    ListDirectory { path: PathBuf, source: std::io::Error },
    Render { artifact: LabelArtifactKind, source: serde_json::Error },
    CreateDirectory { path: PathBuf, source: std::io::Error },
    Write { path: PathBuf, source: std::io::Error },
    SourceValidation { diagnostics: Vec<LabelDiagnostic> },
}
```

A stale publication is a validation diagnostic, not I/O. Check mode reports it
without writing; generation may repair owned publications only after source
relationships validate. Plan-local duplicate or dangling labels are silent by
policy, while unknown square-bracket imports fail because they claim upstream
authority.
