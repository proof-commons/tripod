//! Plan-tree structure and hygiene checks (the `plans-check` lane).
//!
//! Non-semantic documentation checks over `adr/` and `plans/`: census
//! reconciliation, README ownership indexing, heading/link/scaffolding
//! hygiene, phase-gate consistency, and the Markdown weight budget.
//!
//! The weight budget is two budgets, because the tree holds two kinds
//! of document, and the split is by role rather than by directory.
//! Load-bearing planning prose is what the combined `adr/` + `plans/`
//! cap guards: that prose is maintained, so unchecked growth there is
//! duplication rather than content, and the cap keeps one fact to one
//! owner. Documents nobody maintains by hand are different in kind. The
//! executed implementation guides under `plans/guides/`, the static
//! reviews under `plans/reviews/`, and the adopted-source drafts under
//! `plans/drafts/` are verbatim records of a named tree, never edited
//! to fit a budget and never trimmed. The closed records under
//! `plans/history/` join them by role rather than by provenance: they
//! are this repository's own finished gate and finding registers, cut
//! verbatim out of a maintained document once the work they describe
//! closed, and never edited again. The `GENERATED_REGISTERS`
//! publications under `plans/labels/` are regenerated outputs, sized by
//! their inputs and rewritten wholesale by their generators. Charging
//! either class to the maintained-prose cap would make the guardrail
//! fire on files it must not police — an author asked to shrink them
//! could only falsify the record or the generator. Their bytes are
//! excluded from `combined_bytes` and accounted separately against the
//! much larger `ARCHIVE_HARD_CAP_BYTES`, which exists only to catch a
//! runaway paste. Authored prose keeps the combined budget wherever it
//! sits, including `plans/labels/README.md` beside the registers.
//!
//! Subject files arrive by argument from the build system (ADR-014);
//! the checker re-discovers them on disk and hard-fails on any
//! disagreement, so a stale census cannot silently pass. This module
//! replaced the retired `scripts/check_plans.py`, which ran outside
//! the ADR-010 command-line contract.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Context;
use serde::Serialize;

/// Bumped to 2 when the archive budget split `combined_bytes` off from
/// the archive total and added the archive fields to [`PlansReport`].
pub const PLANS_REPORT_SCHEMA: u32 = 2;

const HARD_CAP_BYTES: u64 = 768 * 1024;
const SOFT_TARGET_BYTES: u64 = 520 * 1024;

/// Ceiling for the archived-document budget (4 MiB).
///
/// Deliberately far above the present archive: the archive is verbatim
/// history, so the cap is a runaway-paste tripwire, not a shaping
/// force.
const ARCHIVE_HARD_CAP_BYTES: u64 = 4 * 1024 * 1024;

/// Directories holding verbatim archived documents, excluded from the
/// load-bearing combined budget and accounted against
/// [`ARCHIVE_HARD_CAP_BYTES`] instead.
///
/// `plans/history/` differs from the other three in provenance but not
/// in role. The others hold documents this repository received; history
/// holds records it wrote itself and then closed — completed gate
/// records and the finding registers of remediated reviews, moved
/// verbatim out of the backlog once their batches were done. What every
/// member shares is that the bytes are a finished record nobody edits
/// again, so charging them to the maintained-prose budget would force
/// an author to trim settled history to make room for current work.
const ARCHIVE_DIRECTORIES: [&str; 4] = [
    "plans/drafts/",
    "plans/guides/",
    "plans/history/",
    "plans/reviews/",
];

/// The generated specification label register (ADR-014).
pub const SPECIFICATION_REGISTER: &str = "plans/labels/specification.md";
/// The generated realization label register (ADR-014).
pub const REALIZATION_REGISTER: &str = "plans/labels/realization.md";
/// The generated companion attestation register (ADR-020).
pub const ATTESTATION_REGISTER: &str = "plans/labels/attestation.md";

/// Generated register publications: regenerated outputs, not prose.
///
/// They carry no per-file weight threshold and their bytes are accounted
/// against the archive budget rather than the maintained-prose budget.
///
/// This is the one place the register role is stated. `census.rs` builds
/// its register paths from the same constants, and
/// `generated_registers_match_the_census` fails loudly if a register is
/// ever added to the census without being named here.
pub const GENERATED_REGISTERS: [&str; 3] = [
    SPECIFICATION_REGISTER,
    REALIZATION_REGISTER,
    ATTESTATION_REGISTER,
];

/// Paths deleted from the tree; a surviving textual reference is stale.
const OLD_PATHS: [&str; 8] = [
    "plans/toolchain-architecture.md",
    "plans/decisions/001-typed-rust-is-normative.md",
    "plans/decisions/002-target-independent-realization-layer.md",
    "plans/decisions/003-multiple-backends-tapscript-first.md",
    "plans/decisions/004-translation-validation-over-compiler-trust.md",
    "plans/decisions/005-value-parametric-asset-rigid.md",
    "plans/decisions/006-canonical-transaction-layout-abi.md",
    "plans/research/state-object-constructor.md",
];

/// Leftover drafting-session narration that must not be committed.
const SCAFFOLDING: [&str; 3] = [
    "Below is a complete draft",
    "The next file should be",
    "Notes for applying this file",
];

/// Plans are prose; claiming they are machine input overclaims their
/// authority.
const PROHIBITED_MACHINE_INPUT: [&str; 3] = [
    "Machine-consumed by the toolchain: yes",
    "plans are compiler input",
    "this plan is normative protocol input",
];

/// Retired source-identity phrasing that must not reappear in plans.
const PROHIBITED_SOURCE_IDENTITY: [&str; 4] = [
    "source revision is target identity",
    "exact Elements revision is protocol identity",
    "deployment release pins the Elements source revision",
    "target identity binds the upstream implementation commit",
];

const PLACEHOLDERS: [&str; 4] = ["example.invalid", "TODO_URL", "INSERT_HASH", "TBD_PATH"];

static LINK: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[[^\]]*\]\(([^)#\s]+)(?:#[^)\s]+)?\)").expect("static link pattern")
});
static FILE_SCAFFOLD: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?m)^## File \d+").expect("static scaffold pattern"));
static CONFIDENCE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)(confidence\s+\d{1,3}%|\d{1,3}%\s+confident)")
        .expect("static confidence pattern")
});
/// Deliberately loose: the token after `Phase` is captured whatever it
/// is, so a malformed declaration fails explicitly instead of reading
/// as an absent one.
static CURRENT_GATE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?m)^> \*\*Current gate:\*\* Phase\b[ \t]*(\S*)")
        .expect("static gate pattern")
});
/// The fixed current-phase declaration form used by `plans/README.md`
/// and `plans/roadmap.md`.
static CURRENT_PHASE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?m)^Current: Phase\b[ \t]*(\S*)").expect("static phase pattern")
});
static ACTIVE_STATUS: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?m)^> \*\*Status:\*\* Active").expect("static status pattern")
});
static TASK_HEADING: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^`?([A-Z][A-Z0-9]*-?[0-9]+)`? ").expect("static task heading")
});

#[derive(Debug, Serialize)]
pub struct PlansReport {
    pub schema: u32,
    pub files_checked: usize,
    pub adr_bytes: u64,
    /// Load-bearing `plans/` bytes: the archive directories and the
    /// generated registers excluded.
    pub plans_bytes: u64,
    /// `adr_bytes + plans_bytes`, checked against `hard_cap_bytes`.
    pub combined_bytes: u64,
    pub hard_cap_bytes: u64,
    pub soft_target_bytes: u64,
    pub soft_target_exceeded: bool,
    /// Bytes of documents nobody maintains by hand: verbatim archived
    /// documents under `ARCHIVE_DIRECTORIES` and the
    /// `GENERATED_REGISTERS` publications, checked against
    /// `archive_hard_cap_bytes` alone.
    pub archive_bytes: u64,
    pub archive_hard_cap_bytes: u64,
    pub warnings: usize,
    pub valid: bool,
}

#[derive(Debug)]
pub struct PlansOutcome {
    pub report: PlansReport,
    pub failures: Vec<String>,
    pub warnings: Vec<String>,
}

/// Run every plan-tree check under `root` against the declared census
/// `subjects` (repository-relative or absolute paths).
///
/// Returns `Err` only for environmental faults (an unreadable file or
/// directory); every tree defect is a failure entry in the outcome.
pub fn check_plans(root: &Path, subjects: &[PathBuf]) -> anyhow::Result<PlansOutcome> {
    let root = root
        .canonicalize()
        .with_context(|| format!("resolving repository root {}", root.display()))?;
    let mut failures = Vec::new();
    let mut warnings = Vec::new();

    let files = markdown_files(&root)?;
    verify_census(&root, subjects, &files, &mut failures);
    verify_ownership(&root, &mut failures)?;
    for path in &files {
        check_file(&root, path, &mut failures, &mut warnings)?;
    }
    verify_phase_gate(&root, &mut failures)?;
    verify_task_status_agreement(&root, &mut failures)?;
    verify_unique_row_ids(&root, &mut failures)?;

    let TreeBytes {
        adr: adr_bytes,
        plans: plans_bytes,
        archive: archive_bytes,
    } = tree_bytes(&root, &files)?;
    let combined_bytes = adr_bytes + plans_bytes;
    if combined_bytes > HARD_CAP_BYTES {
        failures.push(format!(
            "weight: combined Markdown {combined_bytes} exceeds hard cap {HARD_CAP_BYTES}"
        ));
    }
    if archive_bytes > ARCHIVE_HARD_CAP_BYTES {
        failures.push(format!(
            "weight: archived Markdown {archive_bytes} exceeds archive hard cap \
             {ARCHIVE_HARD_CAP_BYTES}"
        ));
    }

    let report = PlansReport {
        schema: PLANS_REPORT_SCHEMA,
        files_checked: files.len(),
        adr_bytes,
        plans_bytes,
        combined_bytes,
        hard_cap_bytes: HARD_CAP_BYTES,
        soft_target_bytes: SOFT_TARGET_BYTES,
        soft_target_exceeded: combined_bytes > SOFT_TARGET_BYTES,
        archive_bytes,
        archive_hard_cap_bytes: ARCHIVE_HARD_CAP_BYTES,
        warnings: warnings.len(),
        valid: failures.is_empty(),
    };
    Ok(PlansOutcome {
        report,
        failures,
        warnings,
    })
}

/// Every Markdown file under `adr/` and `plans/`, sorted.
fn markdown_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for tree in ["adr", "plans"] {
        collect_markdown(&root.join(tree), &mut files)?;
    }
    files.sort();
    Ok(files)
}

fn collect_markdown(directory: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in read_dir_sorted(directory)? {
        if entry.is_dir() {
            collect_markdown(&entry, files)?;
        } else if entry.extension().is_some_and(|extension| extension == "md") {
            files.push(entry);
        }
    }
    Ok(())
}

fn read_dir_sorted(directory: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    let listing = fs::read_dir(directory)
        .with_context(|| format!("listing directory {}", directory.display()))?;
    for entry in listing {
        entries.push(
            entry
                .with_context(|| format!("listing directory {}", directory.display()))?
                .path(),
        );
    }
    entries.sort();
    Ok(entries)
}

/// The declared census must equal the on-disk discovery (ADR-014).
///
/// There is deliberately no empty-declaration bypass: discovery stays a
/// verifier, never the membership source, so an empty declaration
/// against a nonempty tree reports every discovered file as outside
/// the census and fails closed.
fn verify_census(root: &Path, subjects: &[PathBuf], files: &[PathBuf], failures: &mut Vec<String>) {
    let declared: BTreeSet<PathBuf> = subjects
        .iter()
        .map(|subject| {
            if subject.is_absolute() {
                subject.clone()
            } else {
                root.join(subject)
            }
        })
        .collect();
    let discovered: BTreeSet<PathBuf> = files.iter().cloned().collect();
    for missing in declared.difference(&discovered) {
        failures.push(format!(
            "census: declared subject absent on disk: {}",
            missing.display()
        ));
    }
    for extra in discovered.difference(&declared) {
        failures.push(format!(
            "census: file outside the build census (rerun meson setup): {}",
            extra.display()
        ));
    }
}

/// Every directory owns a README.md indexing each Markdown child and
/// each subdirectory.
fn verify_ownership(root: &Path, failures: &mut Vec<String>) -> anyhow::Result<()> {
    let mut directories = Vec::new();
    for tree in ["adr", "plans"] {
        collect_directories(&root.join(tree), &mut directories)?;
    }
    directories.sort();
    for directory in directories {
        let readme = directory.join("README.md");
        let relative_directory = relative(root, &directory);
        if !readme.is_file() {
            failures.push(format!("ownership: {relative_directory} has no README.md"));
            continue;
        }
        let contents = read_text(&readme)?;
        for child in read_dir_sorted(&directory)? {
            let Some(name) = child.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name == "README.md" {
                continue;
            }
            let indexed = if child.is_dir() {
                contents.contains(&format!("{name}/README.md"))
            } else if child.extension().is_some_and(|extension| extension == "md") {
                contents.contains(name)
            } else {
                true
            };
            if !indexed {
                failures.push(format!(
                    "ownership: {} not indexed in {}",
                    relative(root, &child),
                    relative(root, &readme)
                ));
            }
        }
    }
    Ok(())
}

fn collect_directories(directory: &Path, directories: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    directories.push(directory.to_path_buf());
    for entry in read_dir_sorted(directory)? {
        if entry.is_dir() {
            collect_directories(&entry, directories)?;
        }
    }
    Ok(())
}

/// The file's lines with fenced-block interiors and fence markers
/// blanked, so line-oriented hygiene checks skip displayed material.
///
/// This is the shared participation scanner's line view; it used to be
/// a second, independent fence loop that could drift from the one the
/// label harvest uses.
fn without_fenced_lines(text: &str) -> String {
    crate::participation::ProseParticipation::of(text).blanked(text)
}

/// Structure and hygiene checks for one Markdown file.
fn check_file(
    root: &Path,
    path: &Path,
    failures: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> anyhow::Result<()> {
    let relative_path = relative(root, path);
    let text = read_text(path)?;

    if !text.lines().next().unwrap_or_default().starts_with("# ") {
        failures.push(format!(
            "heading: {relative_path} does not start with a top-level heading"
        ));
    }
    // Fenced material is displayed without participating: a bracketed
    // pattern inside a code fence is not a Markdown link.
    let linkable = without_fenced_lines(&text);
    for capture in LINK.captures_iter(&linkable) {
        let target = &capture[1];
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("mailto:")
        {
            continue;
        }
        let destination = path.parent().unwrap_or(root).join(target);
        if !destination.exists() {
            failures.push(format!("broken link: {relative_path} -> {target}"));
        }
    }
    for marker in SCAFFOLDING {
        if text.contains(marker) {
            failures.push(format!("draft scaffolding: {relative_path}: {marker}"));
        }
    }
    if FILE_SCAFFOLD.is_match(&text) {
        failures.push(format!("draft scaffolding: {relative_path}: ## File N"));
    }
    if PLACEHOLDERS.iter().any(|marker| text.contains(marker)) {
        failures.push(format!("placeholder data: {relative_path}"));
    }
    if CONFIDENCE.is_match(&text) {
        failures.push(format!("confidence percentage: {relative_path}"));
    }
    if OLD_PATHS.iter().any(|old| text.contains(old)) {
        failures.push(format!("deleted path reference: {relative_path}"));
    }
    if PROHIBITED_MACHINE_INPUT
        .iter()
        .any(|marker| text.contains(marker))
    {
        failures.push(format!("machine-input overclaim: {relative_path}"));
    }
    if relative_path.starts_with("plans/")
        && PROHIBITED_SOURCE_IDENTITY
            .iter()
            .any(|marker| text.contains(marker))
    {
        failures.push(format!("source-identity drift: {relative_path}"));
    }
    if let Some(threshold) = warn_threshold(&relative_path) {
        let size = file_bytes(path)?;
        if size > threshold {
            warnings.push(format!(
                "weight warning: {relative_path} is {size} bytes (threshold {threshold})"
            ));
        }
    }
    Ok(())
}

/// Per-file weight threshold by tree position; `None` means unbounded.
fn warn_threshold(relative_path: &str) -> Option<u64> {
    if is_generated_register(relative_path) {
        // A regenerated output is sized by its inputs, not by an author,
        // so a per-file warning could only ask for the generator to lie.
        // Same reasoning as the verbatim archive below, which is why
        // both now share the archive budget.
        return None;
    }
    let mut parts = relative_path.split('/');
    let tree = parts.next().unwrap_or_default();
    let group = parts.next();
    if relative_path.ends_with("/README.md") || relative_path == "README.md" {
        // An index README is authored maintenance prose wherever it
        // sits, including inside an archive directory, so it keeps the
        // ordinary README threshold.
        return Some(16 * 1024);
    }
    if is_archive(relative_path) {
        // A verbatim archive is unbounded per file: it records a named
        // tree exactly, so a size warning would only ever ask for the
        // record to be falsified.
        return None;
    }
    if tree == "adr" {
        return Some(14 * 1024);
    }
    match group {
        Some("decisions") => Some(12 * 1024),
        Some("packages") => Some(24 * 1024),
        Some("phases") => Some(16 * 1024),
        Some("research") => Some(32 * 1024),
        Some("reference") => Some(40 * 1024),
        _ => None,
    }
}

/// The numbered phase cards on disk, and which of them are Active.
struct PhaseCards {
    numbered: BTreeSet<u32>,
    active: Vec<u32>,
}

/// Read `plans/phases`, collecting every `NN-`-prefixed card.
///
/// Entries are visited in sorted order, so the collected values do not
/// depend on directory traversal order.
fn phase_cards(root: &Path) -> anyhow::Result<PhaseCards> {
    let mut cards = PhaseCards {
        numbered: BTreeSet::new(),
        active: Vec::new(),
    };
    for entry in read_dir_sorted(&root.join("plans/phases"))? {
        let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if entry.extension().is_none_or(|extension| extension != "md") {
            continue;
        }
        let numbered = name.len() > 3
            && name.as_bytes()[..2].iter().all(u8::is_ascii_digit)
            && name.as_bytes()[2] == b'-';
        if !numbered {
            continue;
        }
        let Ok(number) = name[..2].parse::<u32>() else {
            continue;
        };
        cards.numbered.insert(number);
        if ACTIVE_STATUS.is_match(&read_text(&entry)?) {
            cards.active.push(number);
        }
    }
    cards.active.sort_unstable();
    Ok(cards)
}

/// The numeric phase `pattern` declares in `text`.
///
/// A missing declaration and an unparsable one are distinct explicit
/// failures: a weld that skips what it cannot read is not a weld.
fn declared_phase(
    text: &str,
    pattern: &regex::Regex,
    file: &str,
    subject: &str,
    failures: &mut Vec<String>,
) -> Option<u32> {
    let Some(capture) = pattern.captures(text) else {
        failures.push(format!("phase: {file} declares no {subject}"));
        return None;
    };
    let declared = capture[1].trim();
    if let Ok(phase) = declared.parse::<u32>() {
        return Some(phase);
    }
    failures.push(format!(
        "phase: {file} declares a malformed {subject}: \"{declared}\""
    ));
    None
}

/// Weld every current-phase declaration in the planning tree (T8).
///
/// The backlog gate, the roadmap status, the plans README, and the one
/// Active numbered phase card each state the current phase. Only the
/// gate was ever checked, so `plans/README.md` sat a whole phase behind
/// the rest of the tree without any lane noticing. All four are
/// normalized to the numeric phase and compared.
///
/// The single Active card is the reference when there is exactly one,
/// because a card is structural rather than prose; when the Active set
/// is not a singleton that is itself reported and the backlog gate
/// becomes the reference for the two prose declarations.
fn verify_phase_gate(root: &Path, failures: &mut Vec<String>) -> anyhow::Result<()> {
    let mut phase_failures = Vec::new();

    let backlog = read_text(&root.join("plans/backlog.md"))?;
    let gate = declared_phase(
        &backlog,
        &CURRENT_GATE,
        "plans/backlog.md",
        "current gate",
        &mut phase_failures,
    );
    let readme = read_text(&root.join("plans/README.md"))?;
    let readme_phase = declared_phase(
        &readme,
        &CURRENT_PHASE,
        "plans/README.md",
        "current phase",
        &mut phase_failures,
    );
    let roadmap = read_text(&root.join("plans/roadmap.md"))?;
    let roadmap_phase = declared_phase(
        &roadmap,
        &CURRENT_PHASE,
        "plans/roadmap.md",
        "current phase",
        &mut phase_failures,
    );

    let cards = phase_cards(root)?;
    let mut compared = vec![
        ("plans/README.md", readme_phase),
        ("plans/roadmap.md", roadmap_phase),
    ];
    let reference = if let [card] = cards.active.as_slice() {
        compared.push(("plans/backlog.md", gate));
        Some((*card, "the active phase card"))
    } else {
        phase_failures.push(format!(
            "phase: exactly one phase card must be Active, found {}",
            cards.active.len()
        ));
        gate.map(|phase| (phase, "the backlog current gate"))
    };

    if let Some((expected, source)) = reference {
        for (file, declared) in compared {
            let Some(declared) = declared else {
                continue;
            };
            if declared != expected {
                phase_failures.push(format!(
                    "phase: {file} declares Phase {declared} but {source} is Phase {expected}"
                ));
            }
        }
    }

    if let Some(gate) = gate
        && !cards.numbered.contains(&gate)
    {
        phase_failures.push(format!(
            "phase: backlog current gate Phase {gate} has no phase card"
        ));
    }

    phase_failures.sort();
    failures.append(&mut phase_failures);
    Ok(())
}

/// Weld each backlog summary-table status to its own task section
/// (S7).
///
/// The registers carry a status twice: once in a summary table row and
/// once in the task's own `**Status:**` line. Nothing kept them equal,
/// and they drifted — five closed tasks kept `TODO` headers under a
/// table that already said `DONE`. A reader following the register to
/// the task got the stale answer.
///
/// A task ID is a backticked cell in a row whose later cells include a
/// status word; its section is `### <ID> —`. Only IDs appearing in both
/// places are compared, so a table without task sections, or prose
/// mentioning an ID, is not forced into the check.
fn verify_task_status_agreement(root: &Path, failures: &mut Vec<String>) -> anyhow::Result<()> {
    const STATUSES: [&str; 7] = [
        "TODO",
        "DONE",
        "BLOCKED",
        "PARKED",
        "DROPPED",
        "HISTORICAL",
        "IN PROGRESS",
    ];

    let backlog = read_text(&root.join("plans/backlog.md"))?;

    let mut table: BTreeMap<String, String> = BTreeMap::new();
    let mut sections: BTreeMap<String, String> = BTreeMap::new();
    let mut current: Option<String> = None;

    for line in backlog.lines() {
        if let Some(rest) = line.strip_prefix("### ") {
            current = TASK_HEADING
                .captures(rest)
                .map(|capture| capture[1].to_owned());
            continue;
        }
        if let Some(id) = &current
            && let Some(rest) = line.strip_prefix("**Status:** ")
            && let Some(status) = STATUSES.iter().find(|status| rest.starts_with(**status))
        {
            sections
                .entry(id.clone())
                .or_insert_with(|| (*status).to_owned());
            continue;
        }
        if !line.starts_with("| `") {
            continue;
        }
        let cells = line.split('|').map(str::trim).collect::<Vec<_>>();
        let Some(id) = cells
            .iter()
            .find_map(|cell| cell.strip_prefix('`').and_then(|c| c.strip_suffix('`')))
        else {
            continue;
        };
        if let Some(status) = cells.iter().find(|cell| STATUSES.contains(&(**cell))) {
            table.insert(id.to_owned(), (*status).to_owned());
        }
    }

    for (id, row_status) in &table {
        let Some(section_status) = sections.get(id) else {
            continue;
        };
        if row_status != section_status {
            failures.push(format!(
                "status: task {id} is {row_status} in its summary table but \
                 {section_status} in its own section"
            ));
        }
    }
    Ok(())
}

/// Every backticked row identifier in one backlog table is distinct.
///
/// A finding or task identifier is a permanent key: it names one row for
/// good, and prose elsewhere cites it. Two rows sharing one identifier
/// give two answers to what that key names, and a reader following a
/// citation reaches whichever the reader happened to find first — which
/// is how a resolved finding and an open one came to share one number
/// until the sixth static review noticed.
///
/// Scoped to one table, because the tables are separate namespaces: a
/// review register and a toolchain register number their own rows.
fn verify_unique_row_ids(root: &Path, failures: &mut Vec<String>) -> anyhow::Result<()> {
    let backlog = read_text(&root.join("plans/backlog.md"))?;
    failures.extend(duplicate_row_ids(&backlog));
    Ok(())
}

/// The duplicate-identifier failures in one Markdown document.
///
/// Split from the file read so that the scan itself is testable over
/// stated text rather than only over the tree it polices.
pub(crate) fn duplicate_row_ids(markdown: &str) -> Vec<String> {
    let mut failures = Vec::new();
    let mut table = String::from("(before any heading)");
    let mut seen: BTreeSet<String> = BTreeSet::new();

    for line in without_fenced_lines(markdown).lines() {
        if let Some(rest) = line
            .strip_prefix("### ")
            .or_else(|| line.strip_prefix("## "))
        {
            table.clear();
            table.push_str(rest.trim());
            seen.clear();
            continue;
        }
        // The identifier is the row's first cell, and only there: a
        // backticked path or label further along the row is data.
        let Some(id) = line
            .split('|')
            .nth(1)
            .map(str::trim)
            .and_then(|cell| cell.strip_prefix('`'))
            .and_then(|cell| cell.strip_suffix('`'))
        else {
            continue;
        };
        if !seen.insert(id.to_owned()) {
            failures.push(format!(
                "identifiers: {id} names more than one row under \"{table}\""
            ));
        }
    }
    failures
}

/// Markdown bytes for the `adr/` and `plans/` trees, split by budget.
struct TreeBytes {
    adr: u64,
    plans: u64,
    archive: u64,
}

/// Total Markdown bytes for the `adr/` and `plans/` trees, with the
/// archived documents and generated registers separated out of the
/// load-bearing totals.
fn tree_bytes(root: &Path, files: &[PathBuf]) -> anyhow::Result<TreeBytes> {
    let mut totals = TreeBytes {
        adr: 0,
        plans: 0,
        archive: 0,
    };
    for path in files {
        let size = file_bytes(path)?;
        let relative_path = relative(root, path);
        if is_archive_weight(&relative_path) {
            totals.archive += size;
        } else if relative_path.starts_with("adr/") {
            totals.adr += size;
        } else {
            totals.plans += size;
        }
    }
    Ok(totals)
}

/// True for a path inside an archive directory: a verbatim record,
/// either of a document this repository did not author or of its own
/// closed history under `plans/history/`.
///
/// Head validation is what asks this question, and it is skipped for
/// both: an acceptee validates its own heads, never its authority's,
/// and a closed record is evidence of the structure that held when it
/// was written rather than a claim about the structure in force now.
///
/// Public because authorship, not weight, is the question head
/// validation asks of a plans path, and the answer must come from the
/// one place the archive is defined rather than from a second list that
/// could drift from this one.
#[must_use]
pub fn is_archived(relative_path: &str) -> bool {
    is_archive(relative_path)
}

/// True for a path inside an archive directory.
fn is_archive(relative_path: &str) -> bool {
    ARCHIVE_DIRECTORIES
        .iter()
        .any(|directory| relative_path.starts_with(directory))
}

/// True for a generated register publication.
fn is_generated_register(relative_path: &str) -> bool {
    GENERATED_REGISTERS.contains(&relative_path)
}

/// True for a document charged to the archive budget rather than the
/// maintained-prose budget.
///
/// The test is by role, not by directory: an archived verbatim record,
/// or a generated register publication. `plans/labels/README.md` is
/// authored index prose sitting beside the registers, so it stays in the
/// combined budget — which is why the register set comes from
/// [`GENERATED_REGISTERS`] and never from the directory it lives in.
fn is_archive_weight(relative_path: &str) -> bool {
    is_archive(relative_path) || is_generated_register(relative_path)
}

fn file_bytes(path: &Path) -> anyhow::Result<u64> {
    Ok(fs::metadata(path)
        .with_context(|| format!("reading metadata for {}", path.display()))?
        .len())
}

fn read_text(path: &Path) -> anyhow::Result<String> {
    fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
}

/// Repository-relative display path with `/` separators.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// A minimal valid tree: indexed READMEs, four agreeing Phase 2
    /// declarations, one Active card.
    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        fs::create_dir_all(root.join("adr")).expect("adr");
        fs::create_dir_all(root.join("plans/phases")).expect("phases");
        fs::write(root.join("adr/README.md"), "# ADRs\n").expect("adr readme");
        fs::write(
            root.join("plans/README.md"),
            "# Plans\n\nbacklog.md roadmap.md phases/README.md\n\nCurrent: Phase 2 - pilot\n",
        )
        .expect("plans readme");
        fs::write(
            root.join("plans/roadmap.md"),
            "# Roadmap\n\nCurrent: Phase 2 - pilot\n",
        )
        .expect("roadmap");
        fs::write(
            root.join("plans/phases/README.md"),
            "# Phases\n\n02-pilot.md\n",
        )
        .expect("phases readme");
        fs::write(
            root.join("plans/backlog.md"),
            "# Backlog\n\n> **Current gate:** Phase 2\n",
        )
        .expect("backlog");
        fs::write(
            root.join("plans/phases/02-pilot.md"),
            "# Pilot\n\n> **Status:** Active\n",
        )
        .expect("card");
        dir
    }

    /// Every `phase:` diagnostic the tree reports, in reported order.
    fn phase_failures(outcome: &PlansOutcome) -> Vec<String> {
        outcome
            .failures
            .iter()
            .filter(|failure| failure.starts_with("phase: "))
            .cloned()
            .collect()
    }

    fn subjects(root: &Path) -> Vec<PathBuf> {
        markdown_files(&root.canonicalize().expect("canonical root")).expect("discovery")
    }

    #[test]
    fn valid_fixture_tree_passes() {
        let dir = fixture();
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert_eq!(outcome.failures, Vec::<String>::new());
        assert!(outcome.report.valid);
        assert_eq!(outcome.report.files_checked, 6);
        assert!(outcome.report.combined_bytes > 0);
    }

    #[test]
    fn task_status_drift_between_table_and_section_fails() {
        // S7: the registers state each status twice. This is the check
        // that keeps the two equal — five closed A17 tasks once kept
        // TODO headers under a table already reading DONE.
        let dir = fixture();
        fs::write(
            dir.path().join("plans/backlog.md"),
            concat!(
                "# Backlog\n\n> **Current gate:** Phase 2\n\n",
                "| ID | Status | Deliverable |\n",
                "|---|---|---|\n",
                "| `X1-001` | DONE | Agrees |\n",
                "| `X1-002` | DONE | Drifts |\n\n",
                "### X1-001 — Agrees\n\n**Status:** DONE\n\n",
                "### X1-002 — Drifts\n\n**Status:** TODO\n",
            ),
        )
        .expect("backlog");

        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");

        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("task X1-002") && failure.contains("DONE")),
            "{:#?}",
            outcome.failures,
        );
        assert!(
            !outcome
                .failures
                .iter()
                .any(|failure| failure.contains("task X1-001")),
            "an agreeing task must not be reported: {:#?}",
            outcome.failures,
        );
        assert!(!outcome.report.valid);
    }

    #[test]
    fn census_disagreement_fails_both_ways() {
        let dir = fixture();
        let mut declared = subjects(dir.path());
        declared.pop();
        declared.push(dir.path().join("plans/ghost.md"));
        let outcome = check_plans(dir.path(), &declared).expect("check runs");
        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("declared subject absent on disk"))
        );
        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("file outside the build census"))
        );
        assert!(!outcome.report.valid);
    }

    #[test]
    fn empty_declaration_fails_closed_on_a_nonempty_tree() {
        // ADR-014: discovery is a verifier, never the membership
        // source. An empty declared census must not turn the on-disk
        // walk into the authority — every discovered file is outside
        // the (empty) census and the check fails closed.
        let dir = fixture();
        let outcome = check_plans(dir.path(), &[]).expect("check runs");
        assert!(!outcome.report.valid);
        let census_failures = outcome
            .failures
            .iter()
            .filter(|failure| failure.contains("file outside the build census"))
            .count();
        assert_eq!(census_failures, 6, "{:?}", outcome.failures);
    }

    #[test]
    fn empty_declaration_against_an_empty_tree_is_an_environmental_fault() {
        // The chosen empty-versus-empty behavior: the census check is
        // vacuously satisfied and contributes nothing, and the run
        // still fails — a tree with no plans/backlog.md is not a
        // planning tree, reported as an environmental fault rather
        // than a quietly valid empty census.
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("adr")).expect("adr");
        fs::create_dir_all(dir.path().join("plans")).expect("plans");
        let error =
            check_plans(dir.path(), &[]).expect_err("an empty planning tree must not validate");
        assert!(error.to_string().contains("backlog.md"), "{error:#}");
    }

    #[test]
    fn unindexed_file_and_missing_readme_fail_ownership() {
        let dir = fixture();
        fs::write(dir.path().join("plans/orphan.md"), "# Orphan\n").expect("orphan");
        fs::create_dir(dir.path().join("plans/rogue")).expect("rogue dir");
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("ownership: plans/orphan.md not indexed"))
        );
        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.contains("ownership: plans/rogue has no README.md"))
        );
    }

    #[test]
    fn structure_hygiene_defects_are_reported() {
        let dir = fixture();
        fs::write(
            dir.path().join("adr/README.md"),
            "not a heading\n\n[gone](missing.md)\n\nBelow is a complete draft\n\nTODO_URL\n\n95% confident\n",
        )
        .expect("defective readme");
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        let all = outcome.failures.join("\n");
        assert!(all.contains("heading: adr/README.md"));
        assert!(all.contains("broken link: adr/README.md -> missing.md"));
        assert!(all.contains("draft scaffolding: adr/README.md"));
        assert!(all.contains("placeholder data: adr/README.md"));
        assert!(all.contains("confidence percentage: adr/README.md"));
    }

    #[test]
    fn agreeing_phase_declarations_pass() {
        // T8: backlog gate, roadmap, plans README, and the one Active
        // card all say Phase 2 in the fixture.
        let dir = fixture();
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert_eq!(phase_failures(&outcome), Vec::<String>::new());
    }

    #[test]
    fn stale_plans_readme_phase_fails() {
        // The finding itself: plans/README.md sat on Phase 1 while the
        // rest of the tree had moved to Phase 2, and nothing looked.
        let dir = fixture();
        fs::write(
            dir.path().join("plans/README.md"),
            "# Plans\n\nbacklog.md roadmap.md phases/README.md\n\nCurrent: Phase 1 - stale\n",
        )
        .expect("stale readme");
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert_eq!(
            phase_failures(&outcome),
            vec![
                "phase: plans/README.md declares Phase 1 but the active phase card is Phase 2"
                    .to_owned()
            ],
        );
        assert!(!outcome.report.valid);
    }

    #[test]
    fn stale_roadmap_phase_fails() {
        let dir = fixture();
        fs::write(
            dir.path().join("plans/roadmap.md"),
            "# Roadmap\n\nCurrent: Phase 3 - ahead\n",
        )
        .expect("stale roadmap");
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert_eq!(
            phase_failures(&outcome),
            vec![
                "phase: plans/roadmap.md declares Phase 3 but the active phase card is Phase 2"
                    .to_owned()
            ],
        );
    }

    #[test]
    fn stale_backlog_gate_fails() {
        // The gate is compared too, not merely used as the reference:
        // a card is structural, the gate is prose.
        let dir = fixture();
        fs::write(
            dir.path().join("plans/backlog.md"),
            "# Backlog\n\n> **Current gate:** Phase 1\n",
        )
        .expect("stale gate");
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert!(
            phase_failures(&outcome).contains(
                &"phase: plans/backlog.md declares Phase 1 but the active phase card is Phase 2"
                    .to_owned()
            ),
            "{:#?}",
            outcome.failures,
        );
    }

    #[test]
    fn no_active_phase_card_fails() {
        let dir = fixture();
        fs::write(
            dir.path().join("plans/phases/02-pilot.md"),
            "# Pilot\n\n> **Status:** Complete\n",
        )
        .expect("retired card");
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert_eq!(
            phase_failures(&outcome),
            vec!["phase: exactly one phase card must be Active, found 0".to_owned()],
        );
    }

    #[test]
    fn two_active_phase_cards_fail() {
        let dir = fixture();
        fs::write(
            dir.path().join("plans/phases/03-second.md"),
            "# Second\n\n> **Status:** Active\n",
        )
        .expect("second card");
        fs::write(
            dir.path().join("plans/phases/README.md"),
            "# Phases\n\n02-pilot.md 03-second.md\n",
        )
        .expect("phases readme");
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert_eq!(
            phase_failures(&outcome),
            vec!["phase: exactly one phase card must be Active, found 2".to_owned()],
        );
    }

    #[test]
    fn malformed_current_phase_declaration_fails() {
        let dir = fixture();
        fs::write(
            dir.path().join("plans/roadmap.md"),
            "# Roadmap\n\nCurrent: Phase two - words\n",
        )
        .expect("malformed roadmap");
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert_eq!(
            phase_failures(&outcome),
            vec!["phase: plans/roadmap.md declares a malformed current phase: \"two\"".to_owned()],
        );
        assert!(!outcome.report.valid);
    }

    #[test]
    fn missing_current_phase_declaration_fails() {
        let dir = fixture();
        fs::write(
            dir.path().join("plans/roadmap.md"),
            "# Roadmap\n\nNothing.\n",
        )
        .expect("silent roadmap");
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert_eq!(
            phase_failures(&outcome),
            vec!["phase: plans/roadmap.md declares no current phase".to_owned()],
        );
    }

    #[test]
    fn current_gate_without_a_phase_card_fails() {
        let dir = fixture();
        for (path, text) in [
            (
                "plans/backlog.md",
                "# Backlog\n\n> **Current gate:** Phase 7\n",
            ),
            (
                "plans/README.md",
                "# Plans\n\nbacklog.md roadmap.md phases/README.md\n\nCurrent: Phase 7 - absent\n",
            ),
            (
                "plans/roadmap.md",
                "# Roadmap\n\nCurrent: Phase 7 - absent\n",
            ),
        ] {
            fs::write(dir.path().join(path), text).expect("phase 7 declaration");
        }
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert!(
            phase_failures(&outcome)
                .contains(&"phase: backlog current gate Phase 7 has no phase card".to_owned()),
            "{:#?}",
            outcome.failures,
        );
    }

    #[test]
    fn phase_diagnostics_do_not_depend_on_traversal_order() {
        // Diagnostics are sorted before they join the failure list, so
        // two trees differing only in the order their cards were
        // written report the same defects in the same order.
        let mut reports = Vec::new();
        for order in [
            ["03-second.md", "02-pilot.md"],
            ["02-pilot.md", "03-second.md"],
        ] {
            let dir = fixture();
            fs::remove_file(dir.path().join("plans/phases/02-pilot.md")).expect("clear card");
            for name in order {
                fs::write(
                    dir.path().join("plans/phases").join(name),
                    "# Card\n\n> **Status:** Active\n",
                )
                .expect("card");
            }
            fs::write(
                dir.path().join("plans/phases/README.md"),
                "# Phases\n\n02-pilot.md 03-second.md\n",
            )
            .expect("phases readme");
            let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
            reports.push(phase_failures(&outcome));
        }
        assert_eq!(reports[0], reports[1]);
        assert!(!reports[0].is_empty());
    }

    #[test]
    fn weight_thresholds_follow_tree_position() {
        assert_eq!(warn_threshold("plans/labels/specification.md"), None);
        assert_eq!(warn_threshold("adr/README.md"), Some(16 * 1024));
        assert_eq!(warn_threshold("adr/010-x.md"), Some(14 * 1024));
        assert_eq!(warn_threshold("plans/decisions/x.md"), Some(12 * 1024));
        assert_eq!(
            warn_threshold("plans/packages/errors/x.md"),
            Some(24 * 1024)
        );
        assert_eq!(warn_threshold("plans/research/x.md"), Some(32 * 1024));
        assert_eq!(warn_threshold("plans/backlog.md"), None);
        // Verbatim archives are unbounded per file; their index
        // READMEs are ordinary authored prose and stay bounded.
        assert_eq!(warn_threshold("plans/guides/guide_seven.md"), None);
        assert_eq!(warn_threshold("plans/reviews/review-2-0.2.3-dev.md"), None);
        assert_eq!(warn_threshold("plans/guides/README.md"), Some(16 * 1024));
        assert_eq!(warn_threshold("plans/reviews/README.md"), Some(16 * 1024));
        // Every generated register is unbounded per file, and the
        // authored README beside them is not.
        for register in GENERATED_REGISTERS {
            assert_eq!(warn_threshold(register), None, "{register}");
        }
        assert_eq!(warn_threshold("plans/labels/README.md"), Some(16 * 1024));
    }

    #[test]
    fn the_archive_budget_is_a_role_not_a_directory() {
        for register in GENERATED_REGISTERS {
            assert!(is_archive_weight(register), "{register}");
        }
        for directory in ARCHIVE_DIRECTORIES {
            assert!(is_archive_weight(&format!("{directory}archived.md")));
        }
        // Authored prose beside the registers keeps the maintained
        // budget: the role decides, not the directory.
        assert!(!is_archive_weight("plans/labels/README.md"));
        assert!(!is_archive_weight("plans/backlog.md"));
        // Weight class and authorship are separate questions: a
        // register is authored by this repository's generators, so head
        // validation still governs it.
        for register in GENERATED_REGISTERS {
            assert!(!is_archived(register), "{register}");
        }
    }

    #[test]
    fn generated_registers_match_the_census() {
        // The weld: `census::discover` builds its register paths from
        // the same constants, so a register added there without being
        // named in GENERATED_REGISTERS fails here rather than silently
        // landing in the maintained-prose budget.
        let dir = fixture();
        let census = crate::census::RepositoryCensus::discover(dir.path());
        let root = dir.path();
        let discovered: BTreeSet<String> = [
            census.specification_register,
            census.realization_register,
            census.attestation_register,
        ]
        .iter()
        .map(|path| relative(root, path))
        .collect();
        let declared: BTreeSet<String> = GENERATED_REGISTERS
            .iter()
            .map(|register| (*register).to_owned())
            .collect();
        assert_eq!(discovered, declared);
    }

    #[test]
    fn register_bytes_are_charged_to_the_archive_budget() {
        let dir = fixture();
        fs::create_dir_all(dir.path().join("plans/labels")).expect("labels dir");
        fs::write(
            dir.path().join("plans/labels/README.md"),
            "# Labels\n\nspecification.md\n",
        )
        .expect("labels readme");
        let mut register = String::from("# Specification register\n\n");
        register.push_str(&"x".repeat(64 * 1024));
        register.push('\n');
        fs::write(dir.path().join(SPECIFICATION_REGISTER), register).expect("register");
        let readme = dir.path().join("plans/README.md");
        let mut index = fs::read_to_string(&readme).expect("plans readme");
        index.push_str("\nlabels/README.md\n");
        fs::write(&readme, index).expect("indexed labels");

        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert!(outcome.report.valid, "{:#?}", outcome.failures);
        assert!(
            outcome.report.archive_bytes > 64 * 1024,
            "register bytes belong to the archive budget: {}",
            outcome.report.archive_bytes,
        );
        // The authored README beside it still counts as plans prose.
        assert!(outcome.report.plans_bytes > 0);
        assert!(
            outcome.report.combined_bytes < 64 * 1024,
            "combined {} absorbed the register",
            outcome.report.combined_bytes,
        );
        assert!(
            !outcome
                .warnings
                .iter()
                .any(|warning| warning.contains("labels/specification.md")),
            "{:#?}",
            outcome.warnings,
        );
    }

    /// Add an archive directory holding `size` bytes of verbatim
    /// document, indexed by its own README.
    fn write_archive(root: &Path, group: &str, size: usize) {
        fs::create_dir_all(root.join("plans").join(group)).expect("archive dir");
        fs::write(
            root.join("plans").join(group).join("README.md"),
            format!("# {group}\n\narchived.md\n"),
        )
        .expect("archive readme");
        let mut body = String::from("# Archived\n\n");
        body.push_str(&"x".repeat(size));
        body.push('\n');
        fs::write(root.join("plans").join(group).join("archived.md"), body)
            .expect("archived document");
        let readme = root.join("plans/README.md");
        let mut index = fs::read_to_string(&readme).expect("plans readme");
        index.push('\n');
        index.push_str(group);
        index.push_str("/README.md\n");
        fs::write(&readme, index).expect("indexed archive");
    }

    #[test]
    fn archive_bytes_are_excluded_from_the_combined_budget() {
        // The point of the split: an archive directory can hold more
        // than the whole load-bearing cap without moving
        // combined_bytes at all.
        let dir = fixture();
        let bare = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        let bare_combined = bare.report.combined_bytes;
        assert_eq!(bare.report.archive_bytes, 0);

        write_archive(dir.path(), "guides", 900 * 1024);
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");

        assert!(outcome.report.valid, "{:#?}", outcome.failures);
        assert!(
            outcome.report.archive_bytes > HARD_CAP_BYTES,
            "the archive alone must exceed the load-bearing cap: {}",
            outcome.report.archive_bytes,
        );
        // The archive README and the plans README index line are the
        // only load-bearing growth; the 900 KiB document is not.
        assert!(
            outcome.report.combined_bytes < bare_combined + 1024,
            "combined {} grew from {bare_combined}",
            outcome.report.combined_bytes,
        );
        assert_eq!(
            outcome.report.combined_bytes,
            outcome.report.adr_bytes + outcome.report.plans_bytes
        );
        assert_eq!(
            outcome.report.archive_hard_cap_bytes,
            ARCHIVE_HARD_CAP_BYTES
        );
    }

    #[test]
    fn both_archive_directories_are_accounted_as_archive() {
        let dir = fixture();
        write_archive(dir.path(), "guides", 4 * 1024);
        write_archive(dir.path(), "reviews", 8 * 1024);
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert!(outcome.report.valid, "{:#?}", outcome.failures);
        assert!(
            outcome.report.archive_bytes > 12 * 1024,
            "{}",
            outcome.report.archive_bytes
        );
    }

    #[test]
    fn an_oversize_archive_fails_with_a_focused_message() {
        let dir = fixture();
        let over = usize::try_from(ARCHIVE_HARD_CAP_BYTES).expect("cap fits in usize") + 1;
        write_archive(dir.path(), "reviews", over);
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert!(!outcome.report.valid);
        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.starts_with("weight: archived Markdown")
                    && failure.contains("archive hard cap")),
            "{:#?}",
            outcome.failures,
        );
        // The load-bearing cap is untouched by archive growth, so the
        // combined diagnostic must not also fire.
        assert!(
            !outcome
                .failures
                .iter()
                .any(|failure| failure.starts_with("weight: combined Markdown")),
            "{:#?}",
            outcome.failures,
        );
    }

    #[test]
    fn the_load_bearing_cap_still_fails_on_maintained_prose() {
        // Archive growth must not have loosened the core guardrail:
        // an oversize maintained document still trips it.
        let dir = fixture();
        write_archive(dir.path(), "guides", 16 * 1024);
        let mut heavy = String::from("# Heavy\n\n");
        heavy.push_str(&"x".repeat(usize::try_from(HARD_CAP_BYTES).expect("cap fits in usize")));
        fs::write(dir.path().join("plans/research.md"), heavy).expect("heavy plan");
        let readme = dir.path().join("plans/README.md");
        let mut index = fs::read_to_string(&readme).expect("plans readme");
        index.push_str("\nresearch.md\n");
        fs::write(&readme, index).expect("indexed");

        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert!(
            outcome
                .failures
                .iter()
                .any(|failure| failure.starts_with("weight: combined Markdown")),
            "{:#?}",
            outcome.failures,
        );
        assert!(outcome.report.archive_bytes > 0);
    }

    #[test]
    fn oversize_file_warns_without_failing() {
        let dir = fixture();
        let mut heavy = String::from("# Heavy\n\n");
        heavy.push_str(&"x".repeat(15 * 1024));
        fs::write(dir.path().join("adr/001-heavy.md"), heavy).expect("heavy adr");
        fs::write(dir.path().join("adr/README.md"), "# ADRs\n\n001-heavy.md\n")
            .expect("adr readme");
        let outcome = check_plans(dir.path(), &subjects(dir.path())).expect("check runs");
        assert!(outcome.report.valid, "{:?}", outcome.failures);
        assert!(
            outcome
                .warnings
                .iter()
                .any(|warning| warning.contains("weight warning: adr/001-heavy.md"))
        );
        assert_eq!(outcome.report.warnings, outcome.warnings.len());
    }
}
