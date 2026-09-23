# Slice: the gesture says what it does

Status: **contexted** 2026-09-23, every decision taken — ready for validation by two fresh-context
layers.

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

### §0.4 — ✅ DECIDED BY ME ON DELEGATION, recorded as mine so it can be reversed at the right cost

**Measured**: `nav.devices` is « **Appareils** » / "Devices". `inventory.title` — « **Vos fiches** »
— is the REAL section's own `<h2>`, rendered at `_inventory.html:15`, and it is **not** the
navigation. So changing it touches one heading and one lede on one screen, which is ordinary screen
copy.

**The heading and the lede change, because the sentence Guy decided on contradicts them.** Writing
*these machines have no record yet* under a heading saying « Vos fiches » and a lede saying *« une
fiche que vous avez écrite »* is claim and refutation in one viewport — the class story 14.4's third
round paid for, and which this project has now found in its own copy twice.

🔑 **The screen says what it HOLDS rather than what it does not**, and it says it with a word the
binding table already carries — `declared` / « déclaré »:

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

## 2. Acceptance criteria

**AC1 — families (a) and (c) conform to `prd.md:888`.** Ten interface strings stop rendering the
identifier; the ten KEYS are unchanged. A guard asserts that no value under `document.*`,
`triage.documented*` or `identity.no_gesture.*` renders the identifier in either locale, and it is
proven red on the shipped text before it passes.

**AC2 — the manual's false sentence goes.** `user-manual.tex:139`'s `\planned` block is replaced by
what is true: the control exists on `/triage`, and `document-field` does not. Both manuals build.

**AC3 — `/devices`' real section says no record exists yet** and names the epic that creates it.
Its heading and lede say what the screen holds (§0.4), so the sentence does not contradict them.

**AC4 — the eight strings of §0.3 stop rendering the identifier** and say where the state lives
instead, with no new term minted. `inventory.col_documented` moves to family (a) and becomes
« Ajouté ». The absence of a word for *declared by hand* is REGISTERED, not filled.

**AC5 — no regression**: ten gates, clippy `--all-targets`, `RUSTFLAGS="-D warnings"`, fmt, both
store conditions, both browser gates, both manuals. ⚠️ A `.rs` is touched, because `app.yml` is
invisible to Cargo's incremental build.

---

## 3. What this slice must NOT do

- **Not rename a key.** The identifier is bound to `document` by the same row that binds the label.
- **Not extend the binding vocabulary.** §0.3 measured that none is needed; the gap is registered, and inventing a term to fill it is what this slice must not do.
- **Not make the real inventory row clickable**, and not remove the example rows' links — both were
  refused on 2026-09-22, the first as a route that exists to apologise, the second as hiding a
  working demonstration to conceal an absence.
- **Not create a record.** That is Epic 6's, and this slice says so rather than doing it.

---

## Record

- live-count: bin=744 core=191 xtask=110
- base: ec466b2dddebd09472f06443e5ac106014ec88d1
- registered:
- file: _bmad-output/implementation-artifacts/add-gesture-says-what-it-does.md
