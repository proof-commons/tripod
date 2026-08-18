//! The label calculus's adoption parameters, as typed checker data.
//!
//! The calculus is parametric in seven data, and ADR-019 records this
//! repository's fixing of each. This module is that fixing expressed as
//! Rust data rather than as constants scattered across the harvest: one
//! place a reader compares against the ADR-019 adoption-parameter table.
//! The seven, in the ADR's own order, are the owner signature, the owner
//! partition, the profile signature, the reserved kinds, the designated
//! typed-data classes, the citation-index designations, and the
//! scanned-region recognition.
//!
//! # Kind vocabulary: the data-source decision
//!
//! ADR-020 adopts an archived registry draft as the kind vocabulary, and
//! adds to it a recorded extension set the ADR calls `X_A`. Neither
//! document may be edited by the checker: the draft under plans/drafts/
//! is a verbatim archive, and the ADR is hand-maintained prose.
//!
//! Two mechanisms were available: parse both documents at check time and
//! use the parse as the vocabulary, or commit an extracted table and
//! check it against the documents for exactness. This module commits the
//! table, for three reasons. A committed table is a review surface: an
//! edition swap or an `X_A` amendment shows every added and removed token
//! in the diff, where a check-time parse would widen the vocabulary
//! silently and legalize kinds no decision admitted. It keeps the
//! vocabulary inspectable without a Markdown parser in the resolution
//! path. And one mechanism then serves both sources, including the
//! hand-maintained ADR table, which most needs a diff.
//!
//! Exactness is enforced by [`verify_vocabulary_sources`], which parses
//! both documents and fails loudly, naming every token that appears in
//! one and not the other. The tables below can therefore never go stale
//! without the check going red, and regeneration is mechanical from the
//! diagnostic's own token lists.
//!
//! # Scope of kind enforcement
//!
//! ADR-020 governs the planning tree, the decision records, the
//! documentation tree, and the Rust packages. It records, under its
//! consequences, that the attestation LaTeX surface is not yet in scope and
//! carries three tokens it adjudicates none of. Kind validation is
//! therefore enforcing over the owners the ADR governs and reporting
//! only over attestation, so the checker states the ADR's recorded position
//! rather than inventing an adjudication. [`KindScope`] carries the
//! distinction, and the attestation arm becomes enforcing by one edit when
//! that surface enters scope.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    path::Path,
};

use crate::{
    diagnostic::{LabelDiagnostic, LabelErrorCode},
    owner::LabelOwner,
    registry::LabelMint,
    source::SourceLocation,
};

// ---------------------------------------------------------------------
// Parameter 1: the signature of owners (ADR-019, Sigma).
// ---------------------------------------------------------------------

/// One registered prefix of the signature, with the owner it names.
///
/// The signature is a partial map from registered prefixes to owners.
/// The calculus admits families whose prefixes are derived by a rule
/// rather than listed; this repository registers two, the numbered
/// decision records and the Cargo packages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrefixRegistration {
    /// A single prefix naming a single owner.
    Fixed(&'static str, LabelOwner),
    /// One owner per numbered decision record, the prefix derived from
    /// the filename and never written at a mint.
    NumberedRecords { stem: &'static str, digits: usize },
    /// One owner per first-party Cargo package other than the model
    /// crate, the prefix derived from the package directory name.
    Packages,
}

/// The signature of ADR-019: five fixed prefixes and two registered
/// families. Every prefix the checker resolves comes from this table,
/// and a prefix outside it belongs to no owner.
pub fn signature() -> Vec<PrefixRegistration> {
    vec![
        PrefixRegistration::Fixed("A", LabelOwner::Attestation),
        PrefixRegistration::Fixed("RZ", LabelOwner::Realization),
        PrefixRegistration::Fixed("PLAN", LabelOwner::Plan),
        PrefixRegistration::Fixed("DOC", LabelOwner::Doc),
        PrefixRegistration::Fixed("MODEL", LabelOwner::Model),
        PrefixRegistration::NumberedRecords {
            stem: "ADR",
            digits: 3,
        },
        PrefixRegistration::Packages,
    ]
}

/// The owner a registered prefix names, resolved through the signature.
/// This is the whole of the calculus's partial map from prefixes to
/// owners, and the only place a prefix becomes an owner.
pub fn owner_for_prefix(prefix: &str) -> Option<LabelOwner> {
    for registration in signature() {
        match registration {
            PrefixRegistration::Fixed(registered, owner) if registered == prefix => {
                return Some(owner);
            }
            PrefixRegistration::Fixed(..) => {}
            PrefixRegistration::NumberedRecords { stem, digits } => {
                if let Some(number) = prefix.strip_prefix(stem)
                    && number.len() == digits
                    && let Ok(number) = number.parse()
                {
                    return Some(LabelOwner::Adr(number));
                }
            }
            PrefixRegistration::Packages => {
                if let Some(owner) = package_owner_for_prefix(prefix) {
                    return Some(owner);
                }
            }
        }
    }
    None
}

/// The registered package owners: the prefix each Cargo package other
/// than the model crate is cited by, and the package directory it names.
///
/// ADR-019 left these unregistered, so package labels were citeable only
/// within themselves; registering them is this repository's DI-003
/// ruling. The prefix is derived, not chosen: uppercase the package
/// directory name and drop its hyphens, since the calculus's prefix
/// production admits capitals and digits only. The table is committed
/// rather than derived at load so that a new package is a reviewed
/// registration, and [`verify_package_registration`] fails when it
/// disagrees with the census.
pub const PACKAGE_OWNERS: &[(&str, &str)] = &[
    ("ARCHITECTURE", "architecture"),
    ("ARTIFACTS", "artifacts"),
    ("CLICOMMON", "cli-common"),
    ("COMPILER", "compiler"),
    ("DOCUMENTSTAMPS", "document-stamps"),
    ("EXECWRAP", "execwrap"),
    ("FLATTENLATEXMAIN", "flatten-latex-main"),
    ("LABELS", "labels"),
    ("REALIZATION", "realization"),
    ("TAPSCRIPT", "tapscript"),
    ("TARGETELEMENTS", "target-elements"),
    ("TARGETELEMENTSCONFORMANCE", "target-elements-conformance"),
];

/// Derive a package's registered prefix from its directory name, by the
/// rule the package family declares.
pub fn derive_package_prefix(package: &str) -> String {
    package
        .chars()
        .filter(|character| *character != '-')
        .flat_map(char::to_uppercase)
        .collect()
}

/// The package owner a registered prefix names, if any.
pub fn package_owner_for_prefix(prefix: &str) -> Option<LabelOwner> {
    PACKAGE_OWNERS
        .iter()
        .find(|(registered, _)| *registered == prefix)
        .map(|(_, package)| LabelOwner::Crate((*package).to_owned()))
}

/// The registered prefix of a package, if it is registered.
pub fn package_prefix(package: &str) -> Option<&'static str> {
    PACKAGE_OWNERS
        .iter()
        .find(|(_, registered)| *registered == package)
        .map(|(prefix, _)| *prefix)
}

/// Check the committed package registration against the census.
///
/// The packages the census found, and the derivation rule the family
/// declares, must both agree with the table. A package present in the tree and absent from the table is
/// an unregistered owner; a table row naming no package is a stale
/// registration; and either is a decision the tree has not recorded.
pub fn verify_package_registration<'a>(
    packages: impl IntoIterator<Item = &'a str>,
    location: &SourceLocation,
) -> Vec<LabelDiagnostic> {
    let mut diagnostics = Vec::new();
    let found: BTreeSet<&str> = packages.into_iter().collect();
    let registered: BTreeSet<&str> = PACKAGE_OWNERS.iter().map(|(_, name)| *name).collect();

    let unregistered: Vec<&str> = found.difference(&registered).copied().collect();
    if !unregistered.is_empty() {
        diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UnregisteredOwner,
            location,
            format!(
                "Cargo packages carry no registered owner prefix: {}; \
                 register them in the adoption data's package table",
                unregistered.join(", ")
            ),
        ));
    }
    let stale: Vec<&str> = registered.difference(&found).copied().collect();
    if !stale.is_empty() {
        diagnostics.push(LabelDiagnostic::error(
            LabelErrorCode::UnregisteredOwner,
            location,
            format!(
                "the adoption data registers owner prefixes for packages \
                 the census does not carry: {}",
                stale.join(", ")
            ),
        ));
    }
    for (prefix, package) in PACKAGE_OWNERS {
        let derived = derive_package_prefix(package);
        if derived != *prefix {
            diagnostics.push(LabelDiagnostic::error(
                LabelErrorCode::UnregisteredOwner,
                location,
                format!(
                    "registered prefix {prefix} for package {package} \
                     does not follow the family derivation rule, which \
                     yields {derived}"
                ),
            ));
        }
    }
    diagnostics
}

// ---------------------------------------------------------------------
// Parameter 2: the owner partition (ADR-019, Omega).
// ---------------------------------------------------------------------

/// How a partition rule matches a carrier path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathRule {
    /// The rule matches exactly this path.
    Exact,
    /// The rule matches any path under this directory.
    Under,
}

/// One rule of the owner partition: a tree location, and its owner.
///
/// The partition is total on the carrier, and the rules are ordered —
/// the first match wins, so the specific precedes the general.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartitionRule {
    pub path: &'static str,
    pub rule: PathRule,
    pub owner: OwnerSelector,
}

/// The owner a partition rule yields. Two arms name a family member
/// rather than a fixed owner, and are resolved from the matched path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnerSelector {
    Fixed(LabelOwner),
    /// The numbered record the matched filename names.
    NumberedRecord,
    /// The package the matched path lies in.
    Package,
}

/// The owner partition of ADR-019, stated as ordered path rules.
///
/// Version-control internals, build and dependency directories, and
/// generated artifacts are outside the carrier and appear in no rule;
/// the census excludes them before this partition is consulted.
pub fn partition() -> Vec<PartitionRule> {
    vec![
        PartitionRule {
            path: "papers/attestation/main.tex",
            rule: PathRule::Exact,
            owner: OwnerSelector::Fixed(LabelOwner::Attestation),
        },
        PartitionRule {
            path: "papers/attestation/sections",
            rule: PathRule::Under,
            owner: OwnerSelector::Fixed(LabelOwner::Attestation),
        },
        PartitionRule {
            path: "docs/attestation/realization.md",
            rule: PathRule::Exact,
            owner: OwnerSelector::Fixed(LabelOwner::Realization),
        },
        PartitionRule {
            path: "adr",
            rule: PathRule::Under,
            owner: OwnerSelector::NumberedRecord,
        },
        PartitionRule {
            path: "packages/model/src",
            rule: PathRule::Under,
            owner: OwnerSelector::Fixed(LabelOwner::Model),
        },
        PartitionRule {
            path: "packages",
            rule: PathRule::Under,
            owner: OwnerSelector::Package,
        },
        PartitionRule {
            path: "plans",
            rule: PathRule::Under,
            owner: OwnerSelector::Fixed(LabelOwner::Plan),
        },
        PartitionRule {
            path: "docs",
            rule: PathRule::Under,
            owner: OwnerSelector::Fixed(LabelOwner::Doc),
        },
    ]
}

// ---------------------------------------------------------------------
// Parameter 3: the profile signature (ADR-019, Pi -- empty).
// ---------------------------------------------------------------------

/// Where a profile carries its derived label. One choice per profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandardPlace {
    /// In the covered asset's header.
    AssetHeader,
    /// In the covered asset's documentation comment.
    DocumentationComment,
    /// In the owner's own prose.
    OwnerProse,
}

/// One registered inventory profile.
///
/// A profile fixes its kind token, its census, its classification rule,
/// its name transformation, and its standard place. The profile
/// signature is empty in this repository, so no profile is constructed
/// outside tests; the type exists because warrant totality and the
/// two-pass staging are implemented against it, and must already be live
/// when the first profile is registered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    /// The kind token this profile governs. An inventory kind.
    pub kind: String,
    /// Which assets the profile covers, as the recognizing harness,
    /// build, or language reports them.
    pub census: String,
    /// The rule from which the area segment derives.
    pub classification: String,
    /// The rule from the asset's bare identifier to the name segment.
    pub name_transformation: String,
    /// The position at which the derived label is carried.
    pub standard_place: StandardPlace,
}

// ---------------------------------------------------------------------
// Parameters 5 and 6: typed-data classes and index designations.
// ---------------------------------------------------------------------

/// A designated class of typed-data strings, each of which cites a
/// target owner synthetically. No such string is ever a source mint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedDataClass {
    pub description: &'static str,
    pub target: LabelOwner,
}

/// The two designated classes of ADR-019.
///
/// Both target the realization contract, and both are consumed in the
/// architecture harvest, which turns each string into a synthetic
/// citation of the realization owner.
///
/// The model-label publication is deliberately absent: it is a generated
/// artifact checked for exactness, not a citing class.
pub fn typed_data_classes() -> Vec<TypedDataClass> {
    vec![
        TypedDataClass {
            description: "architecture witness semantic tags",
            target: LabelOwner::Realization,
        },
        TypedDataClass {
            description: "exported clause identifiers of the architecture manifest",
            target: LabelOwner::Realization,
        },
    ]
}

/// A designated citation index: a document maintaining a committed index
/// of its citations into one upstream owner. Both are generated
/// registers and participate in nothing they index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexDesignation {
    pub document: &'static str,
    pub upstream: LabelOwner,
}

/// The citation-index designations of ADR-019.
pub fn index_designations() -> Vec<IndexDesignation> {
    vec![
        IndexDesignation {
            document: "plans/labels/specification.md",
            upstream: LabelOwner::Attestation,
        },
        IndexDesignation {
            document: "plans/labels/realization.md",
            upstream: LabelOwner::Realization,
        },
    ]
}

// ---------------------------------------------------------------------
// Parameter 7: scanned-region recognition.
// ---------------------------------------------------------------------

/// The languages whose regions are scanned, and what is scanned in each.
///
/// The recognition itself lives in the single participation scanner,
/// which every check shares; this table states which languages the
/// adoption fixes, so that a language absent here is scanned by nothing.
pub const SCANNED_REGIONS: &[(&str, &str)] = &[
    (
        "markdown",
        "authored prose outside fenced blocks and double-backtick spans",
    ),
    (
        "rust",
        "line, block, and documentation comments, with string and \
         character literals and fenced examples inside documentation \
         comments excluded",
    ),
    (
        "latex",
        "authored body text with percent comments stripped, the label \
         carried by the label macro",
    ),
];

// ---------------------------------------------------------------------
// The kind vocabulary (ADR-020), committed and drift-checked.
// ---------------------------------------------------------------------

/// The kind tokens of the adopted registry: the distinct kinds of the
/// Convention tables of the archived draft. Committed from the draft and
/// checked against it by [`verify_vocabulary_sources`].
pub const REGISTRY_KINDS: &[&str] = &[
    "abst",
    "abuse",
    "ack",
    "adden",
    "agenda",
    "alert",
    "alg",
    "amend",
    "ansatz",
    "api",
    "app",
    "appl",
    "arg",
    "aside",
    "assum",
    "ax",
    "axschema",
    "bench",
    "book",
    "bound",
    "calc",
    "caption",
    "case",
    "casestudy",
    "cav",
    "cex",
    "chap",
    "chron",
    "claim",
    "class",
    "cli",
    "cond",
    "confess",
    "conj",
    "constr",
    "conv",
    "cor",
    "credit",
    "crit",
    "data",
    "dataschema",
    "dataset",
    "dec",
    "dedic",
    "def",
    "defprop",
    "defthm",
    "diag",
    "disc",
    "dossier",
    "dream",
    "endpoint",
    "entry",
    "envvar",
    "epigraph",
    "eq",
    "errat",
    "errcode",
    "event",
    "ex",
    "exer",
    "exhibit",
    "expl",
    "expt",
    "fact",
    "fallacy",
    "fig",
    "fixture",
    "flag",
    "folk",
    "formul",
    "func",
    "fuzz",
    "gate",
    "gedanken",
    "gen",
    "gloss",
    "goal",
    "gram",
    "guess",
    "heur",
    "hint",
    "hist",
    "hope",
    "hyp",
    "ident",
    "iface",
    "impl",
    "inf",
    "intuit",
    "inv",
    "job",
    "joke",
    "jour",
    "judg",
    "just",
    "lang",
    "law",
    "lect",
    "ledger",
    "legend",
    "lem",
    "lemdef",
    "lib",
    "lint",
    "listing",
    "log",
    "macro",
    "mat",
    "memo",
    "metaconj",
    "metaq",
    "metathm",
    "metric",
    "migr",
    "minutes",
    "miracle",
    "mod",
    "model",
    "moral",
    "mot",
    "myth",
    "nonex",
    "ns",
    "ntn",
    "obj",
    "obs",
    "open",
    "outlook",
    "para",
    "paradox",
    "part",
    "persp",
    "pf",
    "pipeline",
    "pkg",
    "por",
    "postc",
    "postmortem",
    "pre",
    "pred",
    "pref",
    "preview",
    "prin",
    "prob",
    "proj",
    "promise",
    "prop",
    "property",
    "proposal",
    "proto",
    "puzzle",
    "q",
    "query",
    "quiz",
    "rec",
    "recall",
    "red",
    "refrain",
    "refut",
    "reg",
    "relnotes",
    "rem",
    "rep",
    "reply",
    "req",
    "result",
    "retro",
    "role",
    "rule",
    "runex",
    "scenario",
    "schema",
    "scheme",
    "schol",
    "script",
    "sec",
    "setting",
    "setup",
    "sig",
    "sketch",
    "slogan",
    "snapshot",
    "sol",
    "sorites",
    "spcase",
    "spec",
    "step",
    "stmt",
    "story",
    "strat",
    "suite",
    "summ",
    "svc",
    "tab",
    "term",
    "test",
    "thesis",
    "thm",
    "thmschema",
    "type",
    "unit",
    "variant",
    "ver",
    "verif",
    "vol",
    "warn",
    "yoga",
];

/// The name-and-kind pairs of the adopted registry.
///
/// Every row of the Convention tables of the archived draft, the row's
/// attestation dagger removed from its name as the registry directs.
/// Committed from the draft and checked against it by
/// [`verify_vocabulary_sources`].
///
/// The pairs are the classification relation itself, where
/// [`REGISTRY_KINDS`] is only its image. A head declares a name and a
/// kind together, and validating one against the other needs the
/// relation, not the image: `tab` is a registry kind and Theorem is a
/// registry name, and the pair of the two is not a registry row.
pub const REGISTRY_PAIRS: &[(&str, &str)] = &[
    ("API item", "api"),
    ("Abstract", "abst"),
    ("Abuse of notation", "abuse"),
    ("Acknowledgment", "ack"),
    ("Activity", "exer"),
    ("Addendum", "adden"),
    ("Afterword", "adden"),
    ("Agenda", "agenda"),
    ("Alert", "alert"),
    ("Algorithm", "alg"),
    ("Amendment", "amend"),
    ("Anecdote", "story"),
    ("Annals", "chron"),
    ("Annex", "app"),
    ("Annotation", "gloss"),
    ("Ansatz", "ansatz"),
    ("Answer", "sol"),
    ("Appendix", "app"),
    ("Application", "appl"),
    ("Argument", "arg"),
    ("Array", "mat"),
    ("Aside", "aside"),
    ("Assertion", "claim"),
    ("Assignment", "exer"),
    ("Assumption", "assum"),
    ("Attribution", "credit"),
    ("Axiom", "ax"),
    ("Axiom schema", "axschema"),
    ("Benchmark", "bench"),
    ("Bogus proof", "fallacy"),
    ("Book", "book"),
    ("Bound", "bound"),
    ("CLI command", "cli"),
    ("Calculation", "calc"),
    ("Caption", "caption"),
    ("Case", "case"),
    ("Case study", "casestudy"),
    ("Catalogue", "reg"),
    ("Caution", "warn"),
    ("Caveat", "cav"),
    ("Challenge", "puzzle"),
    ("Changelog", "log"),
    ("Chapter", "chap"),
    ("Characterization", "crit"),
    ("Chart", "fig"),
    ("Check", "verif"),
    ("Chronicle", "chron"),
    ("Claim", "claim"),
    ("Class", "class"),
    ("Clause", "sec"),
    ("Code", "listing"),
    ("Comment", "rem"),
    ("Computation", "calc"),
    ("Computational note", "impl"),
    ("Condition", "cond"),
    ("Confession", "confess"),
    ("Conjecture", "conj"),
    ("Consequence", "cor"),
    ("Construction", "constr"),
    ("Convention", "conv"),
    ("Corollary", "cor"),
    ("Corollary (of the proof)", "por"),
    ("Corrigendum", "errat"),
    ("Counterexample", "cex"),
    ("Courtesy line", "credit"),
    ("Credit", "credit"),
    ("Criterion", "crit"),
    ("Curiosity", "fact"),
    ("Data", "data"),
    ("Dataset", "dataset"),
    ("Decision", "dec"),
    ("Dedication", "dedic"),
    ("Definition", "def"),
    ("Definition–Proposition", "defprop"),
    ("Definition–Theorem", "defthm"),
    ("Demonstration", "ex"),
    ("Demonstration", "pf"),
    ("Derivation", "calc"),
    ("Desideratum", "goal"),
    ("Diagram", "diag"),
    ("Diary", "jour"),
    ("Dictum", "slogan"),
    ("Digression", "aside"),
    ("Discussion", "disc"),
    ("Disproof", "refut"),
    ("Dossier", "dossier"),
    ("Dream", "dream"),
    ("Drill", "exer"),
    ("Endpoint", "endpoint"),
    ("Entry", "entry"),
    ("Environment variable", "envvar"),
    ("Epigraph", "epigraph"),
    ("Epilogue", "adden"),
    ("Equation", "eq"),
    ("Erratum", "errat"),
    ("Error code", "errcode"),
    ("Estimate", "bound"),
    ("Event", "event"),
    ("Example", "ex"),
    ("Excursus", "aside"),
    ("Exercise", "exer"),
    ("Exhibit", "exhibit"),
    ("Expectation", "pred"),
    ("Experiment", "expt"),
    ("Explanation", "expl"),
    ("Exploration", "proj"),
    ("Fact", "fact"),
    ("Fallacy", "fallacy"),
    ("False proof", "fallacy"),
    ("Fantasy", "dream"),
    ("Feature flag", "flag"),
    ("Figure", "fig"),
    ("Fixture", "fixture"),
    ("Folklore", "folk"),
    ("Foreword", "pref"),
    ("Formulation", "formul"),
    ("Fun fact", "fact"),
    ("Function", "func"),
    ("Fuzz target", "fuzz"),
    ("Gate", "gate"),
    ("Generalization", "gen"),
    ("Given data", "data"),
    ("Gloss", "gloss"),
    ("Goal", "goal"),
    ("Grammar", "gram"),
    ("Graph", "fig"),
    ("Guess", "guess"),
    ("Heuristic", "heur"),
    ("Heuristic argument", "heur"),
    ("Hint", "hint"),
    ("Historical note", "hist"),
    ("Historical remark", "hist"),
    ("Homework", "exer"),
    ("Hope", "hope"),
    ("Hypothesis", "hyp"),
    ("Idea of proof", "sketch"),
    ("Identity", "ident"),
    ("Illustration", "ex"),
    ("Illustration", "fig"),
    ("Image", "fig"),
    ("Implementation remark", "impl"),
    ("Indexing convention", "conv"),
    ("Inequality", "bound"),
    ("Inference rule", "inf"),
    ("Insight", "intuit"),
    ("Interface", "iface"),
    ("Interlude", "aside"),
    ("Intermezzo", "aside"),
    ("Intuition", "intuit"),
    ("Invariant", "inv"),
    ("Inventory", "reg"),
    ("Job", "job"),
    ("Joke", "joke"),
    ("Journal", "jour"),
    ("Judgment", "judg"),
    ("Justification", "just"),
    ("Key", "legend"),
    ("Language", "lang"),
    ("Law", "law"),
    ("Lecture", "lect"),
    ("Ledger", "ledger"),
    ("Legend", "legend"),
    ("Legend", "story"),
    ("Lemma", "lem"),
    ("Lemma–Definition", "lemdef"),
    ("Library", "lib"),
    ("Lint rule", "lint"),
    ("Listing", "listing"),
    ("Log", "log"),
    ("Macro", "macro"),
    ("Marginal gloss", "gloss"),
    ("Matrix", "mat"),
    ("Maxim", "slogan"),
    ("Memo", "memo"),
    ("Memorandum", "memo"),
    ("Meta-conjecture", "metaconj"),
    ("Meta-question", "metaq"),
    ("Meta-theorem", "metathm"),
    ("Metric", "metric"),
    ("Migration", "migr"),
    ("Minutes", "minutes"),
    ("Miracle", "miracle"),
    ("Misconception", "myth"),
    ("Model", "constr"),
    ("Model", "model"),
    ("Module", "mod"),
    ("Module", "sec"),
    ("Moral", "moral"),
    ("Motivation", "mot"),
    ("Motto", "slogan"),
    ("Myth", "myth"),
    ("N.B.", "rem"),
    ("Namespace", "ns"),
    ("Nomenclature", "term"),
    ("Non-example", "nonex"),
    ("Notation", "ntn"),
    ("Note", "rem"),
    ("Objection", "obj"),
    ("Observation", "data"),
    ("Observation", "obs"),
    ("Open Problem", "open"),
    ("Open Question", "open"),
    ("Outline of proof", "sketch"),
    ("Outlook", "outlook"),
    ("Overview", "preview"),
    ("Package", "pkg"),
    ("Parable", "story"),
    ("Paradox", "paradox"),
    ("Paragraph", "para"),
    ("Part", "part"),
    ("Perspective", "persp"),
    ("Philosophy", "persp"),
    ("Photograph", "fig"),
    ("Picture", "fig"),
    ("Pipeline", "pipeline"),
    ("Pitfall", "warn"),
    ("Plausibility argument", "heur"),
    ("Plot", "fig"),
    ("Porism", "por"),
    ("Postcondition", "postc"),
    ("Postmortem", "postmortem"),
    ("Postscript", "adden"),
    ("Postulate", "ax"),
    ("Practice", "exer"),
    ("Prayer", "hope"),
    ("Preamble", "pref"),
    ("Precondition", "pre"),
    ("Prediction", "pred"),
    ("Preface", "pref"),
    ("Preview", "preview"),
    ("Principle", "prin"),
    ("Problem", "prob"),
    ("Problem formulation", "formul"),
    ("Procedure", "alg"),
    ("Project", "proj"),
    ("Prologue", "pref"),
    ("Promise", "promise"),
    ("Proof", "pf"),
    ("Proof idea", "sketch"),
    ("Proof outline", "sketch"),
    ("Proof sketch", "sketch"),
    ("Property", "property"),
    ("Proposal", "proposal"),
    ("Proposition", "prop"),
    ("Protocol", "minutes"),
    ("Protocol", "proto"),
    ("Pseudocode", "listing"),
    ("Punchline", "moral"),
    ("Puzzle", "puzzle"),
    ("Query", "query"),
    ("Question", "q"),
    ("Quiz", "quiz"),
    ("Recall", "recall"),
    ("Record", "rec"),
    ("Reduction", "red"),
    ("Refrain", "refrain"),
    ("Refutation", "refut"),
    ("Register", "reg"),
    ("Registry", "reg"),
    ("Release notes", "relnotes"),
    ("Remark", "rem"),
    ("Reminder", "recall"),
    ("Reply", "reply"),
    ("Report", "rep"),
    ("Requirement", "req"),
    ("Research problem", "open"),
    ("Research question", "open"),
    ("Result", "result"),
    ("Retrospective", "retro"),
    ("Review", "rep"),
    ("Review", "sec"),
    ("Revision", "ver"),
    ("Riddle", "puzzle"),
    ("Roadmap", "preview"),
    ("Role", "role"),
    ("Route", "endpoint"),
    ("Rule", "rule"),
    ("Rule of thumb", "heur"),
    ("Runnable example", "runex"),
    ("Sanity check", "verif"),
    ("Saying", "slogan"),
    ("Scenario", "scenario"),
    ("Schema", "dataschema"),
    ("Schema", "schema"),
    ("Scheme", "fig"),
    ("Scheme", "scheme"),
    ("Scholium", "schol"),
    ("Script", "script"),
    ("Section", "sec"),
    ("Service", "svc"),
    ("Setting", "setting"),
    ("Setting", "setup"),
    ("Setup", "setup"),
    ("Sidebar", "aside"),
    ("Sign convention", "conv"),
    ("Signature", "sig"),
    ("Simulation", "expt"),
    ("Situation", "setup"),
    ("Sketch", "sketch"),
    ("Sketch of proof", "sketch"),
    ("Slogan", "slogan"),
    ("Snapshot", "snapshot"),
    ("Solution", "sol"),
    ("Sorites", "sorites"),
    ("Source line", "credit"),
    ("Special case", "spcase"),
    ("Specification", "spec"),
    ("Speculation", "guess"),
    ("Standing hypothesis", "assum"),
    ("Statement", "stmt"),
    ("Step", "step"),
    ("Story", "scenario"),
    ("Story", "story"),
    ("Strategy", "strat"),
    ("Struct", "class"),
    ("Structure", "class"),
    ("Structure", "constr"),
    ("Structure", "schema"),
    ("Suite", "suite"),
    ("Summary", "summ"),
    ("Supplement", "adden"),
    ("Surprise", "miracle"),
    ("Synopsis", "abst"),
    ("Table", "tab"),
    ("Takeaway", "moral"),
    ("Task", "exer"),
    ("Task", "job"),
    ("Terminology", "term"),
    ("Test", "quiz"),
    ("Test", "test"),
    ("Theorem", "thm"),
    ("Theorem schema", "thmschema"),
    ("Thesis", "thesis"),
    ("Thought experiment", "gedanken"),
    ("Type", "type"),
    ("Unit", "unit"),
    ("Upshot", "moral"),
    ("Use case", "scenario"),
    ("Variant", "variant"),
    ("Verification", "verif"),
    ("Version", "ver"),
    ("Vignette", "story"),
    ("Volume", "vol"),
    ("Warm-up", "exer"),
    ("Warning", "warn"),
    ("Wish", "hope"),
    ("Workflow", "pipeline"),
    ("Working hypothesis", "assum"),
    ("Yoga", "yoga"),
];

/// The recorded extension set `X_A` of ADR-020.
///
/// The kinds this repository adds to the registry's rows as the
/// registry's acceptee. Committed from the ADR's extension table and
/// checked against it by [`verify_vocabulary_sources`].
pub const EXTENSION_KINDS: &[&str] = &[
    "branch",
    "candidate",
    "err",
    "leaf",
    "milestone",
    "obl",
    "op",
    "phase",
    "pin",
    "ref",
    "res",
    "task",
    "trap",
];

/// The name-and-kind pairs of the recorded extension set `X_A`.
///
/// The name column and the kind column of every row of the ADR's
/// extension table. Committed from the ADR and checked against it by
/// [`verify_vocabulary_sources`].
pub const EXTENSION_PAIRS: &[(&str, &str)] = &[
    ("Branch", "branch"),
    ("Candidate", "candidate"),
    ("Error vocabulary", "err"),
    ("Leaf", "leaf"),
    ("Milestone", "milestone"),
    ("Obligation", "obl"),
    ("Operation", "op"),
    ("Phase", "phase"),
    ("Pin", "pin"),
    ("Reference", "ref"),
    ("Residual risk", "res"),
    ("Task", "task"),
    ("Trap", "trap"),
];

/// The document the registry kinds are committed from.
pub const REGISTRY_SOURCE: &str = "plans/drafts/environment-kinds.md";
/// The document the extension kinds are committed from.
pub const EXTENSION_SOURCE: &str = "adr/020-environment-kinds.md";

/// Whether a kind is in the adopted vocabulary: the registry's tokens
/// together with the recorded extension set, which is the effective
/// relation ADR-020 writes `C_A`.
pub fn kind_is_adopted(kind: &str) -> bool {
    REGISTRY_KINDS.contains(&kind) || EXTENSION_KINDS.contains(&kind)
}

/// The pairs of the effective relation `C_A`: the registry's rows
/// together with the recorded extension set's.
pub fn catalogued_pairs() -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    REGISTRY_PAIRS.iter().chain(EXTENSION_PAIRS.iter())
}

/// Every kind `C_A` catalogues for a name: its senses.
///
/// One sense per distinct concept the name carries. The comparison
/// ignores case, since a name heading an environment is capitalized by
/// the format rather than by the catalogue, and a case-sensitive
/// comparison would reject a head whose pair the registry carries.
pub fn catalogued_senses(name: &str) -> Vec<&'static str> {
    catalogued_pairs()
        .filter(|(catalogued, _)| catalogued.eq_ignore_ascii_case(name))
        .map(|(_, kind)| *kind)
        .collect()
}

/// Whether `C_A` carries this name-and-kind pair.
pub fn pair_is_catalogued(name: &str, kind: &str) -> bool {
    catalogued_pairs().any(|(catalogued, catalogued_kind)| {
        catalogued.eq_ignore_ascii_case(name) && *catalogued_kind == kind
    })
}

/// Whether an owner's kinds are enforced against the vocabulary, or only
/// reported. ADR-020 governs every owner but the attestation LaTeX surface,
/// which it records as not yet in scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KindScope {
    Enforced,
    Reported,
}

/// The scope ADR-020 gives an owner.
pub const fn kind_scope(owner: &LabelOwner) -> KindScope {
    match owner {
        LabelOwner::Attestation => KindScope::Reported,
        _ => KindScope::Enforced,
    }
}

// ---------------------------------------------------------------------
// Drift checks against the two vocabulary sources.
// ---------------------------------------------------------------------

/// Parse the distinct kind tokens of the archived registry draft.
///
/// The tokens are the second column of every Convention table. Device
/// rows, whose kind cell is an em dash rather than a token, classify
/// nothing and are skipped.
pub fn parse_registry_source(text: &str) -> BTreeSet<String> {
    let mut kinds = BTreeSet::new();
    let mut in_convention = false;
    for line in text.lines() {
        if line.starts_with("**Convention (") {
            in_convention = true;
            continue;
        }
        if line.starts_with("## ") {
            in_convention = false;
            continue;
        }
        if !in_convention || !line.starts_with('|') {
            continue;
        }
        let cells = table_cells(line);
        if cells.len() == 2
            && let Some(kind) = backticked_token(cells[1])
        {
            kinds.insert(kind);
        }
    }
    kinds
}

/// Parse the name-and-kind pairs of the archived registry draft.
///
/// Every Convention-table row whose kind cell is a token contributes one
/// pair. Device rows contribute none, having no kind; and the
/// attestation dagger a row may carry is a status mark on the row, never
/// a character of the name, so it is stripped before the name is taken.
pub fn parse_registry_pairs(text: &str) -> BTreeSet<(String, String)> {
    let mut pairs = BTreeSet::new();
    let mut in_convention = false;
    for line in text.lines() {
        if line.starts_with("**Convention (") {
            in_convention = true;
            continue;
        }
        if line.starts_with("## ") {
            in_convention = false;
            continue;
        }
        if !in_convention || !line.starts_with('|') {
            continue;
        }
        let cells = table_cells(line);
        if cells.len() == 2
            && let Some(kind) = backticked_token(cells[1])
        {
            pairs.insert((catalogue_name(cells[0]), kind));
        }
    }
    pairs
}

/// Parse the name-and-kind pairs of the ADR's extension table.
pub fn parse_extension_pairs(text: &str) -> BTreeSet<(String, String)> {
    let mut pairs = BTreeSet::new();
    for cells in extension_table_rows(text) {
        if let Some(kind) = backticked_token(cells[1]) {
            pairs.insert((catalogue_name(cells[0]), kind));
        }
    }
    pairs
}

/// The exact catalogue name a table cell carries: the cell's text with
/// the attestation dagger removed, per the status-mark rule of the
/// registry's attestation judgment.
fn catalogue_name(cell: &str) -> String {
    cell.trim_end_matches('†').trim().to_owned()
}

/// Parse the recorded extension set from the ADR's extension table: the
/// kind column of every row of the table the extension section heads.
pub fn parse_extension_source(text: &str) -> BTreeSet<String> {
    let mut kinds = BTreeSet::new();
    for cells in extension_table_rows(text) {
        if let Some(kind) = backticked_token(cells[1]) {
            kinds.insert(kind);
        }
    }
    kinds
}

/// The four-celled rows of the ADR's extension table, in document order.
fn extension_table_rows(text: &str) -> Vec<Vec<&str>> {
    let mut rows = Vec::new();
    let mut in_table = false;
    for line in text.lines() {
        if line.contains(EXTENSION_TABLE_MINT) {
            in_table = true;
            continue;
        }
        if in_table && line.starts_with("## ") {
            break;
        }
        if !in_table || !line.starts_with('|') {
            continue;
        }
        let cells = table_cells(line);
        if cells.len() == 4 {
            rows.push(cells);
        }
    }
    rows
}

/// The label text heading the ADR's extension table. Held as data rather
/// than written as a citation: this crate is a different owner, and the
/// string names a location in a document, not a fact this crate cites.
const EXTENSION_TABLE_MINT: &str = "tab:kinds:extensions";

fn table_cells(line: &str) -> Vec<&str> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect()
}

fn backticked_token(cell: &str) -> Option<String> {
    let token = cell.strip_prefix('`')?.strip_suffix('`')?;
    if !token.is_empty()
        && token
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    {
        Some(token.to_owned())
    } else {
        None
    }
}

/// Check the committed vocabulary against the documents it was extracted
/// from, and fail loudly on any disagreement, naming both sides.
///
/// A source document that is absent is not checked: scoped censuses and
/// the synthetic fixture repositories of the test suite carry neither
/// document, and their absence is not drift. The committed tables are
/// pinned to the real documents by the crate's own unit tests, which run
/// against this repository rather than against a fixture.
pub fn verify_vocabulary_sources(root: &Path) -> Vec<LabelDiagnostic> {
    let mut diagnostics = Vec::new();
    check_source(
        root,
        REGISTRY_SOURCE,
        "the adopted kind registry",
        &parse_registry_source,
        REGISTRY_KINDS,
        &mut diagnostics,
    );
    check_source(
        root,
        EXTENSION_SOURCE,
        "the recorded extension set",
        &parse_extension_source,
        EXTENSION_KINDS,
        &mut diagnostics,
    );
    check_source(
        root,
        REGISTRY_SOURCE,
        "the adopted registry's name-and-kind pairs",
        &|text| rendered_pairs(&parse_registry_pairs(text)),
        &rendered_pair_slice(REGISTRY_PAIRS),
        &mut diagnostics,
    );
    check_source(
        root,
        EXTENSION_SOURCE,
        "the recorded extension set's name-and-kind pairs",
        &|text| rendered_pairs(&parse_extension_pairs(text)),
        &rendered_pair_slice(EXTENSION_PAIRS),
        &mut diagnostics,
    );
    diagnostics
}

/// Render a parsed pair set for comparison and for the drift message,
/// where a pair must name both of its sides to be actionable.
fn rendered_pairs(pairs: &BTreeSet<(String, String)>) -> BTreeSet<String> {
    pairs
        .iter()
        .map(|(name, kind)| format!("{name} = {kind}"))
        .collect()
}

/// The same rendering of a committed pair table.
fn rendered_pair_slice(pairs: &[(&str, &str)]) -> Vec<String> {
    pairs
        .iter()
        .map(|(name, kind)| format!("{name} = {kind}"))
        .collect()
}

fn check_source(
    root: &Path,
    relative: &str,
    description: &str,
    parse: &dyn Fn(&str) -> BTreeSet<String>,
    committed: &[impl AsRef<str>],
    diagnostics: &mut Vec<LabelDiagnostic>,
) {
    let path = root.join(relative);
    let Ok(text) = fs::read_to_string(&path) else {
        return;
    };
    let location = SourceLocation::new(relative, 1, 1);
    let parsed = parse(&text);
    let committed: BTreeSet<String> = committed
        .iter()
        .map(|entry| entry.as_ref().to_owned())
        .collect();
    if parsed == committed {
        return;
    }
    let missing: Vec<&str> = parsed.difference(&committed).map(String::as_str).collect();
    let extra: Vec<&str> = committed.difference(&parsed).map(String::as_str).collect();
    let mut message = format!(
        "the checker's committed kind vocabulary has drifted from {description} at {relative}"
    );
    if !missing.is_empty() {
        let _ = write!(
            message,
            "; the document carries and the checker does not: {}",
            missing.join(", ")
        );
    }
    if !extra.is_empty() {
        let _ = write!(
            message,
            "; the checker carries and the document does not: {}",
            extra.join(", ")
        );
    }
    diagnostics.push(LabelDiagnostic::error(
        LabelErrorCode::KindVocabularyDrift,
        &location,
        message,
    ));
}

// ---------------------------------------------------------------------
// The adoption data, and the invariants stated over it.
// ---------------------------------------------------------------------

/// The adoption data the checker loads before any resolution.
///
/// Loading them is the first stage of the calculus's two-pass staging:
/// the parameters are fixed, then the carrier is harvested, and only
/// then is any resolution judgment derived.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adoption {
    /// The profile signature. Empty in this repository.
    pub profiles: Vec<Profile>,
    /// The reserved kinds. Empty in this repository, and necessarily so
    /// while the profile signature is: a reserved kind no profile
    /// governs admits neither warrant rule.
    pub reserved_kinds: BTreeSet<String>,
}

impl Default for Adoption {
    fn default() -> Self {
        Self::repository()
    }
}

impl Adoption {
    /// This repository's adoption, as ADR-019 records it.
    ///
    /// Both the profile signature and the reserved kinds are empty, so
    /// every mint in the corpus stands on the authorship warrant.
    pub const fn repository() -> Self {
        Self {
            profiles: Vec::new(),
            reserved_kinds: BTreeSet::new(),
        }
    }

    /// The profile governing a kind, if one is registered.
    pub fn profile_for(&self, kind: &str) -> Option<&Profile> {
        self.profiles.iter().find(|profile| profile.kind == kind)
    }

    /// Whether a kind is reserved. Every kind a profile governs is
    /// reserved; a reserved kind need not be governed.
    pub fn is_reserved(&self, kind: &str) -> bool {
        self.reserved_kinds.contains(kind) || self.profile_for(kind).is_some()
    }

    /// The reserved kinds no registered profile governs. A nonempty
    /// result makes those kinds unmintable by either warrant rule, which
    /// is the deliberate hard failure of warrant totality.
    pub fn ungoverned_reserved_kinds(&self) -> Vec<&str> {
        self.reserved_kinds
            .iter()
            .map(String::as_str)
            .filter(|kind| self.profile_for(kind).is_none())
            .collect()
    }
}

/// Enforce warrant totality over the completed minting registries.
///
/// Every mint stands on exactly one warrant, and every kind admits at
/// most one warrant species. With the profile signature empty this is
/// mostly structure, and the inventory arms are vacuous; they are
/// implemented all the same, so that registering the first profile finds
/// the enforcement already live rather than still to be written.
///
/// Three failures are reported. A reserved kind no profile governs
/// admits neither warrant rule, so its bare occurrence is a hard
/// failure. An inventory-kind token away from its profile's standard
/// place warrants nothing, since only an occurrence at the standard
/// place discharges the derivation rule. And a kind outside the adopted
/// vocabulary is no kind at all, over the owners ADR-020 governs.
pub fn validate_warrants(
    adoption: &Adoption,
    mints: &[LabelMint],
    place_of: &dyn Fn(&LabelMint) -> Option<StandardPlace>,
) -> Vec<LabelDiagnostic> {
    let mut diagnostics = Vec::new();
    for mint in mints {
        let kind = mint.label.kind();
        match adoption.profile_for(kind) {
            Some(profile) => {
                if place_of(mint) != Some(profile.standard_place) {
                    diagnostics.push(LabelDiagnostic::error(
                        LabelErrorCode::InventoryKindOutOfPlace,
                        &mint.location,
                        format!(
                            "inventory kind {kind} is warranted only by derivation at its \
                             profile's standard place, and this occurrence is elsewhere"
                        ),
                    ));
                }
                continue;
            }
            None if adoption.is_reserved(kind) => {
                diagnostics.push(LabelDiagnostic::error(
                    LabelErrorCode::ReservedKindWithoutProfile,
                    &mint.location,
                    format!(
                        "kind {kind} is reserved and no profile governs it, so no warrant \
                         rule admits this occurrence"
                    ),
                ));
                continue;
            }
            None => {}
        }
        if kind_is_adopted(kind) {
            continue;
        }
        let message = format!(
            "kind {kind} is in neither the adopted registry at {REGISTRY_SOURCE} nor the \
             recorded extension set at {EXTENSION_SOURCE}"
        );
        diagnostics.push(match kind_scope(&mint.owner) {
            KindScope::Enforced => {
                LabelDiagnostic::error(LabelErrorCode::UnknownKind, &mint.location, message)
            }
            KindScope::Reported => LabelDiagnostic::warning(
                LabelErrorCode::UnknownKind,
                &mint.location,
                format!(
                    "{message}; the attestation surface is recorded as not yet in scope of the \
                     kind registry, and this token awaits adjudication"
                ),
            ),
        });
    }
    diagnostics
}

/// The kinds minted in the corpus, with how many mints each carries.
/// The census the vocabulary is answerable to.
pub fn kind_census(mints: &[LabelMint]) -> BTreeMap<String, usize> {
    let mut census = BTreeMap::new();
    for mint in mints {
        *census.entry(mint.label.kind().to_owned()).or_insert(0) += 1;
    }
    census
}
