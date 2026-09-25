# Story 6.14: The ambiguity explains itself on the page

Status: **contexted, VALIDATED and ARBITRATED 2026-09-25 — `ready-for-dev`.** Guy took all six decisions on
the recommendation (§0.8); ⚠️ **development waits for the planning act (glossary row + 6.14b) to be MERGED**,
because AC8 renders the term that act binds. Both validation layers tried to refute §0.1–§0.3 and all
three survive; the gap-hunt BUILT the recommended shape and it changed two recommendations (D, and the action
bar) and added two decisions.

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
- ~~**Epic 7 owns no such gesture**~~ — ⚠️ *refuted in part by the fact-check*: Epic 7's GESTURE list
  (`epics.md:431-433`) has none, but its COVERAGE line (`:433`) carries **UX-DR14** (`:257`, *"`ambiguous`
  (Resolve badge replaces Document)"*) and **UX-DR43** (`:292`, *"ambiguity always routes to Resolve, never a
  blind document"*). So *Resolve replaces Document* is **Epic 7's by coverage**; **UX-DR21** (the Resolve
  panel, `:264`) is in NO epic's coverage line, and **UX-DR50** (*"[Resolve this pattern]"* on the reach
  line, `:301`) is **Epic 17's** (`:488`). Epic 6's own coverage lists no UX-DR, and FR16 is Epic 5's (`:415`).
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

### §0.3b — ⚠️ Criterion 2's premise is stale

It says the documenting gesture *"belongs to `AbsenceOfProof`"*. Story 6.4's re-aim (Guy, 2026-08-24) moved it
to **`Nouveau`**; an `AbsenceOfProof` cause line carries no gesture at all. This story reads criterion 2 with
that correction.

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

### §0.6 — 🔴 What the validation added (fact-check + a BUILT prototype, each with its own database)

**The prototype** (gap-hunt, uncommitted in its worktree): the split `page.rs` → `triage_view.rs`, a reader
`identity_link ⋈ interface`, Ambigu rows with a candidates pane, the C2 link from Nouveau rows, an end-to-end
test (FixtureConnector → `poll_ingest_resolve` → rendered `/triage`), the seed extended, and both browser gates
run on a booted binary. **Confirmed**: one production pass over two NICs sharing a name renders exactly ONE
Ambigu row whose id `ambigu:{low}:{high}` is stable across a second sweep with fresh ids and round-trips through
`?sel=` (curl); the join through `identity_link.interface_id` is required and exists in no view; the split must
come first.

1. 🔴 **EVERY EXISTING GUARD IS STRUCTURALLY BLIND TO `Ambigu`, and so are the tests the draft described.**
   `build_triage_offering` has **34 call sites**; adding an input through a wrapper that passes an EMPTY one
   leaves the four-kinds test, `no_other_kind_carries_a_live_gesture…`, `the_bar_shows_five_planned_controls…`
   and `the_resolve_gesture_cannot_go_live…` green without ever meeting an Ambigu row — **a Live documenting
   gesture on the Ambigu pane passes all four**. Measured: the whole prototype left the suite, clippy and ten
   gates green (only the stylesheet guard reddened, on a new class); **M1** (the pane shows the OLDEST sighting)
   and **M2** (OPERATOR rows shown) both GREEN, as predicted, under `--baseline`.
2. 🔴 **The pane knows only *Declared | Observed*** (`_triage.html` hard-wires both headings, `DetailPane` has
   only those fields). An ambiguity is TWO OBSERVED candidates; the cheap implementation puts candidate A under
   a **"Declared" heading** — a false heading no guard can see. It needs a candidates shape and, per 6b.4's
   rule, **per-candidate freshness**: a pair is re-judged only when BOTH interfaces are in the sweep, so a NIC
   gone for months keeps its row with stale data (the prototype rendered *"1045 d ago"*, which is what makes it
   visible).
3. 🔴 **One row per PAIR explodes, and *"obelix becomes ONE question"* was false as rows.** 2 NICs → 3 rows (2
   Nouveau + 1 Ambigu); **4 NICs → 10 rows, 6 of them Ambigu** (measured); 100 interfaces sharing a vendor
   default name → **4950** (6.12's measurement) — exactly FR16b's *"not N failures — ONE question"*. And the
   fact-check found the UX specification puts the L2 question on the REACH line as **one pattern with a
   gesture**, in the unit *devices* (`:1043-1044`, UX-DR50) — criterion 2's *"when **the line** is rendered"*
   plausibly means that line. 🔑 **Today an L2 ambiguity only arises from hostname SET EQUALITY, which is an
   equivalence relation**: the interfaces linked by current `Ambiguous` pairs form CLIQUES, so grouping them by
   connected component is EXACT, not the transitive fusion L1 refused. That changes (D).
4. **The action bar on an ambiguity renders *Accept the gap, Snooze, Attach, Exclude*** — `action_bar()` always
   appends Epic 7's four gap gestures — and the draft's AC3 was silent. ⚠️ The glossary's `attach`/*rattacher*
   — *"A link; no data moves"* — is arguably the POSITIVE answer; recorded for (A)'s planning act.
5. **The reach section contradicts the new row**, measured in rendered HTML: directly under the Ambigu pane it
   says *"Every sighting was placed."* and *"Grouping them is still to come."*
6. **Neither browser gate reaches the Ambigu pane even with the seed extended** — axe under all six CI flags:
   10 routes + 5 states, 0 nodes, **exit 0**; kbd-probe: 62 checks, **exit 0**, none on the Ambigu row. The
   seed deletes `identity_link` but NOT `interface` or `l2_pair_decision` (1 interface of residue, measured), so
   fixed ids collide on the re-seed kbd requires; `192.0.2.9` is already used twice by the seed. An
   `AXE_REQUIRE_AMBIGUOUS` flag can find the row by its id prefix `sel=ambigu:` — never a translated word.
7. **The collision of §0.2 already renders today** (fact-check): `gesture.not_built` (`app.yml:507-508`) says
   beside every planned bar *"still to come: resolve an ambiguity…"* — so a Conflit row's *Résoudre* already
   tells the operator it resolves an ambiguity. Story 6b.4's §0f recorded the divergence and nobody registered
   it. `gesture.resolve` is also in `state_vocabulary.rs:389`'s `NOT_A_GLOSSARY_GESTURE` (*"Epic 6 owns FR16's
   ranked candidates"*, arity `; 7]`), and `resolve` has a THIRD use: FR33's alert action (`prd.md:944`).
8. **Two register rows naming 6.14 were missing from the inheritance**: `deferred-work.md:5437` (evidence not
   de-duplicated — live for a display: a multi-homed observation's name appears on BOTH sides) and
   `:6488-6492` (**a flapping reverse-DNS answer churns L2 history** — the pair closes and re-opens, so **the
   Ambigu row appears and disappears**, contradicting the draft AC1's *"stable across sweeps"*).
9. Smaller: reuse the existing **`state.ambiguous`** key rather than mint `triage.kind.ambigu` (6b.6's one term,
   one key — and `one_word_is_rendered_by_one_key` iterates `ObjectState` only, so minting reds nothing);
   verdict sentences must be TOTAL over unknown tokens (`verdicts` has only a non-empty CHECK); `hostname_of`
   returns the short name; the FixtureConnector refuses a `Hostname` its capabilities do not declare, so the
   replacement tripwire needs 5.14b's `kinds()` widened; the interface↔observation join reads a table growing
   by about one row per host per sweep (≈13k/day, reasoned) — a per-interface latest-sighting read is the shape
   that scales (14.3a's precedent); (A2) also needs a non-empty `verdicts` and a `ruleset_version`; confidence
   (UX-DR21) is omitted because nothing produces it — say so; `count_engine_reach` lives in `repo.rs:1425`.
10. **The split**: `page.rs` 550–1359 → `triage_view.rs` gives 1148 / 812 code lines, green; it needs three
    `pub(crate)` and a re-export. ⚠️ Two hazards measured: a scripted cut left a doc-comment tail on the wrong
    side (caught only because it sat at end of file), and a `#[cfg(test)]` helper placed mid-file reddened
    `no_handler_pairs_a_status_with_an_untranslated_literal` through `code_half` truncation.

### §0.7 — The decisions, REVISED on §0.6

- **(A) unchanged — recommendation (A1)**, 6.14b inserted; add to its planning act that `attach`/*rattacher*
  may already be the positive answer's word.
- **(B) unchanged — (B1)**, and the planning act must also touch `gesture.not_built`'s sentence,
  `NOT_A_GLOSSARY_GESTURE`, and FR33's third *resolve*.
- **(C) unchanged — (C2)**, now also because *Resolve replaces Document* is **Epic 7's** by coverage (UX-DR43).
- **(D) REVISED — ONE Ambigu row per GROUP of interfaces linked by current `Ambiguous` pairs**, the pane listing
  every candidate, and ONE reach line counting those groups in the unit the UX specification uses (*machines
  possibly counted more than once* / devices), never summed with sightings. Exact today because hostname
  equality is an equivalence (§0.6 3); the day a non-transitive L2 rule can produce `Ambiguous`, the grouping
  must be re-examined — registered with the rule's story.
- **(E) NEW — the action bar on an Ambigu row: the Resolve control ALONE** (labelled), not Epic 7's four gap
  gestures, which answer a GAP and not a doubt. Recommendation: Resolve alone.
- **(F) NEW — flapping names.** A pair whose PTR answer flaps closes and re-opens, so the row comes and goes.
  Recommendation: show the CURRENT state and say so; the churn stays registered, owner re-set to the story that
  gives L2 a memory of past answers (or Epic 6's retrospective).

### §0.8 — ✅ GUY'S ARBITRATION, 2026-09-25 — all six on the recommendation

| | decision | refused, and why |
|---|---|---|
| **A** | **Display + a LABELLED Resolve control; the gesture goes to an INSERTED story 6.14b**, whose planning act first says what each answer writes (and whether `attach`/*rattacher* is already the positive answer's word) | (A2) half a gesture with a whole write route's cost; (A3) a device mint and a new `match` outcome inside a display story |
| **B** | **« Résoudre » / *Resolve* names the AMBIGUITY gesture** — a new glossary row, Guy's planning act — **and a Conflit row's primary becomes the documenting gesture at FIELD level** (`gesture.document`, `Planned`, owner **Epic 7**, FR13(b)): two sources disagreeing about one field are answered by declaring the field's value, which needs no new word. `gesture.not_built`'s sentence and `NOT_A_GLOSSARY_GESTURE` change in the same act | (B2) a new word for the ambiguity while the UX specification binds *Resolve* to it in six places |
| **C** | **Nouveau rows keep *Ajouter*** and say they belong to an open question, linking to it | removing the product's only live gesture from the machine on screen to put a control that does not act — and *Resolve replaces Document* is Epic 7's by coverage (UX-DR43) |
| **D** | **ONE Ambigu row per GROUP** of interfaces linked by current `Ambiguous` pairs, and ONE reach line counting groups | per pair: 10 rows for 4 NICs, 4950 for 100 same-named devices — FR16b's *"not N failures"* |
| **E** | **The Ambigu pane carries the Resolve control ALONE** | Epic 7's four gap gestures answer a gap, not a doubt |
| **F** | **Flapping names: the row follows the CURRENT state, said on the screen**; the churn stays registered | a memory of past answers is a new decision rule, not display |

🔑 **And the sequence, Guy's too**: **6.14 → the release (its IPAM notes are BLOCKING, action B2) → 6.14b**, so that
`obelix` is SEEN on the NAS and used before its gesture is written — the project's own rule, *use it before
writing more of it*.

---

## 1. Acceptance criteria — written for Guy's arbitration (§0.8)

⚠️ *Rewritten on §0.6/§0.7 for (A1)+(B1)+(C2)+(D revised)+(E)+(F).*

**AC1 — ONE `Ambigu` row per group of interfaces linked by current ENGINE `Ambiguous` pairs**, labelled from the
existing `state.ambiguous` key, id derived from the group's smallest interface id, stable across sweeps with
fresh observation ids while the answers are (F: when a name flaps the row follows the current state, said on
the screen). OPERATOR rows and `NoMatch` rows are NOT shown, each asserted by a test. 🔴 **The existing kind and
gesture guards are FED an ambiguity input** — the four-kinds test becomes five kinds, and
`no_other_kind_carries_a_live_gesture…`, `the_bar_shows_five_planned_controls…` and
`the_resolve_gesture_cannot_go_live…` each meet an Ambigu row; no wrapper passing an empty input is allowed to
stand in for them.

**AC2 — the pane shows EVERY candidate and the evidence, in a candidates shape**: no *Declared* heading over an
observed candidate; per candidate its hardware address, its current addresses and names and **its own
freshness** (6b.4's rule); the names **as currently observed** (decision F, said on the screen); one keyed
sentence per verdict, TOTAL over an unknown token. No raw token, no UUID. A **two-sweep test where a name or
address changes while the vector stays the same** is what separates *as seen now* from *as first seen* (the
validation's M1 was green without it).

**AC3 — the Ambigu pane carries the Resolve control ALONE (E), labelled and not acting** (`Gesture::Planned`,
owner 6.14b), with the sentence saying what it will do; not Epic 7's four gap gestures; the amber stays the
documenting gesture's. **No documenting gesture on the Ambigu row** (criterion 2, read as §0.3b).

**AC4 — the Nouveau rows of an ambiguous group keep *Ajouter*, and say they belong to an open question** (C2),
with ONE link to the group's Ambigu row (defined because the row is per group, not per pair).

**AC5 — the reach section gains ONE line for L2 questions, counted in GROUPS** (the UX specification's unit:
machines possibly counted more than once), never summed with sightings (5.14b's arbitration 10); and the two
sentences the prototype measured contradicting the row — *"Every sighting was placed."*, *"Grouping them is
still to come."* — are corrected. ⚠️ Criterion 3: **grouping does not exist** (no L2 `Match`), so the sighting
unit stays true for L1, and the story says so rather than renaming it.

**AC6 — 5.14b's tripwire is REPLACED, not deleted**: a production pass over two NICs sharing a name yields
exactly one Ambigu row on `/triage`; the old test's false doc is corrected.

**AC7 — both browser gates VISIT the Ambigu pane** (the validation measured both exiting 0 without doing so):
the seed deletes `interface` and `l2_pair_decision` before inserting, uses addresses it does not already use,
and creates two interfaces, their links and one `Ambiguous` pair; `AXE_REQUIRE_AMBIGUOUS=1` finds the row by
the id prefix `sel=ambigu:`, never by a word; the keyboard gate reaches the row and its labelled control; the
floor moves with the checks, prose and constant in one commit.

**AC8 — (B) in the code**: the Ambigu pane's control renders `gesture.resolve` (*Résoudre*/*Resolve*), bound by
the glossary row the planning act adds; a **Conflit row's primary becomes `gesture.document`**, `Planned`, owner
**7** — the `"gesture.resolve" => "6"` arm at `page.rs:727-730` goes; `gesture.not_built`'s sentence stops
promising *"resolve an ambiguity"* under a conflict; `NOT_A_GLOSSARY_GESTURE` (`state_vocabulary.rs:389`) loses
`gesture.resolve` now that a row binds it; `copy-vocabulary` green.

**AC9 — `page.rs` is split before it grows**, behaviour-neutral, in its own commit — with the two hazards the
prototype measured checked by name: no doc comment left on the wrong side of the cut (the compiler catches it
only at end of file), and no `#[cfg(test)]` item placed mid-file (it truncates `code_half`).

**AC10 — the join from interface to observations reads per-interface LATEST sightings**, not the whole
`identity_link` table per render (≈13k rows a day at the reference cadence, reasoned) — 14.3a's precedent.

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

**Under (A1) with (D) revised: `obelix` becomes VISIBLE as one question** — one Ambigu row per group, beside its
two Nouveau rows (the validation measured the per-pair shape at 10 rows for 4 NICs) — two interfaces, their names and addresses, why the
engine will not decide, and a control that says what answering will do. **The run of engine-spine stories
giving the operator nothing ends here** (six: 6.5, 6.6, 6.7, 6.9, 6.11, 6.12). ⚠️ **They still cannot ANSWER
it** until 6.14b — say so on the screen, not only here.

## 5. Change Log

| when | what |
|---|---|
| 2026-09-25 | contexted on `dabe1f5` (two Explore passes: planning documents, and the triage/route/gate code); four findings; four decisions posed (§0.5) |
| 2026-09-25 | validated by two fresh-context layers with their own databases — §0.1–§0.3 survive; the fact-check corrected the Epic 7 claim (UX-DR43 is Epic 7's by coverage), found the collision already rendering and two missing register rows; the gap-hunt BUILT the shape: every existing guard blind to `Ambigu`, a false *Declared* heading, per-pair rows exploding (10 rows for 4 NICs), the gates exiting 0 without visiting the pane. (D) revised to one row per group; (E) and (F) added (§0.7) |
| 2026-09-25 | **Guy's arbitration**: all six on the recommendation (§0.8); sequence 6.14 → release → 6.14b; the planning act (glossary row, 6.14b) is a separate PR |

## References

- `_bmad-output/planning-artifacts/epics.md` story 6.14 and 6.13's note; `:264` (UX-DR21), `:431-433` (Epic 7)
- `_bmad-output/planning-artifacts/prd.md:896-897` (FR16, FR16b), `:985-1025` (glossary), `:1019` (`ambiguous`)
- `_bmad-output/planning-artifacts/ux-design-specification.md:766-769, 1174-1175, 1248-1250, 1264, 1332-1373`
- `_bmad-output/planning-artifacts/architecture.md` D14, D15 case A, D16, D21 `:1459-1460`
- `crates/opencmdb-bin/src/page.rs:565, 631, 719-765, 852, 880, 929-936, 953, 1030-1248, 1401, 5231, 5775, 5956, 6455`
- `crates/opencmdb-bin/src/identity_view.rs`, `l2_repo.rs:169`, `scan_pass.rs:739-819`, `document.rs`, `main.rs:726-786, 1812`
- `crates/opencmdb-bin/locales/app.yml:117, 145-147, 497-552`; `a11y/seed.sql`, `a11y/axe-gate.mjs`, `a11y/kbd-probe.mjs`
- Previous story: `6-12-resolver-writes-device-groupings.md`
