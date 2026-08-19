//! The three-part shape rule, enforced at every label-intended entry.
//!
//! A label is `kind:area:name` and nothing else. The markdown and Rust
//! surfaces have demanded three segments structurally for as long as
//! their owner shapes have; the LaTeX surface did not, and nothing in
//! the checker guaranteed that a two-segment token failed *wherever* it
//! was written. This module holds the arity rule itself, so the
//! guarantee is one decision in one place rather than a property that
//! happened to hold shape by shape.
//!
//! # Survey: where a label token enters the system
//!
//! Each row is a path that reads a token, and what a two-segment token
//! did there before this rule. `a` fails loudly with a location, `b` is
//! silently read as something other than a label, `c` is harvested as a
//! valid label.
//!
//! | Entry point | Before | Now |
//! |---|---|---|
//! | `latex::harvest` — `\label{...}` arguments in attestation sources | c | a: `MalformedLabelShape` at the label's line |
//! | `latex::macro_labels` — `\OpenProblem`, `\ProposedInvestigation` prefix synthesis | c | a: the synthesized token is checked like any other |
//! | `latex::open_subproblem_labels` — the optional-argument form | c | a |
//! | `latex::insert` — values carrying `#` | b | b: unchanged, decided below |
//! | `rust_source::harvest_label` — acute spans whose first segment is a model type | a | a, now coded as a shape defect |
//! | `rust_source::harvest_label` — acute spans with any other first segment | a | a: unknown Rust label type, unchanged |
//! | `repository` markdown harvests — bare and parenthesized spans in the realization document, the records, and the planning tree | b | b: unchanged, decided below |
//! | `repository::harvest_realization_import`, `import` — bracketed imported citations | a, except c under the `RZ` prefix, whose shape admitted two segments | a for every owner alike, coded as a shape defect |
//! | `repository::add_architecture_citations` — synthetic citations from typed witness and clause data | c: the designated classes target `RZ`, so a two-segment tag resolved | a, coded as a shape defect |
//! | `heads::parse_head` — the delimited label of an environment head | b | b: unchanged, decided below |
//! | `nearmiss::classify` — repaired candidate spans | b | b: a repair is tested against the three-segment planning shape, so a near miss is warned about and never failed |
//! | `check`, `render` — generated registers | b | b: registers are compared as bytes and mint nothing |
//! | `adoption` — the pair and package tables | b | b: those tables carry names and kinds, never labels |
//!
//! # The decisions the `b` rows record
//!
//! **Prose and code spans that fail to parse are text.** This is the
//! calculus's own answer and not an oversight: a span is an occurrence
//! only when the owner's grammar accepts it, so a two-segment span in a
//! record or a plan is ordinary inline code. Erroring there would mean
//! erroring on every colon-bearing span in the corpus. The near-miss
//! families warn where a reader would nonetheless read a label, and
//! this rule never fires where they do — see the coordination rule
//! below.
//!
//! **A head declines rather than fails.** Head recognition ends at a
//! label that parses, and declining is not a defect there: the leader
//! grammar is narrow precisely so that bold prose emphasis forms no
//! judgment. A head whose label is two-segment is therefore read as
//! emphasis, and its span falls to text by the rule above — one
//! reading, not two. What that costs is visibility, not soundness: the
//! label mints nothing either way, so nothing false enters the
//! registry.
//!
//! **Macro bodies are not occurrences.** A `\label{}` inside a macro
//! *definition* carries parameter tokens rather than a name, and the
//! harvest declines any value containing `#` before parsing it. The
//! definition mints nothing; each *use* of the macro is the occurrence,
//! and the use is checked. So `invest:attestation:#2` in the macro file is
//! read as a body, not as a label, and this rule cannot reach it.
//!
//! **Registers are bytes.** A generated register is checked by
//! comparing it against a re-render of the registry it indexes, so its
//! tokens are never re-parsed and cannot smuggle a shape past the rule
//! that governed their mint.
//!
//! # Coordination with the near-miss warnings
//!
//! One occurrence yields at most one of the two. The near-miss families
//! see only spans the harvest let fall to text, and every family tests
//! its repaired span against the three-segment planning shape — so a
//! near miss is by construction a *non*-occurrence, while this rule
//! fires only where the surface has already committed the token to
//! being label-intended. A backtick span in comment text is warned
//! about and never failed; an acute span there is failed and never
//! warned about.
//!
//! # The realization document's top-level divisions
//!
//! The realization contract once carried twenty two-segment anchors,
//! its sections and its one appendix, minted before the shape was
//! settled and cited across the corpus. They were frozen here by name
//! until the migration wave retired them: each takes the document's
//! own-division area, `realization`, a division's home being the
//! document itself, exactly as the paper's divisions take
//! `attestation`. The rule is
//! therefore total and carries no enumerated residue — a two-segment
//! token in that document now fails like any other, with no list to
//! consult. The rename moved no identity: labels are presentation
//! under the denotation law, and the calculus's presentation
//! invariance metatheorem is why nothing that hashes the corpus
//! moved.

/// The form every label takes, quoted in the diagnostic.
pub const EXPECTED_FORM: &str = "kind:area:name";

/// The number of colon-separated segments a label carries.
pub const SEGMENTS: usize = 3;

/// Whether a token carries an admissible number of segments.
///
/// Every owner answers the same way, by arity alone: there is no
/// surface exemption and no enumerated residue to consult.
pub const fn arity_admitted(parts: usize) -> bool {
    parts == SEGMENTS
}
