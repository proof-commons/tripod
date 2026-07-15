//! Guard-listing weld: `listing:domains:guard` is a projection of the
//! model's failure vocabularies — complete, and inventing nothing.
//!
//! Implements `´test:verification:guard-listing-weld´`: the machine
//! welds identifiers; humans own only the comments. Comments inside the
//! listing fences stay lowercase (the 𝗜-glyphs are not ASCII CamelCase
//! and fall through), so every identifier-shaped token in the fenced
//! blocks must be a real variant name.
//!
//! The v13 markdown has landed; the document check is live. The
//! corrected listing is held here as a fixture — the paste source for
//! the document — and the projection logic runs against it
//! unconditionally.

use std::collections::BTreeSet;
use std::path::Path;

use crate::*;

const DOC_RELATIVE_PATH: &str = "../../docs/attestation/realization.md";

fn doc_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(DOC_RELATIVE_PATH)
}

const ALLOWED_NON_VARIANTS: &[&str] = &["Guard", "InvariantError"];

/// Concatenates the ```-fenced blocks that follow the mint line
/// carrying the backticked `listing:domains:guard` label, up to the next
/// heading.
fn listing_fences(doc: &str) -> String {
    let mut block = String::new();
    let mut seen_label = false;
    let mut in_fence = false;

    for line in doc.lines() {
        if !seen_label {
            seen_label = line.contains("`listing:domains:guard`");
            continue;
        }

        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }

        if in_fence {
            block.push_str(line);
            block.push('\n');
        } else if line.starts_with('#') {
            break;
        }
    }

    block
}

/// Maximal alphanumeric words shaped `[A-Z][A-Za-z0-9]+`; single
/// letters, lowercase prose, and non-ASCII glyphs fall through.
fn camel_tokens(text: &str) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();

    for word in text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
        let mut chars = word.chars();

        if let Some(first) = chars.next()
            && first.is_ascii_uppercase()
            && word.len() >= 2
            && chars.all(|c| c.is_ascii_alphanumeric())
        {
            tokens.insert(word.to_owned());
        }
    }

    tokens
}

/// Omission and invention findings for a candidate listing body.
fn projection_violations(block: &str) -> Vec<String> {
    let tokens = camel_tokens(block);

    let guards: BTreeSet<&str> = Guard::ALL.iter().map(|g| g.name()).collect();
    let reasons: BTreeSet<&str> = InvariantError::ALL.iter().map(|r| r.name()).collect();

    let mut violations = Vec::new();

    // Completeness: every variant appears.
    for name in guards.iter().chain(&reasons) {
        if !tokens.contains(*name) {
            violations.push(format!("listing omits {name}"));
        }
    }

    // No inventions: every identifier-shaped token is real.
    for token in &tokens {
        if !guards.contains(token.as_str())
            && !reasons.contains(token.as_str())
            && !ALLOWED_NON_VARIANTS.contains(&token.as_str())
        {
            violations.push(format!("listing invents {token}"));
        }
    }

    violations
}

/// The corrected `listing:domains:guard` body: the paste source for the
/// document, welded here until the document lands and is welded itself.
const LISTING_FIXTURE: &str = r"
`listing:domains:guard`
```rust
pub enum Guard {                               // why a *transition* was rejected
    // recognition & shape
    NoSuch, WrongAsset, WrongShape, WrongPool, WrongTarget, WrongClass,
    // domains & arithmetic
    Domain, BadConstant, Overflow, Underflow, CycleOverflow,
    ActiveBackingCapExceeded,                  // the 𝗜₂ cap, by name
    // construction exactness
    DuplicateInput, DuplicateOutputIndex, MissingOutputIndex,
    // authorization
    BadSignature, BadAuthorization,
    // terminal & progress discipline
    Sealed, NoTrap, ZeroProgress,              // sealing, no-trap, and zero-progress paths
    // value & recipient pins
    OverDraw, ValuePin, PartitionPin, RecipientPin, ClassCross,
    // issuance & destruction
    MissingAuthority, BadIssuance, BadDestruction,
    // the two exact partitions
    CanonicalDeltaMismatch, OpenFlowMismatch,
    // fee & sponsor
    SponsorMismatch, FeeMismatch,
    // roots & welds
    RootMultiplicity, RootSuccession, ResvWeld, ControlVaultWeld,
    // cadence & maturity
    CadenceTooEarly, CadenceOperatorOnly,
    MaturityAlreadyAnnounced, MaturityLeadTooShort,
    MaturityLeadTooLong, MaturityNotComplete,
    // history, checkpoint, codec
    WrongCheckpoint, UnsupportedSchema, HistoryOrder, DuplicateEvent,
    // wrapper
    InvariantFailure,
}
```

```rust
pub enum InvariantError {                      // which *state/audit* clause failed → clause_of
    IdentityAuthority, CanonicalClosure,       // 𝗜₁
    Domains, ActiveBackingCap,                 // 𝗜₂
    Floor,                                     // 𝗜₃  — Y > Ω lives here, not in Guard
    SealedTerminal,                            // 𝗜₄
    Backing,                                   // 𝗜₅
    DistributionPayability,                    // 𝗜₆
    EntitlementLifecycle,                      // 𝗜₇
    ReceiptAccountingPreMaturity,
    ReceiptAccountingPostMaturity,             // 𝗜₈
    ConsensusValueAuthority,                   // 𝗜₉ — declared, never produced
    StateSuccession, ResvSuccession,
    HistoryProjection,                         // 𝗜₁₀
    MaturityCoherence,                         // 𝗜₁₁
}
```

### next heading
";

#[test]
fn listing_fixture_is_a_projection_of_the_model() {
    let violations = projection_violations(&listing_fences(LISTING_FIXTURE));

    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn projection_rejects_an_invented_variant() {
    // The retired `Insolvent` name must be reported as an invention.
    let forged = LISTING_FIXTURE.replace(
        "    Floor,",
        "    Insolvent,                                 // Y > Ω\n    Floor,",
    );

    let violations = projection_violations(&listing_fences(&forged));

    assert!(violations.contains(&"listing invents Insolvent".to_owned()));
}

#[test]
fn projection_rejects_an_omitted_variant() {
    let truncated = LISTING_FIXTURE.replace("    InvariantFailure,\n", "");

    let violations = projection_violations(&listing_fences(&truncated));

    assert!(violations.contains(&"listing omits InvariantFailure".to_owned()));
}

#[test]
fn fences_stop_at_the_next_heading() {
    let block = listing_fences(LISTING_FIXTURE);

    assert!(block.contains("pub enum Guard"));
    assert!(block.contains("pub enum InvariantError"));
    assert!(!block.contains("next heading"));
}

#[test]
fn guard_listing_is_a_projection_of_the_model() {
    let document = std::fs::read_to_string(doc_path())
        .expect("realization.md must exist once the label freeze is declared");

    let violations = projection_violations(&listing_fences(&document));

    assert!(violations.is_empty(), "{}", violations.join("\n"));
}
