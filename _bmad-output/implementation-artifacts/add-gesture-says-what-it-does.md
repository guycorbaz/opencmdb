# Slice: the gesture says what it does

Status: **ready-for-dev** — contexted 2026-09-23, VALIDATED the same day by two fresh-context layers
whose findings rewrote §2 entirely, withdrew §0.4 and dissolved §0.3. Every decision is taken.

**Issues:** #200, #201. **Base:** `ec466b2`. **In no epic file**, on 6.4b's and 14.4c's precedent:
*a slice a finding creates has no epic.*

---

## 0. Where this comes from, and what is already decided

🔑 **This slice was created by Guy USING THE PRODUCT**, not by a plan. On 2026-09-22 he added a
machine from `/triage` on his own NAS — the first time that gesture has been pressed outside a test
— and said: *« Documenter est un peu trop fort : aucune fiche ne peut être créée pour l'instant. »*

That single sentence closed milestone **J3** and opened two issues. It also confirmed R1's own
wording on the day R1 closed: *the real network reveals what the tool cannot see.* **Two defects,
neither reachable by any gate, any review layer or any retrospective.**

### §0.1 — What is already decided, and needs no arbitration

`prd.md:888` binds it: **"the interface names this gesture *Add* / « Ajouter »; documentation, API
and code use `document`"** — and calls it **the ONE concept whose UI label differs from its
identifier**, which is precisely what makes this drift possible here and nowhere else.

⚠️ **The KEYS do not change, only their VALUES.** `document.*` is the identifier and the binding row
says code uses it. A rename would be a second defect wearing the fix's clothes.

**Family (a) — the gesture's own answers, 7 keys.** `document.done_one` · `done_other` ·
`unknown_subject` · `already_documented` · `nothing_to_document` · `triage.documented_one` ·
`documented_other`.

🔑 **`triage.documented_one` is the sentence Guy actually read**: `document.done_*` is the 201's
body, which a browser never sees because it follows the `HX-Redirect`; `triage.documented_*` is what
`/triage` renders after it, from `?documented=N`. *The word came back through the one string the
button's own v0.3.1 correction did not cover, and it is the one the operator reads after acting.*

**Family (c) — the identity abstention lines, 3 keys.** `identity.no_gesture.absence_of_proof` ·
`ambiguous` · `unrecognised`, all « Rien à **documenter** ici ». Same family: they name the gesture.

⚠️ **And the shipped user manual carries a false sentence**, `user-manual.tex:139`: a `\planned`
block reading *"Document-all exists as a route and is reached from no screen"* — **false since story
6.4, 2026-08-26**, and published in `v0.5.0` and `v0.5.1`. The manual's prose may say `document`
(documentation does, per the row); what is wrong is the claim, not the word.

### §0.2 — THE FINDING: the inventory promises a record it cannot deliver

🔴 **The screen is called « Vos fiches » / "Your records".** Its lede says *« Une ligne est **une
fiche que vous avez écrite** »*; its empty state says *« **Une fiche s'écrit** dans la file de
triage »*. **Not one row opens.**

Measured: `templates/_inventory.html` — the REAL section — carries **exactly one `href`**, the empty
state's link to `/triage`. `templates/_devices_example.html:42` links **every example row** to
`/devices/{id}`. *So the invented machines open and the operator's own does not, under a title
saying these are their records.*

⚠️ **The copy already contradicts itself two lines apart**: `inventory.none_after` says « pressez le
bouton **qui l'ajoute** », conforming, directly after the lede says « documenté ».

**Guy's decision of 2026-09-22 was: say it in `/devices`' real section, and name the epic that
creates the record.** ⚠️ **His first remedy — say it on the record screen — was refuted by the
measurement above**: that screen is unreachable for a machine the operator added.

**What his decision left open** — whether the heading and the lede change too — is settled in §0.4.

### §0.3 — ✅ DISSOLVED BY MEASUREMENT: no vocabulary act is needed

**It was posed as an arbitration and it is not one.** Reading each of the seven strings in place
shows the state is **already named by its own sentence**, or expressible with a word the binding
table already holds. Nothing must be minted.

| Key | What the word is doing | What replaces it |
|---|---|---|
| `ipam.finding.documented` | *redundant* — the sentence already says **« dans l'inventaire »** | drop the word |
| `ipam.check.documented` | *redundant* — same | drop the word |
| `ipam.next_free_none` | one item in a list of exclusions | « dans l'inventaire » |
| `ipam.next_free_caveat` | same list | same |
| `inventory.unnamed` | « aucun nom **documenté** » | « aucun nom **déclaré** » — `declared` IS a binding pair |
| `error.store_unreachable` | « les enregistrements **documentés** » | « **vos** enregistrements » |
| `example.alert.unseen` | example content, FR30's alert kind | « une adresse de **votre inventaire** » |

🔑 **And one key was MISFILED by my own analysis**: `inventory.col_documented` is the column holding
**when the gesture was made** (`InventoryRow::documented`, *"When it was documented"*). It names the
ACT, so it belongs to family (a) and becomes « **Ajouté** » / "Added". *The four families were a
reading; the reading was checked and one row moved.*

⚠️ **What this does NOT do**: it does not give the product a word for *declared by hand*. The
binding table's STATE axis still has none, and three screens go on describing that state by saying
where it lives rather than what it is. **Registered rather than minted** — a term invented to fill a
gap nobody has met on a screen is minted by accident, which the PLAN axis's own act of 2026-09-10
says in as many words.

### §0.4 — ~~DECIDED BY ME~~ **WITHDRAWN 2026-09-23 by §0.6's measurement**, and kept visible

**Measured**: `nav.devices` is « **Appareils** » / "Devices". `inventory.title` — « **Vos fiches** »
— is the REAL section's own `<h2>`, rendered at `_inventory.html:15`, and it is **not** the
navigation. So changing it touches one heading and one lede on one screen, which is ordinary screen
copy.

**The heading and the lede change, because the sentence Guy decided on contradicts them.** Writing
*these machines have no record yet* under a heading saying « Vos fiches » and a lede saying *« une
fiche que vous avez écrite »* is claim and refutation in one viewport — the class story 14.4's third
round paid for, and which this project has now found in its own copy twice.

🔴 **WITHDRAWN.** §0.6 measures that « fiche » is not the defect, so retitling the screen solves a
problem that does not exist and creates one §0.5(i) measures (« Ce que vous avez déclaré · 4
fiches »). *A decision taken on a reading, refuted when the reading was checked.* It is struck
rather than deleted, because the next reader must see what was nearly shipped and why it was wrong.
What it proposed, for the record:

- `inventory.title` « Vos fiches » → **« Ce que vous avez déclaré »** / "What you have declared"
- `inventory.lede` « une fiche que vous avez écrite » → **« une adresse que vous avez déclarée »**,
  the rest of the sentence unchanged
- `inventory.none_before` « Une fiche s'écrit dans la » → **« Une déclaration s'écrit dans la »**

⚠️ **Refused: keeping the heading and adding the sentence under it**, which is what Guy's 2026-09-22
decision reads as on its own. It is cheaper by three strings and it leaves the screen asserting and
denying one thing in one viewport. ⚠️ **Also refused: calling them « Vos enregistrements »** — it
dodges *fiche* while keeping the promise, since an enregistrement is no more openable than a fiche.

⚠️ **This decision is MINE, not Guy's.** He decided the sentence; the heading and the lede follow
from it and are ordinary screen copy — `nav.devices` is « Appareils » and is untouched. It is
recorded here so that reversing it costs one commit and no archaeology.

### §0.5 — 🔴 WHAT THE VALIDATION CHANGED, and it reframes the slice

Two fresh-context layers refuted enough of §0.1–§0.4 that the criteria are rewritten below rather
than patched. **Four things, each measured.**

**(i) The audit is of a VERB; the promise is a NOUN, and it lives in other keys.** Twenty keys carry
`document`. **`inventory.n_entities_one/other` renders « %{n} fiche(s) » INSIDE THE SAME `<h2>` as
the title** (`_inventory.html:15`), carries no form of the identifier, and is outside every
criterion. §0.4's rewrite would have produced « **Ce que vous avez déclaré** · 4 fiches » with AC3's
*« ces machines n'ont pas encore de fiche »* three lines below — **claim and refutation in one
section, which is the class §0.4's own argument invokes.** Six more record-promising keys sit
outside the twenty, `nav.device` = « **Fiche appareil** » among them.

**(ii) And the promise is not quite what §0.2 said.** The binding glossary has **no row for
`record`/`fiche`** — checked — but it uses the noun in its own definitions: *"write observed values
into the **declared record**"*, *"the operator **CREATES THE MISSING RECORD**"*. 🔑 **So the product
DOES create a record in the glossary's sense. What it does not create is a PAGE** — and « fiche »
carries *a sheet you open and fill*. `/devices/{id}`, labelled « Fiche appareil » in the navigation,
serves invented content. *§0.2 said the product promises a record it cannot deliver; it delivers the
record and not the page.* ⚠️ **This refinement is NOT taken here** — it changes what the slice is
about and it is posed to Guy.

**(iii) AC1 asked for a guard that already exists TWICE, and its scope was the story's own recorded
defect.** `copy_vocabulary.rs`'s `RETIRED` (per-locale, whole file, real YAML parse, located
findings, keys protected by construction) and `state_vocabulary.rs`'s `RETIRED_IN_COPY` (read by
`no_resolved_value_carries_a_retired_term`, on the **rendered** string), kept in step by
`the_two_carriers_agree_on_what_is_retired`. Twelve entries in each and the property holds over all
twenty — **measured: 40 findings on the shipped tree, exactly 20 keys × 2 locales, and green after
the repair**, with « votre documentation » surviving because `contains_word` refuses a glued needle.
🔴 **AC1's three namespaces would have left eleven of twenty-one strings with no carrier**, including
the one §0.3 itself re-filed. *A guard scoped by a family is scoped by a reading of it* — the
instrument defect §1 records, reproduced inside the criterion written about it.

**(iv) 🔴 A stale fixture pins the defect Guy corrected by hand.** Three `xtask` tests carry
`gesture.document: en "Document" / fr "Merger"` **as correct copy**, green since before `v0.3.1`.
Adding the terms reds all three, and the honest repair is the fixture, not the list.

⚠️ **And AC5's stated reason was FALSE**: `build.rs:38` has carried
`cargo::rerun-if-changed=locales/app.yml` since story 6b.10 closed that hazard. Measured — a sentinel
planted in `app.yml` alone reaches the binary. **Both twins and four story files went on citing the
hazard as live for a month**, which is Epic 14's retrospective's own class in mirror: *a closure
written only where it was made does not reach the file the next story reads.* Corrected in the twins.

### §0.6 — 🔑 WHAT GUY'S SENTENCE ACTUALLY MEANS, measured — and it is the slice's real subject

Taken to the letter — *« aucune fiche ne peut être créée pour l'instant »* — and checked:

```
grep -rn "insert_declared_attribute(" crates/opencmdb-bin/src --include=*.rs
```

**Every call site of the only `'manual'` writer is inside a `#[cfg(test)]` module. There is no
production call site at all.** An operator cannot write a single declared field of their own. The
one write route that exists, `POST /document-all`, copies what the connector observed and nothing
else; `document-field` (FR13(b)) is Epic 7's and exists nowhere in `crates/`.

🔑 **So the gesture ADOPTS; it does not AUTHOR** — and that is what Guy's sentence names. Not *no
record exists* (one does, and the glossary calls it that), and not *no page opens* (true, and
`inventory_view.rs:21` explains why: the row shows everything the store knows). **You cannot create
a record because you cannot write a field the network did not show you.**

**Three consequences, and they settle every open point above:**

1. **« fiche » / « record » STAYS.** The row IS the declared record, in the glossary's own sense
   (*"the operator CREATES THE MISSING RECORD"*). « Vos fiches », « 4 fiches » and `inventory.lede`
   are **true**. §0.4 is withdrawn, §0.5(i)'s heading collision dissolves with it, and the six
   record-promising keys outside the twenty need no change.
2. **`nav.device` = « Fiche appareil » is the ONE real over-promise left** — an entry that addresses
   no particular device and serves an invented one. It is Epic 6's screen and not this slice's;
   registered.
3. **AC3's sentence changes subject.** Not *these machines have no record yet* — false, they have one
   — but **what the operator can and cannot do with it**: these records hold what the network showed;
   you cannot yet add a field it did not; there is nothing more to open, because the row shows
   everything the store knows.

⚠️ **This is taken by me on delegation and recorded as mine**, reversible at one commit. It rests on
a measurement rather than a preference, which is why it is taken rather than posed a fourth time.
**Refused: retiring « fiche »** — it would remove a true word to fix a false sentence, and leave the
operator with no name for the thing they made.

---

## 1. Measured facts

- **20 keys** carry the word in `app.yml`, both locales, `documentation` excluded — walked by parsing
  the file, not by grepping one namespace. ⚠️ **Issue #200 opened saying FIVE**, which came from
  grepping `document.*` alone: *an instrument that measures a named region leaves the defect to the
  region beside it*, met in the issue written about that very class. Corrected on the issue.
- The **button** conforms — `gesture.document` renders « Ajouter » / "Add" — corrected by Guy himself
  in `v0.3.1`, and the key's own comment records why.
- `Screen::Device` is `Nature::Example(ExampleContent::DeviceRecord)`; `entity` and `device` exist in
  the schema with **no producer**, by story 6.5's own criterion; `interface` stays outside the
  supertype until story 6.12.
- What the gesture gives the operator today: `declared_attribute` rows (`origin='adopted'`) —
  **up to THREE on the shipped connector — `ipv4`, `hostname` when reverse DNS answers, and `mac` since PR #163** — the question leaving the queue, and an
  inventory row. 🔴 **This bullet said *`ipv4` alone* and the validation refuted it**: `gap::project`
  maps `IpV4`, `Hostname` AND `Mac`, and `arp_ping::emitted_facts` pushes all three. The figure was
  true before 2026-09-10 and was copied into **six documents** — this story, issue #201, both twins
  and two files of the project record — from a claim I wrote rather than measured. ⚠️ And the tree's
  own prose said **TWO** (`inventory_view.rs:11`, written after the reverse-DNS story and before the
  MAC one), so the slice took ONE from an issue while the file it cites said TWO and the code did
  THREE. *Three figures for one fact, none of them measured until now.*
- 🔑 **`inventory_view.rs:21` already carries the ANSWER to Guy's question, and has all along**:
  *"There is no drill-in, and that is today's data speaking rather than a design. A documented entity
  carries at most \[three\] fields, so the row shows everything the store knows and a record page
  would repeat it."* **True, correct, and written where no operator can read it.** That is issue #201
  in one sentence: the reason exists, for the next author and never for the person who pressed the
  button. AC3's sentence is that paragraph, said on the screen.

---

## 2. Acceptance criteria — rewritten 2026-09-23 on the validation

**AC1 — the twelve terms join the TWO carriers that already exist; no new guard is written.**
`copy_vocabulary.rs`'s `RETIRED` gains `document / documents / documented / documenting` in the `en`
column and `documenter / documenté / documentée / documentés / documentées` in the `fr` one;
`state_vocabulary.rs`'s `RETIRED_IN_COPY` gains the same, and
`the_two_carriers_agree_on_what_is_retired` keeps them in step. **Proven red on the shipped tree
before it passes: `🔴 copy-vocabulary 40 finding(s)`, which is exactly the twenty keys × two
locales** — so the denominator falls out of the measurement instead of being asserted. Green after
the repair, with « votre documentation » surviving untouched because `contains_word` refuses a glued
needle.

🔑 **Two carriers and not one, because story 6b.10 measured they see different things**: the file
carrier is blind to a key with no `fr` half (the resolver falls back and the French page serves
English), the resolver carrier is blind to a key NAME. ⚠️ **The KEYS are protected by
construction** — the identifier column is the `<key>` pseudo-locale and gets no term — so §3's *not a
key rename* is held by the gate's shape rather than by care.

**AC2 — three stale `xtask` fixtures are repaired, not the list.** `gesture.document: en "Document" /
fr "Merger"` is pinned GREEN as correct copy in three tests, and has been since before `v0.3.1`. It
pins **the product's primary button rendering the word `prd.md:888` forbids** — the defect Guy
corrected by hand. Three one-line edits; `cargo test -p xtask` back to 110.

**AC3 — the manual's false sentence goes, and so does the retired term beside it.**
`user-manual.tex:140-141` stops claiming document-all *"is reached from no screen"* (false since
2026-08-26, shipped in `v0.5.0` and `v0.5.1`); `document-field` really is unbuilt, so that half
stays. ⚠️ And `:135` carries **`drifted`**, retired by name in both binding tables since story 6b.6
and shipped twice — *`docs/manuals/` is walked by no gate, and this slice is the only occasion
anyone opens the file.*

**AC4 — `/devices`' real section says what the operator can and cannot do**, per §0.6 and NOT per
§0.2's withdrawn framing: these records hold what the network showed; a field it did not show cannot
yet be added; there is nothing more to open because the row shows everything the store knows. It is
**a key**, in both locales, and 🔴 **its field is covered** — replacing
`rust_i18n::t!("inventory.no_record_yet")` with an English literal in `inventory_strings()` currently
leaves **744 tests green**, because `every_field_of_the_shared_strings_comes_from_a_key` is bounded
to `page.rs`'s `fn strings()`. Widened to reach `inventory_view.rs`, proven red on that literal.
⚠️ Placed inside the non-empty branch — after `{%- endif %}` it renders on the EMPTY state where
*"these records"* names nothing, and immediately above the example marker it can be read as belonging
to the wrong list. No guard can say that; the position is a decision, written here.

**AC5 — the one string the operator actually reads gains a rendered-DOM carrier.**
`a11y/kbd-probe.mjs` already **presses the gesture for real** and reads
`document.querySelector(".documented")?.textContent` — *the exact string Guy read* — and asserts only
that it is non-empty. One assertion more: it carries no form of the identifier. 🔑 That is story
6b.11's amended AC5 applied where it belongs — *a source guard does not suffice where the defect
lives in the DOM* — and it costs one line in a check that already runs.

**AC6 — no regression.** Ten gates, clippy `--all-targets`, `RUSTFLAGS="-D warnings"`, fmt, both
store conditions, both browser gates, both manuals. ⚠️ **No claim is made that `app.yml` needs a
`.rs` touched to rebuild**: `build.rs:38` has carried `cargo::rerun-if-changed=locales/app.yml` since
story 6b.10 closed that hazard, measured again here. A `.rs` is touched because AC1 and AC4 touch
Rust, not for a reason a month out of date.

**AC7 — three rows are registered**, and the `## Record

- live-count: bin=745 core=191 xtask=110
- base: ec466b2dddebd09472f06443e5ac106014ec88d1
- registered: says « Fiche appareil » and the screen behind it serves an invented machine
- registered: An operator cannot AUTHOR a declared field
- registered: bounded ONE constructor while five more existed
- file: CLAUDE.md
- file: _bmad-output/implementation-artifacts/add-gesture-says-what-it-does.md
- file: _bmad-output/implementation-artifacts/deferred-work.md
- file: a11y/kbd-probe.mjs
- file: crates/opencmdb-bin/locales/app.yml
- file: crates/opencmdb-bin/src/inventory_view.rs
- file: crates/opencmdb-bin/src/page.rs
- file: crates/opencmdb-bin/src/state_vocabulary.rs
- file: crates/opencmdb-bin/templates/_inventory.html
- file: docs/manuals/user-manual/user-manual.tex
- file: docs/project-context.md
- file: xtask/src/copy_vocabulary.rs

### File List

The `## Record` block's `file:` lines are this slice's File List (checked by `cargo xtask record`).
