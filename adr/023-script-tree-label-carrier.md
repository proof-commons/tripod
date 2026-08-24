# ADR-023: The Script Tree Enters the Label Carrier

**Status:** Decided and implemented
**Scope:** The Python sources under `scripts/`, and the two ADR-019
adoption parameters that kept them outside the label graph
**Amends:** two of ADR-019's adoption-parameter entries, the owner partition (`[ADR019-rule:labels:owner-partition]`) and the scanned-region recognition (`[ADR019-rule:labels:scanned-regions]`). Neither entry's own text is edited; both are read through this record
**Does not establish:** any new owner, any new prefix, any mint in the
script tree, and no scanning of shell sources

---

## Context · `sec:scripts:context`

The native executor's funding path mines each transaction directly
because the miner takes only what it is handed, and the same executor
boots its node with pegin validation disabled. Both adaptations are
load-bearing and both live in `scripts/elements-native-executor.py`,
which ADR-019 could not reach: it scanned Markdown, Rust, and LaTeX and
nothing else, and the census excluded `scripts/` outright.

So both friction rows said the register alone — the words for a friction
this repository does not adapt to. Deleting a mint is how a correction is
collected, the check failing at every citation that outlived it; these
two produced silence instead of that worklist. Backlog DI-F06 recorded
it as a gap in the carrier, not a property of the frictions.

---

## Decision · `dec:scripts:carrier`

The Python sources under `scripts/` join the label carrier. Two adoption-parameter entries are amended and no other.

**Owner partition Ω.** The partition gains one rule: `scripts/` to the
`DOC` owner, for files whose extension is `.py`. The signature Σ is
untouched, because no owner is added. `DOC` is already the residual
owner — the last rule of the partition, claiming authored repository
text outside the named trees, the root and per-directory READMEs among
them. Script comments are that same residual text, and the citations
this record exists to make possible are imports of `PLAN`-owned labels,
which need their region to have an owner rather than a register.

**Scanned-region recognition.** The entry gains a fourth language:

> Python: line comments, introduced by a number sign, with string
> literals of every quote form — single, double, and triple, prefixed or
> bare, docstrings included — excluded.

That is the Rust rule applied to Python for the Rust reason. The
executor is full of protocol text, RPC arguments, and fixture strings;
scanning them would manufacture citations and near misses out of data
the file merely carries. Docstrings go with the rest: separating one
from any other string means knowing a statement's position in its block,
which is a Python parser, and a parser is more than this record needs.

**Unchanged.** Σ, Π, K, the designated typed-data classes, and the
citation-index designations stand as ADR-019 fixes them.

---

## Consequences · `rem:scripts:consequences`

**A Python citation is written in a comment.** Where the reasoning lives
in a docstring, as at the executor's funding path, the citation sits in
a comment at the adapting line: it marks the site, the docstring keeps
the prose. A narrowness of the rule above, stated not repaired.

**Shell sources stay outside.** Nothing in them adapts to a friction,
and shell quoting — heredocs carrying whole scripts among it — needs a
scanner whose narrownesses are harder to state honestly than Python's.
A shell adaptation that needs a citation gets its own record.

**The census reaches the tree, which can mint.** The categorical
exclusion narrows from every path under `scripts/` to the shell sources
there, and the Python files are declared and re-discovered like any
other subject; a file added there that is neither Python nor shell fails
the census audit until it is classified. Nothing mints there today, but
a bare acute span in a Python comment would mint a `DOC` label, as one
in a Rust comment mints a crate label — the partition being total, not a
permission granted here.

---

## Rejected alternatives · `rem:scripts:rejected`

**Register a new `SCRIPT` owner.** It would amend the signature as well
as the partition, and register a prefix nothing mints under. An owner
whose register is empty is a promise the corpus does not keep, and a
citation of it could never resolve.

**Scan docstrings.** The pull is real, the reasoning being conventionally
written there. It needs a parser, and it would put the executor's
protocol prose inside the scanned region — which is what the string
exclusion exists to prevent.

---

## Adoption gate · `gate:scripts:carrier`

Adoption holds when:

- the two amended entries match the checker's partition data and its scanned-region table row for row, and the scanner scans comments and no string literal of any quote form;
- every tracked Python file under `scripts/` is a declared census
  subject, and the shell sources are the only categorical exclusion
  there;
- the two adaptations in the native executor cite the frictions they
  adapt to, and their register rows name those sites;
- removing either mint fails the label check at the Python site.

Every item holds as of this record, the last by demonstration.

---

## Amendments · `sec:scripts:amendments`

Every later change to this record's text is listed here as its own entry under its own label, as an adopting record lists the amendments it carries.

### Parameter citations · `rem:scripts:parameter-citations`

ADR-019's seven adoption parameters stood in one table when this record was written, and they now stand as seven entries under seven labels, the table's own label having retired with the split (`[ADR019-rem:labels:parameter-entries]`). The header of this record could therefore only cite the whole table and name the two rows it amends in prose beside the citation. It now cites the two entries themselves, the owner partition (`[ADR019-rule:labels:owner-partition]`) and the scanned-region recognition (`[ADR019-rule:labels:scanned-regions]`), which is the citation this record wanted and could not write; the wording that called them rows reads entries throughout, in the decision and in the adoption gate alike.

Nothing this record decides moves. The partition rule it adds, the language it adds to the scanned-region recognition, and the five parameters it leaves standing are exactly as adopted, and the two amended entries carry the fixings their rows carried, word for word.
