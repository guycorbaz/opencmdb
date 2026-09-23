# Slice: the gesture says what it does

Status: **draft** — contexted 2026-09-23, awaiting Guy's two arbitrations (§0.2 and §0.3) before
validation by two fresh-context layers.

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

### §0.2 — ⚠️ ARBITRATION 1: the inventory promises a record it cannot deliver

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

**What is still open** is whether the title and the lede change too. Saying *these have no record
yet* under a heading that says *Your records* is a contradiction in one viewport — the class story
14.4's third round paid for, and which this project has twice found in its own copy.

### §0.3 — ⚠️ ARBITRATION 2: seven keys name a STATE, and no binding row covers it

`ipam.finding.documented` (*"Documented in the inventory, so triage asks nothing about it"*) ·
`ipam.check.documented` · `ipam.next_free_none` · `next_free_caveat` · `inventory.col_documented` (a
**timestamp** column — *when* it was documented) · `inventory.unnamed` · `error.store_unreachable` ·
`example.alert.unseen`.

🔑 **These describe a STATE — *this address is in the declared record* — and not the act.**
`prd.md:888` binds the gesture's label and says nothing about the resulting state, and the binding
table's STATE axis (`concordant`, `gap`, `conflict`, `ambiguous`, `undeclared`) **has no word for
*declared by hand* either.**

So this is a vocabulary question, and **a story may not extend the binding table** — Guy mints a
term or decides the existing words suffice. ⚠️ Naming it before its screen exists is what the PLAN
axis's own act of 2026-09-10 warns against; naming it now is defensible because the state is
already rendered on three screens.

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
- What the gesture gives the operator today: `declared_attribute` rows (`origin='adopted'`) — **on
  the shipped connector, `ipv4` alone** — the question leaving the queue, and an inventory row.

---

## 2. Acceptance criteria (draft — AC3 and AC4 depend on the arbitrations)

**AC1 — families (a) and (c) conform to `prd.md:888`.** Ten interface strings stop rendering the
identifier; the ten KEYS are unchanged. A guard asserts that no value under `document.*`,
`triage.documented*` or `identity.no_gesture.*` renders the identifier in either locale, and it is
proven red on the shipped text before it passes.

**AC2 — the manual's false sentence goes.** `user-manual.tex:139`'s `\planned` block is replaced by
what is true: the control exists on `/triage`, and `document-field` does not. Both manuals build.

**AC3 — `/devices`' real section says no record exists yet** and names the epic that creates it.
⚠️ Shape depends on §0.2.

**AC4 — the state's word.** ⚠️ Depends on §0.3.

**AC5 — no regression**: ten gates, clippy `--all-targets`, `RUSTFLAGS="-D warnings"`, fmt, both
store conditions, both browser gates, both manuals. ⚠️ A `.rs` is touched, because `app.yml` is
invisible to Cargo's incremental build.

---

## 3. What this slice must NOT do

- **Not rename a key.** The identifier is bound to `document` by the same row that binds the label.
- **Not extend the binding vocabulary.** §0.3 is Guy's act, not this slice's.
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
