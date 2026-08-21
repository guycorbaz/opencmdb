# Story 6b.10: The copy — FR and EN, every string a key

Status: in-progress

Epic: 6b — *L'interface de la maquette*. **Tenth numbered slot, eleventh story file.** It is the
first story whose subject IS `crates/opencmdb-bin/locales/app.yml`, and — measured below — the first
whose scheduled deliverable **was already written by the ten story files before it**. What is left is
not the text. It is a **cross** the validation measured and the first draft had only half of: the
English render carries ~470 assertions and **no human look**, the French render carries nine human
looks and **no assertion at all** — and the gates that keep either from regressing once this epic
stops watching.

## Story

As the operator,
I want the interface in my language,
so that the product is usable in both without one being a second-class rendering of the other.

## Acceptance Criteria

Transcribed from `epics.md:2280-2296`, **unmodified** — divergences are raised in §0 (a story may not
edit an AC; only a retrospective or a correct-course may).

1. **Given** every string the ten screens introduce
   **When** it ships
   **Then** it exists as an i18n key **with both translations** — FR and EN, neither derived from
   the other. *(Guy's arbitration, 2026-08-13; the reference mock is a FRENCH RENDERING of a
   bilingual GUI, not a French-first decision.)*

2. **Given** the new keys
   **When** `cargo xtask ci` runs
   **Then** the forbidden-word lint and the glossary uniqueness test cover them, and the glossary's
   FR column is **authoritative over the mock's wording** wherever the two differ.

3. **And** ⚠️ this is the epic's largest single block of text — **32 keys today against roughly a
   hundred in the mock** — and **a string added later in one language only is exactly the defect
   this story exists to prevent.**

### The three ACs this story adds to itself, and why each is here rather than in `epics.md`

A story may not edit an AC. These are **obligations this story takes on**, recorded here and
registered for Epic 6b's retrospective, which owes the epic's AC a correction:

4. **The build hazard is CLOSED, not registered.** `app.yml` is invisible to Cargo's incremental
   build (§0e, re-measured on this tree today). Every measurement this story takes is taken through
   that hazard. It is closed before anything else is measured, and the closure is itself measured.
5. **The English rendering is LOOKED AT, in a browser, screen by screen** (§0b). It is the default
   locale and no story in this epic has ever rendered it.
6. **The floors and premise counts this story touches state what they are FOR** (§0f) — the register
   row assigned to this story asks exactly that question and refuses to answer it.

## §0 — What contexting found, and what it refutes

### §0a. 🔴 THE CENTRAL MEASUREMENT: AC3's premise is STALE, and the block of text is already written

`epics.md:2296` says *"32 keys today against roughly a hundred in the mock"*. ⚠️ The epic was created
by correct-course on **2026-08-13** (`epics.md:2090`) but that sentence entered the tree on
**2026-08-14**, in commit `001e24a` — `git log -S`, because a story about stale figures may not date
one by association. Measured on `master` at `97c0e9a`, key counts at each merge commit:

| commit | story | top-level keys |
|---|---|---|
| `d284cfb` | before Epic 6b (6.3) | **31** (+ `_version` = the AC's 32) |
| `0b42bd0` | 6b.1 | 31 |
| `d8cc438` | 6b.2 | 47 |
| `d1bb128` | 6b.3 | 63 |
| `cea6853` | 6b.4 | 90 |
| `20e854c` | 6b.4b | 98 |
| `b8a5795` | 6b.5 | 108 |
| `7cc1602` | 6b.6 | 153 |
| `f32c268` | 6b.7 | 184 |
| `b18495a` | 6b.8 | 221 |
| `81589ff` | 6b.9 | **284** |

🔴 **The 6b.7 row was MISSING from this table when the story was first written, and the validation
found it.** Its absence made 6b.8 look like a 68-key story where it added **37**, and it turned *ten*
story files into *"the nine stories"* — in a table introduced as *"key counts at each merge commit"*,
in the story whose whole subject is figures that have stopped matching what is there. It is kept
visible rather than silently repaired: **an enumeration written from memory of which stories exist is
exactly the shape story 5.13b shipped when it reserved a UUID prefix by enumerating one directory.**
The table is now `git log --follow -- crates/opencmdb-bin/locales/app.yml`, not a recollection.

**253 keys were added by the TEN story files** (6b.1–6b.9 plus the 6b.4b insertion), against an
estimate of *"roughly a hundred"*. ⚠️ **The multiple depends on what is compared and the story first
got it wrong**: AC3's ~100 is the mock's TOTAL string count set beside 32 existing keys, so the
comparable estimate for the ADDITION is ~68 and the overshoot is **≈3.7×**, not the 2.5× a
total-against-addition comparison gives. And the property AC1 asks for **already holds**: 284 keys,
**284 `en:` values, 284 `fr:` values, zero missing on either side, and no empty value on either side**
(independent lint over all 568 entries).

🔑 **This is not a reason to shrink the story; it is the story's finding, and it changes what the
story is.** AC3's closing sentence — *"a string added later in one language only is exactly the
defect this story exists to prevent"* — describes a **defect that has not happened**, because every
story added its keys in pairs. What AC3 could not anticipate is that ten story files writing copy in
French, browser-checking it in French, against a French mock, would leave the **English** side
written and never read. AC1's letter is met. §0b is what its spirit is actually about.

⚠️ **Do not "confirm" AC1 by re-running the count.** A count of `en:`/`fr:` lines is a count of what
is IN the file; a string that never became a key is not in that population at all — story 6b.6's
sentence about `every_key_carries_both_locales`, and the reason its two i18n defects were found by
looking at the page. AC1's real question is *what is on the screen that is not a key*, and the sweep
in §0c is how it gets asked.

### §0b. 🔴 THE ENGLISH UI IS THE DEFAULT AND NO HUMAN HAS LOOKED AT IT — and the validation inverted the rest of this section

- `main.rs:373` — `std::env::var("OPENCMDB_LOCALE").unwrap_or_else(|_| "en".to_string())`. **The
  default locale is `en`.** A fresh deployment renders English.
- Every browser look in this epic was French, and **the evidence comes in two grades, kept apart
  because the draft mixed them**. `grep -n OPENCMDB_LOCALE` over the story files hits **eight times in
  six files** — `6b-2:448, :678`, `6b-3:282, :623`, `6b-4:470`, `6b-4b:312`, `6b-5:326`, `6b-6:409`
  (6b.2's own note: *"**Export** `OPENCMDB_LOCALE=fr` first**: the default locale is `en`"*, quoted
  verbatim this time — the draft paraphrased it inside quotation marks). **6b.7, 6b.8 and 6b.9 carry
  the variable zero times**; they say *"in French"* in prose (`6b-7:635`, `6b-8:499`, `6b-9:62, :611`).
  ⚠️ The conclusion holds for all nine; the *"measured by grepping"* sentence supported six of them,
  and it named three files the grep does not reach while omitting one it does.
- 🔴 **The draft claimed the epic's i18n defects were *"both on the English side, both found by
  accident"*, and the validation refuted half of it — the correction is sharper than the claim.** Of
  the epic's three known instances, **one** is genuinely English-side and invisible to a French look:
  story 6b.6's *"Drift"* at four sites for `gap` (the French was right everywhere, the code already
  said `gap`, and the sole test pinning it asserted `contains("Drift")` under a message reading *"a
  **Gap** must be a row"*). The other two are the **opposite** case — an English literal surfacing on
  the **French** UI (6b.3's example data, 6b.8's *"ARP/ping sweep"*) — and **both were found BY the
  French browser look**, which is the pass §0b says cannot see this class.

🔴 **AND THE GAP-HUNT LAYER INVERTED WHAT IS LEFT OF THIS SECTION, BY MEASURING THE OTHER AXIS.**
The draft said the English side *"has none"*. That is true of **human** readings and false of
automated ones, and the true picture is the opposite of what the section's title suggests:

```
gaphunt: ambient locale = "en"
shell contains "Tableau de bord" = false
shell contains "Dashboard"       = true
```

**Every render-side test in this crate renders in ENGLISH.** Measured over the code halves: **241
`t!(` call sites and ZERO carrying a `locale =` override**; the only `set_locale` is `main.rs:374`, at
boot, and four separate doc comments forbid a test from calling it. So:

| | automated readings | human readings |
|---|---|---|
| **English render** | ~470 tests | **0** |
| **French render** | **0** | 9 browser looks |

🔴 **And the French half of that table has a measured witness.** Set `nav.dashboard`'s **`fr`** value
to its own key name — story 6b.6's defect, on the other side — and the result is **472 + 161 + 70
tests green and nine gates green**. `no_screen_renders_a_key_name_as_a_label` exists for precisely
that defect and is **blind to it**, because it renders in `en`. *Epic 5's dominant class again: the
guard is correct about what it tests, and the defect is one locale away.*

🔑 **This also explains 6b.6's *"Drift"* better than the draft did.** The tests DID render English —
one of them *required* the defect (`assert!(contains("Drift"))`). The English render is not unread; it
is read by ~470 assertions that were written by the same author, in the same session, as the copy they
assert. *A test written beside the string it checks reads the string back, not the language.*

🔑 **So the corrected shape is a cross, and both diagonals are this story's:**
- **English: heavily asserted, never looked at.** A grep and a human eye find what assertions written
  alongside the copy cannot — a retired term, a machine-turned sentence, a register that slips.
  `gesture.merge` is the specimen, and a **grep** found it. → **AC5.**
- **French: looked at nine times, never asserted.** A look catches an English word that leaks into
  French; it cannot catch a French value that silently regresses to its key or to English, because
  the reader has no second copy to compare against. → **the guards T5 and T6 must be able to render a
  chosen locale at all**, which today nothing can.

⚠️ **T9's French control pass is therefore not a courtesy — it is covering the genuinely unasserted
side**, and the draft had it as an afterthought.

⚠️ **And the risk is WORDING, not layout — the UX spec has already reasoned the layout half away
and this story must not claim it back.** `ux-design-specification.md:1438-1442`: *"French runs longer
than English for the same sentence, so a layout measured in French holds in English while **the
reverse does not follow.** The mock is therefore the safe side to measure — and *safe* is not
*verified*."* Nine French looks are the safe side. What they cannot cover is what the English words
SAY.

⚠️ **State the limit before doing it**: a look is not a proof, and this one will find what a reader
notices — an untranslated string, a retired term, a sentence that reads as machine-turned English, a
label that resolves to its own key. It will not find a subtly wrong nuance. Say what it found; do not
promise what it covered.

🔑 **And it has a written checklist, which is better than an impression**: the *Microcopy Rules* at
`ux-design-specification.md:1446-1451` — (1) button = action verb, feedback the same verb as a past
participle; (2) error = cause + next step, **never blame the user**; (3) the tool's *"I"* only for an
action it attempted and failed, never for a state; (4) **one term = one translation, always, no
"elegant" synonyms**; (5) **empty ≠ failure — calm, never alarming.**

⚠️ **There are FIVE and the story's first draft said four, citing a range that contained three.** The
validation counted them. The missing one is **rule 5**, and it is the rule that matters most on a
product where nine screens of ten are placeholder or example surfaces — *the* rule an English sweep of
this particular interface should be carrying. **A checklist quoted from memory is a checklist short by
whatever memory dropped**, and what it dropped was the relevant entry.

### §0c. 🔴 THE ONE GLOSSARY VIOLATION IN THE FILE, AND IT IS THE PRIMARY CONTROL

Every value in `app.yml` was linted against the retired vocabulary, per locale, both the values and
the key names. **One hit, and one only:**

```
gesture.merge:
  en: "Merge"
  fr: "Merger"
```

- Both binding tables retire **`merge` in ENGLISH** by name — `prd.md:1037` and
  `ux-design-specification.md:1386`. The PRD's wording, **verbatim and attributed to it alone**: *"it
  names the forbidden operation — the founding pillar is **linked, never merged**; the French UI verb
  « Merger » is the fixed translation of `document` and carries no such claim."* ⚠️ The UX spec says
  the same thing in different words (*"it names the forbidden operation; **the pillar** is* linked,
  never merged *— … and **makes** no such claim"*); the draft cited `:1352`, which is the blank line
  after the gesture table, and hung one wording on two sources. *Attributing one sentence to two
  documents is not quotation.*
- The binding EN term for this gesture is **`document`** (`document-field` / `document-all`).
- The FR half is **correct** — « Merger » is the fixed translation.
- The key NAME is also in scope: the PRD's first column is *"EN (docs, API, **code**)"* (`prd.md:991`)
  — and the UX spec's twin heading is **stronger and is the one to cite**: *"EN (docs, API, code —
  **and a UI locale**)"* (`ux-design-specification.md:1339`), corrected deliberately at `:1423-1426`
  so that English is named as an interface language rather than only as the code side. 🔑 *The
  strongest version of this story's own argument was in the document its first draft did not quote.*

🔴 **This is the gesture the whole product is built around**, the primary control of `/triage`
(`page.rs:909`, `:1002` — `action_bar("gesture.merge")`), and the English UI labels it with the one
word the glossary forbids. It is **story 6b.6's *"Drift"* defect, one story later, on a different
term**: the FR right, the EN retired, no guard able to see it — and 6b.6's repair was scoped to the
`gap`/écart pair alone.

🔑 **And it is the epic's own founding sentence turned on the epic**: *"if a gesture is named after
an operation we forbid, someone eventually implements the operation."* → **Arbitration 1, §0h.**

**The rest of the sweep, and what it found:**

| Surface | Method | Result |
|---|---|---|
| Template **text nodes** (17 files) | strip `{{ }}` / `{% %}` / `{# #}` and tags, look for letters | **clean of copy** — `opencmdb` (a proper noun) at `_shell.html:12` and `:19`, plus a bare **`v`** at `:21` (`v{{ version }}`), locale-neutral. ⚠️ The draft said *"only `opencmdb` … twice"* and the `v` is a third literal a sweep presented as exhaustive should have named |
| Template **human-text attributes** (`aria-label`, `title`, `alt`, `placeholder`) | grep, minus interpolations | 🔴 **ONE**: `_gap_card.html:1` — `aria-label="Reconciliation result"`, an English literal on the French UI, invisible to every existing guard **and** to a sighted browser look. ⚠️ **Scoped honestly below** |
| Rust view code, prose literals in the CODE half | multi-word string literals, keys and paths filtered out | clean of screen copy; the residue is error/log text (§0g) |
| Rust code half, **accented** literals | code half only, comments stripped | three: `example_data.rs:1010` *Invités*, `:1086` *Comptabilité*, `:1114` *Supervision caméras* — **all deliberate**, proper nouns under Guy's arbitration of 2026-08-20 (`ExampleApp::owner`'s doc states it) |

⚠️ **The `aria-label` is the sharpest of the three, and the reason is what it teaches**: it is copy
that a sighted browser look **cannot see**. AC5's browser pass would have walked past it; the
attribute sweep found it in one grep. *A look at the page reads what the page shows; a screen reader
reads what it does not.*

⚠️ **And it is scoped rather than inflated, because AC1 says *the ten screens* and this is not one of
them.** `_gap_card.html` is `GapFragment`'s template (`page.rs:344-350`), served at **`/gap`**
(`main.rs:452`) — a live, reachable, authenticated address, and an HTMX fragment **embedded by no
page**, which is story 6b.4's own registered row. So: a served English literal in the product's copy,
on the eleventh address, **outside AC1's letter**. It is the clearest single instance of *a string
that never became a key*, which is what AC1 is for — and it goes in under arbitration 2, not by
pretending the route is a screen.

### §0d. AC2's GATE: a seam noted since story 3.8, never built, and it must be TWO carriers

- `app.yml:2`, in the file's own header, since story 3.8 (2026-07): *"the D65 vocabulary/forbidden-word
  gate **can later** lint these strings."* `3-8-transversal-anchors.md` records the seam **four
  times** (`:15`, `:24`, `:43`, `:72`) — `:43` is the one whose wording matches `app.yml:2`; the draft
  said *twice* and named the two that do not. **Later is this story.**
- Measured: `gate_vocabulary` (`xtask/src/main.rs:426-481`) has two volets. **Volet B** walks seven
  planning documents. **Volet A** walks `crates/` and skips anything whose extension is not `rs`
  (`:461-463`). **`app.yml` is read by neither.** `CODE_RETIRED` (`:419`) is also **locale-blind** —
  it could not carry `merge`, which is forbidden in one language and binding in the other.
- The glossary side: `state_vocabulary.rs:153` pins the **STATE axis** against `BINDING_STATE_AXIS`
  (`:106`), a **transcribed, not derived** constant — its doc says deriving it from the locale file
  *"would make the check compare the locale file to itself"*. There is **no equivalent for the
  GESTURE axis**, which is where §0c's violation lives.

🔑 **Two carriers of different kinds, and neither subsumes the other — story 6.3's design, applied.**

1. **A ninth gate**, in `cargo xtask ci`, over the FILE. It sees the key **names** as well as the
   values, it runs when the crate does not compile, and it is what AC2 names literally.
2. **A test in `opencmdb-bin`**, over the **RESOLVED** value in both locales. Story 6b.4b's lesson,
   which cost that story four findings: **read the key names from the file and the values from
   `t!()`, and the YAML syntax becomes irrelevant.** A `fr: |` block scalar defeated a line-based
   parse and put a forbidden sentence on all five controls with every guard green.

🔴 **AND THE GATE'S SCOPE IS A DECISION THE DRAFT LEFT UNSTATED, WHICH THE VALIDATION PRICED.** §0c
says the key NAME is in scope because the glossary's first column names *code*; T4 says *"key names in
scope too"*. Applied to `app.yml`, that is one rename. **Applied to `crates/` as the column literally
reads, it reds the committed tree in ~135 places** — `Expectation::MustMerge`, 87 `must-merge` tokens,
`Merged`/`merged` throughout the identity engine — including **87 occurrences across fourteen
sha256-locked fixture files**, which cannot be renamed without reddening the `fixtures` gate.

🔑 **So the gate is scoped to `crates/opencmdb-bin/locales/app.yml` and nothing else, and that
narrowing is WRITTEN rather than inherited from where the code happens to be.** `merge` is a real word
about a real operation the engine must talk about (*must-merge* is a trap expectation, and the pillar
is that the product does **not** merge, which the corpus has to be able to say). What the glossary
forbids is `merge` **as the name of the operator's gesture**, and the locale file is exactly where
gesture names live. ⚠️ **Stated as a limit, not as a property**: this gate cannot stop a future story
naming a route `/merge` or a handler `merge_entity`. That closure is the D65 volet-A denylist, and
extending it means deciding what to do about `MustMerge` first. **Register it; do not widen the gate
into a red tree.**

⚠️ **The direction of the failure decides which parse is acceptable where.** For a *denylist*, a
parse that misses a value is a **hole**, not a false alarm: a forbidden word inside a block scalar
goes unseen and the gate reads green.

🔴 **THE DRAFT LEFT THIS AS A TRADE-OFF AND THE VALIDATION MEASURED IT INTO A DECISION.** It built
both parses and ran twelve legal YAML shapes through each:

| shape | naive line split | real YAML parse |
|---|---|---|
| double-quoted · single-quoted · unquoted · escaped quote · quoted key | caught | caught |
| **block scalar `\|`, `>`, `\|-`** · **flow mapping `{en: …}`** · **4-space indent** · **plain multi-line continuation** · **nested key name** | **MISSED (7)** | caught |

**And `yaml-rust2` 0.11.0 is ALREADY IN `Cargo.lock`, via `config`.** Adding it to `xtask` is **one
line in the lockfile**, and it builds `--offline`. 🔑 *There is no trade-off to state — the limit the
draft was prepared to write down costs one already-vendored crate to remove.* **T4 does not take the
"state the limit" branch.** ⚠️ Two of the three shapes the draft suspected — single quotes and escaped
quotes — are **caught by the naive parse**; four it never named are missed. *The enumeration was wrong
in both directions, which is why it had to be run rather than reasoned.*

⚠️ **A real parser has its own failure mode, and it is silent.** The validation's first hand-rolled
event walker **dropped a whole key from the parse** and the gate then reported ✅ on `en: "Merge"` — the
denylist hole arrived at from the opposite direction, and it survived a re-run by hand before the
cause was found (a parent mapping's key/value flag never reset after a nested value). **Prescribe a
COMPLETENESS assertion** — `entries == 2 × keys + 1` for `_version` — or the gate can go blind with no
error and no warning.

⚠️ **And the word matcher must not be `gate_vocabulary`'s.** `contains_word` treats `_` as a word
character — right for Rust identifiers, wrong here, where `_` is this file's own separator. Measured:
`gesture.merge` reds and **`gesture.merge_all` is GREEN**, on a file where **107 of 284 keys carry an
underscore**. The draft's M5 prediction fails on the ordinary snake_case spelling.

🔴 **THE TWO CARRIERS' SEPARATING ROW IS NOT THE ONE THE DRAFT CHOSE, AND THE REAL ONE IS STRONGER.**
With a real parse, the block-scalar row (M4) **reds both carriers** — carrier 1 subsumes carrier 2
there, so the row the draft picked to prove independence proves the opposite. What actually separates
them, measured with a control on one tree: **a word retired in `fr`, in a key that has no `fr` half.**

```
carrier 1 (reads the file)     →  all gates GREEN
carrier 2 (reads the resolver) →  FAILED: renders "zzretired wording" in fr
```

🔑 **The separating property is the FALLBACK, not the syntax.** `rust-i18n` falls back to `en`, so a
French screen can render a French-retired word **that exists nowhere in the French column** for a
file-reading gate to find. *Carrier 2 is load-bearing because the file is not what the operator
reads* — which is the same sentence as story 6b.4b's *"a guard that reads the source measures what was
written, never what was served"*, arrived at from the locale axis.

🔴 **The gate does NOT go in `xtask/src/main.rs`.** Measured: **1908 code lines against the 2000
ceiling**, 92 lines of headroom. Story 6.3 hit this exact wall and put `observed-immutable` in its
own module; `CLAUDE.md` requires *split, not grown*, and 6b.6 is the only story in this epic that
split **before** the gate asked. New module: `xtask/src/copy_vocabulary.rs`. And `main.rs`'s module
doc enumerates the gates and **says of itself** that story 5.12's review caught it listing six while
the file implemented seven — *"adding a gate below without adding it here is the same defect again."*
Add the row in the same edit as the gate.

### §0e. 🔴 THE BUILD HAZARD — re-measured on this tree, today, and it is AC4

`deferred-work.md` (story 6b.9's section) records it. **Re-measured on `master` at `97c0e9a`, with
the workspace already built:**

🔴 **THE STORY'S FIRST DRAFT OVER-GENERALISED THIS, THE VALIDATION REFUTED IT, AND THE REFUTATION IS
KEPT IN PLACE BECAUSE IT CHANGES AC4, T8 AND FIVE MUTATION ROWS.** The draft said *"any mutation that
edits `app.yml` alone measures nothing"*. **That is true of `cargo build` and FALSE of `cargo test`**,
and a mutation pass runs `cargo test`. Measured, with a real content change rather than a `touch`:

```
$ sed -i 's/^  en: "Merge"$/  en: "ZQXSENTINEL"/' crates/opencmdb-bin/locales/app.yml
$ cargo build --workspace --locked
    Finished `dev` profile in 0.08s              # NO "Compiling" line
$ strings target/debug/opencmdb | grep -c ZQXSENTINEL
0                                                # the BINARY never saw it
$ cargo test --workspace --locked --no-run
   Compiling opencmdb-bin v0.1.1
$ strings target/debug/deps/opencmdb-<hash> | grep -c ZQXSENTINEL
3                                                # the TEST binary DID
```

🔑 **And the cause is already in the tree**: `screens.rs:654` and `page.rs:3823` both
`include_str!("../locales/app.yml")` inside `#[cfg(test)]` modules, and rustc records `include_str!`
targets in dep-info, which is cargo's fingerprint input. The test target therefore has a dependency
edge on the file **that the binary does not have** — visible in `target/debug/deps/opencmdb-*.d` and
absent from `target/debug/opencmdb.d`. The crate recompiles, `rust_i18n::i18n!` re-expands, and the
new value reaches the test binary. ⚠️ **The edge is incidental**: it exists because two guards happen
to read the file, not because anything declares the dependency. **Delete both `include_str!`s and the
edge vanishes — measured, not argued**: with the two replaced by runtime reads,
`grep -l app.yml target/debug/deps/*.d` returns **nothing** and the same mutation goes **GREEN with no
`Compiling` line**. ⚠️ And the two lines in question are `screens.rs:654` and `page.rs:3823` — *both
inside guards this story's own T5 and T6 edit.*

**So what is actually true**: `rust_i18n::i18n!("locales", …)` (`main.rs:42`) reads the file through a
proc macro that registers no Cargo dependency, and **there is no `build.rs` in
`crates/opencmdb-bin/`** (verified by `find`). The hazard bites **the produced binary** — a browser
look, a `strings` check, a real boot, a CI job that only builds — and **not `cargo test`**.

⚠️ **Two consequences, both live, and both land on THIS story.** (1) CI restores a cached `target/`
(`ci.yml:46-48` uses `actions-rust-lang/setup-rust-toolchain@v1`, whose own comment says it *"sets up
the build cache"*), so a translation-only PR can be validated against the OLD strings — **and this
story's PR is exactly a translation-heavy PR.** (2) 🔴 **The rows the hazard really bites are M0's
`strings` receipt and T9's browser pass**, which run against `target/debug/opencmdb`. The
assertion-carrying rows (M1, M2, M3, M5) measure what they claim on today's tree.

🔑 **The transferable half is not the hazard; it is how the false sentence got here.** The
over-generalisation was inherited **verbatim** from `deferred-work.md`'s own row — *"any mutation that
edits `app.yml` alone and does not touch a `.rs` file measures nothing"* — which was measured on
`cargo build --locked --bin opencmdb` and then stated about everything. *A register row carries the
measurement's conclusion and not its command, so the next reader inherits the scope the measurer
happened to use.* The register row is corrected in the same push.

**The closure is a `build.rs` emitting `cargo::rerun-if-changed=locales/app.yml`** — the standard
mechanism, no dependency, a few lines. ⚠️ **And the measurement of the fix must be the STRING, not
the rebuild.** A rebuild happening proves the trigger fired; it does not prove the new value reached
the binary. The house has already been burned by exactly this distance twice — 6b.4b's *"a screenshot
of a stale binary is not a look at your code"*, and 6b.5's *"a key can be in `app.yml` and absent
from the BINARY, and `every_key_carries_both_locales` cannot tell — it reads the file, `t!()` reads
the embedded copy."*

🔴 **AND THE INSTRUMENT THE DRAFT PRESCRIBED FOR THAT PROOF IS WRONG IN THE DIRECTION THAT MATTERS.**
It said, in bold, *"prove it with `strings target/debug/opencmdb | grep`"*. Measured:

```
$ strings target/debug/opencmdb | grep -c "Rafraîchir"   →  0
$ grep -ac "Rafraîchir" target/debug/opencmdb            →  1
```

The string **is** in the binary. GNU `strings` looks for runs of printable **single-byte** characters,
so a UTF-8 multibyte character terminates the run and the value is never emitted (`-e S` gives 0 too).
Over the file: **163 of 284 `fr:` values — 57% — carry a non-ASCII character**, and 28 `en:` values do.
🔑 **A developer following T1 on any accented value would have seen *absent before, absent after* and
concluded the fix had failed** — an instrument that cannot confirm presence, prescribed for a proof of
presence, in the story that spent §0e explaining why the rebuild is not the receipt. **Use
`grep -a <needle> target/debug/opencmdb`, never `strings | grep`.**

🔑 **And the hazard's second half is a claim about the PAST that this story is positioned to settle
and must not overstate — and it is now SMALLER than the register row says.** The row says *"every
mutation pass in this epic that claimed to mutate a translation must be re-read with that in mind"*;
since `cargo test` did see those changes, an assertion-carried translation mutation in **nine**
merged stories (6b.2–6b.9, 6b.4b included — not eight) measured what it claimed. What did not is any
row whose oracle was a **rendered page from the built binary** or a browser look. Re-reading them is
Epic 6b's retrospective's work, not this story's; **correcting the row's scope is this story's**,
because this story is what measured it.

### §0f. THE FLOORS THAT NO LONGER MEASURE ANYTHING — and the question the register refuses to answer

`screens.rs:686-690`:

```rust
assert!(
    checked >= 47,
    "the premise: 48 entries minus `_version` ({checked} scanned) — a scan that found \
     nothing would assert nothing"
);
```

**`app.yml` holds 284 keys.** The floor is six times below what is there, and its message states a
figure — *"48 entries"* — that is **false today and was TRUE when it was written**: at `d8cc438`,
story 6b.2's own merge, the file held exactly 48 top-level entries (`git show`). It went stale one
story later, at 6b.3's 64. ⚠️ The draft said *"false since story 6b.2"* and blamed the story that got
it right — **a one-story misattribution of when a figure went stale, in the story about figures that
went stale.** *A number is not wrong because it is old; it is wrong when what it counts moved.* Story 6b.9 added 63 keys and left it
deliberately, so the drift would be visible rather than quietly papered over.

⚠️ **The register row assigns this to 6b.10 and names the decision rather than taking it**: *"whoever
fixes it must decide what the floor is FOR — a premise check that the file was read at all, or a
count that must track the file — because the two want different numbers."* → **Arbitration 3, §0h.**

🔑 **There is a third answer neither option names**: make the guard's oracle a **second, independent
parse of the same file** — `checked` must equal the count of `  en:` lines — so the number is
*derived* rather than *believed*, and a low constant floor stays only as the *"the scan matched
something"* premise. That is `fixtures.rs`'s `expected()` idiom and `BINDING_STATE_AXIS`'s, both of
which `CLAUDE.md` protects by name.

🔴 **ITS COST WAS STATED ABSTRACTLY AND THE VALIDATION BUILT IT AND MEASURED IT.** *"Two parses of one
file can be wrong the same way"* now has three witnesses, and they are worse than the sentence:

| attack on option (c) | result |
|---|---|
| delete one `fr:` line | **RED** ✅ |
| delete one `en:` line | **RED** ✅ |
| **delete a whole key block** | **GREEN** — both parses agree the key is gone |
| **truncate the file at a clean key boundary** | **GREEN** |
| **nest the key and remove its `fr` half** | **GREEN** |

🔴 **THE LAST ROW IS THE SHARPEST FINDING OF THE WHOLE VALIDATION, and it is reachable through one
ordinary valid idiom.** YAML nesting resolves identically —

```yaml
gesture:
  badge:
    en: "Not yet"
    fr: "À venir"
```

— `t!("gesture.badge")` returns the same value, measured. Nest a key, remove its `fr` half, and the
result is **471 + 161 + 70 tests green, all nine gates green**, while
`t!("gesture.badge", locale = "fr")` renders **`"Not yet"`** — *English in the French UI, which is
literally the defect AC1 exists to prevent.* Both parses are line-shape based, so **both agree the key
does not exist**, and a key that is not in the population is not missing a locale.

⚠️ **The existing guard does red on the nested shape — for the wrong reason** (`gesture carries []`, an
unrelated false positive). Apply the ordinary repair that message invites — give the parent its own
pair — and everything goes green. *A guard that fails for the wrong reason is worth nothing* (story
5.14b's sentence), and here it is worse than nothing: it teaches the developer a repair that opens the
hole.

🔑 **So arbitration 3 is no longer a choice between (a), (b) and (c): none of the three closes the
class, and the option that does is a FOURTH.** Whatever the floor is for, the guard's **parse** must be
the same one carrier 1 uses — a real YAML parse over the resolved key set — or the two carriers
disagree about what a key even is. → **arbitration 3 is re-put below with option (d).**

⚠️ **And one more thing nothing guards**: `nav.dashboard: en: ""` leaves **471 + 161 + 70 green and
nine gates green**. §0a's *"no empty value on either side"* is a **measurement of today's file, not a
property** — say it that way, and decide whether the guard should carry it.

**Sister floors this story is standing next to** — do not sweep them silently, and do not leave one
you touched stale:

- `example_screens.rs` `every_literal_key_in_the_view_code_resolves` — `checked >= 60`.
- `page.rs` `no_gesture_copy_names_the_story_that_will_build_it` — `keys.len() >= 8`, and its
  `assert_eq!(checked, keys.len() * 2)` is the good shape: **derived, not believed.**
- 6b.7's own sentence, quoted here because this story is where it bites: **a floor is only a guard
  while it equals what is there.**

### §0g. WHAT THIS STORY DOES **NOT** CLOSE, AND SAYS SO

- 🔴 **The word guard on `/diagnostic` is an enumeration and a paraphrase slips it.** Register row,
  owner **this story** (6b.9's section): a translation value reading *"none stored — this deployment
  cannot be breached by a remote attacker"* left 696/696 green and reached the served page.
  ⚠️ **The register row's assignment and this story's ability are not the same thing.** The first line
  of that AC is the **shape** (`security_rows` takes a `SecurityPosture` and nothing else, so a claim
  cannot be *typed* into the row builder); the enumeration is the second line, against a translation
  value. What this story can do is **widen the enumeration once, deliberately not to exhaustion, and
  say so** — story 5.12's sentence, fourth application in this epic: *an enumeration cannot claim the
  completeness of a property.* The real closure is a schema for security rows that has no free-text
  arm at all, which is not this story's. **Say which of the two shipped.**
  ⚠️ **And do not carry a count with that sentence.** The draft wrote *"fourth application in this
  epic"*; the tree's own counters disagree — `deferred-work.md:4562`, `6b-9…md:247` and
  `diagnostic.rs:34` all say *third*, while `example_screens.rs:968` says *fourth*. 🔑 *A tally
  maintained by hand across four files is a number nobody can check*, which is the very class this
  epic keeps catching. Cite the rule; do not number it.
- **The strings that are not screen copy stay as they are, and the boundary is stated**: connector
  and boot refusals (`arp_ping.rs:84, :86, :88`, `dburl.rs:44-68`), the write route's response bodies
  (`document.rs:39` `CSRF_REFUSED_BODY`, `:205-208` the `CREATED` line, `:230`, `:284`, plus the
  404/409/422 bodies from `refusal.to_string()` at `:215, :219, :223` — ⚠️ the draft cited five line
  numbers and **not one of them carried a body**; three were comments and one a closing brace), and
  `expect()` / log text. AC1 says *"every string the ten screens introduce"*. → **Arbitration 2, §0h.**
- 🔴 **And the boundary has a class the draft missed entirely, which arbitration 2(a)'s *"and nothing
  else"* asserted a measurement nobody had taken**: the **500 bodies served at the ten screens' own
  addresses** — `page.rs:1196` (`"internal error"`), `:1417`, `:1611`, `:1654`, `:1701` and
  `diagnostic.rs:725` (all `"template error"`), beside the render-error fallback at `:105-106`. These
  are English strings the operator reads **at `/triage`, `/dashboard`, `/sources`** — `CLAUDE.md`'s
  6b.9 row records exactly that (*"still answer a bare `500 internal error` when the pool is down"*).
  Whether they are in or out is a real fork; **it is now arbitration 2's third option instead of a
  clause claiming there is nothing else.**
- **THREE gestures in `app.yml` have NO glossary row, not one**: `gesture.resolve` (« Résoudre »),
  and 6b.9's `gesture.check_now` (`app.yml:899`) and `gesture.export_log` (`:902`). ⚠️ The draft named
  only the first, **under a heading that reads as exhaustive** — which understates what the
  retrospective inherits, and is the same shape as §0a's missing table row. `baseline` is a fourth,
  already registered to Epic 9. Story 6b.7's precedent governs all of them: **extending a binding
  table is a planning act and Guy's**, refused there as *premature, not wrong*. Register the set; do
  not add a row.
- ⚠️ **And the triage queue's `kindLabel` axis is a fourth list nobody has decided about**:
  `triage.kind.absence` and `.nouveau` (`app.yml:240`, `:246`) are outside every binding table, which
  `CLAUDE.md`'s 6b.6 row already registers. AC2's *"the glossary uniqueness test cover them"* does not
  say whether *them* reaches that axis. **This story does not decide it and says so** — reconciling
  `Nouveau` with `undeclared` rewrites shipped copy on a story that does not own it, which is the PRD's
  own sentence at `:1031-1033`.
- ⚠️ **The two binding tables DISAGREE**: `ux-design-specification.md:1341-1351` carries **eleven**
  gesture rows, `prd.md:993-1002` carries **ten** — `attach`/« rattacher » is missing from the PRD's.
  Both are called binding. A gate transcribing "the glossary" must name **which file** it transcribed
  and the divergence must be registered, not silently resolved by picking one. **Owner: Epic 6b's
  retrospective.**
- **The three French proper nouns in the example dataset render French under the English UI** and
  that is Guy's decision of 2026-08-20, not a defect — but it is worth one sentence in AC5's report,
  because it is what an English-speaking reader will notice first and mistake for one.
- **`OPENCMDB_LOCALE` is read raw at `main.rs:373`, outside `AppConfig`, and an unrecognised value is
  accepted in silence.** → **Arbitration 4, §0h.**
- **NFR26 is not "closed" by this story** any more than NFR5 was closed by 5.12. Two locales exist
  and are gated; a third language, locale negotiation, per-user locale and pluralisation beyond
  `%{n}` are none of them here.

### §0h. THE ARBITRATIONS — ALL FIVE TAKEN BY GUY 2026-08-21, each with the option refused

🔑 **Guy took the recommendation in all five, and in four of them the recommendation had been CHANGED
by the validation** — arbitration 1's cost, arbitration 2's option set, arbitration 3's winning option
and arbitration 4's premise were each wrong in the first draft and each corrected by a measurement.
*The arbitration that was put to him is not the arbitration the story first wrote, and that difference
is what the validation bought.* Arbitration 5 is the one the UX spec demanded be **taken rather than
defaulted**, and it was put for that reason alone.

**Arbitration 1 — `gesture.merge`'s English half (§0c). ✅ TAKEN: (a)** (Guy, 2026-08-21).
- **(a)** EN becomes **"Document"**; the key is renamed **`gesture.document`**. ⚠️ **Blast radius
  measured by doing it**: the three literal sites are `page.rs:909, :954, :1002` — **`:687` is
  `fn action_bar` and contains no `gesture.merge` at all**, so the draft's list was one site wrong —
  and **exactly ONE test breaks**, `page.rs:3897`, whose assertion reads
  `assert_eq!(…label, "Merge", "a drift offers Merge")`. 🔑 *That message carries `drift`, the synonym
  story 6b.6 retired* — so the one test standing in the way of retiring `merge` is a test that still
  says `drift`. Closes the violation on both columns the glossary binds
  (the UI string and the code identifier), and the retired word can then be added to the gate's
  denylist without the gate reddening on the committed tree. ⚠️ **That last clause is true only under
  §0d's stated narrowing** — the gate reads `app.yml` and nothing else. Scoped to `crates/` as the
  glossary column literally reads, it would red in ~135 places on the committed tree, 87 of them in
  sha256-locked fixtures. The validation measured it; the narrowing is not optional.
- **(b)** EN becomes "Document", key stays `gesture.merge`. Cheaper; leaves a retired term in the
  column the glossary's own heading calls *"EN (docs, API, **code**)"*, and the gate must then carry
  an exemption for the one identifier it most wants to catch.
- **(c)** Leave it. Refused on the record: the English UI would go on labelling its primary control
  with the word the founding pillar forbids, and the gate this story ships would have to be written
  blind to it.

**Arbitration 2 — the perimeter of "every string the ten screens introduce" (§0g). ✅ TAKEN: (a′)** —
the ten screens' rendered surface **plus the six `500` / `template error` bodies served at those same
addresses** (Guy, 2026-08-21). 🔑 *An operator whose store is down must not be the one person who reads
English on a French deployment.* ⚠️ (a′) did not exist when the arbitration was first drafted; the
validation found the class and `CLAUDE.md`'s own 6b.9 row had already recorded the symptom.
- **(a)** The **rendered surface of the ten screens under a successful render**, text nodes and
  human-text attributes alike — which adds `_gap_card.html`'s `aria-label` and, if the arbitration
  admits the eleventh address, nothing else. Boot refusals, logs and `expect()` stay English; the
  write route's bodies are registered to story 6.4, which is what makes them visible.
- **(a′)** The same, **plus the six `500`/`template error` bodies served at those same addresses**
  (§0g). 🔑 *An operator whose store is down reads English on a French deployment, at the URL the
  product told them to use* — and that is the one path where the interface language silently stops
  being the interface language. Cheap: six strings, two keys. ⚠️ Its cost is real and belongs in the
  decision: a translated 500 body is one more thing that can fail while something is already failing,
  and `t!()` on a dead-store path is a call the fallback must survive.
- **(b)** Add `document.rs`'s response bodies. They are operator-readable, but **no template calls
  that route today** (`CLAUDE.md`: *"`POST /document-all` is called by no template in the product"*),
  so translating them now is copy nobody can reach, written against a gesture whose shape 6.4 may
  change.
- **(c)** Everything a human could read, logs included. Refused: it makes the story unbounded and
  puts French in a log file an operator greps in English.

**Arbitration 3 — what the floor is FOR (§0f). ✅ TAKEN: (d)** (Guy, 2026-08-21) — derived over a
**real YAML parse**, on the resolved key set, with the completeness assertion. ⚠️ (c) was the draft's
recommendation and was **measured insufficient** before it could be taken; (d) did not exist until the
nested-key witness did.
- **(a)** A low premise constant. Honest about its job, never rots — and never notices a parse that
  silently stops seeing two thirds of the file.
- **(b)** An exact count. Notices everything, and turns every story that adds a key into a story that
  edits a number in a message reading *update this number* — which story 6b.6's review measured a
  developer **follows**, leaving the suite green over a real defect.
- **(c)** **Derived**: `checked` must equal a second, independent parse of the same file, plus a low
  constant as the *"the scan matched something"* premise. Both properties, no number to maintain.
  🔴 **Measured insufficient** (§0f): a deleted key block, a truncated file and a **nested key with its
  `fr` half removed** all leave it GREEN — the last one rendering English in the French UI with 702
  tests and nine gates green.
- **(d)** ⭐ **Recommended after the validation: (c)'s derivation over a REAL YAML parse** — the same
  parse carrier 1 uses — asserting over the **resolved key set** rather than over line shapes, plus the
  completeness assertion §0d prescribes (`entries == 2 × keys + 1`). It is the only option measured to
  catch the nested shape, and it makes the two carriers agree about what a key is. ⚠️ **It does not
  catch a deleted key block either** — nothing here can, since a key that is gone is gone from both
  sides — and that class is registered rather than claimed.

**Arbitration 4 — does `OPENCMDB_LOCALE` move into `AppConfig` and get refused by name? (§0g).
✅ TAKEN: YES** (Guy, 2026-08-21) — into `AppConfig`, **refused by name at boot**, and the refusal
**accepts what `rust-i18n` accepts, measured** (`fr-CH` works and must keep working), never an
enumerated list. 🔴 **This is a BREAKING change**: a deployment carrying `OPENCMDB_LOCALE=FR` stops
booting. It owes story 6b.12 a release-note line, and `README.md:124` and
`docker/README.dockerhub.md:85` describe the old behaviour.
- **Yes.** Two precedents point the same way: story 6.1 refuses malformed configuration **by name at
  boot**, and story 6b.9 moved `OPENCMDB_METRICS_TOKEN` into `AppConfig` and thereby **removed** a
  reader rather than adding one — 6b.2's shipped M12 (*two readers of one variable, two blank-value
  behaviours*) is the defect both were avoiding. Today `OPENCMDB_LOCALE=FR`, `=fr-CH` or `=""` is
  accepted in silence and the operator gets English with no explanation, on the one configuration
  knob this story owns.
- **No.** It is a locale, not a credential; a wrong value degrades to the fallback rather than to an
  insecure state, and moving it grows a story that is already touching the gates.
- 🔴 **THE DRAFT'S EXAMPLE LIST WAS WRONG AND A REFUSAL BUILT FROM IT WOULD REGRESS THE PRODUCT.**
  Measured: `OPENCMDB_LOCALE=fr-CH` renders **« Tableau de bord »** — `rust-i18n` strips the region and
  it **works today**. `FR`, `fr_CH`, `""`, `zz` and `en-US` fall back to English. The draft named
  `fr-CH` as a silent-failure example; it is the **one region-qualified form that succeeds**, so a
  refuse-by-name written against `available_locales!()` would **reject a working configuration**.
  🔑 *An arbitration argued from an unmeasured example list decides the wrong thing precisely where
  the example was wrong.* If (yes) is taken, the refusal must accept what `rust-i18n` accepts —
  measured, not enumerated.
- ⚠️ **Whichever way**: `rust_i18n::set_locale` is **process-wide**, and two sites record it as a
  hazard in tests — `page.rs:2016`, *"it is process-wide, so a test that calls it makes the suite
  order-dependent"*, and `page.rs:3278`, where story 6b.6's validation found it had been called.
  ⚠️ The draft put a sentence of its own — *"a test that calls it corrupts its neighbours"* — in
  quotation marks against those two citations; **it appears nowhere in the repository**, and the
  validation's `grep` is what found that. The substance survives; the quotation did not. Every new
  locale assertion uses the **per-call `locale =` override**, never `set_locale`.

**Arbitration 5 — is LOCALE a second snapshot axis? (asked by the UX spec, by name). ✅ TAKEN: (b)**
(Guy, 2026-08-21) — **left to review, and the review is written down.** ⚠️ The spec forbids letting
this one default, so it was put and taken rather than assumed. 🔑 **And §0b's inversion is what makes
(b) the right answer rather than merely the affordable one**: the French render has no automated
reading at all, so what that axis is short of is not a visual baseline — it is a **locale-parameterised
render helper** (T5). *A snapshot of a screen no test can render in French would be a baseline of the
language nobody checks.*

`ux-design-specification.md:1440-1442` hands this decision to whoever implements it, in so many
words: *"visual snapshots run per THEME today, and a bilingual GUI makes locale a second axis.
Whether that axis is snapshotted (doubling the baselines) or left to review is an implementation
arbitration, and it is named here **so that it is taken rather than defaulted**."*
- **(a)** Snapshot both locales. Doubles the baselines, and the product has **no visual-snapshot
  harness at all today** — this would be building one, which is not this story.
- **(b)** **Left to review, and the review is AC5's browser pass, written down.** Matches what the
  tree can actually do, and the spec's own *safe is not verified* is discharged by looking rather
  than by a baseline that nothing generates.
- **(c)** Defer to Epic 22 (*"first-light soigné & bilingue complet"*). ⚠️ Refused as a **default**
  rather than as an option: the spec's `:1434-1436` says in so many words that Epic 22 *"does not
  license shipping a single-language screen before it"* — it owns completeness and the **in-UI
  selector**, which this story therefore does not build.
- **Whichever way, it is TAKEN and recorded**, which is what the spec asks for.

### §0i. THE VALIDATION'S OWN RECORD — recountable rather than believed

**Fact-check layer** (fresh context, read-only, `master` @ `97c0e9a`), 2026-08-21. Every finding
below is applied ABOVE, in the section it belongs to, rather than listed here and left there.

| # | What the draft said | What is true | Where it is fixed |
|---|---|---|---|
| H1 | *"any mutation editing `app.yml` alone measures nothing"* | **False for `cargo test`** — two `include_str!`s in `#[cfg(test)]` modules give the test target a dep-info edge the binary lacks | §0e, T8, M0, M1 |
| H2 | *"Four Microcopy Rules"*, cited `:1445-1449` | **Five**, at `:1446-1451`; the range held three; **rule 5 was missing** | §0b, T9 |
| H3 | the retirement at `ux…:1352` | `:1352` is a **blank line**; it is at `:1386`, and the quotation is the **PRD's alone** | §0c |
| H4 | a table of *"key counts at each merge commit"* | **6b.7 (`f32c268`, 184 keys) was missing**; *"nine stories"* is ten story files; 6b.8 added 37, not 68 | §0a |
| H5 | *"a test that calls it corrupts its neighbours"*, in quotation marks | **Appears nowhere in the repository** — the story wrote it and attributed it | §0h arb. 4 |
| H6 | arb. 1(a) costs one rename | Only under a narrowing the draft never stated: scoped to `crates/`, the gate reds **~135 sites**, 87 in **sha256-locked fixtures** | §0d, §0h arb. 1 |
| M1 | *"measured by grepping the story files"*, eight citations | The grep reaches **six files**; three named carry the variable zero times, one it does reach was omitted | §0b |
| M3 | *"both defects on the English side"* | **One** of three is; the other two leaked English into FRENCH and were found **by the French look** | §0b |
| M4 | five `document.rs` body citations | **Not one carried a body** — three comments, one closing brace | §0g |
| M5 | *"false since story 6b.2"* | **True when 6b.2 wrote it** (48 entries exactly); stale since 6b.3 | §0f |
| M6 | *"6b.6 discovered this the hard way"* | 6b.6 says it **avoided rather than discovered**; the discovery was story 5.9's validation | T2 |
| M8 | arb. 2(a) *"and nothing else"* | Six **`500`/`template error`** bodies are served at the ten screens' own addresses | §0g, arb. 2(a′) |
| M9 | one glossary-less gesture | **Three**, plus `baseline`, plus the `kindLabel` axis | §0g |
| L1–L9 | — | the column heading, a third template literal, ten off-by-one citations, the 2.5× multiple (**≈3.7×**), the AC's date (**2026-08-14**), *"eight stories"* (**nine**), two different *"eleventh addresses"*, an uncheckable *"fourth application"*, *"twice"* (**four times**) | throughout |

🔑 **Six HIGH findings and every one of them was a sentence of mine, not a design fault** — the shape
the blind review layer has produced for four stories running, arriving one step earlier this time
because the story was checked before it was built. ⚠️ **And the two sharpest are the same defect
twice: an enumeration written from what I remembered rather than from what I ran** — the commit table
that skipped 6b.7, and the Microcopy checklist that dropped rule 5. *Story 5.13b's sentence, met
again: an enumeration cannot establish absence, and it cannot establish completeness either.*

**Carried forward as SUSPICION, never as fact** — the fact-check layer labelled these unmeasured and
they must not be ticked without measurement:

1. M10's prediction (a variable-held nonexistent key: green on the literal guard, red on the render
   guard) — read in both guards, never planted.
2. Arbitration 4's premise, that `OPENCMDB_LOCALE=FR`, `=fr-CH` and `=""` all degrade silently to
   English — structurally sound, **not executed**.
3. That the `aria-label` is *"invisible to every existing guard"* — every template-reading guard was
   read and none inspects attribute text, but no mutation was planted. **A reading, not a
   measurement**, and this project's own rule says which of the two counts.

**Gap-hunt layer** (own worktree, base `97c0e9a`; built the ninth gate **twice** — naive and real
parse — plus carrier 2, the `build.rs` and arbitration 1(a)'s rename, and ran mutations against all of
them). Baseline: **697 tests**, eight gates green.

| # | What the draft said or left open | What it measured | Where it is fixed |
|---|---|---|---|
| H1 | *"measures nothing"* | Independently reached the same refutation, **and added the control**: replace the two `include_str!`s and the same mutation goes GREEN | §0e |
| H2 | *"prove it with `strings \| grep`"* | 🔴 **The instrument cannot see 163 of 284 `fr:` values** — GNU `strings` breaks on any multibyte char. Prescribed for a proof of presence, blind to presence | §0e, T1, M0 |
| H3 | *"handle the shapes, or state the limit"* | **Not a trade-off**: naive misses **7 of 12** shapes; `yaml-rust2` is **already in the lock**, one line | §0d, T4 |
| H4 | (nothing) | 🔴 **Nested YAML resolves identically and defeats every parse**: nest a key, drop its `fr` half → **702 tests + nine gates GREEN** with English rendering in French | §0f, arb. 3(d), M4″ |
| H5 | *"the English side has none"* | 🔴 **Inverts it.** 241 `t!(` sites, **zero** `locale =` overrides — English has ~470 assertions and 0 looks; **French has 9 looks and 0 assertions** | §0b, T5 |
| H6 | M4 separates the carriers | **Refuted** — with a real parse it reds both. The separator is the **fallback** (M4′) | §0d, M4/M4′ |
| M1 | M5's prediction | `contains_word` counts `_`: **`gesture.merge_all` is GREEN**, and 107 of 284 keys use `_` | §0d, M5 |
| M2 | (nothing) | A hand-rolled walker **silently dropped a key** and the gate read ✅ — prescribe a completeness assertion | §0d |
| M3 | *"no empty value on either side"* | An empty value leaves everything green: a **measurement of today's file, not a guard** | §0f |
| M4 | *"touches `page.rs:687, 909, 954, 1002`"* | `:687` holds **no** `gesture.merge`; **exactly one** test breaks — and its message still says `drift` | arb. 1(a) |
| M5 | `fr-CH` degrades silently | 🔴 **`fr-CH` WORKS** (region stripped). A refusal built from the draft's list would reject the one qualified form that succeeds | arb. 4 |
| M6 | arb. 3(c) | Deleted key block, truncated file, nested key → all **GREEN** | §0f, arb. 3 |
| M7 | T10's document list | `README.md:124` and `docker/README.dockerhub.md:85` document `OPENCMDB_LOCALE`; and the story said *release note* / *6b.12* / *breaking* **zero times** | T10 |
| L1–L4 | — | `merge` has 310 legitimate whole-word uses in `crates/`; clippy needs a type alias; the gate costs **5** lines (1908 → 1913) | §0d, T4 |

✅ **REFUTED BY MEASUREMENT — do not re-chase these.** Single-quoted values and escaped quotes are
**caught** by the naive parse (the draft grouped them with block scalars). The template sweep is
**right**: all 17 templates re-scanned for `aria-label|title|alt|placeholder|summary|abbr|label`
literals — exactly one hit, `_gap_card.html:1`. There is **no `<svg>` anywhere**, so no SVG
`<title>`/`<desc>` channel; the one `<title>` interpolates. `assets/app.js` carries **no user-visible
text**; `assets/app.css` has **no `content:` string** (its three hits are `justify-content`). And story
5.12's offset→line class **does not recur** — `Marker::line()` located line 329 correctly with 45 `é`
immediately above it.

⚠️ **WHAT THE GAP-HUNT COULD NOT REACH, stated as absence and not as coverage**: no database (every
DB-backed test returned early, bin suite 0.15 s — nothing through the store, no route-level partition
test); **no browser — AC5/T9's English look was NOT performed**, and it measured the *testability* of
the locale axis, never the *quality* of the English copy; M0, M6, M8, M9 and M11 not run; carrier 2 as
built lints the `en` column only, and §0g's eleven-rows-versus-ten divergence between the two binding
tables **is still unpicked**.

🔑 **What the two layers together change about this story.** The draft's design was *two carriers,
separated by YAML syntax*. Both halves of that were wrong: the syntax question has a one-line answer,
and what separates the carriers is the **fallback**. And the draft's framing was *the English side is
unread*. It is unread by humans and read by ~470 assertions; the **French** side is the one nothing
asserts. **The story is the same story and both of its arguments have been replaced by better ones** —
which is the whole reason this project makes validation mandatory.

## Tasks / Subtasks

- [x] **T0 — ALL FIVE arbitrations TAKEN by Guy, 2026-08-21 (AC: 1, 2, 5, 6)**
  - [x] (1) **(a)** — EN `"Document"`, key renamed `gesture.document`.
  - [x] (2) **(a′)** — the ten screens' rendered surface **plus the six `500` / `template error`
        bodies** served at those same addresses.
  - [x] (3) **(d)** — the floor derived over a **real YAML parse**, on the resolved key set, plus the
        completeness assertion.
  - [x] (4) **YES** — `OPENCMDB_LOCALE` into `AppConfig`, refused **by name** at boot, the refusal
        accepting **what `rust-i18n` accepts, measured** (`fr-CH` works). 🔴 **Breaking.**
  - [x] (5) **(b)** — the locale axis is **left to review**, and AC5's browser pass IS that review,
        written down. The UX spec forbids defaulting this one; it was put and taken.
  - [x] Each recorded in §0 with the option refused and what refusing it costs.
- [ ] **T1 — Close the build hazard FIRST, before any other measurement (AC: 4)**
  - [ ] `crates/opencmdb-bin/build.rs` emitting `cargo::rerun-if-changed=locales/app.yml`.
  - [ ] **Measure the STRING, not the rebuild**: change one value, `cargo build`, then
        **`grep -a <needle> target/debug/opencmdb`** for the new and the old value. Record both runs.
        🔴 **NOT `strings | grep`** — the validation measured it blind to **163 of 284 `fr:` values**,
        because GNU `strings` breaks its run on any multibyte character (§0e).
  - [ ] Record which of this story's own mutation rows the fix makes meaningful. Do **not** re-audit
        the eight earlier stories — that is Epic 6b's retrospective, and the row says so.
- [ ] **T1b — Arbitration 4: `OPENCMDB_LOCALE` into `AppConfig`, refused by name (AC: 1)**
  - [ ] The refusal **accepts what `rust-i18n` accepts, measured** — `fr-CH` renders
        « Tableau de bord » today and must keep doing so. **Do not enumerate `available_locales!()`**;
        that rejects the one region-qualified form that works (M5).
  - [ ] It **removes a reader** rather than adding one (6b.9's precedent), and no new test mutates an
        env var (6.1's rule).
  - [ ] 🔴 **Breaking**: `OPENCMDB_LOCALE=FR` stops a deployment booting. Register the release-note
        line with 6b.12 **now**, and update `README.md:124` and `docker/README.dockerhub.md:85` in
        this push.

- [ ] **T2 — Stand the bench up (AC: 5)**
  - [ ] `mariadb:10.11` on a port that is **not 3306** (another project's container holds it).
        ⚠️ 6b.6 **avoided it rather than discovering it**, and says so in its own text; the hard-way
        discovery was **story 5.9's validation**, which caught the trap *"before it could migrate
        another project's database"*. Apply migrations, export `DATABASE_URL`.
  - [ ] Baseline the suite **both ways**: without `DATABASE_URL` and with. The **clock** is the tell
        that the database-backed tests genuinely executed. Measured at `97c0e9a` by the validation:
        **697 tests** — 470 bin + 161 core + 66 xtask — eight gates green, bin suite **0.15 s** with
        no database.
- [ ] **T3 — The sweep, re-run and widened (AC: 1)**
  - [ ] Re-run §0c's four passes on the tree you are working on, not on the numbers in this file.
  - [ ] Fix `_gap_card.html:1`'s `aria-label` → a key, both locales — **if arbitration 2 admits it**
        (`/gap` is the eleventh address, not one of the ten screens; §0c states the scope).
  - [ ] **Arbitration 2(a′): the six `500` / `template error` bodies become keys** —
        `page.rs:1196`, `:1417`, `:1611`, `:1654`, `:1701`, `diagnostic.rs:725`, beside the
        render-error fallback at `:105-106`. ⚠️ **A `t!()` on a dead-store path is a call the fallback
        must survive**: the store being down is exactly when the process is least healthy, so resolve
        the key **before** the failing work where the shape allows it, and keep the last-resort
        fallback (`page.rs:105-106`) a plain literal — a render error is not the moment to depend on
        the renderer.
  - [ ] Whatever else the re-run finds. ⚠️ **A finding this file does not list is not a reason to
        doubt the re-run; it is the re-run working.**
- [ ] **T4 — Carrier 1: the ninth gate (AC: 2)**
  - [ ] `xtask/src/copy_vocabulary.rs` — **not** `main.rs` (1908/2000 code lines, §0d).
  - [ ] **Arbitration 1(a) lands FIRST** — the gate reds on the committed tree until it does
        (measured: exactly two findings, `app.yml:325` the key name and `:326` the `en` value).
  - [ ] **Per-locale** denylist, which no existing gate shape has: `merge` forbidden in `en`,
        « Merger » binding in `fr`. Key **names** in scope too — with a matcher whose boundary set
        includes `_` and `.`, or `gesture.merge_all` passes (M1).
  - [ ] 🔴 **A REAL YAML parse — `yaml-rust2`, already in `Cargo.lock` via `config`, one line to add.**
        The *"state the limit"* branch is **withdrawn**: the naive parse misses 7 of 12 legal shapes
        and removing that limit costs one already-vendored crate (§0d).
  - [ ] **A COMPLETENESS assertion** (`entries == 2 × keys + 1`) — the validation's own walker
        silently dropped a key and the gate then read ✅ (§0d).
  - [ ] ⚠️ Two things the validation hit while building it: wiring the gate costs **5 lines** in
        `main.rs` (1908 → 1913, 87 of headroom left — the inline gate would have breached 2000, so
        §0d's conclusion holds), and clippy `-D warnings` **rejects** the natural
        `Result<(Vec<Entry>, Vec<(String, usize)>)>` return as *"very complex type"* — it needs a type
        alias.
  - [ ] Add the gate's row to `main.rs`'s module-doc list **in the same edit**, and correct the
        *"Eight gates"* sentence. The file's own doc says missing that is the defect story 5.12
        caught.
  - [ ] Located verdicts, both directions: probes that must red **and** probes that must stay green,
        each pinned with the file and the line the gate names. Story 5.12's *"a pinned boolean proves
        THAT a gate fires and never WHERE."*
- [ ] **T5 — Carrier 2: the glossary uniqueness test over RESOLVED values (AC: 2)**
  - [ ] The **gesture axis** transcribed as a constant beside `BINDING_STATE_AXIS`, in
        `state_vocabulary.rs` or its own module — **transcribed, not derived**, and it must name
        **which** of the two disagreeing tables it transcribed (§0g).
  - [ ] Key names from the file, values from `t!(key, locale = …)`, **both locales**, never
        `set_locale`.
  - [ ] 🔴 **And this is where §0b's second diagonal is closed or is not**: today **no test in this
        crate can render a screen in a chosen locale** (241 `t!(` sites, zero `locale =` overrides),
        so a French value regressing to its own key leaves 702 tests and nine gates green — measured.
        A locale-parameterised render helper is what makes the French half assertable at all. **If it
        is out of scope, say so and register it**; do not leave the measurement unmentioned.
  - [ ] Measure that neither carrier subsumes the other: a violation only carrier 1 catches (a key
        NAME), and one only carrier 2 catches (a value inside a block scalar).
- [ ] **T6 — The floors (AC: 6)**
  - [ ] `every_key_carries_both_locales` per **arbitration 3(d)**: the oracle is the SAME real YAML
        parse carrier 1 uses, over the **resolved key set**, plus the completeness assertion — **and
        its message rewritten**, since it states a false figure today.
  - [ ] **Prove it against the nested shape** (M4″), which is the reason (d) exists: nest a key, drop
        its `fr` half, and the guard must red where every other option leaves 702 tests green.
  - [ ] ⚠️ **A deleted key block is caught by NOTHING and (d) does not claim it** — register it.
  - [ ] Check the sister floors in §0f. Leave none you touched stale.
- [ ] **T7 — The `/diagnostic` word guard, widened once and NOT to exhaustion (AC: 2)**
  - [ ] Widen the enumeration; **state the limit in the doc**, do not imply completeness.
  - [ ] Re-run the register row's own paraphrase and record whether it now reds. If the shipped
        answer is *the enumeration only*, say so plainly.
- [ ] **T8 — Mutation pass; run every row (AC: 4, 6)** — the table below.
  - [ ] ⚠️ **Corrected by the validation (§0e): an `app.yml`-only mutation IS seen by `cargo test`**
        (two `include_str!`s in `#[cfg(test)]` modules give the test target a dep-info edge the binary
        does not have). What it is **not** seen by is anything reading `target/debug/opencmdb` — M0's
        `strings` receipt and T9's browser pass. **Run T1 first anyway**, because the edge is
        incidental and a refactor deleting either `include_str!` removes it silently.
  - [ ] The driver exits **non-zero when a mutation fails to apply** (6b.6), **touches restored
        files** (askama compiles templates into the binary — 6b.7), and **never mixes a scratchpad
        restore with `git checkout --`** (four occurrences, most recently in 6b.9's own review).
  - [ ] **Commit before every prove-to-red.**
- [ ] **T9 — LOOK AT THE ENGLISH UI, screen by screen, in a browser (AC: 5)**
  - [ ] `google-chrome` **151.0.7922.169** and `firefox` **154.0** are installed — re-measured
        2026-08-21 (6b.4b found them after four stories had deferred on an unmeasured assumption;
        6b.6 recorded Firefox at 153, so **re-run `--version` rather than quoting this line**).
  - [ ] **`OPENCMDB_LOCALE` unset or `=en`**, against a live database, on a **rebuilt** binary —
        `cargo test` builds the test target, not `target/debug/opencmdb` (6b.4b).
  - [ ] All ten routes, **plus `/devices/{id}` AND `/gap`** — ⚠️ the draft's T9 visited neither `/gap`
        nor its own §0c finding, while calling `/gap` *"the eleventh address"* and `/devices/{id}`
        something else. **Both are outside the ten; both get looked at.**
  - [ ] Read against **all five Microcopy Rules** (`ux-design-specification.md:1446-1451`), not
        against an impression, and **rule 5 — *empty ≠ failure, calm never alarming*** — is the one
        this interface most needs, nine of its ten screens being placeholder or example surfaces.
        Report what it found; **do not report what it covered.**
  - [ ] Then one French pass as a control, so a repair made for English is not measured only there.
  - [ ] 🔑 **Arbitration 5(b): this pass IS the locale coverage, so it is WRITTEN DOWN** — which
        screens, which locale, which build, what was read and what was found. The UX spec's *"safe is
        not verified"* is discharged by looking; a look nobody recorded discharges nothing.
- [ ] **T10 — Both runs, the gates, the documents (AC: all)**
  - [ ] `cargo fmt --all --check` on the **committed** tree (6b.1 shipped a tree where it was red),
        `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` **both
        ways**, `cargo xtask ci` — **nine** gates after T4.
  - [ ] **Derive the register rows this story OWES from its own §0 and its arbitrations, then diff
        `deferred-work.md` before the commit** — 6b.9's review found the register untouched under a
        section headed *REGISTERED RATHER THAN FIXED*. A re-read that reads only what you wrote
        cannot find what you did not write.
  - [ ] The twins (`CLAUDE.md`, `docs/project-context.md`) and `sprint-status.yaml` in the same push.
  - [ ] 🔴 **And the documents the draft's list MISSED**, found by the validation: `OPENCMDB_LOCALE` is
        documented at **`README.md:124`** and **`docker/README.dockerhub.md:85`**, and arbitration 4
        changes what those sentences promise. The docs-current-before-push rule names both.
  - [ ] 🔴 **REGISTER THE RELEASE-NOTE OBLIGATIONS — the draft contained the words *release note*,
        *6b.12* and *breaking* exactly zero times.** Two operator-visible changes ship here: the
        English primary control relabels **Merge → Document**, and under arbitration 4-yes an existing
        deployment carrying `OPENCMDB_LOCALE=FR` **stops booting**. 6b.1 owes 6b.12 a colour line and
        6b.2 an address line; this story owes it one or two more, *and a breaking boot refusal is not
        a line a release discovers.*

## Dev Notes

### Prescribed mutations

⚠️ **Read T8's first bullet before running any of these**, and read §0e before believing the draft's
warning: it said rows M1–M5 *"measure nothing"* on a tree without `build.rs` and **the validation
refuted it** — `cargo test` sees an `app.yml`-only change today. The rows that genuinely need T1 are
**M0** (a `strings` receipt on the produced binary) and anything whose oracle is a browser look.

| id | Mutation | Prediction |
|---|---|---|
| M0 | **Revert `build.rs`**, change one value, `cargo build`, **`grep -a <needle> target/debug/opencmdb`** (🔴 **never `strings \| grep`** — it cannot see 57% of the French file, §0e) | 🔴 **The story's own AC4 receipt, and it is ALREADY MEASURED on the committed tree** (§0e): with `en: "Merge"` replaced by a sentinel, `cargo build` finishes in 0.08 s with no `Compiling`, the sentinel is **absent from the binary (0)** — and after `cargo test --no-run` it is **present in the test binary (3)**. Re-run it as the receipt; the prediction is not open. |
| M1 | Delete one `fr:` line | RED on `every_key_carries_both_locales`, **and it measures what it claims with or without T1** (§0e). ⚠️ The **silent** direction — `rust-i18n` falls back to `en`, so no render-time guard can see it (`screens.rs:644-646`). |
| M2 | Delete one `en:` line | RED. The loud direction; kept as the pair that shows the guard is symmetric. |
| M3 | Restore `gesture.merge`'s EN value to `"Merge"` | RED on **both** carriers after T4/T5 — and the row must say which assertion each reddened on. |
| M4 | Put a forbidden word in a **block scalar** (`en: \|` + next line) | ✅ **Measured: with a real YAML parse it reds BOTH carriers**, so it does NOT separate them — the draft picked it to prove independence and it proves the opposite. Kept as the receipt that T4 took the real-parse branch. |
| **M4′** | A word retired in **`fr`**, in a key that has **no `fr` half** | 🔴 **THE row that separates the carriers, and it was found by measurement.** Carrier 1 **GREEN** (the word is nowhere in the French column to find), carrier 2 **RED** (`rust-i18n` falls back to `en` and the French screen renders it). The separating property is the **fallback**, not the syntax. |
| **M4″** | **Nest a key and delete its `fr` half** | 🔴 GREEN on everything today — 702 tests, nine gates — while the French UI renders English. §0f's headline. Reds only under arbitration 3(d). |
| M5 | Rename a key to contain a retired word, values clean | Carrier 1 RED, carrier 2 **GREEN**. ⚠️ **The prediction fails on the ordinary spelling**: `gate_vocabulary`'s `contains_word` counts `_` as a word character, so `gesture.merge_all` is **GREEN** — 107 of 284 keys carry an underscore. Use a matcher whose boundary set includes `_` and `.`. |
| M6 | Replace the gate's whole body with `Ok((true, …))` | 🔴 RED, or T4 has story 5.12's structural defect: *the whole body of `gate_declared_authorship` was deletable with the xtask suite green*, because every test attacked the helper. **The gate needs an end-to-end test, not only a matcher test.** |
| M7 | Break the offset→line map (multibyte value) | RED on a **located** probe. 5.12's finding: a pinned boolean proves THAT a gate fires and never WHERE, and `é` is in half this file. |
| M8 | Restore the `aria-label` literal | RED — and the guard must read the **rendered** page, not the template (6b.4b: *every guard read the source and every defect lived in the render*). |
| M9 | Set the floor's oracle to a constant equal to today's count | Under arbitration 3(c): GREEN, then RED once a key is added. **Prediction: this row exposes whether (c) was implemented or (b) was implemented under (c)'s name.** |
| M10 | `t!()` with a key that does not exist, spelled in a variable | GREEN on the literal-key guard by its **stated limit** (`example_screens.rs:1214`), RED on `no_i18n_key_reaches_the_screen` (`:1509`). A limit re-measured, not re-argued. |
| M11 | `OPENCMDB_LOCALE=zz` | Under arbitration 4-yes: a **named boot refusal**. Under 4-no: silent English, and the row records that as the shipped behaviour. |

### What the previous stories leave you

🔴 **The BLIND review layer — the diff alone, no repository, no build — found both HIGH findings for
FOUR stories running (6b.6, 6b.7, 6b.8, 6b.9), and every time they were the author's own sentences.**
Keep it blind. ⚠️ **And hand the review layers `deferred-work.md`** — 6b.7's auditor reported *"no
register rows"* on a diff that was `crates/` only, and the lesson was the reviewer's, not the layer's.

⚠️ **This story is unusually rich in sentences about counts**, which is the exact material four
consecutive reviews have found false. Every number in §0 is dated to `97c0e9a` and was measured, not
recalled. **Re-measure before restating; and put the live count in this file, per 6.1's AC8.**

### The house rules that bite here

- **`CLAUDE.md`'s DRY rule protects DELIBERATE redundancy by name.** `BINDING_STATE_AXIS` and
  `fixtures.rs`'s `expected()` are second independent oracles. A gesture-axis constant is the same
  shape and **must not** be "DRY-ed" into a derivation from `app.yml` — that would compare the file
  to itself.
- **D47's frontier**: everything is `opencmdb-bin` and `xtask`. `opencmdb-core` gets **no BEHAVIOUR
  change** — never *byte-identical* (5.13b: a promise of non-modification shelters false sentences).
- **`xtask` is a dependency of nobody**; a new dependency there costs the product nothing. It is
  still a decision to state.
- **Every `pub` item carries a TRUE doc.** A false doc is a defect — this epic has found at least one
  per story for five stories.
- **Prove-to-red**: a guard is observed failing before it passes, and the mutation is recorded.
  **Commit first.**

### What the operator will be able to DO — asked on purpose

**Nothing new.** No form, no button, no write — the count of well-lit dead ends stays at ten, and
Epic 6b's retrospective owes that count a look.

🔑 **What changes is who the product is for.** Today an operator who boots it without setting
`OPENCMDB_LOCALE` gets the half of the interface nobody has read, with the product's primary control
labelled with a word the product's own founding pillar forbids. After this story the English
deployment is a deployment rather than a fallback — and two gates make it stay one after this epic
stops watching.

### References

- `epics.md:2280-2296` (the three ACs), `:2086-2108` (goal, Guy's four premises at `:2092`, the six
  constraints, the DoD — ⚠️ which says *seven* gates and the tree has eight, nine after T4).
- `prd.md:985-1046` (the binding vocabulary: **ten** gesture rows, five state rows, the retirement of
  `merge` in English at `:1035-1037`) against `ux-design-specification.md:1332-1356` (**eleven** gesture
  rows — `attach` is missing from the PRD's table; the same retirement at `:1386`),
  `:1420-1442` (the interface-language decision, *no string is born in one language only*, Epic 22's
  boundary, and the layout direction), `:1446-1451` (the **five** binding **Microcopy Rules** — the story's draft said four and cited a range holding three; rule 5, *empty ≠ failure*, is the one it dropped).
- `deferred-work.md`, story 6b.9's section — the four rows touching this story: the build hazard, the
  `checked >= 47` floor (**owner: this story**), the paraphrased security claim (**owner: this
  story**), and `epics.md:2108`'s stale gate count.
- `crates/opencmdb-bin/locales/app.yml` — 958 lines, **284 keys**, `:2` (the seam noted since 3.8),
  `:325` (`gesture.merge`).
- `crates/opencmdb-bin/src/`: `main.rs:42` (`i18n!`), `:110-139` (`AppConfig`), `:373` (the raw
  locale read); `screens.rs:642-691` (`every_key_carries_both_locales` and its floor);
  `page.rs:89` (`lang`), `:687-725` (the action bar), `:909, 953-954, 1002`, `:2016` and `:3278` (the
  `set_locale` hazard), `:3818-3852` (the key-names-from-file / values-from-`t!()` idiom);
  `example_screens.rs:1219-1256` (`every_literal_key_in_the_view_code_resolves`
  and its stated limit), `:1509` (`no_i18n_key_reaches_the_screen`, the render-side half),
  `screens.rs:624` (`no_screen_renders_a_key_name_as_a_label`); `state_vocabulary.rs:96-119`
  (`BINDING_STATE_AXIS`), `:153` (the state-axis test), `:186` (`one_word_is_rendered_by_one_key`,
  UX-DR64's glossary uniqueness);
  `diagnostic.rs:31-35` (why a word list is a second line), `:1005-1039` (the word guard).
- `crates/opencmdb-bin/templates/_gap_card.html:1` (the `aria-label`).
- `xtask/src/main.rs:1-42` (the module doc that enumerates the gates), `:393-481`
  (`gate_vocabulary`, its `DOCS`, `PAIRS`, `CODE_RETIRED`, and the `rs`-only filter at `:460`);
  `xtask/src/observed_immutable.rs` (the precedent for a gate in its own module).
- `.github/workflows/ci.yml:46-48` (the cached `target/`, which is the hazard's second half).
- `3-8-transversal-anchors.md:24, 72` — the forbidden-word seam, noted twice, in July.
- `~/travail/Projets actuels/opencmdb/documents/opencmdb - maquette.html` — the reference mock.
- 🔑 **The gap-hunt layer's WORKING CODE is kept rather than discarded**, on branch
  `worktree-agent-a3b64a3115e908a71` (worktree at `.claude/worktrees/agent-a3b64a3115e908a71`), three
  commits off `97c0e9a`: `3fc314c` the prototype gate + `build.rs` baseline · `31e24da` arbitration
  1(a)'s rename with the suite green · `8c66eca` the **real-YAML** gate (`xtask/src/copy_vocabulary.rs`,
  298 lines) plus carrier 2 in `screens.rs`, and `yaml-rust2` added to `xtask/Cargo.toml` for **one
  line in `Cargo.lock`**. ⚠️ **It is a PROTOTYPE, not a patch to apply**: it carries M2's fixed walker
  but not the completeness assertion, its matcher still has M1's `_` defect, and carrier 2 lints the
  `en` column only. Read it to save the day of building it twice; do not merge it.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
