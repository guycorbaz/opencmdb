---
date: '2026-08-13'
trigger: 'The reference mock (`~/travail/Projets actuels/opencmdb/documents/opencmdb - maquette.html`) became the interface reference, and the plan has no epic that builds it.'
decidedBy: Guy
scope: Moderate — backlog reorganization, no MVP change, no requirement added or removed
outcome: 'Epic 6b INSERTED between stories 6.3 and 6.4. Release v0.2.0 moves inside Epic 6.'
---

# Sprint Change Proposal — 2026-08-13

**The interface epic, and the release that ends it.**

---

## 1. Issue Summary

**The trigger is not a defect. It is an artefact that did not exist a day ago.**

On 2026-08-13 Guy produced a **reference mock** of the whole interface — ten screens, French, a complete
design system — and settled its status the same day: *"la maquette est la référence car je n'ai rien vu
d'autre pour l'instant."* Three divergences between it and the UX specification were arbitrated, and two
more were registered unarbitrated (see `ux-design-specification.md`, `editHistory` 2026-08-13).

**What the plan does not have is an epic that builds it.** The mock's ten screens map to **sixteen of the
seventeen remaining epics** — it is the shape of v1.0, not of an increment — so *"implement the mock"* is
not a step the plan can absorb anywhere. Meanwhile the product's own record says why this matters:

> Epics 1–3 = **21 stories and everything usable today**. Epics 4–5 = **43 stories and nothing
> operator-visible.** *(Epic 5 retrospective, 2026-08-12.)*

**Guy's decision (2026-08-13), in his words:** finish the epic in progress, put the new interface in place,
cut a release carrying that interface and the functionality already built — **then** choose what comes next.

**The evidence that the change is real and not a preference:**

| Fact | Measurement |
|---|---|
| The shipped UI | **two templates** (`gap.html`, `_gap_card.html`), one page, 5 routes, all read-only |
| The mock | **ten screens**, ~100 French UI strings, a 17 KB token sheet |
| The copy that exists | **32 i18n keys** in `locales/app.yml` |
| Epic 6's state | **zero code**; 6.1 `ready-for-dev` with a rewrite brief as its §0, 6.2–6.19 `backlog` |

---

## 2. Impact Analysis

### Epic impact

- **Epic 6 is SPLIT IN SEQUENCE, not in content.** No story of it is modified, added or removed. Stories
  6.1–6.3 (the write route, the adapter, the NFR5 guards) are **backend-only and touch no screen**; story
  **6.4 is a screen story** — the abstention line that carries the documenting gesture. Building 6.4 before
  the new interface means building it twice: once in today's card, once in the triage screen. It moves
  **after** Epic 6b. *(Guy's arbitration, 2026-08-13.)*
- **No epic is invalidated.** Epic 6b renders screens that later epics FILL: the Inventory it draws from an
  example dataset is the one Epic 6's L2 grouping feeds; Applications is Epic 15's; IPAM is Epic 14's;
  Alerts is Epic 16's. **Epic 6b buys their frame, never their content** — and the marker of story 6b.3 is
  what keeps that promise legible instead of implied.
- **Epic 7 gains a precondition it did not have**: its rich triage inbox now lands in a screen that exists.

### Artifact impact

| Artifact | Impact |
|---|---|
| **PRD** | **None.** No FR or NFR is added, removed or reinterpreted. NFR26 (EN + FR UI) and NFR25 (WCAG 2.1 AA on key views) are exercised earlier than planned, not changed. |
| **Epics** | One epic inserted (**6b**), one sequencing note on Epic 6. `epics.md` edited — permitted for a correct-course, forbidden to a story. |
| **UX spec** | Already updated on 2026-08-13 (three divergences settled, two registered). **One further correction is owed by this proposal**: dark mode. |
| **Architecture** | **None.** Askama + HTMX + Tailwind + `rust-embed` are unchanged; the mock's React runtime is not adopted, and D47's frontier is untouched — this is all `opencmdb-bin`. |
| **Docs** | The User Manual, `README.md`, the `gh-pages` landing site and `docker/README.dockerhub.md` all describe a one-page product. The house rule is docs-current-before-push; story 6b.12 carries it. |

### Two measurements taken during this analysis, both of which change the work

1. **🔴 "Dark mode deferred" is not deferring an unbuilt feature — it is turning off the only one that
   ships.** `gap.html` hardcodes `data-theme="dark"`, and `assets/app.css` carries **both** token sets
   (`#0f1420` dark, `#f6f7f9` light) with no switcher. Whoever runs `v0.1.1` today sees a dark product; the
   mock is light-only. **The release therefore changes the product's colour scheme, and the release notes
   must say so** rather than let a user discover it. The dark token set is kept in the sheet, unreferenced,
   so the day a dark reference exists it is a story and not an excavation.
2. **🔴 The example data must NEVER be written to the database.** `docker/seed-example.sql` is already a
   *shipped* writer of `declared_attribute` and story 5.12 had to name it as a third sanctioned site. If the
   demo screens seed rows, an example row becomes **indistinguishable from a real one at the storage
   layer** — the authorship gate gains a site, and the product's central promise (observed and declared are
   never confused) is broken by the very thing that was supposed to be visibly fake. **The example dataset
   lives in the handler/template layer, in code, and no demo screen opens a connection.**

---

## 3. Recommended Approach

**Option 1 — Direct adjustment. Viable, and selected.** Effort: **Medium**. Risk: **Low**.

*Rejected, and why:* **Rollback** (option 2) has nothing to roll back — Epic 6 has no code. **MVP review**
(option 3) is not triggered: no requirement changes, no scope is cut; only the ORDER in which the product
becomes visible does.

**The sequence:**

```
6.1 · 6.2 · 6.3          the write route, the adapter, the NFR5 guards   (no screen)
      ↓
EPIC 6b                  the interface: ten screens, the marker, the copy
      ↓
6.4                      the documenting gesture, landing in the triage screen
      ↓
RELEASE v0.2.0           story 6b.12
      ↓
6.5 … 6.19               L2 grouping — replaces Inventory's example data with real devices
```

**Why the release sits INSIDE Epic 6 rather than at its end:** the release's content is *the new interface
plus what is already built*. Waiting for 6.5–6.19 would hold it for fifteen stories of grouping work whose
only visible effect is to turn one example screen real. **The epic does not close at the release; it
continues under it** — and `sprint-status.yaml` says so rather than leaving a reader to infer it.

---

## 4. Detailed Change Proposals

### 4.1 — `epics.md`: insert Epic 6b

**The epic's body — goal, measured constraints, twelve stories with their acceptance criteria — is written
in `epics.md` and NOT duplicated here.** *(Deliberate: this project has been caught six consecutive times
by a fact stated in two documents where one went stale. A dated proposal and a living plan holding twelve
copies of the same ACs is that defect, pre-arranged.)* This section records the DECISION; `epics.md` carries
the work.

**Identity:** `Epic 6b`, on the house convention for insertions — stories 5.4b, 5.9b, 5.11b, 5.13b and 5.14b
were all inserted with a letter suffix rather than by renumbering their successors. **Renumbering seventeen
epics to make room for one is a change whose only product is churn.**

**Title:** *L'interface de la maquette* — the mock's interface.
**Goal:** the product looks like the reference mock, and **every screen says truthfully whether what it
shows comes from the product or from an example dataset.**
**FRs covered:** **none new.** It re-renders the surfaces of FR10, FR11, FR16 and FR16b, and it builds the
site where FR13(a) lands in story 6.4. **NFRs:** 25 (a11y on the key views), 26 (EN + FR).

**Twelve stories:**

| # | Title | One line |
|---|---|---|
| 6b.1 | The design system: tokens, typography, and the accent that stays reserved | The mock's palette and Barlow, with amber kept for the documenting gesture |
| 6b.2 | The shell: header, navigation, and ten routes | One URL per screen, deep-linkable — never a client-side switch |
| 6b.3 | The example-data marker, and the gate that keeps it honest | The story that makes "show all ten screens" safe |
| 6b.4 | The triage screen, on the real gap | Today's card becomes the mock's two-pane triage |
| 6b.5 | The dashboard: the real reach section beside labelled example sections | The mixed screen, and the rule that lets it be mixed |
| 6b.6 | Inventory and device record (example) | The frame Epic 6's grouping will fill |
| 6b.7 | Applications and IPAM (example) | The frames of Epics 15 and 14 |
| 6b.8 | Sources and alerts | Example, plus the source facts the product really holds |
| 6b.9 | Self-diagnostic and commissioning | Example, plus the version/migration/scan facts really held |
| 6b.10 | The copy: FR and EN, every string a key | ~100 mock strings land with their English twins |
| 6b.11 | The keyboard layer and the focus contract | Navigation keys yes; **no letters** — the spec forbids assigning them piecemeal |
| 6b.12 | The release v0.2.0 and the documents that describe it | Image, manuals, README, landing site, release notes that promise only what exists |

### 4.2 — `epics.md`: one sequencing note on Epic 6

Epic 6's decomposition note gains the split: 6.1–6.3, then Epic 6b, then 6.4, then the release, then
6.5–6.19. **No acceptance criterion of Epic 6 is touched.**

### 4.3 — `ux-design-specification.md`: the dark-mode correction

*Customization Strategy* states **"Dark mode is first-class from MVP"**. Guy's arbitration of 2026-08-13
defers it. **The sentence is corrected by decision, with the measurement above attached** — that today's
shipped page is dark and the new one is light — so that nobody later reads the deferral as an oversight and
nobody reads it as "we never had dark".

### 4.4 — `sprint-status.yaml`

Twelve `6b-*` entries at `backlog`, `epic-6b: backlog`, `epic-6b-retrospective: required` (mandatory since
2026-08-10 — an epic is not closed until the retrospective has run, and the project review follows it).

---

## 5. Implementation Handoff

**Scope: Moderate** — backlog reorganization, no replan.

| Recipient | Deliverable |
|---|---|
| **PO / DEV** | `epics.md` + `sprint-status.yaml` as amended by this proposal |
| **DEV (Amelia)** | Stories 6.1–6.3 first, under the normal cycle: `create-story` → **`create-story validate` (mandatory here)** → `dev-story` → `code-review` |
| **UX (Sally)** | The two unarbitrated divergences registered in the spec — the accent doctrine is settled, the second (dark) is settled by §4.3; nothing else is open |

**Success criteria for this change:**

1. Every one of the ten screens is reachable at its own URL and **states its own nature** — fed, or example.
2. **No demo path opens a database connection**, and a test says so.
3. Every new string exists in **both** locales.
4. The release notes name what the release does **not** do — the gestures Epic 7 owns, and the colour change.

**⚠️ The risk this proposal carries and does not resolve:** ten screens of which eight are examples is a
product that *looks* far more finished than it is. The marker of story 6b.3 is the whole defence, and it is
a defence against a misreading by the person who installs it — **not** against the one who builds it. The
question *"is this screen fed?"* has to stay askable of every screen, every day, by a test rather than by
memory.
