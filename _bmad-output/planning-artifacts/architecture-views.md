---
viewsOf: architecture.md
sourceSha256: 'e2ae176d3f49ae5a96a83d040dda4d6cfb1a69ba491a1edd326962b3e21ccd47'
sourceLines: 4887
generatedAt: '2026-07-17'
status: 'DERIVED — never edit by hand. Regenerate from the source.'
whatThisIs: 'CROSS-CUTTING VIEWS, not a summary. Each section gathers something the source scatters across 4886 lines and that NOBODY can reconstruct by scanning: every named renunciation, every measured number, every recorded dissent, every author amendment, every piece of named theatre, everything still open. It POINTS at the source; it does not restate it.'
whatThisIsNot: 'A short version of the architecture. It carries NO decision bodies. It cannot be applied from — every entry cites a line number, and the argument at that line is the part that makes the decision hold. For the state of any REQUIREMENT, read prd.md editHistory. Never here.'
warning: 'A view is a SNAPSHOT. This project has been burned three times reading a snapshot as a state (F56). If sourceSha256 no longer matches architecture.md, THIS FILE IS STALE — regenerate it. cargo xtask ci is specified to verify this hash (D65, same mechanism as D56 fixture MANIFEST). It already caught itself once: this file went stale 3 minutes after birth.'
history: 'Born 2026-07-17 as a 3004-line DISTILLATE (ratio 1.63:1 — a short copy, not a compression). Guy cut the ~2100 lines that paraphrased the decisions: they restated the source, they were the only real duplication produced by a pass whose purpose was removing duplication, and they went stale on every edit. What survives is the part that exists nowhere else.'
---

# Architecture Distillate — opencmdb (D1–D65, F1–F59)

## 0. What this file is, what it deliberately LOST, and why that was the right trade

**This is not a summary of the architecture. It carries no decision bodies.** Each section below gathers
one thing the source **scatters across 4 886 lines and that nobody can reconstruct by scanning**: every
named renunciation (§6), every measured number (§8), every recorded dissent with its author (§7), every
decision amended by its own author (§9), every piece of named theatre (§10), everything still open (§11).
**It POINTS. It does not restate.** That is the whole design: a file that restates cannot help diverging
from its source, and divergence is the failure this project has already paid for three times (F56, F59).

**Measured fact about the source: it has ZERO repeated sentences over 90 characters.** There was no fat to
remove. **So the honest conclusion of the 2026-07-17 cleanup pass was that the file it was asked to shrink
had nothing to shrink** — and the pass's real output was this file, an index (in the source), and the
removal of a trap.

> **🔴 What was cut from this file, on 2026-07-17, and it is the point.** This was born as a **3 004-line
> DISTILLATE — 61% of the source, a ratio of 1.63:1. That is a short copy, not a compression.** ~2 100 of
> those lines paraphrased the decisions one by one. **They restated the source, they were the only real
> duplication that the whole cleanup pass produced — created by the pass whose stated purpose was removing
> duplication — and they went stale on every edit of the source.** Guy cut them. **What survives is only
> what exists nowhere else.**
>
> **The mechanism proved itself before the ink dried: the hash detected this file as stale THREE MINUTES
> after it was written**, because two bugs were fixed in the source meanwhile. That is the mechanism
> working, and it is also the argument against ever letting this file restate anything.

**What is deliberately LOST, recorded so the loss is a decision and not an accident:**
- **The narrative of each debate** — outcome and deciding argument are kept; the texture that showed a
  conclusion was contested is gone. *Mitigated by §9: every author amendment is listed, and an amendment is
  the evidence that a decision was tested.*
- **Rhetorical force** — the load-bearing phrase is kept, the paragraph around it is not. **This was doing
  real work: rhetoric is what makes a rule survive a Tuesday evening at 23:00.**
- **The full argument of every rejected alternative** — kept: what was rejected and the one reason it lost.
  **Named risk: a rejected alternative reduced to one line gets re-proposed. That has happened three times
  here already (F56).** When re-proposing, open the source at the cited line.
- **Every decision body** — deliberately. **Not one entry here may be applied from its one-liner.**

## 1. What this document IS — and how to read it

- **opencmdb**: Rust single-binary network reconciliation engine. Observed (auto-discovered) vs
  declared (hand-entered). **The gap is the product.** Self-hosted, Synology/Docker priority.
- **The defining constraint, and it drives half the architecture: solo developer + AI assistant, NO
  SECOND HUMAN REVIEWER.** Every load-bearing invariant is therefore held by the compiler
  (`E0117`, `E0004`, `E0603`, borrowck, name resolution), never by discipline. *An ADR does not move a
  cost gradient. It adds guilt.* (l. 2468)
- **The value of this document is not its conclusions — it is that its conclusions were ATTACKED BY
  THEIR OWN AUTHORS AND SURVIVED.** Five proposed gates died to D45, four withdrawn by whoever proposed
  them; the superset theorem was attacked for an hour by its own opponent and held; at validation four
  authors amended their own graven decisions on arguments from people not in the room (l. 4042–4047).
- **Status: NOT READY. Confidence: HIGH.** Not in tension: *"Confidence is about what is decided;
  readiness is about what is left"* (l. 4042). Completeness checklist: **11/16** (l. 4029).
  *"A green here would be exactly what this document spends 3 400 lines dismantling: a gate that cannot
  fail. The status is the instrument working."* (l. 4039)
- **It is a GRAPH, not a list.** The source states its own most-cited nodes: **D1 (46×), D45 (34×),
  D21 (25×)** — *"if your question touches identity, gates or the schema, those three have almost
  certainly spoken already"* (l. 50). D57 was left on the critical path for a day while **D25 and D21
  already answered it**.
- **Read order that actually works:** §2 (the rules) → §6 (what we do NOT guarantee) → the decision you
  need, at its line. **No decision may be applied from its one-liner** — the source says this of its own
  index (l. 54), and it is true of this file with more force.

> **⚠️ FINDING, new here: the source's own Decision Index (l. 59–134) carries STALE line numbers —
> offset ~94 lines (it points D1 at 164; D1 is at 258; it points D45 at 2358; D45 is at 2452).** The map
> is wrong about its own terrain. **That is F56's exact error class, committed by the index whose job was
> to prevent it.** Every line number in THIS file was verified against the sha256'd source above. If you
> jump by the source's index, you will land ~94 lines early.

## 2. The governing rules — verbatim, and they are cited more than any decision

- **D45 (l. 2454) — the gate criterion, adopted project-wide:**
  > **"A gate is a gate when its red has exactly ONE repair, and it is the one we wanted."**

  The corollary is what does the work (l. 2464): *"a gate whose red is repairable from both sides is not
  a gate, it is a negotiation."* **And a negotiation with oneself, on a project with no second reviewer,
  has a known outcome.** Not a discipline problem — **a cost gradient**: at 10 a.m., rested, you also
  take the cheap repair, because it is usually the right one.
  Companion standing rule: **"a gate that has never caught anything is a tax on the gates that do"**
  (l. 847, 2470). And: **"A CI you would not trust to merge a patch bump blind is not a CI — it is a
  suggestion."** (l. 847)
  **D45 was applied against its own authors** (l. 2471): it killed the parity counter,
  `MemoryRepository`, the `trait-vocabulary` grep, **15 of the 50 traps**, and **a `kek_id` that nothing
  verified** — four of five withdrawn by whoever proposed them (l. 3555).

- **D10 (l. 503) — testing decisions that gate the architecture.** The load-bearing half:
  > **"Portability is obtained by refusing SQL, not by abstracting it"** (l. 554). All value comparison
  > and normalisation (`norm()`: trim, lowercase, MAC→canonical hex, IP→canonical form) happens **in
  > Rust**. **No comparison descends into the engine.** Same for `now()`: computed in Rust, bound as a
  > parameter, **never `NOW()`/`CURRENT_TIMESTAMP` in SQL** (this also makes accept deadlines
  > deterministic). Timestamps stored as **ISO-8601 UTC `TEXT`** so lexicographic order == chronological
  > order.
  > **D64 re-reads D10 and it is the whole reason D65 exists: D10 is a CORRECTNESS decision, not a
  > portability one** (l. 4510) — identity IS the product, and `norm()` is what makes hostname/MAC/IP
  > equality deterministic.
  Also D10: **NFR4 becomes a TRIPLET** — precision, recall, **and false-merge rate ≤ ~0.01** (later
  superseded by D18's binary gate). **False-merge is catastrophic and asymmetric** (two hosts fused →
  the operator loses trust and uninstalls); **false-split is benign**. *"They must not carry equal
  weight in the gate, or we ship an engine that fuses aggressively to inflate recall."*
  And the doubt NFR, valid in all three regimes: **`for all cycles c: D_new(c) <= Δ_observed(c)`** —
  *"the engine has no right to invent a doubt the outside world did not provoke"* (l. 536).
  **AC-7b/7c are an indivisible couple** (l. 543): silence (`D_new = 0` — **exactly zero, not "≤5%"; a
  non-zero threshold here is a licence to make noise**) AND wake (mutate one observed field → `D_new = 1`,
  exactly that field). *"Without 7c, 7b is satisfied by `return;` — a silence test without a wake test
  tests a corpse."*

- **D21 (l. 1395) — three flags resolved by the reframe.** Three rules, each cited downstream:
  1. **`revert` DOES NOT EXIST. BAN the word** — *"it carries a destructive-reversible intuition that is
     false here, and the word will eventually produce code that does a `DELETE`."* Three distinct
     annulments on three objects, **none is a revert: annulment is an addition.**
  2. **`source_state` granularity: per `(connector, subnet)`** — *per connector is too coarse (35 of 36
     subnets scanned and `OK` is a **lie** — and **a false "gone" costs as much as a false merge**);
     per host is too fine and **circular***. **Consequence: an absent observation in a `STALE` subnet
     produces NO gap.** *(Retracted by D34(3) to `(connector, SCOPE)` — see the source.)*
  3. **NO connector precedence — deliberately.** *"A precedence is a merge rule in disguise."* **What
     replaces it: disagreement is an `Opposes`.** A source that cannot know answers `Neutral` — **it
     does not LOSE an arbitration.** *"We do not guess. We expose. That is the product."*
  Plus D21's cascading implications (l. 1423), each load-bearing: identity first, gap second (a data
  dependency) · **read-your-own-writes ⇒ identity resolution runs INSIDE the writer actor; the read pool
  only serves the API** · transaction unit ~100 decisions or 1000 rows, **an identity decision is NEVER
  split across two transactions** · **`device` has NO business columns** (*"if anyone proposes adding
  `hostname` to it, they have just restored the OBSERVED/DECLARED merge we forbade"*).

- **D25 (l. 1535) — Caching: NONE. Explicitly.**
  > **"Recording 'no cache' as a DECISION, not an omission."** The session's governing allergy: **a cache
  > without an invalidation key is how the merge and the effective-value view come back.**
  > The ONLY permitted in-memory index is the writer actor's per-batch identity index — **and only
  > because its lifetime is strictly shorter than any possible writer, so there is no invalidation
  > window. If any future cache outlives its batch, it is an ordinary cache and the rule applies in
  > full. Red line.**
  **This rule closed D57 a day after D57 was written** (a closure table is a cache that outlives its
  batch) — and nobody had put them side by side.

- **D59 (l. 4175) — the reference scale is a PRODUCT TARGET, not the developer's install. Nobody had
  checked.** Every document said ~300 hosts / 36 subnets; the UniFi gateway says **a few hundred clients, single-VLAN /
  4 networks / 1 VLAN / 1 site**. It travelled through four documents unqualified **and it is the
  denominator of several arguments**.
  > **The sharpest consequence (l. 4193): "The dimension the product's target is DEFINED by (36 subnets)
  > is the one dimension the developer's own network CANNOT exercise (1 VLAN)." This is D46's `:memory:`
  > argument one storey up: there it was "you would be testing a database you do not ship." Here it is
  > "you would be testing a NETWORK YOU DO NOT TARGET."**
  **D59's lesson generalised by D64 (l. 4381): where a fact is not probe-able, D59 does not say "measure"
  — it says DO NOT FOUND THE DECISION ON IT.**
  **D59 is the error this file must not commit: a number that was measured must never be
  flattened into an adjective.** *"'Small' is an adjective dressed as a number."* (l. 4666)

- **F56 (l. 4771) — the process finding, and it is a property of the document, not an anecdote:**
  > **"FOURTH instance in 24 h of reasoning from the map instead of the terrain."** (D64's pre-brief ×2 ·
  > D57's brief mislabelling Journey 4/6 · the orchestrator's own editHistory line claiming keyboard
  > bindings "E/G/S/I" that **had never existed in the UX spec at all** — a standing question built for
  > two sessions on a line of its own making. **A journal entry is not a state either.**)
  > **"The register is now larger than its authors' working memory, and it discovers holes where its own
  > rules already stand."** Not an anecdote — **a property of the document at 4 700 lines.**
  *(The keyboard non-subject was withdrawn 2026-07-17 by Guy: the UX spec **assigns no keyboard letters
  BY DECISION** — the criterion is muscle memory of the Superhuman/Gmail corpus, never a mnemonic for our
  own vocabulary, which is why `E` works and `I` never should have existed. **Dividend: bindings are
  decoupled from our vocabulary — a rename can never orphan a key again.**)*

- **The two rules the source states about ITSELF** (frontmatter l. 26–31, and l. 1805/2122):
  > **"This document records what was DECIDED. `prd.md` records what the requirements ARE. For the state
  > of any requirement, read prd.md's editHistory frontmatter — never a table in here."**
  > **"A feedback table with no applied-date is a dead document that looks alive."** (l. 3617)
  **All F-items F1–F59 are APPLIED. Nothing in the source is outstanding feedback.** The 28 F-rows no
  decision referenced were REMOVED on 2026-07-17 — *"a snapshot of finished work sitting inside a
  decision register, read as open work three times in 24 h."* **31 F-rows physically remain**
  (F1, F2, F11–F14, F16, F17, F35, F37, F38, F40–F58, F59); *the frontmatter says "the 13 that survive
  are the ones other decisions CITE" — the two counts do not reconcile in the source, and I have not
  resolved it.*

## 3. The stack, the versions, and the BANS

**Verified 2026-07-16 — re-verify at implementation time** (l. 679–688):

| Dependency | Version | Note |
|---|---|---|
| `axum` | **0.8.9** (2026-04-14) | latest STABLE; `0.9` is unreleased breaking work on `main` |
| `askama` | **0.16.0** (2026-04-29) | `askama_axum` **removed since 0.13** |
| `sqlx` | **=0.9.0** (2026-05-21) | decided; repo moved to `transact-rs` (May 2026) |
| `tailwindcss` standalone CLI | **v4** | JS-config `safelist`/`content` **gone** → `@source` / `@source inline()` (v4.1+) |
| `tokio` | `^1` | the most boring of the set |
| MariaDB | **10.11.11** exact pin in CI = DSM 7 package **10.11.11-1551** (verified 2026-07-16), LTS to **Feb 2028** | *10.6 reached EOL 2026-07-06 — had the package been a 10.6, this would be a different conversation* |
| crypto (D31) | `chacha20poly1305 = "0.11"` · `argon2 = "0.6"` · `age = "=0.11.3"` [BETA] · `zeroize = "1"` · `rand_core = "0.9"` | confirm at `cargo add` — **a compilation check, not a maintainer audit** |
| `rust-embed`, `uuid`(v7), `serde`, `chrono`, `tracing`, `thiserror` | verify at `cargo add` | **pin from the real lock; do not invent numbers** |

**`sqlx = "=0.9.0"` (l. 690) — why, on verified facts:** 0.9.0 is the head of the line (no 0.9.1 after
~8 weeks, **read as stability, not immaturity**); the crate is not renamed/yanked → the blocking check
**passes**; and the clincher: **`0.8.6` is 14 months old (2025-05-19) — not "the mature version", a dead
line. The "prudent" option was in fact the more exotic of the two.** Risk near-nil for US specifically:
the skeleton touches only `query()`, `.bind()`, `fetch_*`, `Row::try_get()`, concrete pools, `migrate!`
— **sqlx's major-release breaking changes historically concentrate exactly where we do not go.**
*"Pinning 0.8.6 is insuring against a fire in a room we bricked up."*
**The residual risk, named honestly:** *"the risk of 0.9.0 is not 'sqlx is broken', it is 'your copilot
thinks we are on 0.8'. That risk does not decrease with sqlx's quality — it decreases with CALENDAR
TIME."* Mitigated by the compiler (an API break is an `E0599`, not a heisenbug) — **and, at the time,
by the dual-backend CI. D64 removed that half. See §6.1.**
**The sqlx governance move is NOISE — read correctly, a de-risking** (l. 751): the change happened years
ago; the repo name caught up. **The risk was the situation BEFORE** — an inactive corporate entity
holding the namespace of a foundational dependency. Contrast `event-stream` (npm, 2018) where the signal
was **a new identity obtaining publish rights**: here that signal is absent. **An inverted event-stream.
Delta ≈ 0. Mitigating a null delta is theatre.**

**NO starter template — `cargo new` + a curated dependency set** (l. 624). Four candidates web-verified
July 2026; **all four are demonstration TODO apps, all four predate askama 0.13's removal of
`askama_axum` — stale at the root.** The decisive argument is **not** "our decisions are already made":
> **"The available starters carry decisions ACTIVELY CONTRARY to ours."** Scored against our locked
> decisions: **~1.5 of 8**. All use `sqlx::query!` (banned — it is sqlx's default path), none has the
> `Repository` trait, none has the writer actor, their Tailwind is a v3 whose config no longer exists.
> **"We would start BELOW zero: dismantling before building."**
**What a starter really provided was a PROOF OF INTEGRATION — and that, only, must be bought back, via
the walking skeleton.** Corollary, standing rule: **do not test that axum routes. Do not test that askama
renders. If a test would fail only because of an upstream bug, it is not our test.** We READ the four
repos for `rust-embed` + Tailwind-standalone plumbing. **We fork nothing. Cost, stated honestly: ~1 day
of boilerplate we write instead of inherit** — not a zero price.

**Dependency version policy — Guy's principle, adopted** (l. 655): *"Je préfère l'utilisation de
librairies récentes car de toute manière, il faudra migrer une fois."* **D1's own logic applied to
dependencies: the migration cost is invariant — pay it while the codebase is empty.**
> **"Boring technology and 'prefer recent' do not conflict — they measure different quantities."**
> Boring is a **budget of unknowns** (the whole budget goes into the domain — *that* is where we want to
> be surprised). "Recent" is **migration debt**.
> **"A two-year-old version is not boring — it is SLOW EXOTIC. Old and brand-new are both exotic, at the
> two ends of the same axis. What makes a version boring is not its age — it is that other people have
> already hit its bugs. 'Boring' is not a property of the library; it is a property of what the world
> knows about it."**
> **"'Prefer recent' is a COST policy, not a security one — it is only affordable if the marginal cost of
> a bump tends to zero. The whole policy reduces to: make the bump free, or do not prefer recent."**
**⇒ grouped Renovate + auto-merge on patch/minor when green, in the SAME story as the CI** — *not later,
or you will never have the policy, only the starting point.* Breaking changes: **dedicated PR, never
grouped, never auto-merged. Veto: never two breaking changes in one commit — bisection must stay trivial,
because there is nobody else to bisect.** Plus **pinned MSRV + `rust-toolchain.toml`** and **`cargo-deny`
limited to `advisories` + `licenses`** — *licences are not cosmetic: **a binary links statically, and a
GPL/AGPL entering transitively into a distributed binary is a licence problem for our users**; and **a
library can change licence on a minor.***
**Pin semantics: a RATCHET, not a wall** (l. 810). *"A defensive pin has no exit condition; that is how
you wake up on 0.9.0 in 2028."* The exact pin means **"do not advance on your own, without my seeing
it"** — Renovate opens the PR, CI arbitrates, the pin rises.
**`Cargo.lock` COMMITTED + Docker build `--locked`. Non-negotiable** (l. 804) — *"nothing resolves on the
fly at build time"*; neutralises ~90% of the supply-chain scenario at ~zero cost. **And: pinning in
`Cargo.toml` what the lockfile already guarantees is superstition.**

**REFUSED / BANNED outright** (l. 737, 778, 849):
- **`askama_web` and `askama_axum`.** We write our own Askama→Axum `IntoResponse` (~15 lines).
  *"`askama_axum` was removed in 0.13 precisely because bridge crates are a maintenance burden.
  `askama_web` is the same bridge, repainted, with one fewer maintainer."* It would be a **version lock
  between our two most likely upgrades.** > **"A dependency whose entirety we can rewrite in 12 readable
  lines is not a dependency — it is a coordination debt."** Bonus: we own the error path — *on a
  reconciliation engine, a silently failed template render is worse than a clean 500.*
- **`sqlx::Any` / `AnyPool` — FORBIDDEN.** The natural temptation of a dual backend: **it levels down,
  masks type divergences, and would blind us to exactly the dialect bugs the trait must isolate.**
- **`sqlx::query!` / `SQLX_OFFLINE`** — banned (it is sqlx's default path).
- **Any crate re-abstracting sqlx** (SeaORM, Diesel, a "multi-dialect" query builder) — *two abstractions
  stacked on the same problem is where we would lose the collation bugs.*
- **`native-tls`** — a system OpenSSL link is the death of the static single binary on Synology. Use
  `tls-rustls-ring`.
- **Any crate that brings Node back into the build path. Red line.**
- **A CDN for any JS/CSS asset** (D37).

**The Tailwind v4 trap, specific to us** (l. 796): any class **built in Rust** (a `match` on a state enum
returning `"bg-pending"`) is **invisible to the static scanner. Miss one and there is no build error, no
red test — just a status pill with no colour in production.** On a product whose core is the visual
distinction of observed vs declared, **that is a silent product bug.** The CI drift-check catches only
ONE of the two drifts (it sees "class added to a template, CSS not regenerated"; it does **not** see
"class built in Rust, `@source inline()` forgotten"). **Hence AC-1.12.**

**What actually mitigates sqlx is D1, not the `Repository` trait — do not confuse them** (l. 762): the
trait covers **API coupling** (two-file blast radius) and cold replacement. It does **NOT** cover a
pooling bug under WAL, a driver regression, an unpatched CVE, or a behavioural divergence between
drivers — **those cross the trait without seeing it. The trait is an abstraction of API, not of
behaviour: it protects us against sqlx refactoring, not against sqlx being wrong.**
**D1 was the behavioural mitigation** — a permanent differential test across two independent drivers.
*"We have built, without naming it, a sqlx bug detector that runs on every commit."*
**⚠️ D64 deleted that detector. See §6.1 — it is renunciation #1.**

## 6. 🔴 NAMED RENUNCIATIONS AND NON-GUARANTEES — the document's most valuable content

> *"Non-guarantees are written as non-guarantees" is listed as a KEY STRENGTH (l. 4055). "A threat model we
> cannot enforce is a delayed lie." "A gate that claims to cover what it does not cover is worse than no
> gate: it EXTINGUISHES VIGILANCE."*
> **These read like weaknesses. They are the opposite: each one is a place where the authors refused to
> claim a coverage they did not have. Deleting one does not remove the hole — it removes the knowledge of
> the hole.**

### 6.1 D64's renunciation (l. 4488) — "and Guy just made it more expensive"

1. **🔴 THE DRIVER-LIE DETECTOR IS GONE.** D46b was **the only instrument that could say *sqlx is wrong on
   one of them*. We are removing it in the same month we adopt `sqlx =0.9.0`** — eight weeks old, no 0.9.1,
   repo moved to a new org in May 2026 — **under the "prefer recent" rule, which was itself carried by "if
   your CI is good, age is a superstition". HALF OF WHAT MADE THE CI GOOD WAS TWO DRIVERS. These two
   decisions were taken separately and THEY COMPOSE BADLY. We know. We accept it. We do not pretend to
   cover it.**
2. **The "the trait absorbed a SQL assumption" class returns to FULL SIZE, with no gate** — D51's named
   renunciation, **now with Winston's corrected price tag** (§9).
3. **🔴 Guy re-opened PostgreSQL as a possible future addition ("on verra"), which CASHES ITEM 2 IN.**
   Winston had defused his own amendment by noting *"NFR21 locks PostgreSQL out, so the realistic port is
   intra-engine and the discounted cost is still ~0"*. **That lock is now SOFT. The amendment becomes
   decisive: a PostgreSQL port is exactly the scenario where an unguarded, MariaDB-shaped trait is paid for
   in full, on a schema that has grown for months.** ⇒ **If PostgreSQL ever enters the roadmap, THIS TRAIT
   IS AUDITED BEFORE, NOT DURING** — and the audit is scoped by D51's four leaks, **at full size**.
4. **The evaluator loses five minutes.** Not a user; an evaluator. *Five minutes do not buy a two-engine
   test matrix.* **Recorded as a real, small, accepted cost — not argued away.**

### 6.2 D51's renunciation (l. 2865) — one bug class, assumed, with no gate

> **"This class has no gate. The cost is paid on the day of the port. We know it, we assume it, and we do
> not pretend to cover it."** If a non-relational backend enters the roadmap, **this trait is to be
> re-audited BEFORE, not during.**
> *A gate that claims to cover what it does not cover is worse than no gate: **it extinguishes
> vigilance.***
**The class, named exactly: *"the trait absorbed a SQL assumption"*** — both backends stayed green **because
both ARE SQL. The trait covers API coupling; D1 covered sqlx being wrong; NOBODY COVERS THE TRAIT CEASING TO
BE AN ABSTRACTION.**
**⚠️ D51 point 4 as written — *"the discounted cost of the class is zero; PostgreSQL is SQL; both real
porting scenarios are intra-SQL"* — was AMENDED BY WINSTON ON 2026-07-17 AND IS FALSE. See §9.**

### 6.3 NFR9c (l. 1574) — the security non-guarantee, in the quickstart, not in a FAQ

> **NFR9c — NOT GUARANTEED, documented as a non-guarantee:** *"If your backup tool copies both the key and
> the DB, encryption at rest no longer protects you. **opencmdb cannot and will not prevent this.**"*
> **In the quickstart, BEFORE the install — not in a FAQ.**
Companion non-guarantees from the same table: **backup stolen with secrets included → NOT protected** ·
**local root on the NAS → NOT protected** (NFR9 already says so). *"We will not pretend to defend the
local-root case. **Theatre is worse than a narrow frontier we actually hold.**"*
**And the meta-rule it produced, reused four times since:** *"we cannot measure the world; we can measure
our reaction to it."*

### 6.4 D65 / F57 (l. 4803) — the DDL grep is BLIND to derived expressions

> **NON-GUARANTEE, named: the grep reads the DDL, so it is structurally blind to DERIVED expressions.**
> `INSTR(CONCAT(path, ',', dst), …)` compares an expression **whose collation is declared in no DDL. True
> for columns. False for expressions.** Moot today because D57 refused the recursive CTE — **real the day
> any epic compares an expression rather than a column. We do not claim to cover it.**
**And D65 volet B's own two non-guarantees:** **it cannot see a rename in a document that never discussed
the subject at all** (zero old, zero new = green, correctly) · **it does not police the frontmatter, which
is a journal by construction and must be free to record the old names.** *"We are not claiming a proof."*
**Plus the weakness that RUNNING it found and reasoning about it did not: `ignore` is the only retired term
that is also an ordinary English word.** *"`CHECK` is parsed and IGNORED below MariaDB 10.2.1"* is not the
UI gesture. **A document that used `ignore` only in that sense and never discussed exclusion would go red,
and its "repair" — mention `exclude` — would be NONSENSE: the cheap repair and the correct repair would come
apart, which is D45's own definition of a bad gate.** *Measured, not assumed: it does not happen in this
corpus — all six documents describe the triage inbox, so all six contain `exclude`, and all six are green
with margin.* **Recorded so that the day a seventh document joins the set and reds on an innocent
"ignored", the gate is UNDERSTOOD rather than DISABLED.** *(The clean answer if it ever fires: `ignore`
moves to volet A only.)*

### 6.5 D57 (l. 4637) — `MAX_DEPTH` is NOT structurally enforced

> **The mechanism is an ABSENCE** (same gesture as D49's missing `commit()`, D50's missing `new()`).
> **NON-GUARANTEE, stated: nothing STRUCTURAL enforces this absence — a name-grep is theatre (D45 killed
> `trait-vocabulary` because "the spelling of a leak is free"). We do not claim a mechanism we do not
> have.**
**Why it matters:** if the visited-set is broken, **`MAX_DEPTH` converts a loud hang into a SILENTLY
TRUNCATED impact list. `MAX_DEPTH` makes our failure silent.**

### 6.6 The renunciations recorded at Step 6 (l. 3486), "in D51's spirit — recorded, not hidden"

- **Nothing stops `bin` bypassing `core` and putting business logic in a handler.** No gate; one there
  would be repairable from both sides (D45). **Assumed — the symptom is visible: it means writing SQL in
  `http/`, and that you will see.**
- **AC-8b DIES if someone writes `pub(crate)` on the `FieldDecision` store.** The grep helps; **it is a
  grep. AC-8a (the purge test) is the real gate.**
- **`Web`/`Api` can still drift into an enum. Rust cannot forbid it.** Three stacked mechanisms make the
  path long and smelly (D53); **NONE CLOSES IT.**
- **The subdomain organisation has no compiler-enforced falsification.** Its condition is recorded instead
  (D54).
- **🔴 THE BUILD ORDER OPTIMISES FOR TRUTH, NOT FOR EARLINESS — and no decision in this document balances
  that.** *"Recorded as a renunciation because it is the largest one and it is not technical: **every
  decision here makes the product more TRUE. None makes it arrive EARLIER.** On a solo project with no
  reviewer, **the most likely death is not a fragile `identity_link` — it is ABANDONMENT.** A build order
  whose first contact with the real network comes last **defers every bad surprise to the moment it costs
  most.**"* The identity→gap dependency is causality and it stands; **"`identity_link` must be STABLE before
  the gap has meaning" is NOT — "stable" is a quality threshold, therefore a product arbitration that
  presented itself as a law of physics.** *Product decisions do not disappear when no product voice is in
  the room; **they disguise themselves as technical constraints.*** (= F41) **Now PARTIALLY PAID by D1-rev
  and D19-rev** — *the only amendments that pay the renunciation in acts rather than in confession.*

### 6.7 Other non-guarantees, scattered but load-bearing

- **D18 (l. 1257): no `CHECK` detects a false merge. The schema makes a false merge revisable and
  traceable; IT DOES NOT MAKE IT IMPOSSIBLE. A 300-host corpus is small. That is a RELEASE risk, not a test
  detail.** And: **"a trap suite proves nothing about what I failed to imagine. At v0.1 the gate is weak
  and honest rather than strong and false."**
- **D47 (l. 4643): the dependency graph does NOT prevent `ConnectorError::Other(String)`, which is `anyhow`
  in disguise. The closed taxonomy is not guarded by the compiler** — it is guarded by D18 Tier 1 requiring
  a decision on every variant.
- **D53 (l. 3071): the `StatusCode::` gate FAILS D45 in the strict sense — six exits, two watched.**
  Reclassified as **a REFLEX gate, not a proof gate. "That is less than we wanted, and it is real."**
- **D61 (l. 4266): `network_id ≡ l2_domain` is UNVERIFIED and CANNOT be verified on the developer's
  network.** *(See §8.)*
- **D31 (l. 3742): four lines no test can check** (`OsRng` · **the AAD on both sides — the bug is symmetric,
  therefore invisible** · `decrypt()`'s un-zeroized intermediate `Vec`).
- **D17 (l. 1196): OPEN — do the ingestion probes capture probe requests? If yes the order of magnitude is
  off by ~30×. The policy holds either way; the default does not.**

## 7. 🔴 RECORDED DISSENT — with its author

- **🧪 Murat, on D64 (l. 4538) — dissented on the TIMING, not on the decision: *"not no, not yet."***
  Story 1 is **already funded as dual by D1**, and it returns **four numbers this room is currently
  guessing**: (1) trap-suite time on each engine · (2) **whether the dual constraint catches anything in
  the first DDL — D51 predicted "2-3 reds, all in story 1"** · (3) whether the HRTB goes green in a day ·
  (4) whether `docker run` is still one line after D27. *"Three hours now or six weeks later — here: one
  story already in the plan, or **half a day of documentary surgery on a prediction**."*
  **He also named the cost nobody else did: D50 makes `IdentityIndex::for_unit` the only constructor, so
  `tests/traps/` — the ~50-trap suite, THE PRODUCT'S ONLY GATE (D18/NFR4) — becomes DOCKER-DEPENDENT, and
  D19-rev puts weeks of calendar inside that loop.** *(He named and refused the cheap repair: **an in-memory
  `WriteUnit` "just for the traps" is `MemoryRepository` in a smaller hat, and D51's four legs apply word
  for word.**)*
  **His abandonment calculus does NOT move: |Δ| < 5 points**, low confidence in the number, **high in the
  decomposition** — *"P(abandon) measures whether GUY stops. F35 measures whether STRANGERS arrive. **They
  are not the same variable, and my instrument only ever measured the first.**"*
  **⇒ Guy OVERRODE the timing and closed the debate.** *"The four numbers will be observed in story 1
  anyway; what they can no longer do is RE-OPEN (B)."*
- **🔴 The three technical voices vs John, on D57-scope (l. 4760).** **All three judged the graph cheap
  enough to keep whole** (Amelia: **p95 ≈ 24 ms, zero new architecture**). **John conceded the cost argument
  before it was made:** *"if the edge table costs half a day, I have no cost argument. **I never had one.**
  My argument is that the product may not assert what nobody re-reads. **That one does not depend on the
  price.**"* **⇒ Guy decided on the PRINCIPLE, not the price.**
- **D11 (l. 571) — three agents argued against « Merger »/"Merge"**, Guy chose the word anyway: *"Vocabulary
  is architecture — if the action is called merge in the UI, someone will end up implementing it as a
  merge"* (**Winston**) · the word names **the forbidden operation** · **a "Merge" button in a product whose
  pillar is "linked, never merged".** **Mitigation: the tests, not the label. Verdict: a PEDAGOGY risk, not
  an architecture risk.** *(Closed 2026-07-16: EN says `document`; « Merger » is the FR UI label only.)*
- **D31 (l. 1733) — "a genuine disagreement, recorded rather than papered over"** (`age` vs pure
  RustCrypto). **Resolved at validation, and the resolution names the disagreement as false:** *"Two correct
  answers to two different questions, stacked… **each camp was answering a different half.**" The note itself
  recorded: "they partly talk past each other."*
- **D20 (l. 1368) — the strength/no-strength disagreement, resolved by a contract, not a winner:** *"You get
  your strength, I keep my table."* **If strength returns it returns as an ORDINAL under a four-condition
  ADR — a weight producing an `f64` out of the cascade is an automatic reject.**
- **D10's `<5%` doubt budget (l. 542) — Sally's number dropped one rung, not out:** *"Sally's <5% drops one
  rung: from CI gate to **R2 telemetry canary**, never blocking."*
- **D33 (l. 1905) — the variant count is OPEN and the disagreement is recorded** (5 / 7 /
  6+`ImplausibleResponse`). **"They converge on what matters: the default is safe, `Cancelled` is the
  exception, and the split serves the operator, not the engine."**

## 8. 🔴 MEASUREMENTS — every number, with its source. Never flatten one into an adjective.

> **D59 is what happens when you do: a number nobody had checked became "the reference scale" across four
> documents. "'Small' is an adjective dressed as a number." "Refuse to invent a threshold."**

### 8.1 The cardinality probe (l. 4135–4212) — D19-rev step 0, executed 2026-07-16

*A throwaway read-only reader, run once, locally, against the developer's a UniFi gateway. **Nothing entered the
repo** — counts and shapes, never values; correlation on a per-run salted hash, discarded at exit. The
connector is deleted. **It never crossed the trait.*** **It answered its question — favourably — and brought
back four findings nobody was looking for.**
Measured over the **few hundred known clients** (**the real denominator; only some were active at probe time**),
using **only structural facts** — *a reading, not an inference.* **The number itself is the output of an
engine that does not exist; the probe measured BOUNDS.**
```
locally administered (U/L bit) :  a large share     <-- D17 dormant territory
universal (stable) MACs        :  most                <-- the denominator that matters
LOWER bound, multi-interface   :  a modest share      (demonstrably grouped by a structural fact)
UPPER bound, cardinality 1     :  a large majority      (not demonstrably grouped)
```
**The threshold was: *"80% → abstention is an edge detail, the specified screen is right."* The upper bound
is a large majority.** ⇒ **The gap is the content; abstention is the exception. The UX hierarchy holds. F44's grouped
view is a bootstrap MODE, not the product** — **and F44 and F12 do NOT collapse into the same feature.**
**Three reservations, stated BECAUSE the answer is favourable and that is when one stops looking:**
1. **`hostname` is unusable on nearly half** (**MISSING or empty — but null never**). Hostname is **one of L2's
   three named signals.** ⇒ **the multi-interface lower bound is an UNDER-estimate: you cannot group by a hostname you
   do not have. The true multi-interface fraction is above a modest share by an unknown margin, and the margin is
   exactly the population we are blind to.** (= F51)
2. **a large share of the inventory is locally administered. D17/FR38b is not an edge case — it carries nearly half
   rows.** *The feature that keeps the central indicator from drifting upward forever is half the data.*
   (= F50)
3. **the UniFi infrastructure devices carry many radio/vap entries — several MACs per box. D12 is validated in the wild.**
   **Infra is the heavily multi-interface population, and it is counted separately from the general population.**

### 8.2 D59 (l. 4175) — the scale nobody had checked

| Every document says | The a UniFi gateway says |
|---|---|
| **~300 hosts** | **a few hundred clients, single-VLAN** |
| **36 subnets** | **4 networks** (2 wan + 2 corporate), **1 with a VLAN id** |
| — | **1 site** · **10 infra devices** |
**Confirmed with Guy: 300/36 is the product's TARGET AUDIENCE, not his install.** It must be **written as
such** (= F48).
**What survives: D18 is untouched.** Its `n=300` is the size of the **seeded bulk fixture**, which the
generator produces at the target scale **by construction**. *The generator MAKES the reference scale; it
does not MEASURE it.* **The binomial argument stands.**
**And the consequence for the seeded generator, which is NEW WORK:** the generator must produce a
cardinality distribution at 300/36 — **and that distribution is now a PARAMETER someone must choose, with
exactly ONE empirical anchor: a single-VLAN home lab.** **The direction of the bias is knowable even
if its size is not: infra grows with segmentation, and infra is the multi-interface population.** ⇒ **a large majority is
very likely an OVER-estimate of cardinality-1 at the target scale. Not fatal to F42 — a large majority has room — but
the generator must not encode a large majority as if it were measured at 300/36. It was measured on a single-VLAN home lab.**
**And a correction to the build-order round:** *"the real mitigating factor: Guy is the user — 300 hosts,
his NAS, his network, his pain"* — **Guy IS a user. He is not the REFERENCE user.** He has a single-VLAN home lab8 on 4
networks. **The estimate's only positive modifier rested on a number nobody had measured. The direction of
the correction is not knowable without re-running it; the fact that it rested on an unmeasured number is.**

### 8.3 D60 (l. 4214) — the UniFi capability descriptor, MEASURED

**🔴 The biggest single risk is CLOSED: `Fact::Uplink` EXISTS at zero privilege.**
```
Fact::Mac              mac          present on all
Fact::IpV4             ip           present on nearly all
Fact::Hostname         hostname     present on most active; usable on about half
Fact::OuiVendor        oui          often empty
Fact::DhcpLease        dhcpend_time present        network_id present on all
Fact::Uplink (wired)   sw_mac / sw_port present -> the large majority OF WIRED CLIENTS
Fact::Uplink (wifi)    ap_mac       present on most
Fact::Rtt              rtt absent · latency absent   <-- ABSENT (a scanner fact, not UniFi)
```
**L2 keeps its topology signal.** *The percentages on `sw_mac`/`sw_port` look low only because they are
computed over all clients; over WIRED clients, where the fact is meaningful, it is the large majority.*
**`Fact::Rtt` is absent from UniFi — and that is NOT a reason to remove it from the enum. It is the first
MEASURED capability descriptor, and it is exactly what D34 exists for.** The generic ARP/ping scanner
**does** produce an RTT; UniFi does not. *D19 named a Fact no single connector emits; **D34 made that legal
before it was known to be necessary.***
> **🔴 The trap inside the good news: `satisfaction` is present on nearly all of clients. IT IS NOT A LATENCY.** An
> implementer looking for `Fact::Rtt` will find **a plausible-looking number at nearly all coverage and map it.
> `satisfaction` is a UniFi score, not a measurement of ours** — *the exact shape of a silent wrong: **it
> compiles, it is populated, it is meaningless.***

### 8.4 D61 (l. 4244) — `l2_domain` = `network_id`. The question stays OPEN, and it is now NAMED.

```
client.vlan       : MISSING on 48/48
client.network_id : present 100%   —   DISTINCT VALUES: 1
rest/networkconf  : 4 networks (2 wan, 2 corporate), 1 carrying an explicit vlan id
```
> **On the developer's network, `(l2_domain, mac)` DEGENERATES to `(constant, mac)` = `mac`. The L1 key has
> no scope dimension to exercise.**
**The model is not wrong** — a segmented network has VLANs, and the target is 36 subnets. **But:**
**`network_id ≡ l2_domain` is UNVERIFIED and cannot be verified here** (*with one distinct value there is no
way to distinguish a genuine L2 domain from a UniFi config object that merely correlates with one*) · **the
"same MAC on two subnets" trap is not reproducible on the developer's network** — it stays a synthetic trap,
**which is legitimate (its oracle is the spec, not the terrain) — but it means the L1 key's scope dimension
ships with ZERO CONTACT WITH REALITY until a segmented user appears** · ⇒ **named as an OPEN RISK, not
closed by silence. The first user with real VLANs is the first verification of the foundation.
`network_id` is adopted as `l2_domain` ON EVIDENCE OF SHAPE, NOT OF BEHAVIOUR.**

### 8.5 D62 (l. 4270) — the UniFi version matrix: the real axis

```
UniFi OS console    : a 5.x build          (<gateway-model>.<build>)
ucore               : a 5.x build
Network application : <10.x build>      (previous: <prior 10.x build>)   <-- THIS is what the connector talks to
```
**D35 bounds the fixture matrix as `unifi-3.x/` and `unifi-4.x/` and calls it a product decision. Those
numbers correspond to NOTHING on the target device. The matrix was bounded without anyone looking at a real
controller.**
- **The axis that matters is the Network application version (10.x), not the UniFi OS console version.**
  They are different products on different release trains, **and the connector speaks to the former.**
- **The version is also the fixture's provenance tag. Tagging a capture `unifi-4.x` when the payload came
  from Network a 10.x build is a fixture that LIES ABOUT ITS OWN PROVENANCE — and the re-capture job diffs
  against that tag.**
- ⇒ **`fixtures/capture/network-10.x/`, not `unifi-4.x/`.** **The supported matrix is a product decision
  that is now RE-OPENED, on the correct axis, with a real number to anchor it.** (= F49)

### 8.6 Wire format (l. 4291) — the ~15 traps, now written from MEASUREMENT instead of belief

| Field | Measured | Resolves |
|---|---|---|
| `mac` | **lowercase, `:`-separated, 100%** — no `AABBCC`, no `aa-bb-cc` | one trap, closed |
| `hostname` | **MISSING and empty both occur · null NEVER** | **TWO of the three behaviours occur. `null` does NOT.** *The trap set must encode absent and empty — and must NOT encode null, or it tests a case the source cannot produce* |
| `last_seen` | **10 digits = SECONDS epoch** (not ms) | one trap, closed |
| `oui` | **`""` on a large share** | `Fact::OuiVendor` is often empty — **a named Fact that is usually absent** |
| `vlan` | **MISSING 100%** | see D61 |
| `sw_port` | integer, 1–2 digits | |
| `is_wired` | bool, 100% | the wired/wifi split is total |
| `network_id` | `str(len=24)`, 100%, **1 distinct** | see D61 |
| **distinct keys** | **127** | ***D19's `Fact` enum names 7.*** **The payload is many-fold the model's surface — that ratio is the drift surface D35's mutation fixtures exist to cover** |

### 8.7 Amelia's CTE benchmarks (l. 4598–4620) — measured on `mariadb:10.11.11`, container destroyed after

| Form | Result (MEASURED) |
|---|---|
| `UNION ALL`, no guard, 2-node cycle A↔B | **1001 rows, exit 0, 99 ms, `Warning 1931` — and `sqlx` does not surface warnings.** *The product does not crash. **It lies.*** |
| `UNION DISTINCT` on `dst` alone, 200 nodes / 600 edges | **200 reached, 0.9 ms — correct** |
| **the same + a `depth` column** (the one the UI wants) | **199 331 rows instead of 200, 36.5 ms.** `DISTINCT` keys on `(dst, depth)`, `(B,1)` ≠ `(B,3)`, **the guard EVAPORATES** |
| `UNION ALL` + per-path `INSTR(path,dst)=0` | **50 nodes → 549 721 paths · 600 edges → TIMEOUT > 60 s · 1500/51 000 → killed at 327 s** |
**`max_recursive_iterations` = 1000, MEASURED.** *(Amelia, against herself: she was about to assert
`4294967295`. **"Had I not started the container I would have walked into this room with an invented number,
in the document that wrote 'refuse to invent a threshold'."**)*
**The Rust BFS alternative, measured: 51 000 edges = 32 KB; full `SELECT` + `ORDER BY` while a writer
inserts 300 000 rows = 10.7–23.9 ms ⇒ p95 ≈ 24 ms = 1.6% of the NFR2 budget.**
> **🔴 And 36.5 ms PASSES NFR2 with 40× of margin. NFR2 cannot catch this failure: THE FAILURE IS FAST.**

### 8.8 D18's binomial (l. 1204) — why zero is the only threshold

**With n=300 and zero observed events, the 95% upper bound on the true rate is 3/300 = 1%** — the strongest
statement production can make. **If the true rate is 1%, the observed count is Binomial(300, 0.01): mean 3,
sd 1.72. Observing 0 and observing 6 are both compatible with a true rate of 1%.** A `≤ 0.01` threshold
cannot distinguish 0.5% from 2%. **"It is not a gate, it is a coin toss wearing a badge of authority."**

### 8.9 D41's telemetry arithmetic (l. 2272) — the numbers that killed FR52

**40 installs at 3 months × 5% opt-in = n_opt-in = 2** · **Wilson 95% on 4/8: [22%, 78%]** — *"not a
measurement — a paraphrase of ignorance"* · **rule of three: zero returns out of 8 → upper bound 37%** —
*"even the blackest result refutes nothing"* · **power to distinguish 30% from 40%: n ≈ 172. You have 2.**
**Selection bias, toy model: 8 enthusiasts at a modest share opt-in / 70% retention; 32 curious at 1% / 8% → true
retention 20%, observed 57% — a factor of 2.8.** **At 40% opt-in: observed 49%, truth 20%. Still 2.4×.**
**P(usable signal at 3 months) ≈ 0 · P(trust incident) ≈ 10–20% · impact: potentially terminal.**
**D42's unblock condition: ~1000 plausible installs — n_opt-in ≥ 100 for a Wilson interval narrower than 20
points.**

### 8.10 Other measured or measurable numbers

- **D17:** ~50–200 `local` interfaces/month · ~600–2400/year vs ~300 stable `universal` · **~2400 rows ≈
  500 KB against NFR18's 512 MB — three orders of magnitude** · dormancy window default **30 days** ·
  FR38 retention **90 days**.
- **D13:** `blocking_recall >= 0.999` · **90k pairs at 300 hosts is noise on a NAS i5** · `confidence` =
  **INTEGER milli-units 0..1000**.
- **D21:** transaction unit **~100 decisions or 1000 rows, whichever first**.
- **D29:** Argon2id **`m = 19 MiB, t = 2, p = 1`** (RFC 9106 second recommended) · **AC: < 300 ms per hash,
  MEASURED on the target** — *"a Celeron J4125 is not a desktop"*. **UNKNOWN until measured.**
- **D31:** the wrap = **22 lines of body** · FR48 = **120–180 lines** · **the wrapped blob is 88 bytes, not
  ~60** (`kek_id` 16 + nonce 24 + ct 48) — **D28's figure is wrong.**
- **D48:** ~20 bytes × ~10⁶ history rows = **tens of MB**, InnoDB secondary indexes included. **On a
  Plus-class x86, noise.**
- **D50:** real retries ≈ **1–5/year**.
- **D53:** the newtype tax is **~7 lines once, not per handler**.
- **D64:** credit **~2 days on story 1** · Murat: two DDLs ≈ **10–20 h across the MVP, ~1–2% of a ~1000 h
  MVP** — *"nobody abandons a project over 15 hours"* · **7 of 63 decisions served an engine nobody named
  will run** · HRTB risk **one day → half a day** · Murat's abandonment calculus **|Δ| < 5 points**.
- **D65:** the vocabulary gate was **RUN BEFORE IT WAS WRITTEN — simulated against the six documents on
  2026-07-17: 18 (document × retired-term) pairs in scope, 0 red. A measurement, not a prediction.**
  Against yesterday's state it goes red exactly where it should: **`architecture.md` `pending_accept` ×5 /
  `pending_commit` ×0**; **the brief `accept-as-declared` ×2 / replacement ×0**. **And
  `ux-design-specification.md`, which legitimately narrates its own rename, is GREEN with `pending_accept`
  ×2 alongside `pending_commit` ×8 — confirming the co-presence shape does not punish a document for having
  a memory.**
- **D57-scope:** the FR26/27 probe's decision rule: **11 instances → the graph is a `SELECT` and D57 will
  never have existed. 300 → D57 is a real decision, taken on a fact.**
- **D64 condition 3:** the stopwatch rule — **8 minutes → the objection is dead by measurement. 40 → we
  re-open, on a fact.**
- **Measured at D65 gate 1:** `collation_server = utf8mb4_general_ci` **on the target. The exposure is real,
  not theoretical.**
- **Uninstrumented bets, NAMED AS BETS (l. 3626):** **NFR2 (p95 ≤ 1.5 s on-read)** · **Argon2id AND age's
  scrypt on the real Celeron — TWO measurements, not one.**

## 9. 🔴 DECISIONS AMENDED BY THEIR OWN AUTHORS — the amendment IS the evidence the decision was tested

> *"Seven decisions moved. **Each was amended by the person who wrote it, on an argument from someone who
> was not in the room. That is the finding, not an anecdote."*** (l. 3650)
> *"At validation, four authors amended their own graven decisions on arguments from people who had not been
> in the room."* (l. 4046) · **"Four were withdrawn by whoever proposed them."** (l. 3557)

- **🔴 Winston amends D51, point 4 (l. 4467).** He wrote *"the discounted cost of the class is zero —
  PostgreSQL is SQL, replacing sqlx is SQL, both real porting scenarios are intra-SQL"*, **and signed it one
  day ago. It was FALSE.** D51's four-leak table: `implicit order`, `LIKE semantics`, `UNIQUE(hostname)` —
  **those reds are produced by the DIFFERENTIAL, not by MariaDB. Remove SQLite and they all go green.** The
  argument that made the renunciation bearable was precisely *"2/4 refute the premise — LIKE and order are
  NOT in D1's blind spot"*: **he had halved the class by showing D1 saw half of it. Remove SQLite and the
  class returns to full size while his signature does not move.** Worse: *"PostgreSQL is SQL"* **was doing
  the work of "PostgreSQL AGREES WITH MariaDB", and those are not the same claim — PostgreSQL's `LIKE` is
  case-sensitive, its unordered scan is heap order, not PK order. The only proof our trait did not depend on
  one dialect's defaults was HAVING A SECOND DIALECT IN CI.**
  ⇒ **AMENDED: the cost of the class is not zero. It is UNKNOWN, UNMEASURABLE, and paid on the day of the
  port — including an intra-SQL port.** *"I make the amendment because it is true, not because it wins."*
- **🔴 Murat amends D45's own table, on the D1 row (l. 4479).** He wrote *"D1 — the red is MariaDB says no —
  repairs: none, we do not patch InnoDB."* **False.** We do not patch InnoDB — **but D23 gave us two DDL
  files.** Take `UNIQUE(hostname)`: SQLite accepts `Foo` and `foo`, MariaDB `general_ci` refuses the second.
  Red. **Repairs: (a) lift the comparison out of SQL into `norm()` — the one we wanted; (b) put `ascii_bin`
  on the column in ONE of the two DDL files — one line, an honest commit message ("align collations"),
  green, AND THE COMPARISON STAYS IN SQL.** **The red was repairable from both sides. By his own criterion
  (D45), D1 was a NEGOTIATION.** *"**D23 is D1's negotiation surface. I wrote both and I never put them side
  by side.** What we lose by dropping SQLite is smaller than I made it look."*
- **🔴 Winston amends D57's own wording, on the cycle (l. 4623).** He wrote *"an undetected cycle is a
  traversal that does not terminate"* — **false on MariaDB: it bounds itself and returns a plausible
  truncated answer, which is worse.** *(And Murat then removed the obligation rather than the mechanism: **"a
  visited-set does not DETECT the cycle — it makes it moot. You do not get a decision for writing a BFS
  correctly."**)*
- **Winston withdraws D53's `WebError` ruling (l. 3005)** — *"my refusal of `WebError` was a preference —
  **`E0117` is a compiler. A taste argument against a compiler is zero rounds.**"*
- **Winston, on D57's representation (l. 4596)** — *"**I presented a non-choice as a blocking decision and
  left it on the critical path for a day. That is the step-4 defect — stack categories, not product ones —
  committed by the man who named it.**"*
- **Winston reverses himself INTO scope on D16 (l. 1120)** — *"An error of judgement on my part, not a scope
  arbitration."* **And he holds the same test when it cuts the other way on D41/FR52:** *"I brought
  `virtual_device` INTO the MVP with exactly this test, so I hold to it when it cuts the other way."*
- **D18's author turns his own accusation on himself (l. 1213)** — *"**I gated the easy and hedged the hard:
  exactly the cowardice I accuse the engine of, applied to myself.**"*
- **D19-rev — its author names his own theatre (l. 3910)** — *"I disqualified real captures because they
  have no oracle — and I was about to manufacture an oracle by introspection. **That is my own theatre, and
  it carries my name.**"*
- **Amelia amends herself twice (l. 4618, 4644)** — the invented `max_recursive_iterations` she was about to
  assert (**"in the document that wrote 'refuse to invent a threshold'"**), and **AC-D57-1's first draft,
  "decoration, thrown out by its author"** because a `BTreeSet` assertion **passes under a `MAX_DEPTH`-only
  implementation.**
- **The validation step records its OWN error rather than quietly fixing it (l. 3608, 3994).** It asserted
  the 39 feedback items were unapplied and that `prd.md` was *"a fossil"*. **False — and false for the exact
  reason this document spends 4 000 lines refusing.** The tables were written at **14:57**; the PRD was
  edited at **16:01** and the UX spec at **16:25**. **"The tables were read as the state of the world when
  they were only the state of the world at 14:57 — a claim true where it was written and false where it
  counted. THAT IS NFR9'S ERROR, COMMITTED BY THE VALIDATION STEP ITSELF."** **The structural fix is worth
  more than the correction, because the next reader would have concluded the same thing.**
- **The amendment that names the pattern (l. 4000):** *"identity first, gap second — not negotiable, it is a
  data dependency."* **The dependency is causality and stands** — the link must **EXIST**, satisfied by one
  correct row. **"STABLE" speaks about the false-positive rate over 300 hosts. That is NFR4 — a RELEASE
  criterion, not a compilation precondition.** *"I stacked the two in one sentence with 'non-negotiable' in
  front, which turned a quality arbitration into a law of physics. **That was ENGINEERING THEATRE: the
  vocabulary of hard dependency borrowed to armour a preference for rigour.**"*
- **D64's own pre-brief was wrong on the two largest entries of its own ledger, one in each column
  (l. 4441)** — *"the pre-brief reasoned from the map instead of the terrain. **That is this document's own
  signature error, committed by the party that convened the room to avoid it.**"*

## 10. NAMED THEATRE — do not build. (A standing practice, including against its own proposers.)

**Killed by D45, with their proposers' consent (l. 3555): the parity counter · `MemoryRepository` · the
`trait-vocabulary` grep · 15 of the 50 traps · a `kek_id` that nothing verified.** *"Four were withdrawn by
whoever proposed them."*
- **Dependencies/CI (l. 841):** `cargo-audit` **on top of** `cargo-deny` (*redundancy disguised as defence
  in depth*) · **a `cargo outdated` CI step that blocks nothing** (*a report that blocks nothing is not a
  gate, it is an RSS feed*) · auditing transact-rs's maintainers · forking/vendoring sqlx · **an SBOM at
  MVP** · **testing that axum routes.**
- **Security (l. 1796):** SBOM at MVP · **a Docker image scanner (Trivy/Grype) on a static Rust binary in
  distroless** — *"signal tends to zero, noise tends to CVE-of-the-month-in-a-libc-we-do-not-have. **Weak
  position, revisable** once the base image grows"* · **a pentest/DAST of the MVP** — *"a solo dev triaging
  DAST noise is time stolen from AC12-3..7"* · **"testing that axum authenticates our routes"** — *"axum
  routing correctly is axum's problem."*
- **Connectors (l. 2073):** **wiremock / httpmock** — *"the fixture IS the trait. Adding a network stack to
  a test that must never touch the network is testing `reqwest`"* · **chaos engineering / toxiproxy at
  MVP** — ***"the chaos here is the vendor's schema, not network jitter — and toxiproxy cannot rename a
  field"*** · testing `Display` on `ConnectorError` (*testing the derive macro*) · **a "degraded mode" test
  asserting the process still runs** — *"`assert!(true)` with a heartbeat"* · **fuzzing the parser at MVP**
  (weakly held) — *"arbitrary bytes prove you do not panic on garbage UniFi will never send. **The mutation
  set derived from a real body is worth 100× at a tenth of the cost.** After, not before."*
  **Nuance, not rejection:** retry/backoff tests on the tokio clock test the library's timer — **but the
  POLICY is ours and depends on the variant** (`AuthFailed`: **never retry — you would DoS your own
  controller with a dead key**; `Unreachable`: always). **Test the `variant → decision` table, pure, no
  clock.**
  > *"If I had to reduce it all to one assertion: **the faulted run cannot invent a single fact.** Everything
  > else is observability."*
- **Errors (l. 2649):** a hand-maintained markdown catalogue of error codes **next to the `match` that is
  already the source of truth** · **`.context()` on every `?` — noise simulating traceability.**
- **Identifiers (l. 2721):** `BINARY(16)` plus a hex view (**rebuilding the readability you just removed**)
  · a benchmark spike (*it costs more than the 20 MB it saves*) · **`ORDER BY entity_id` as a business
  sort — tempting and false (two nodes, two clocks); business ordering goes through `observed_at`.**
- **Repository (l. 2773):** a Fowler-style `UnitOfWork` with `register_dirty` · **a `Repository` per
  aggregate** (*a single transaction crosses them all, and the two-file argument literally requires one
  trait*) · **a `savepoint()` "just in case"** — *"an identity decision is never split in two, so a
  savepoint inside one is a CATEGORY ERROR"* · **`RepositoryError::Sqlx(#[from] sqlx::Error)` "to keep the
  context" — the leak, dressed as prudence.**
- **Telemetry (l. 2398):** the "reconciliation ran" ping · **cross-instance aggregation of the north-star —
  "the purest theatre in FR52"** · the isolated install event.
- **Structure:** **a third `opencmdb-web` crate — modularity theatre** (l. 3025) · **`status_exhaustive.rs`
  — the name that would have lied** (l. 3097).
- **Process:** **an in-memory `WriteUnit` "just for the traps"** — *"`MemoryRepository` in a smaller hat,
  and D51's four legs apply word for word"* (l. 4548) · **`MAX_DEPTH`** (l. 4637) · **an impact trap
  suite** — *"it would test that `HashSet` works"* (l. 4655) · **AC12-1 (`wrap/unwrap == identity`) — it
  tests someone else's crate** (l. 3755).
> **The two standing rules that generate this list: "a gate that has never caught anything is a tax on the
> gates that do" · "a requirement your worst-case scenario satisfies is not a requirement" — and the
> validation earned a third: it applies to YOUR OWN WORK. Before writing a test, ask what would make it
> red. If nothing would, you are writing decoration.** (l. 4091)

## 11. OPEN — and open is a state. Nothing here is resolved by this file.

**Status: NOT READY** (frontmatter, l. 7). **Confidence: HIGH.** **Checklist: 11/16.** *"Five items
unchecked, two under Architectural Decisions… **you cannot write `Cargo.toml` today.**"*

**The three open critical gaps, as the source's own frontmatter names them (l. 8–11):**
1. **Four crate selections: i18n** (**must be greppable/diffable — the glossary gates run over the
   translation files**) · **config** (**three boot cross-invariants as startup failures**) · **`/metrics`**
   · **Docker base image.** **Plus the sqlx 0.9 verification list, `migrate!` FIRST. THIS IS NOW THE ONLY
   THING BETWEEN HERE AND `Cargo.toml`.**
2. **D65 — BOTH remaining gates, grouped into `cargo xtask ci`, BEFORE story 1.** (`--ddl-collation` with
   **no allowlist — the absence IS the mechanism**; `--vocabulary` **volet A absence + volet B
   co-presence**.)
3. **🔴 STORY 1, found at the D57 table and it is not D57: `Reads` as a single trait DOES NOT COMPILE.**
   *(`ReadRepository` is `&self`, `Unit<'u>` is `&mut self`, and `core` cannot name `sqlx::Executor`.)*
   **D49's prose is false in the singular — it must be TWO traits delegating to a generic free function in
   `bin`.**

**Open questions carried, each with its owner (do NOT resolve these by inference):**
- **D24 — temporal-history growth: "OPEN and unaddressed". A NUMBER, one page, before the epic.** *(D64:
  it changes name, not nature.)*
- **Product decisions parked (l. 3969):** D52's supported floor *(answered by D64: (a) ≥ 10.11)* · **the
  `ImplausibleResponse` threshold** · **the `blind → live` hysteresis** (*"a flap creates a gap on
  return"*) · **the recall threshold that defines "stable"** (F43) — *the metric can be supplied (join
  recall over the traps); **the number facing it is a product arbitration.** Until it is set,
  "identity_link must be stable" is a preference, not a requirement.*
- **D61 — `network_id ≡ l2_domain`: OPEN, named, unverifiable on the developer's network.**
- **D62 — the supported UniFi matrix: RE-OPENED on the correct axis** (Network application 10.x).
- **D33 — the exact `ConnectorError` variant count.**
- **D10 / AC-8e — value flapping:** answer on `v1`, observed → `v2` (re-fires), **observed returns to
  `v1`** — does the doubt re-fire? **Position: yes** (*silently re-approving `v1` rebuilds a `RESOLVED`
  verdict via value equality — the banned auto-following field disguised as an optimization*). **Held
  WEAKLY; the UX cost is real for an oscillating port. Must be a named, owned, reversible test:
  `#[test] fn flapping_value_refires_doubt()`.**
- **D17 — do the ingestion probes capture probe requests?**
- **Five Tier 2 patterns, never party-tested, two biting at story 2 (l. 2970):** **HTMX fragment routing
  convention** (*agents will diverge on story 2*) · **🔴 axum 0.8 uses `{id}`, NOT `:id` — the training
  corpus says `:id`, and axum 0.8 PANICS AT ROUTER BUILD TIME; it does not fail to compile. The drift
  vector #1, dead centre.** Needs a convention and a smoke test · **JSON envelope + error shape** ·
  **`tracing` conventions + a `batch_id` correlation id in every span** (*on a reconciliation engine,
  correlating observation → link decision → gap **is** the debugging*) · **i18n key convention.**
- **Category 3 open (l. 2105):** per-ENDPOINT blindness — *"I cannot settle it without NFR8's version-tagged
  fixtures. Do not freeze it before we have them"* — **and it touches the schema** · the coalescing key.
- **Three measurements outstanding (l. 3977):** **Argon2id on the real Celeron (< 300 ms)** · **age's scrypt
  derivation on the same CPU — its own cost, unmeasured** · **NFR2's p95 on-read.**
- **The verification list before `Cargo.toml` (l. 3979):** **`sqlx::migrate!` — folder format,
  `_sqlx_migrations` schema, checksums** (*"the only thing that could cost us a day"*) · **`zeroize` on key
  types** · **is the Synology keystore reachable from a container** (*"unknown; would not build on it
  without proof"*).

**First Implementation Priority — NOT `cargo new` (l. 4095), as amended by D57-scope and D65:**
1. **The cardinality probe (D19-rev step 0) — DONE 2026-07-16.** *(Its trigger was: cardinality 1 minority →
   STOP and rewrite the PRD before a single screen exists. **It returned a large majority. The screen holds.**)*
2. **A SHORT PRD/UX pass — F42–F47 only.** *(F1–F37 were applied 2026-07-16 at 16:01 and 16:25. **There are
   no "five bombs".**)*
3. **D57 — CLOSED 2026-07-17. At MVP there is no graph at all.**
4. **D65's two gates + the four crate selections + the `sqlx 0.9` verification list — `migrate!` first.**
5. **Then story 1 (D1-rev):** the walking skeleton **that shows a real gap on a cardinality-1 perimeter, and
   abstains — visibly, with a counter — everywhere else.** The `WriteRepository` + `transact` skeleton and
   two empty adapters **COMPILE before one line of identity logic exists.** **If the HRTB is not green in a
   day, take the `Box<dyn>` escape hatch — the risk carried alone is bounded to one day, and that bound is
   the point.** *(⚠️ And `Reads` must be two traits — gap 3 above.)*

**The one structural weakness, and it is the same one three times (l. 4060):**
> **A frame with no opposition does not produce a blind spot by mistake — it produces one BY
> CONSTRUCTION.**
> **(1)** Step 4's categories were **stack** categories, not **product** categories → **four FR domains
> never landed.** **(2)** 56 decisions were taken with **nobody mandated to push the other way** → *every
> decision makes the product more TRUE; none makes it arrive EARLIER.* **(3)** The ethic is **entirely
> negative** → the product knows with surgical precision how not to be unpleasant, **and has no idea how to
> be good.**
> **All three were found by voices that were NOT IN THE ROOM** — the PM, then the designer. **That is not
> luck, and it is the lesson: THE COUNTERWEIGHT MUST BE SUMMONED, because a frame cannot see its own
> edge.**

**AI Agent Guidelines, verbatim in force (l. 4079):**
- **Follow the mechanisms, not the prose.** Where a rule is held by the compiler (D47, D49, D50, D53, D54),
  **the rule IS the signature** — do not restate it in a comment, **and do not weaken the signature to make
  something compile.**
- **When a gate reds, apply D45 BEFORE reaching for the fix:** *how many repairs does this red have, and is
  the cheapest the one we wanted?* **If the cheapest repair is the wrong one, the gate is telling you
  something about the DESIGN, not about the code.**
- **`core` never learns about `anyhow`, `axum`, `sqlx` or `askama`.**
- **Refuse to invent a threshold.** Every float that decides, every magic number, every "we'll tune it" is a
  **reopened decision — bring it back to Guy.**
- **In `crypto/`, four lines cannot be tested and three will be hallucinated (D31). Review them by hand,
  every time.**
- **"A requirement your worst-case scenario satisfies is not a requirement" — it applies to your own work.
  Before writing a test, ask what would make it red. If nothing would, you are writing decoration.**

**Areas for future enhancement (l. 4069):** **D58 — the narrator** (Growth) · **D24's retention/growth
policy before a long-running instance exists** · **IPAM and Topology as they are reached — deliberately,
and the restraint is recorded as a decision** · **the Postgres port — D51 says re-audit the trait BEFORE,
not during** *(and D64 renunciation 3 made that audit more expensive)*.
**Why IPAM (FR21-25) and Topology (FR53) have a folder and zero decisions — "correct proportionality"
(l. 3578):** *"**What makes IPAM safe: no decision here cannot be taken at the moment of writing it, because
none constrains anything else.** A domain whose choices are local and reversible needs patterns, not
architecture."* · Topology: **one FR.** *"If a single FR justified an architectural decision, we would have
53."* · **Insight — acceptable, not free: the risk is the VOLUME, not the feature. A sizing hole that
becomes an architectural hole the day the answer is "we need a time-series table".** ⇒ **an
order-of-magnitude calculation, one page, before the epic. Not a decision — a number.**

## 12. What this file is NOT a source for

- **❌ The state of any REQUIREMENT.** The source of truth is **`prd.md`'s `editHistory` frontmatter** —
  never a table in `architecture.md`, and never this file. *"This document records what was DECIDED.
  `prd.md` records what the requirements ARE."* **That mistake has been made three times.**
- **❌ The ARGUMENT behind any decision.** Every entry here is a compression of an argument that was
  attacked, **and the argument is the part that makes it hold. No decision may be applied from its
  one-liner.**
- **❌ Anything you intend to ACT on without opening `architecture.md` at the cited line.**
- **⚠️ This file is itself in D65 volet B's scope** — it is the sibling of `product-brief-opencmdb-distillate.md`,
  which **is** named in the six-document set (l. 4845). **If it holds a retired term (`accept-as-declared`,
  `revert`, `reverting`, `pending_accept`, `ignore`/`ignorer`, `merge` in English) without its live
  replacement, it is RED — and the repair is to apply the rename, not to delete the word.** *A stale
  document is a gap. This one has a sha256 above so you can tell.*
- **⚠️ The source's own Decision Index is stale by ~94 lines** (§1). **Use the anchors in this file, or
  `grep -n`. Do not trust either map.**

---
*Distilled 2026-07-17 from architecture.md @ 96897b06 (4 886 lines) — **~1.6:1 on lines, ~1.4:1 on bytes,
and §0 says why that is the honest ratio rather than a failure to try.** Every line number verified against
that hash. **When the hash moves, this file is a fossil that looks alive.***
