# The Attestation, Explained Like a Human

*A plain-language report on what this system is, what it does, how the smart contract works, and why it can be trusted.*

---

## Part One: The Problem This Thing Solves

### The oldest problem on the internet

Every online community eventually faces the same question: **how do you know someone is serious?**

Anyone can create a thousand fake accounts. Anyone can sign up, make promises, and vanish. This is called the *Sybil problem* — one person pretending to be many — and there are only a few known ways to fight it. You can check government IDs (which requires trusting an ID-checker and destroys privacy). You can make people solve puzzles (which just measures who owns computers). Or you can do what this system does:

**You can make seriousness *expensive*, and make the expense *provable*.**

The attestation contract is a machine for doing exactly that. It lets anyone, anywhere, publicly and permanently prove: *"I destroyed something of real value, and I did it in favor of this name."* Not spent — **destroyed**. Given up forever, in a way everyone can verify and nobody can fake, undo, or counterfeit.

That permanent, verifiable record of sacrifice is called an **attestation**. It is the only product this contract exists to manufacture. Everything else — the vault, the receipts, the cycles, the fees — is machinery to make that one record trustworthy.

A note on what you are reading: the contract described here is also the proving ground for the software that builds it. The repository's product is a compiler — a tool that turns a precise description of a contract like this one into the exact bytes a blockchain enforces — and this contract is the hard case the compiler must handle correctly before its guarantees mean anything. The contract needs nothing above it to be complete; the record it produces is finished the moment it is written. What others may build on that record — identity, membership, standing — is their act, conferring their meaning, and none of it is why the record exists.

### Why destruction, of all things?

Because destruction is the one act that can't be faked or quietly reversed. If I *pay* someone to vouch for me, the money moved from one pocket to another — maybe my own other pocket. If I *lock up* money, I get it back later. But if I verifiably *burn* money, the loss is real, permanent, and exactly measurable. A record built on burning is a record built on the one thing an adversary cannot counterfeit: genuine, irrecoverable cost.

The system is deliberately humble about what the record *means*. The attestation gives you nothing here — no payout, no interest, no claim. It's a fact, like a diploma on a wall: this address destroyed this much value on this date. Other systems that choose to read the record decide what that fact is *worth* — membership, standing, reputation, whatever they choose. This contract only guarantees the fact is *true*.

---

## Part Two: The Design — A Warehouse With a Strange Rule

### The warehouse

Imagine a warehouse that stores Bitcoin (technically L-BTC, Bitcoin on the Liquid network — think of it as Bitcoin in a particular vault system). The warehouse works like an old-fashioned grain depot:

1. **You deposit coins**, and the warehouse gives you a **receipt**.
2. **The receipt is a claim**: bring it back any time, and the warehouse pays out your share of everything it holds.
3. **The warehouse is full-reserve.** It never lends, never invests, never holds less than what backs every receipt. Every receipt is covered, always, by real coins sitting in the vault.

The payout rate is simple arithmetic: **coins in the vault ÷ receipts in circulation**. This ratio is called the **floor**, written φ (phi). If the vault holds 1,000 coins and 1,000 receipts exist, each receipt redeems for exactly 1 coin.

### The strange rule

Here's the twist that makes this a proof-of-seriousness machine rather than just a boring vault:

**Instead of redeeming your receipt, you may *burn* it.**

When you burn a receipt, you tear it up in public, forever. You get nothing back — but the system writes down, permanently: *"Address X burned receipts worth Y coins."* That's the attestation.

And now notice something beautiful: when you burn a receipt, the coins that backed it **stay in the vault**. The vault didn't shrink, but the number of receipts did. So the floor — coins ÷ receipts — *goes up* for everyone else. Every act of burning makes every remaining receipt slightly more valuable.

This single fact drives the whole economy:

- **The floor can never go down.** Deposits mint new receipts at the current rate (neutral). Redemptions remove coins and receipts in exact proportion (neutral). Burns remove receipts only (floor rises). There is no operation, anywhere in the system, that lowers it. It's a ratchet.
- **Burning is honestly costly.** The moment you burn, you destroyed something you could have redeemed for real coins. The immutable record says exactly how many receipt units you burned; their coin valuation uses the latest *officially swept* floor (see the ashtray, below). Because the official floor lags behind unswept burns and only ever understates, the recorded value never exceeds what your receipts could have redeemed for at that instant — it can only be conservatively lower. The record never flatters a burner. The record itself is permanent within the chain history you are reading; its *coin valuation* is always a reading taken against a particular history, never a number frozen into the record. If Bitcoin reorganizes and changes which official sweep comes before your burn, that same immutable record is simply re-valued under the new history — possibly up, possibly down. To treat a valuation as settled, wait until the sweep it depends on is buried deep enough that a reorganization is implausible.
- **Everyone else is your auditor.** Other receipt-holders profit from your burns, so they have every incentive to watch the vault, redeem at the higher rate, and generally keep the market honest.

### The two flavors of receipt

There's one complication, and it exists to solve the "day one" problem: at launch, before outsiders join, the founder holds all the receipts. If the founder burns their own receipts, the floor rises — but the extra value flows back to receipts *the founder also holds*. They'd be moving money from one of their pockets to another, "proving" sacrifice for free.

The fix: receipts come in two flavors.

- **Live receipts** — the normal kind. Tradeable, redeemable, burnable.
- **Time-locked receipts** — tradeable, but **structurally impossible to burn or redeem**. Not "forbidden by a rule that might have a bug" — the operations for burning and redeeming them *literally do not exist* in the contract. There's no door to lock because there's no door.

At launch, the founder's stake splits between the two (say, half live, half time-locked). Now there's a hard ceiling on how much can be burned before real outsiders join — the time-locked half sits in the denominator, capping how far the floor can climb and how much attestation the founder can self-manufacture. The math works out to a precise limit that can be printed on day one.

Later — after real depositors have arrived and market discipline has teeth — a one-time event called **maturity** relabels every time-locked receipt into a live one. The operator announces it once, publicly and irrevocably, with a lead time bounded on both sides (not too soon, not absurdly far away), and at the appointed cycle it happens automatically, in one atomic step. No value moves; the floor doesn't budge; the training wheels come off.

---

## Part Three: How the Contract Actually Works

### First, understand the stage

This runs on Liquid, a Bitcoin sidechain, which uses the **UTXO model** — the same accounting model as Bitcoin. Forget "accounts with balances." Instead, picture the blockchain as a table covered in **sealed boxes**. Every transaction opens some boxes, and creates new sealed boxes from the contents. A box, once created, never changes — it can only be consumed. Each box has conditions written on its seal saying who (or what) is allowed to open it.

A **covenant** is a box whose seal says not just *who* can open it, but *what the opener must do with the contents* — "you may open this box only if, in the same breath, you create a new box that looks exactly like this, plus these exact other boxes." The rules chain forward, box to box, forever. That's the "smart contract" here: not a program running on a server, but an unbroken chain of self-enforcing boxes.

### The cast of boxes

The whole system is about fourteen kinds of box. The important ones:

**The pool — two boxes welded together.**
- **STATE** is the system's brain: a box containing the official ledger numbers — how many coins in the vault (Ω), how many live and time-locked receipts exist (Y_L, Y_T), how much deposit money is queued (Q), which cycle we're on, and whether maturity has been announced. It's marked by a one-of-a-kind token (an asset that has existed exactly once since genesis), so it can't be impersonated — a forger can copy what the box *looks* like but can never hold the token that makes it *the* STATE.
- **RESV** is the actual vault: a box holding the real L-BTC. Operations that move backing — admission, cycle processing, and ordinary redemption — consume STATE and RESV together and recreate the required successors. Some operations change only authenticated state: clearing ash, relabeling matured receipts, and announcing maturity advance STATE without spending RESV. The final sealing redemption advances STATE while terminating RESV. In every case, the transition certificate and global invariant require the active vault's real value to equal the committed backing relation `Ω + Q`, so the books and the money cannot drift apart.

**The receipts.** Each receipt is a box holding units of a native asset called **U**, tagged with an owner and a class (live or time-locked). Because U is a *native* asset, the blockchain itself — not just the contract — enforces that units can't be conjured from nothing. And crucially, new U can only be minted by whoever presents a specific **authority token** (called PACE)… which lives inside the covenant and can only be spent by one specific operation. So counterfeiting is blocked by the blockchain, and over-printing is blocked by the covenant. Two independent locks.

**The clock.** That same PACE token doubles as the system's metronome. Its age — how many blocks since it was last consumed — *is* the time since the last cycle. Clever: the clock and the minting-key are the same object, so "minting happened" and "the clock reset" are physically the same event and can never disagree.

**Ash.** When receipts are burned, they don't vanish instantly — they turn into **ash**: an ownerless box of U that nobody can ever spend back into circulation. Think of it as a public ashtray. Why the intermediate step? Performance. The nasty details are in the next section, but the short version: burns happen constantly and shouldn't queue up behind the single STATE box, so burning is instant and share-nothing, while a housekeeping step later sweeps the ashtray into the official ledger.

### A day in the life

**Someone deposits.** Maria wants in with 1 L-BTC. She creates a **deposit request** — an open box with her coin and a note: "for the pool, receipts to Maria, refundable to Maria." The pool ignores it (anyone can create any box; junk boxes must be harmless, so unprocessed offers are just… inert). She can cancel any time and get every satoshi back. Eventually, anyone — a bot, a volunteer, the operator — sweeps pending requests into the pool: her coin joins the vault's escrow queue, and she receives an **entitlement**: a scarce, mint-controlled token that *is* her place in line. Not a note saying she's owed receipts — an actual asset only the covenant could have created, in exactly her amount. (Why so paranoid? Because a note can be forged; an asset cannot. An attacker who wants a fake entitlement would have to actually deposit. The forgery *is* the participation.)

**The cycle turns.** Periodically — the operator can trigger it after a minimum wait, and *anyone* can trigger it after a maximum wait, so the operator can pace the system but never hold it hostage — the **cycle** runs. In one atomic transaction: the escrow queue merges into the vault; new receipts are minted at *exactly* the current floor, so existing holders are neither diluted nor enriched; the operator takes a fee slice (a fixed formula — no discretion, no knobs); and the depositors' receipts go into a **distribution** — a counter box plus a money box, welded together — from which anyone can later pay Maria out. That last part matters: even if Maria loses interest, even if the operator disappears, any stranger can execute the payout, and the payout can only go to Maria at the formula amount. Permissionless *and* theft-proof, because the trigger and the destination are separated: anyone may pull the lever, but the lever only does one thing.

**Someone burns.** Bob holds 50 live receipts and wants to attest for his address. He signs one transaction: his receipts go in; one ash box comes out; and the transaction carries little data notes — "credit address `bob-address` with this burn." That's the whole hot path. No pool boxes touched, no queue, no permission — thousands of burns can happen in parallel.

**Someone sweeps the ashtray.** The ash sits there, publicly visible, and here's the honest subtlety: until it's swept, the official ledger still counts those burned receipts in the supply, so the floor is *understated*. Never overstated — understated. The error is always in the safe direction. Then anyone (typically someone who profits from the floor rising — a big holder about to redeem, or a burner who wants their next burn valued at the true rate) runs **clear**: ash boxes in, STATE updated, supply decremented by exactly the ash's real value, floor officially rises. The economy self-schedules its own bookkeeping: sweeping happens when someone finds it worth a transaction fee, and if nobody ever does, nothing breaks — the floor just stays conservatively low.

**The indexer reads it all.** The attestation map itself — "address X has attested Y" — is not stored on the blockchain at all. It's *recomputed* from the blockchain by anyone who cares, following one deterministic public rule. This sounds like a cop-out; it's actually the trust-minimal choice. An on-chain "total" would be a number you'd have to trust; the raw history is something you can *check*. The rule has two locks: a record counts **only** if it rode a genuine burn transaction (right shape: real live receipts in, one fresh ash out — so moving old ash around, or just writing "I burned!" in a transaction note, credits nothing), and the credited amounts **can't exceed** the ash actually created (over-claim and *all* your records are voided, though your burn still counts as a burn — you just donated). Two independent indexers following the rule must get byte-identical answers. The model ships differential harnesses that compare event recognition, query bytes, and receipt accounting — and its tests prove those harnesses catch a mismatching candidate; a deployment release additionally requires reports from a *separately implemented* indexer before the claim counts as independent evidence.

---

## Part Four: Why It's Safe — The Honest Version

Safety here isn't one big proof; it's a stack of small, boring, verifiable refusals. Let me walk the ones that carry the weight.

### 1. The system never trusts what an attacker can choose

This is the deepest principle in the design. On a public blockchain, anyone can create a box shaped like anything — a fake vault, a fake deposit request, a fake "state" with fantastic numbers. So the contract recognizes objects only by properties an attacker *cannot* choose: one-of-a-kind genesis tokens, native asset IDs the chain itself enforces, actual coin values read from consensus (never from a label someone wrote), and unbroken lineage back to genesis. Fake boxes aren't detected and rejected — they're simply *invisible*. They can't even cause a false alarm: junk is defined to be inert until something tries to spend it under a real rule, at which point the rule fails. (This closes a sneaky attack where an adversary plants garbage just to make honest auditors report errors forever.)

### 2. Value can't be misdirected — even by the people allowed to act

Ordinary contracts check "does the money balance?" This one checks something stronger: **every single input and output must be claimed by exactly one declared purpose**, and each purpose balances on its own. Why? Because totals can lie. "Alice: 5→4, Bob: 7→8" balances perfectly at 12 while robbing Alice. Here, that transaction is unrepresentable: Alice's coins belong to Alice's flow, which must balance to Alice's destinations. Fees can't nibble a payout; a helper's gas money can't mingle with the vault; a refund can't be shaved.

And on every path a *stranger* may trigger — sweeping deposits, paying distributions, clearing ash, the forced cycle — the recipients and amounts are fixed by formula before the stranger shows up. A random triggerer can make the system *go*; they can never make it go *to them*.

### 3. Every rounding error favors the vault

The contract works in whole units, and division rarely comes out even. Every single division in the system rounds *down*, in the direction that leaves the remainder in the pool. Your redemption rounds down a hair; the leftover fraction stays behind and lifts the floor for everyone else. And because on-chain scripts can't divide, the contract makes you *show your work*: you supply the answer, and the script checks it from both sides — `answer × divisor ≤ total < (answer+1) × divisor` — which pins exactly one correct value. Miss either check and a clever spender could pick a "close enough" answer and pocket the gap; with both, there is no gap.

### 4. Printing money requires a physical key, held in a locked room

The receipt asset is reissuable — new units *can* be minted (that's what deposits do). Consensus alone would let the key-holder mint any amount. So the key (PACE) lives inside the covenant, can only be spent by the cycle operation, and that operation pins the minted amount to the formula and enumerates every destination the fresh units may land in — the operator's fee slice and the depositors' distribution vault, nothing else, to the unit. Unauthorized minting isn't "checked for and refused"; it's impossible-by-absence — the key simply cannot appear in any other kind of transaction. The same pattern guards the entitlement mint and the distribution counters with their own separate keys, so no operation is ever one forgotten check away from being a mint.

### 5. The floor ratchet is arithmetic, not policy

"The floor never falls" isn't a rule someone enforces — it's an inequality checked on every state change: new-vault × old-supply ≥ old-vault × new-supply, in exact integer math, no rounding, no ratio ever computed. Each operation was designed so this holds by construction (deposits: equality; redemptions: equality-or-better thanks to rounding; burns and clears: strictly better), and then it's checked *anyway*. Belt, and suspenders.

### 6. The endgame can't trap anyone

Dying systems are where funds get stranded, so the terminal states are engineered: the last holder can redeem everything and seal the pool — but *only* if no depositor's money is still queued (so nobody's escrow gets locked behind a dead pool). The ash-sweeper is clamped so supply can never hit zero through housekeeping. Lost keys can't hold shared machinery hostage: payouts and relabels are permissionless precisely so that one vanished user can't freeze everyone's cleanup. And once sealed, the pool is a tombstone — provably un-resurrectable, so "the pool" can never be respawned as an imposter.

### 7. The whole thing is checked three different ways

- **A typed manifest** — a machine-readable declaration of every asset, box-kind, operation, and permission — is the single source of truth. The model *derives* its rules from it (no hand-copied tables to drift), a semantic hash pins its exact content, and mutation tests verify that corrupting any declared fact breaks the build.
- **An executable model** — a full Rust simulation of the contract — runs thousands of randomized adversarial scenarios: fake boxes injected, malformed deposits, wrong signers, corrupted histories, hostile orderings. Every accepted state must satisfy eleven global invariants (the books balance, the vault matches, distributions stay payable, history replays cleanly from genesis…); every rejected transaction must leave the world bit-for-bit untouched. A deliberately seeded catalogue of corruptions — hundreds of them — each must fail with the *specific named error* it should.
- **Independent recomputation** — the attestation ledger is checked by comparing two separately-built indexers, both on raw event lists *and* on final answers, because (subtle but real) two errors can cancel in a total while being flagrant in the event list.

### 8. And here is what you still have to trust — stated, not hidden

This is, honestly, the part that most convinced me the design is serious: it keeps a public list of its own residual assumptions rather than rounding them to zero.

- **The Liquid network itself.** The vault holds L-BTC, and L-BTC settlement is run by Liquid's federation of functionaries. If Liquid fails, the vault fails. This is named, not buried.
- **The gap between model and deployed script.** The Rust model is proven; the *actual on-chain bytecode* must be separately verified to match it. Every such claim in the spec is tagged `{verify}` — an IOU that a deployment must pay with test evidence before release, and the release process mechanically refuses to ship with unpaid IOUs.
- **Liveness is economic, not guaranteed.** Nobody is *forced* to sweep ash or trigger cycles; incentives make it likely, fees make it possible, and — critically — every lazy outcome is safe (the floor reads low, never high; deposits wait, never vanish).
- **Known sharp edges, catalogued.** A dust deposit too small to mint even one receipt gets absorbed (predictable in advance; wallets should refuse). Tiny remainders from rounding become permanently locked vault-value that nobody can claim — which, note, only ever *understates* the floor. The operator can drag their feet on maturity — but the spec shows why that strategy starves the operator's own fee income, and once announced, the schedule is bounded and irrevocable.

### The shape of the safety argument, in one paragraph

Nothing here asks you to trust intentions. The founder's self-dealing is capped by arithmetic printed at genesis. The operator's take is a fixed formula, their scheduling power is fenced by a deadline after which anyone can act, and their one true discretion (when to announce maturity) is disciplined by a market price that punishes stalling. Strangers can drive every essential process but can capture value from none of them. Every number is either enforced by Bitcoin-style consensus, pinned by the covenant, or recomputable by you from public data. And in every place where the system must choose between being generous and being conservative — rounding, lazy bookkeeping, valuation timing — it is conservative: it never overstates a payout or a recorded attestation. Rounding may leave value in the pool, and a lagged clearing floor may under-credit the recorded sacrifice relative to its burn-instant redeemable value.

---

## Part Five: Stepping Back

What has actually been built here is a **notary for sacrifice**. The vault, the receipts, the floor, the cycles — all of it exists so that one sentence can be written indelibly into public record and trusted by strangers: *this address gave up this much real value, forever, on purpose.*

It's a strange kind of financial machine — one whose product isn't profit but *proof*. The receipt is designed to be destroyed; the record of destruction is designed to outlive everything, including the pool itself. Other systems may read that record and build identity, membership, and standing on it, each conferring its own meaning on the sacrifice — but the record is complete without them. This contract refuses to promise what the sacrifice is worth. It only promises — with consensus rules, welded boxes, ratchet arithmetic, adversarial testing, and a candid list of what remains assumed — that the sacrifice was *real*.

That refusal to over-promise is, in the end, the most trustworthy thing about it.
