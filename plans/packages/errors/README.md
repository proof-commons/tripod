# Illustrative Package Error Vocabularies

These files sketch typed fail-closed boundaries for planned packages. They are
implementation aids, not frozen public APIs.

## Purpose · `sec:error-sketches:purpose`

An error vocabulary identifies package ownership, identities that must agree,
invalid or incomplete inputs, assurance-boundary failures, and focused negative
tests. It prevents vague public boundaries such as `Result<T, anyhow::Error>`.

## Status · `rule:error-sketches:status`

Implementations may rename, split, merge equivalent internal variants, attach
typed context, or separate validation from runtime errors. They must preserve
the represented failure classes and fail-closed behavior unless an owning
contract or accepted decision changes.

## Index · `tab:error-sketches:index`

| Package | Error sketch |
|---|---|
| labels | [labels.md](labels.md) |
| realization | [realization.md](realization.md) |
| compiler | [compiler.md](compiler.md) |
| target-elements | [target-elements.md](target-elements.md) |
| tapscript | [tapscript.md](tapscript.md) |
| simplicity | [simplicity.md](simplicity.md) |
| linker | [linker.md](linker.md) |
| transaction | [transaction.md](transaction.md) |
| vectors | [vectors.md](vectors.md) |
| release | [release.md](release.md) |

## Design rules · `rule:error-sketches:design`

Public errors prefer typed context such as `MissingRelation(RelationId)` over
strings. Avoid public `Other(String)`. External I/O may retain a structured
source error. Display text is presentation, not error identity.

| Class | Meaning |
|---|---|
| invalid | Value violates its schema or semantic domain. |
| unsupported | Valid elsewhere but unsupported here. |
| incomplete | Required declaration or evidence is absent. |
| mismatch | Independently supplied values disagree. |
| ambiguous | More than one interpretation remains. |
| stale | Evidence or publication applies to an earlier identity. |
| unavailable | Infrastructure or required witness cannot be obtained. |
| rejected | Relevant semantic or target predicate failed. |
| nondeterministic | Equal explicit inputs produced unequal canonical output. |

Pure validators may aggregate independent defects deterministically. A
transformation that cannot safely proceed normally returns the first typed fatal
error. Errors and their `Debug` or `Display` forms never expose private keys,
nonces, blinders, unpublished openings, RPC credentials, URLs with credentials,
or secret-bearing argv.

Every release-relevant variant class requires focused positive or negative
coverage. Plan-local labels and Rust code blocks here are non-normative and
unlinted.
