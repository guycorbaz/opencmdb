# Story 6.14: The ambiguity explains itself on the page

Status: **contexted 2026-09-25 — `ready-for-dev` in `sprint-status.yaml`, and ⚠️ NOT developable until the
mandatory two-layer validation and Guy's arbitration on §0.5.** §0 carries four findings: two of them
change what this story can honestly ship, and one is a word the product already uses for something else.

**Epic 6**, on Guy's act of 2026-09-24 (PR #210): 6.12 (done) → **6.14** directly, 6.13 absorbed into 6.12.
**Base:** `master` at `dabe1f5`, clean tree. **Live count at base:** bin=782 core=219 xtask=110
(`cargo test -p <crate> --locked -- --list`, the command `cargo xtask record` runs).

---

## Story

As the operator,
I want an unresolved grouping to show its candidates and their evidence,
so that I can lift the doubt the engine refused to guess at.

## The epic's criteria (`epics.md`, story 6.14), with the two notes that now bind them

1. **Given** an abstention whose cause is `Ambiguous`, **When** it is displayed, **Then** its candidates and
   their evidence are shown **from the persisted `link_candidate` rows** *(from 6.12's L2 decision table since
   2026-09-24 — see story 6.13's note)* (FR16) — the abstention explains itself.
2. **Given** the operator's three cases, **When** the line is rendered, **Then** it carries the gesture that
   **LIFTS A DOUBT** — choose among candidates — and **not** the documenting gesture, which belongs to
   `AbsenceOfProof`.
3. **And** the count keeps the honest unit story 5.14b's arbitration 13 fixed, and ⚠️ **Epic 6 is where that
   unit stops being honest**: once grouping exists, *sighting* is no longer the true word, and the locale keys
   change with it.

**Binding since 2026-09-24/25 and read with them:**

- 6.13's note: 5.14b's tripwire cannot red on an L2 ambiguity; **its replacement — a slice with two
  interfaces and a shared name, read from wherever L2 decisions live — is this story's**.
- Guy, 2026-09-25 (6.12's code review): AC3 of 6.12 is **superseded by decision F** — `l2_pair_decision`
  stores the pair and the verdict VECTOR, no evidence; **this story reads the NAMES from the two interfaces'
  CURRENT observations at display time**, cost stated: today's names, not those the decision was reached on.
- Guy, 2026-09-25: a pair holding a current OPERATOR row is **left to the operator** by the engine
  (`operator_held`). This story is the first that could WRITE one.

---

## 0. What contexting measured, and what it will not settle alone

### §0.1 — 🔴 NOTHING SPECIFIES WHAT *"LIFT THE DOUBT"* WRITES, and the positive answer needs a device nobody mints

Searched across `prd.md`, the UX specification, `architecture.md`, `epics.md` and both manuals:

- **The glossary has no gesture row for it.** *"The operator LIFTS THE DOUBT"* is the MEANING of the
  `ambiguous` STATE row (`prd.md:1019`, UX `:1367`) — not a gesture with an EN/FR pair. The gesture axis
  has `attach`/*rattacher* and `exclude`/*exclure* (`prd.md:1000-1001`); `triage` lists
  *document / accept-gap / create / attach / exclude / snooze* (`prd.md:1003`). **No `resolve`, no
  *choose among candidates*.**
- **The UX specification's Resolve panel is one line**: *"candidate matches + evidence + confidence; never a
  blind document"* (`:1248-1250`, UX-DR21 at `epics.md:264`), listed as *MVP periphery* (`:1264`). **No
  layout, no controls, no keyboard key** (`epics.md:2161`: keys are assigned *"when all six gestures exist,
  at once"*).
- **Epic 7 owns no such gesture** (`epics.md:431-433`: document, accept-gap, exclude, snooze, create, attach).
- **What an answer WRITES is unspecified.** The only operator write on a link the architecture spells out is
  D21's *"undo a link → supersede with a `NO_MATCH` `decided_by='OPERATOR'`"* (`architecture.md:1459-1460`).
  The POSITIVE answer — *"these two interfaces are one machine"* — is a grouping, i.e. a **device**, and
  6.12 left the device mint **ownerless by decision** ("whichever story first produces an L2 `Decisive`"),
  with `0012` refusing `outcome = 'match'`.

🔑 **Consequence:** criterion 2's *"it carries the gesture that LIFTS A DOUBT"* can be met in two honest ways
and one dishonest one — a LABELLED control that does not act yet (story 6b.4b's shape), a LIVE control for
the NEGATIVE answer only (D21), or a live positive answer that invents the device mint and its semantics
inside a display story. §0.5 (A).

### §0.2 — 🔴 *"Résoudre"* ALREADY LABELS SOMETHING ELSE ON THIS SCREEN

`page.rs:1112` gives every **Conflit** row (`AbstentionCause::ConflictingObservations` — two SOURCES disagree
about one field) the primary control `gesture.resolve` — *Résoudre*/*Resolve* — `Planned`, owner **"6"**
(`page.rs:727-730`, comment: *"Résoudre needs FR16's ranked candidates and is Epic 6's"*). The UX
specification's *Resolve* is the **ambiguity** gesture (`:766-769`: *"such a card shows a Resolve badge
instead of Document"*; `:1174`: *"ambiguity always routes to Resolve"*). The mock put *Résoudre* on a
conflict, and story 6b.4 followed the mock.

So **one word would name two different acts on one screen** — which the binding glossary's preamble forbids
in so many words — and the Conflit row's owner "6" is **false** once 6.14 exists: no Epic 6 story resolves a
source-against-source conflict. `resolve` is also **in no glossary row**. §0.5 (B).

### §0.3 — ⚠️ *"Ambiguous always routes to Resolve (never a blind document)"* collides with the one live gesture

`obelix`'s two addresses (`.8`, `.9`) are undeclared, so each is a **Nouveau** row carrying the product's
ONE live control — *Ajouter*, the documenting gesture — and 6.12 makes them also ONE `Ambiguous` pair. The UX
specification says an ambiguous card shows **Resolve instead of Document from the start** (`:766-769`) and
*"never a blind document"* (`:1174-1175`). Applied literally, this story **removes the working gesture** from
exactly the machine Guy can see on the NAS, and — if the Resolve control is only labelled (§0.5 A1) — leaves
those rows with **no live control at all**. Not applying it lets the operator document `.8` and `.9` as two
entities *before* answering whether they are one machine — the very double count Epic 6 is named after
(*"Ne pas compter deux fois la même boîte"*). §0.5 (C).

### §0.4 — Measured facts the dev agent inherits

- **`page.rs` is at 1954 of 2000 code lines** (last top-level `#[cfg(test)]` at `:1955`). **46 lines of
  headroom: split BEFORE adding** — `build_triage_offering` and its view-model types (`QueueRow` `:565`,
  `DetailPane` `:852`, `TriageView` `:880`) into a `triage_view.rs`, on `identity_view.rs`'s precedent
  (6.4) and `ipam_rail.rs`'s (14.5). The split is its own commit, behaviour-neutral, suite green before and
  after.
- **The queue is keyed on ADDRESSES; an ambiguity is about two INTERFACES.** Row kinds today: Écart, Absence,
  Conflit, Nouveau (`page.rs:1030-1248`); row id doubles as the `?sel=` selector (`nouveau:{ipv4}`,
  `ecart:{entity}:{field}`). **`Ambigu` was omitted by story 6b.4 because nothing produced it**
  (`page.rs:929-936`) — 6.12 produces it. A new row id must be stable across sweeps: `ambigu:{low}:{high}`
  over the two interface ids.
- ⚠️ **`the_queue_carries_the_four_kinds_the_engine_can_produce_and_no_others`** (`page.rs:5231`) — its doc
  says it asserts Ambigu's ABSENCE, its body does not (`:5262-5291`). The first guard this story must fix
  rather than trip.
- **Nothing reads `l2_pair_decision` for display.** `load_current_l2_decisions` (`l2_repo.rs:169`) reads the
  whole current set (small by decision G) and is the natural reader; the names come from
  `load_observation_facts` (already loaded by `triage_view`, `page.rs:1401`) filtered to each interface's
  observations — ⚠️ **the join from interface to observations is `identity_link.interface_id`**, a read no
  view makes today.
- **Evidence to show = the stored verdict vector** (`verdicts`: `rule=verdict;…`) **plus the names read now**.
  Each rule needs an operator-readable sentence in both locales (e.g. *both answer to `obelix.home.arpa`*;
  *neither is a virtual-router address*) — keys, never literals; the `copy-vocabulary` gate lints them
  (`merge` is retired).
- **The reach section** (`identity_view.rs`, `count_engine_reach`) reads `identity_link` only; its
  `identity.no_gesture.ambiguous` line already says *"this is a doubt to lift, not an entity to create"*
  (`app.yml:145-147`). An L2 ambiguity is **invisible to it** today.
- **5.14b's tripwire** `the_production_pass_produces_no_ambiguous_abstention` (`scan_pass.rs:776-819`) runs
  on a slice with ONE interface and reads `identity_link`; its doc (`:762-765`) still says 6.12 turns it red
  — **false**, owed a correction here.
- **Write-route pattern, should a gesture go live** (`document.rs`): a port-backed, pool-free state (so
  `State<MySqlPool>` cannot compile in the handler — 6.1's M4); `write_guard::same_origin` first (403); keyed
  refusal bodies; `HX-Redirect` to `/triage`; its path added to a `PATHS` list; mounted in `main.rs` `app()`
  above `auth_deny`; and `every_write_route_is_refused_without_a_credential_and_exists` (`main.rs:1812`)
  **hard-codes `declared.len() == 11`**. No `xtask` gate covers `l2_pair_decision` (authorship guards
  `declared_attribute` only).
- **Browser gates**: `a11y/seed.sql` creates **no `interface` and no `l2_pair_decision` row**, so neither gate
  would ever see an Ambigu row — the *"green on residue"* shape story 6b.11 paid for. The seed must create two
  interfaces, their links and one `Ambiguous` pair, and axe gains an `AXE_REQUIRE_AMBIGUOUS` flag (a seeded
  run with no Ambigu row is *the gate could not run*, not a pass). `kbd-probe.mjs` finds rows by
  `.queue .queue-row > a` and its floor `MIN_CHECKS = 62` moves with any new check.

### §0.5 — The decisions that are Guy's, each with a recommendation

**(A) What *"carries the gesture that lifts a doubt"* means here.**

- **(A1) Display + a LABELLED control** (6b.4b's `Gesture::Planned` with an owner) — the row, its candidates,
  their evidence, and a control that says it does not act yet and why. Honest, bounded, and **`obelix`
  becomes visible as one question**. The gesture's semantics go to a story of their own (6.14b), which first
  needs a planning act: a glossary row for the gesture, and what each answer writes.
- **(A2) Display + a LIVE negative answer only** — *"not the same machine"* writes an OPERATOR `no_match` row
  (D21's shape; the engine then leaves the pair alone, 6.12's decision), while *"the same machine"* stays
  labelled. Half a gesture, a new write route with its whole front-loaded cost (Origin, keyed refusals, both
  browser gates, the route-table test), and a `rule_id` to mint for an operator's `no_match` (the CHECK
  demands one).
- **(A3) Display + the full gesture** — including the device mint, a `match` outcome in `0012`, and what a
  grouped device means to the gap (still keyed on addresses). A second story's worth inside a display story.

**Recommendation: (A1)**, with 6.14b INSERTED on the 6b.4/6b.4b and 6.4/6.4b precedent — *a display story
whose gesture's semantics are unspecified is two stories*. ⚠️ Guy's call because it adds a story to the epic.

**(B) The word.** Options: (B1) the ambiguity takes *Résoudre* and the Conflit row gets another word (the
conflict's gesture is none of Epic 6's — its owner "6" is false, re-owned to Epic 7 with the conflict itself);
(B2) the ambiguity gets a new word and *Résoudre* stays on conflicts. **Recommendation: (B1)**, because the
UX specification binds *Resolve* to ambiguity in three places and the conflict's *Résoudre* came from the
mock; the conflict's control becomes `Planned` under a word the glossary already has, or a new row. ⚠️ Either
way **extending the binding glossary is a planning act** (6b.6's precedent: Guy added five rows to both
documents), so the term is Guy's, in `prd.md` and the UX specification together.

**(C) Does an ambiguity take the documenting gesture away from its addresses?** (C1) Yes — the UX
specification's *"never a blind document"*: a Nouveau row whose address sits on an interface in a current
`Ambiguous` pair shows the Resolve control instead of *Ajouter*, and under (A1) that control is labelled, so
`obelix` cannot be documented until 6.14b. (C2) No — Nouveau rows keep *Ajouter*, and the Ambigu row stands
beside them; the pane of each Nouveau row SAYS that its address is part of an open question. **Recommendation:
(C2) now, (C1) with 6.14b** — removing the product's only live gesture from the machine the operator is
looking at, to replace it with a control that does not act, trades a working gesture for a promise.

**(D) One row per PAIR or per connected GROUP?** Three NICs sharing a name are three pairs. FR16b says *"96
multi-interface devices is not 96 failures — it is ONE question"*, but that sentence is about the reach
section's CAUSE line, and ambiguity is not transitive (L1 refused connected components for that reason).
**Recommendation: one row per pair** — literal to the data, `obelix` is one row either way — with the reach
section gaining ONE line counting L2 questions (FR16b) and the pair-vs-group question registered.

---

## 1. Acceptance criteria — DRAFT, written for (A1)+(B1)+(C2)+(D)

**AC1 — an `Ambigu` row per current `Ambiguous` pair**, id `ambigu:{low}:{high}`, read from
`l2_pair_decision`, stable across sweeps (an unchanged network gives the same row and the same `?sel=`).
The queue's four-kinds guard is rewritten to assert FIVE kinds, and its doc stops claiming an absence it
does not test.

**AC2 — the pane shows both candidates and their evidence**: for each interface its hardware address, its
addresses and names **as currently observed** (Guy's decision F, cost said ON THE SCREEN: *"names as seen
now"*), and one keyed sentence per verdict of the stored vector. No raw token, no UUID shown (6b.4's lesson).

**AC3 — the control that lifts the doubt is present, labelled, and does not act** (`Gesture::Planned`, owner
6.14b), with the one sentence story 6b.4b's shape requires saying what it will do; the amber stays the
documenting gesture's (`the_resolve_gesture_cannot_go_live_and_take_the_amber_with_it` holds). **No
documenting gesture on the Ambigu row** (criterion 2).

**AC4 — the Nouveau rows of an ambiguous pair keep *Ajouter*, and say they belong to an open question** (C2),
with a link to the Ambigu row.

**AC5 — the reach section gains ONE line for L2 questions** — its own unit (*pairs of interfaces*, not
*sightings*), never summed with the sighting lines (5.14b's arbitration 10). ⚠️ Criterion 3's *"the unit stops
being honest once grouping exists"*: **grouping does not exist** (no L2 `Match`), so the sighting unit is
still true for L1 and the story says so rather than renaming it.

**AC6 — 5.14b's tripwire is REPLACED, not deleted**: a production pass over two NICs sharing a name yields
exactly one Ambigu row on `/triage`; the old test's false doc is corrected.

**AC7 — both browser gates see it**: `a11y/seed.sql` creates two interfaces, their links and one `Ambiguous`
pair; `AXE_REQUIRE_AMBIGUOUS=1` in CI; the keyboard gate reaches the Ambigu row and its labelled control; the
floor moves with the checks, prose and constant in one commit.

**AC8 — the glossary term is the one Guy chose (B)**, in both locales, through keys; `copy-vocabulary` green.

**AC9 — `page.rs` is split before it grows**, behaviour-neutral, in its own commit.

## 2. What this story must NOT do

- **Not write any row** under (A1) — no route, no OPERATOR row, no device.
- **Not use the amber** for the Resolve control.
- **Not show a UUID or a raw verdict token** to the operator.
- **Not sum** the L2 question count with the sighting counts.
- **Not edit `epics.md`, `prd.md` or the UX specification** — (A) and (B) are planning acts.
- **Not delete** 5.14b's tripwire — replace it and correct its doc.
- **Not claim the trap gate moves** — still RED 26/15/11.

## 3. Previous-story intelligence (6.12)

- The L2 table's pair is ordered by INTERFACE id, not by L1 key — a display keyed on keys will disagree.
- `load_current_l2_decisions` reads operator rows too; the display must say which rows are the engine's.
- One read per sweep, never per pair — 6.12's review measured the per-pair shape at 10–50×.
- Derive a mutation's carriers by grepping the mutated token; a CHECK's readers are every row it judges
  (6.12's M6). `cargo fmt` reflows anchors; the driver refuses a two-site anchor, and that is a result.
- Each validation and review layer gets a database of its own (port 13419; never 3306; `env -u DATABASE_URL`).
- `Status:`, the Change Log and the criteria are the regions `cargo xtask record` does not read.

## 4. What the operator gains

**Under (A1): `obelix` becomes VISIBLE as one question** — two interfaces, their names and addresses, why the
engine will not decide, and a control that says what answering will do. **The run of engine-spine stories
giving the operator nothing ends here** (six: 6.5, 6.6, 6.7, 6.9, 6.11, 6.12). ⚠️ **They still cannot ANSWER
it** until 6.14b — say so on the screen, not only here.

## 5. Change Log

| when | what |
|---|---|
| 2026-09-25 | contexted on `dabe1f5` (two Explore passes: planning documents, and the triage/route/gate code); four findings; four decisions posed (§0.5) |

## References

- `_bmad-output/planning-artifacts/epics.md` story 6.14 and 6.13's note; `:264` (UX-DR21), `:431-433` (Epic 7)
- `_bmad-output/planning-artifacts/prd.md:896-897` (FR16, FR16b), `:985-1025` (glossary), `:1019` (`ambiguous`)
- `_bmad-output/planning-artifacts/ux-design-specification.md:766-769, 1174-1175, 1248-1250, 1264, 1332-1373`
- `_bmad-output/planning-artifacts/architecture.md` D14, D15 case A, D16, D21 `:1459-1460`
- `crates/opencmdb-bin/src/page.rs:565, 631, 719-765, 852, 880, 929-936, 953, 1030-1248, 1401, 5231, 5775, 5956, 6455`
- `crates/opencmdb-bin/src/identity_view.rs`, `l2_repo.rs:169`, `scan_pass.rs:739-819`, `document.rs`, `main.rs:726-786, 1812`
- `crates/opencmdb-bin/locales/app.yml:117, 145-147, 497-552`; `a11y/seed.sql`, `a11y/axe-gate.mjs`, `a11y/kbd-probe.mjs`
- Previous story: `6-12-resolver-writes-device-groupings.md`
