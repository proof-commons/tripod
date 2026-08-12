# Labels Error Vocabulary · `err:labels:vocabulary`

> Illustrative boundary for `tripod-labels`.
> **Package contract:** [labels.md](../labels.md)

```rust
pub enum LabelErrorCode {
    Io, UnclosedMarkdownFence, UnclosedInlineCode, AsymmetricCitation,
    DuplicateMint, MissingMint,
    UnknownOwner, UnknownImportedLabel, InvalidImportedCitationForm,
    InvalidLabel, DuplicateLatexLabel,
    ArchitectureLabelMissing, AttestationAnchorSetMismatch, AttestationIndexStale,
    GeneratedRegisterMissing, GeneratedRegisterStale,
}

pub enum GenerateError {
    Validation(Vec<LabelDiagnostic>),
    Io(std::io::Error),
    Json(serde_json::Error),
}
```

A stale publication is a validation diagnostic, not I/O. Check mode reports it
without writing; generation may repair owned publications only after source
relationships validate. Every owner's duplicate mints and unresolved citations
fail — planning and documentation labels included, under
(`[ADR013-inv:labels:total-resolution]`) — and a `AttestationIndexStale`
diagnostic means the committed upward-citation index no longer presents
exactly the body's anchor set.
