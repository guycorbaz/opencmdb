# Story 6b.7: Applications and IPAM (example)

Status: done

Epic: 6b — *L'interface de la maquette*. **Seventh numbered slot, eighth story file** (6b.4b was
inserted at 6b.4's validation). It takes the `Empty` screen count from six to four.

⚠️ **THIS FILE WAS REWRITTEN AFTER ITS VALIDATION PASS (2026-08-20) AND THE REFUTATIONS ARE KEPT IN
WRITING.** Two fresh-context layers ran: one checked every claim against its sources, one **built both
screens end to end in a scratch copy and ran twenty mutations against them**. Between them they
refuted **eleven** claims of the first draft, three of which would have shipped a screen with no guard
over it. The draft is preserved in the session scratchpad; what follows is the corrected story, and
where a correction matters the refuted sentence is quoted rather than quietly replaced.

## Story

As the operator,
I want the application inventory and the subnet occupancy to exist as screens,
so that Epics 14 and 15 have a frame rather than a blank page.

## Acceptance Criteria

Transcribed from `epics.md:2234-2242`, **unmodified** — divergences are raised in §0 (a story may not
edit an AC; only a retrospective or a correct-course may). Verified verbatim by the fact-check layer.

1. **Given** the IPAM screen, **when** the occupancy grid renders, **then** it is **CSS Grid with an
   `aria-label` per cell**, and its cells come from **explicit example data, never from a computed
   pattern**. ⚠️ The mock fills them with `(i * 37) % 256 < used`: **a fake that varies is a fake no
   test can pin and no screenshot can compare.**
2. **Given** the applications screen, **when** it renders, **then** owner and criticality are shown
   as **declared and unobservable** — *nothing will ever observe them*, which is the mock's own
   sentence and the reason the screen exists.
3. **And** both carry the marker of 6b.3.

**Added by contexting and CORRECTED by the validation** (numbered from 4 so the three above keep the
epic's numbering):

4. **Given** the vocabulary these two screens introduce, **when** it is written, **then** every noun
   **and every value set** is checked against the canonical glossary, and anything the glossary does
   not carry is **REGISTERED rather than introduced** — story 6b.6's AC2 applied to the five nouns and
   the three value sets §0a names. ⚠️ *A story may not extend a binding artefact; Guy did that himself
   on 2026-08-19 and the same rule holds here.*
5. 🔴 **REWRITTEN — the first draft's AC5 was measured UNSATISFIABLE.** It read *"the marker partition
   and the body-serves-what-it-declares guard cover them **without either being edited to accommodate
   them**"*, and the two new variants force `E0004` in the guard itself. What is required instead:
   **every guard this story edits is edited to become a PROPERTY, never to accommodate two more
   entries**, and the three edits are named in T3 rather than discovered.
6. 🔴 **NEW, and it is the validation's largest single finding.** Three guards the draft listed as
   *"will red"* were **measured GREEN** on the two new screens (§0g). This story does not ship until
   each is a property over `Screen::ALL` or over the dataset's key-bearing fields, **with the
   mutation that reds it recorded**.
7. **Given** the whole delivery, **when** `cargo xtask ci`, `cargo clippy --workspace --locked -- -D
   warnings` and `cargo fmt --all --check` run, **then** all eight gates are green and the suite is
   run **both ways** — without `DATABASE_URL` and against a live `mariadb:10.11` — with both
   wall-clocks recorded, the clock being the tell that the database-backed tests genuinely executed.
8. **And** the story's live test count is stated **in this file** and nowhere else, and the two
   screens are **looked at in a browser** before the story is called done. 🔴 The validation's browser
   pass already found one defect no guard can see (§0h).

---

## §0 — What contexting found, and what the validation measured

🔑 **This story's centre of gravity is NOT the pixels.** AC1 and AC3 are example surfaces on a
mechanism three stories have already built. What is new is that **this story is required by its own
epic to render vocabulary the previous story deliberately refused and registered** (§0a), that **the
guards it inherits do not see it** (§0g), and that **its central accessibility requirement is carried
by nothing** (§0f).

### §0a. 🔴 AC2 IS THE REGISTERED DEBT OF STORY 6b.6, AND THE REGISTER NAMES A DIFFERENT OWNER

`deferred-work.md:4267`, registered by story 6b.6 on 2026-08-19 — quoted exactly, owner included, and
verified verbatim by the fact-check layer:

> ⚠️ **`criticality` is in the mock's *Hosted here* block and was deliberately NOT rendered.** It
> would introduce a third vocabulary axis (critical / normal / low) with no glossary row and no
> producer, which is exactly what AC2 says to register rather than introduce. Same for `app` and
> `owner`. **Owner: the story that gives the product containment data** (Epic 6 / Growth).

The same refusal is in the code, at `example_data.rs:141`, also verified verbatim. 🔴 **And
`epics.md:2240` — this story's own AC2 — requires exactly what that register refused.** The register's
owner assignment is falsified by the plan it was written under; correcting that row belongs with this
story whichever way the arbitration goes.

#### 🔴 THE DRAFT'S CRITERION WAS SELF-CONTRADICTORY, AND THE CORRECTION ENLARGES THE QUESTION

The draft cleared `host` — *"it is FR28's `hosts` containment, already in the plan's vocabulary"* —
and condemned `exposure` as *"a FOURTH axis"*. **`prd.md:921` names `exposes` in the same clause as
`hosts`**, so that criterion clears `exposure` too. And FR27 (`prd.md:919`) names *applications*,
*owner* and *criticality* verbatim, so *"in the plan's vocabulary"* discriminates nothing at all.

🔑 **The real discriminator is BINDING-GLOSSARY MEMBERSHIP, and on it all five fail — `host`
included.** The question to arbitrate is therefore **five nouns on one criterion**, not four bad ones
beside a good one.

| Noun on the screen | In a binding table? | What it would introduce |
|---|---|---|
| **application** | ❌ | The object FR27 groups software into. An entity noun — neither gesture nor state. |
| **owner** / *propriétaire* | ❌ | A person-shaped attribute (FR27, FR40). ⚠️ **And its VALUES are a vocabulary too** — see below. |
| **criticality** / *criticité* | ❌ | 🔴 An axis **with its own value set**. |
| **exposure** / *exposition* | ❌ | 🔴 A second value-bearing axis, **including one value that collides with a shipped concept**. |
| **host** / *hôte* | ❌ | Cleared by the draft on a criterion that does not hold. FR28 names it; the glossary does not. |

**The value sets, measured against the mock's own `APPS` array — both counts in the draft were wrong:**

- **criticality ∈ { Critique, Élevée, Moyenne, `Faible` }** — **four**, not the three the draft named.
- **exposure ∈ { Interne, Reverse proxy, `Publiée · 443`, `Hors périmètre` }** — **four**. The draft
  split one value across its own `·` separator and **omitted `Hors périmètre` entirely**. 🔴 That is
  the one value which collides with a concept the product already ships: the scan perimeter
  (`OPENCMDB_SCAN_CIDR`, and story 5.14b's *"out-of-perimeter sightings"*).
- **owner ∈ { Direction, Prestataire IT, Comptabilité, Atelier, Marketing }** — 🔴 **a fifth axis the
  draft did not see at all**, because it listed `owner` beside `name` and `host` as plain data. The
  gap-hunt measured the consequence: with the owners as literals, **`/apps` renders five French words
  inside the English UI and the whole suite stays green** (mutation M-I). *A literal is not a key, and
  the locale guard can only see keys* — story 6b.6's `role_key` defect, verbatim, one story later.

#### ⚠️ THE TWO "BINDING" MIRRORS ARE NOT IDENTICAL, AND THAT IS A FINDING OF ITS OWN

`ux-design-specification.md` carries **eleven** gesture rows; `prd.md` carries **ten**. The UX spec has
a row the PRD lacks — `| Attach a discovery to an existing record | attach | rattacher | A link; no
data moves |` — which in the PRD survives only inside the `triage` row's meaning cell and in FR14.

🔴 **So for a story whose AC4 is *"checked against the canonical glossary"*, there is no single
glossary: there are two mirrors that have drifted.** This is not this story's to fix. **Registered,
with the drift measured, owner Epic 6b's retrospective.** (The draft wrote *"eleven rows"* for the
PRD, which is how it failed to notice.)

⚠️ **And a METHOD defect the validation named, which matters more than the count**: the draft measured
the glossary against **one** document. The conclusion survives — all five nouns are absent from
`prd.md`, from `ux-design-specification.md` **and** from the user manual's appendix, verified by a
whole-tree search — but *an enumeration cannot establish an absence* is this project's rule since
5.13b, and 6b.6's headline defect was that exact gesture. **Reproduced one story later, in the story
that cites it.**

#### ✅ GUY'S ARBITRATION, 2026-08-20 — option (c), with the value question answered in the same breath

**Taken: (c) — render only what AC2 NAMES, register the rest.** The screen carries **owner** and
**criticality**; **`exposure` is NOT rendered**; the five nouns and the three value sets are registered.
🔑 *The story introduces nothing its criterion did not ask for, and the question that remains for the
plan is smaller and better posed than the one this story would have forced.*

**Refused, and recorded with the reason** — a house rule, not a formality:

- **(a) extending the binding table with an object axis** was refused **for now**, on the cost the
  validation measured: it is not five rows but five rows **plus three value sets**, and a value set is
  a *scale*, not a term — a new KIND of row in a table whose every row reads *one concept, one
  translation*. ⚠️ Refused as **premature, never as wrong**: the day Epic 15 gives these nouns a
  producer, (a) is the closure, and this paragraph is what the story owes it.
- **(b) render all five and register** was refused because it introduces `exposure` — a whole axis with
  four values — that no criterion asks for. *An AC is a floor for what must ship, not a licence for
  what may.*

**And the values (Guy, same arbitration): criticality is an i18n KEY, exposure would be a KEY, owner
values are DATA.** The reason is not stylistic: criticality and exposure are a **closed classification
the product will one day compute**, so they are copy the operator reads in their own language; an
owner is a **proper noun**, like a device's `name`. The gap-hunt measured what the wrong answer costs:
with the owners as literals, `/apps` renders **five French words inside the English UI with the whole
suite green** (mutation M-I) — story 6b.6's `role_key` defect verbatim.

⚠️ **The exposure half of that answer is recorded for a screen this story does not ship.** It costs
nothing today and it means Epic 15 inherits a decision instead of re-opening one. **It must not be read
as licence to render `exposure` here.**

🔴 **`host` STAYS ON THE SCREEN, and this is the one place the arbitration needs reading rather than
applying.** It is one of the five nouns with no glossary row, and (c) says *render what the AC names*.
But an application that runs nowhere is not an application: the host is FR28's containment, it is the
mock's second column, and the device record already renders *Hosted here* from the same relation.
**Dropping it would gut the screen to satisfy a rule about vocabulary.** So: rendered, and **registered
with the other four** — the register row is what carries it, not a silence.

### §0b. 🔴 `conflit` MEANS TWO DIFFERENT THINGS ON TWO SCREENS, AND THE TABLE FORBIDS EXACTLY THAT

The mock's IPAM side panel carries a *"Conflit d'adresse"* block: *"Deux appareils répondent"* — FR24,
*"same IP on two MACs"*. The binding table's state axis (`prd.md:1016`, identical at
`ux-design-specification.md:1365`) says:

> | Two observations disagree **with each other** | **conflict** | **conflit** | Source against
> source, **not observed against declared** […] |

🔴 Two different facts. *Source against source* is one field, two sources, two values. *Two appliances
answering on one address* is a fact about the NETWORK. And `prd.md:988` is explicit: *"Neither
language carries two meanings for one word."*

✅ **The gap-hunt measured that nothing in the codebase can see this**: `one_word_is_rendered_by_one_key`
compares `ObjectState` keys only, and planting `fr: "Conflit"` on a new IPAM key left the suite
**green** (mutation M-K). So the collision is real, undetectable, and must be decided rather than
discovered.

✅ **GUY'S ARBITRATION, 2026-08-20: qualify and register.** The panel is titled with the mock's own
phrase — **« Conflit d'adresse »** — on the precedent of *"Écart · 2 champs"*, where a qualifier
disambiguates **without minting a term**; and the collision is **registered** beside the
`Nouveau`/`undeclared` one (`deferred-work.md:4260`, owner Epic 6b's retrospective).

**Refused, with the reason**: adding a binding row for the address conflict — right on the merits, and
**a planning act that does not wait on this story**; and dropping the word for FR24's own phrasing —
which loses the mock's copy to buy a distinction the qualifier already makes. 🔑 *The qualifier is the
cheapest true answer, and it is the one the product already uses on its busiest screen.*

⚠️ **§0b arbitrates the WORD and the gap-hunt found the DATA is also unhandled**: the mock's conflict
panel names **`PC-COMPTA-02`, a ninth device that is not in the example dataset**, at `192.168.10.41`.
T1 imposes app→device coherence and the draft imposed none for IPAM. Note what the dataset already
offers: `fw-edge` carries `ObjectState::Conflict`, and `vm-billing` sits at the mock's own conflict
octet. **Use two devices that exist, at an address the grid really draws as used.**

### §0c. ⚠️ AC1's `aria-label` PER CELL AND THE MOCK'S `role="img"` ARE INCOMPATIBLE — AND SHAPE (i) IS NOW MEASURED

The mock has **one** `aria-label`, on a `role="img"` container, and a `title` per cell. AC1 requires an
`aria-label` **per cell** — a deliberate divergence from the mock, and the right one.

🔴 **Under `role="img"` the 256 labels are swallowed**: ARIA `img` is *Children Presentational: True*.
⚠️ **This is REASONED, not measured** — the gap-hunt tried and reports that Chrome's devtools tree
still shows the children, because the pruning is a platform/AT mapping and not visible over CDP. *Say
so rather than claiming a measurement.* NFR25 is this epic's DoD, and **UX-DR71 (`epics.md:316`) names
the occupancy grid BY NAME as one of five WCAG-2.1-AA key views.**

**What the gap-hunt DID measure, in Chrome 151 over `Accessibility.getFullAXTree`, on four built shapes:**

```
list name=''
  listitem name='192.0.2.1 - utilisée'     ← (i) <ul> + CSS Grid + aria-label per <li>   ✅ name survives
  listitem name='192.0.2.2 - libre'
generic name=''
  generic name='192.0.2.1 - utilisée'      ← (iii) bare <div aria-label>                 ⚠️ see below
```

- ✅ **Recommendation (i) is CONFIRMED**: `list` → `listitem` with the accessible name intact, and
  `display: grid` + `list-style: none` did **not** cost the list role in Chrome. ⚠️ Safari/VoiceOver is
  known to drop `role=list` under `list-style: none` and was not testable here — **add an explicit
  `role="list"`**, which the gap-hunt measured costs nothing.
- 🔴 **A stronger argument against the naive shape than the draft had**: `aria-label` on a bare `<div>`
  maps to `generic`, and `aria-label` on `generic` is **prohibited by ARIA 1.2** — an axe-core
  `aria-prohibited-attr` violation, not merely a keyboard inconvenience.
- 🔴 **The draft's option (ii) — a table with a `<th>` per row — does NOT satisfy AC1**, which requires
  a label per CELL. It is withdrawn.
- 🔴 **A third requirement neither the draft nor the mock's colours cover**: the mock encodes *reserved*
  as a **hatched pattern**, not a flat grey. The gap-hunt's flat-colour build put reserved and free at
  near-identical greys — **three states distinguished by colour alone is WCAG 1.4.1**. Keep the
  pattern.

### §0d. ⚠️ "EXPLICIT EXAMPLE DATA" — MEASURED, AND THE DRAFT WORRIED ABOUT THE WRONG HALF

The mock's formula, verbatim: `const used = (i * 37) % 256 < sub.used;`. The ban is on a **fake that
varies**, not on deriving 256 cells from a committed list. **The shape prescribed**: per subnet, two
committed lists of host octets, and the cell's state is a **membership lookup**, never an arithmetic
predicate.

🔴 **The draft called ~110 literals for one subnet *"verbose but honest"* and offered to shrink the
mock's numbers. Measured, that worry was misplaced by an order of magnitude**: the gap-hunt committed
all three subnets — **175 octet literals — and after `cargo fmt` they occupy 16 lines**; the 96
literals of the first subnet are **6 lines**. `example_data.rs` goes 775 → 919 of 2000. **Commit the
mock's numbers; do not shrink them.** What actually costs lines is the apps table (~70).

🔴 **AND `page.rs` IS NOT "the file the `file-size` gate is closest to"** — a false sentence of the
draft's, measured by the gate's own counter: **`xtask/src/main.rs` is at 1908/2000**, `page.rs` at
1524. The advice built on it was misdirected; the file to keep an eye on is xtask's.

🔴 **A DEFECT INHERITED FROM THE MOCK, FOUND BY BUILDING IT**: a /24 has **256 addresses and 254
hosts**. The gap-hunt's build rendered `.0` and `.255` as ordinary **free** cells and announced *"next
free address: 192.0.2.0"* — the network address. The mock has the same bug (`for i = 0..256`,
`findIndex(transparent)`). ⚠️ **And the draft's own T1 prescribed a test asserting the counts *"sum to
256"*, which would have PINNED the defect as expected behaviour** — *a test that pins the ugly thing
is a test that requires it* (6b.4). **The honest denominator is 254, or `.0`/`.255` get a fourth
state.** Decide it; do not inherit it.

⚠️ **Two representations of one fact, pinned by an equality test**: the occupancy line is `count()`
over the same data the grid renders, never independent scalars. The mock computes
`256 - used - reserved` from its own numbers and **can disagree with the cells it drew** — confirmed
by the fact-check layer reading the mock.

### §0e. FIVE INHERITED MECHANISMS — three confirmed, two corrected

1. ✅ **`Nature::Example(ExampleContent)`** — CONFIRMED exactly: two new variants force **two `E0004`
   sites**, one production (`ExampleContent::render`) and one test (`main.rs:976`). ⚠️ The test one is
   `#[cfg(test)]`, so `cargo build` alone shows only the first.
2. 🔴 **CORRECTED — the draft was wrong about which marker rule applies.** It said the per-section
   guard *"counts per section"*. For a screen whose nature is `Example` (both of these), the guard
   asserts **exactly ONE marker for the whole page**; per-section counting applies only to the `Mixed`
   dashboard. Read literally, the draft pointed the developer at 6b.6's *"four identical banners"*
   defect. ⚠️ **And per §0g the guard cannot see these screens at all.**
3. ⚠️ **`ScreenQuery`** — the subnet selector is a **link with a query parameter**, server-rendered.
   🔴 The draft justified this with *"this screen is on the pool-free `Router<()>`"*, which is a false
   reason — nothing about `Router<()>` bears on form controls. The true reasons are that a link is
   deep-linkable and needs no JavaScript. ⚠️ Adding the field breaks **three** existing struct literals
   with `E0063` (compiler-carried, one line each), and `/devices?subnet=x` **already** parses-and-ignores
   today, so nothing changes there.
4. ⚠️ **`Gesture::Planned { owner }` — BUILT by the gap-hunt: it works, all 653 tests green, no 6b.4b
   guard broken, ~15 lines.** So reuse beats copying. Four costs the draft did not name, all measured:
   `action_bar()` is hardcoded to the mock's five triage controls and cannot yield one, so a **new
   function in `page.rs`** is required; `_action_bar.html` reads `s.gesture_badge` and
   `s.gesture_not_built` off the CALLER's strings struct, so `IpamStrings` must resolve both — a second
   reader of one fact, and **Askama warns about nothing**; the partial emits a fixed DOM id
   (`id="gesture-not-built"`), a duplicate the day two bars share a page; and 6b.4b's render guards
   read `rendered_triage_body()` only, so a native `disabled` in the IPAM bar would be invisible.
   ⚠️ **`epics.md:2092`'s premise (2) speaks of *implemented* screens and the four gestures Epic 7
   owns**; applying it to an example screen and an Epic 14 gesture is an extension — defensible, but
   it is a decision to write down. And the gesture is **FR21** (manage subnets), not FR23 (find a free
   address), which is the panel above it.
5. ✅ **Absent values are `Option`, never a `"—"` literal** — CONFIRMED, `ExampleSighting::mac` is the
   precedent. The mock's *"non évalué"* observed version is such an absence. 🔴 And so is the eighth
   app: **`Site vitrine` is hosted OUTSIDE the perimeter, on no device at all** — the row of which
   *"declared and unobservable"* is most true. The draft's T1 required every app's host to name a
   device, which that row falsifies. **The host is an `Option` too.**

### §0f. ⚠️ AC1'S CENTRAL REQUIREMENT IS CARRIED BY NOTHING, AND THE EPIC'S a11y GATE DOES NOT EXIST

Measured by the gap-hunt: `rg -i 'axe-core|a11y|accessib|wcag'` over `crates/`, `xtask/` and
`.github/` returns **four hits, all prose comments, zero code**. There is no axe-core, no headless
browser check, and **not one test in this repository asserts on an `aria-*` attribute** — while
`epics.md:2108` makes axe-core on the ten routes this epic's Definition of Done.

| mutation | result |
|---|---|
| **M-G**: every cell's `aria-label` replaced by `title` (the mock's own spelling) | 🔴 **GREEN — 0 red** |

**So AC1's *"an `aria-label` per cell"* would ship guarded by nothing.** This story owes a test that
reds on M-G. ⚠️ The epic's axe-core gate stays owed by the epic (6b.11 / 6b.12) and is **not** this
story's to build — but a story may not leave its own criterion carried by nothing.

### §0g. 🔴 THE VALIDATION'S LARGEST FINDING: THREE INHERITED GUARDS ARE MEASURED GREEN HERE

The draft listed these under *"Will red (already written)"*. **Each is an ENUMERATION of screens, not
a property**, and each was measured with a CONTROL that reds — which is what makes the measurement
decisive.

| Guard | Mutation planted on the new screens | Result | Control |
|---|---|---|---|
| `every_example_section_is_covered_by_exactly_one_marker` (`page.rs:3791`) — loops a hardcoded `[("/devices", …), ("/devices/{id}", …)]` + the dashboard | `_ipam_example.html` loses `class="example-section"` | 🔴 **GREEN** | the same on `_devices_example.html` → **RED** |
| same | a second `example-marker-badge` planted in the apps body (6b.6's *"four banners"* defect) | 🔴 **GREEN** | as above → **RED** |
| `no_i18n_key_reaches_the_screen` (`example_screens.rs:735`) — `pages` is `/devices` + the eight records + the unknown page | `criticality_key: "apps.criticality.bogus"`, a key that does not exist | 🔴 **GREEN**, and `/apps` renders the literal key name | `role_key: "devices.role.bogus"` → **RED ×2** |
| `the_example_copy_is_translated_rather_than_typed` (`example_data.rs`) — iterates **two** fields | a new key-bearing field on the app rows | 🔴 **GREEN** | — it already misses five existing key-bearing fields |

🔑 **None of the three can be "extended".** Each must become a property — over `Screen::ALL` for the
first two, over the dataset's key-bearing fields for the third — **and that is AC6.** ⚠️ The third one
is the sharpest: it *reads* as strong coverage (it is the first thing to red in the control) and it is
an enumeration of two fields out of seven.

⚠️ **The draft also listed `no_screen_renders_a_key_name_as_a_label` (`screens.rs:593`)**, which
renders `shell_html(Screen::Triage)` and checks nav labels only — it never sees a screen body. And
`every_variant_of_a_navigated_enum_is_listed_in_all` covers `Screen`, `NavGroup`, `ObjectState` and
`DeviceKind`, **not `ExampleContent`**, which has no `ALL`: nothing this story does can red it.

#### 🔴 The witness guard works, and its witness is guarded by nothing

| mutation | result |
|---|---|
| `/ipam`'s witness set to `/apps`'s witness | **RED** (1 test, assertion-carried) |
| the two render arms swapped, correct witnesses | **RED** — the partition is the SOLE carrier |
| **both witnesses set to `t!("example.badge")`** — a string the dispatch prepends to *every* example body | 🔴 **GREEN — 0 red** |

Any string from the marker satisfies every witness for all time. §0e(1)'s *"distinctive to that
screen"* is advice; **nothing enforces it**. ⚠️ The gap-hunt's own driver defect proves how much rides
on this: a `python .replace()` silently did nothing because `cargo fmt` had reflowed the anchor line,
**a stale placeholder body was served, and `cargo fmt`, clippy, all eight gates, the marker partition,
the per-section marker guard, the stylesheet guard and both i18n guards were GREEN** — only the
witness assertion caught it.

#### 🔴 The third forced edit, which the draft named nowhere

`main.rs:1023` carries a hardcoded `assert_eq!(example_contents.len(), 2, …)` whose message reads
*"a third is a screen that grew example content without a story deciding it should"*. This story makes
it **4**, and the message would then assert the opposite of the truth. **Change the number AND the
sentence** — 6b.6's own record warns that bumping one without the other *"leaves a false explanation
standing over a true count"*.

### §0h. 🔴 EPIC CONSTRAINT 5 IS LIVE HERE, AND THE DRY GESTURE IS WHAT BREAKS IT

`every_class_a_template_names_is_defined_in_the_stylesheet` **skips any `class="…"` containing `{`**.

| mutation | result |
|---|---|
| a static literal class with no rule (`ipam-nostyle`) | **RED** (control) |
| `class="ipam-cell cell-{{ cell.modifier }}"` with no `.cell-*` rule | 🔴 **GREEN** |
| delete `.ipam-cell-used`, legend written as three **literal** `<span class="ipam-cell ipam-cell-used">` | **RED** |
| **delete `.ipam-cell-used`, legend rendered from the same `CellView` data** (the DRY shape) | 🔴 **GREEN — 0 red** |

🔑 **The coverage comes entirely from the legend duplicating the three modifier names as literals.**
Write the legend the DRY way — one source for the three states, which is what this codebase's rules
ask for FIRST — and the used cells ship **with no colour**, on a screen whose entire content is a
visual distinction. That is `spark-h8` (6b.5) reproduced, **caused by the tidy gesture**. The legend's
three literals are therefore a **deliberate redundancy**: label them as such, and pin them with a test,
which is this codebase's sanctioned form (`CLAUDE.md`, DRY rule).

#### And a rule that EXISTS and renders nothing — found only in the browser

The gap-hunt looked at its build in Chrome 151 at 1280 px: **the legend's three colour swatches do not
render.** `.ipam-cell { aspect-ratio: 1; display: inline-block }` on an empty `<span>` collapses to
zero width, so the legend reads *"used reserved free"* with no chips — and the legend is the only thing
explaining the grid's colours. **Every guard green: the class is used, the rule exists.** This is the
stylesheet guard's own written limit live on this story's central screen.

### §0i. ⚠️ WHAT THE MOCK'S IPAM CARRIES THAT THE DRAFT'S T2 DID NOT

- A **two-column layout** (`grid-template-columns: minmax(0,1fr) 320px`) with a right-hand rail.
- The free-address panel uses the mock's `.blueprint` + four `.corner` decorations — **which exist in
  no template today**.
- The occupancy line sits **inside the legend row** (`margin-left: auto`), not under the grid.
- The conflict panel ends with a link to `/triage` (*"Trancher dans le triage"*).

---

## Tasks / Subtasks

- [x] **T0 — ✅ DISCHARGED 2026-08-20. Guy arbitrated all three; development is UNBLOCKED (AC: 2, 4)**
  - [x] §0a: **option (c)** — render owner and criticality, **do not render `exposure`**, register the
        five nouns and the three value sets. (a) and (b) refused with their reasons, in §0a.
  - [x] §0a: **criticality = i18n key · exposure = key (recorded for Epic 15, not rendered here) ·
        owner values = data.**
  - [x] §0a: **`host` stays rendered and is registered with the other four** — see §0a's last
        paragraph for why this needs reading rather than applying.
  - [x] §0b: **« Conflit d'adresse »**, qualified and registered.
  - [x] ⚠️ **No binding artefact was edited**: option (a) is refused as premature, so `prd.md` and
        `ux-design-specification.md` are untouched by this story and by Guy.
- [x] **T1 — The example data (AC: 1, 2)**
  - [x] `ExampleApp`, **fixed by T0's arbitration**: `name` (data — a proper noun), `host:
        Option<&'static str>` (a device slug; `Site vitrine` is hosted outside the perimeter),
        `declared_version`, `observed_version: Option<…>` (*"non évalué"* is an absence),
        `owner` (**data**, a proper noun), `criticality_key` (**an i18n KEY**).
        🔴 **No `exposure` field** — option (b) was refused; adding one is out of scope.
  - [x] The eight rows, **re-hosted onto slugs that exist** — only `NAS-01` maps today; seven of eight
        must be re-pointed at `nas-01`, `switch-core`, `vm-billing`, `ct-registry`, `srv-app-02`, …
  - [x] A test: every app's host that IS `Some` names a device the inventory ships.
  - [x] `ExampleSubnet` — three subnets, the mock's counts, **committed octet lists** (~16 lines).
  - [x] 🔴 Decide `.0` / `.255` (§0d) and write the decision at the site. **Do NOT assert "sums to
        256"** unless that decision makes it true.
  - [x] A test: the occupancy line's numbers are `count()`s over the same lists the grid renders.
- [x] **T2 — The two screen bodies (AC: 1, 2, 3)**
  - [x] `apps_body` / `ipam_body` in `example_screens.rs`, each with its own strings struct, **keys not
        literals** — including the value sets if T0 makes them keys.
  - [x] `_apps_example.html`: **six columns** — application · host · declared version · observed
        version · owner · criticality (the mock's seventh, *exposition*, is **not rendered**, T0) — and
        **AC2's sentence on the screen**, not only in this file. ⚠️ The mock's sentence names exposure
        in neither half, so it survives the drop unchanged: *"La version observée vient de l'hôte ; le
        propriétaire et la criticité sont déclarés — rien ne les observera jamais."*
  - [x] `_ipam_example.html`: the subnet selector (links), the grid as **`<ul role="list">` + CSS Grid,
        one `aria-label` per `<li>`** (§0c), the **hatched** reserved pattern, the legend with its three
        literal classes (§0h), the occupancy line inside the legend row, the two panels, the two-column
        layout, the `/triage` link.
  - [x] The next-free-address derived from the same lists, and a test pinning that the address it names
        is one the grid draws as free **and is a host address**.
  - [x] The conflict panel titled **« Conflit d'adresse »** (T0), on **two devices that exist**, at an
        address the grid draws as used.
  - [x] The planned *Réserver* control per §0e(4) — or the written decision not to render it.
  - [x] 🔴 Decide the unrecognised-`?subnet=` policy and make it **match the sibling screen**:
        `inventory_body`'s own doc says an unrecognised `kind` narrows to nothing, *"a filter that
        ignores its input is the shape story 6b.4's review caught"*. The gap-hunt measured its build
        silently serving the first subnet — two screens, opposite policies for one gesture.
- [x] **T3 — Wire the two screens, and the THREE forced edits (AC: 3, 5)**
  - [x] Two `ExampleContent` variants; both natures flipped; the two `render` arms.
  - [x] `main.rs:976` — two witness arms, each naming a string **distinctive to that screen**, and a
        guard or a comment that says why a marker string is not one (§0g).
  - [x] `main.rs:1023` — the count **and its sentence**.
  - [x] ✅ The partition's `probed` count is derived from `Screen::ALL` and follows automatically —
        confirmed by measurement, no edit needed.
- [x] **T4 — Turn the three enumerations into properties (AC: 6)** — 🔴 *the story does not ship
      without this*
  - [x] The per-page/per-section marker guard: a property over `Screen::ALL`, keyed on the nature.
  - [x] `no_i18n_key_reaches_the_screen`: a property over every `Example` screen's real HTTP body.
  - [x] `the_example_copy_is_translated_rather_than_typed`: over **every** key-bearing field of the
        dataset, not two. ⚠️ It already misses five that exist today — closing them is in scope here
        because this story is what makes the hole load-bearing.
  - [x] Each observed RED before it passes, with the mutation recorded.
- [x] **T5 — Copy, in both locales (AC: 2, 4)**
  - [x] Every new key in `app.yml` with `en` and `fr`. ⚠️ `every_key_carries_both_locales` sees only
        keys that are IN the file: 6b.6 shipped two headings resolving to keys that did not exist.
  - [x] ⚠️ Its floor reads `checked >= 47` on a message saying *"48 entries minus `_version`"* while
        `app.yml` carries **153**, and its doc claims a baseline of 32. Three stale numbers in one
        guard. **Registered, owner Epic 6b's retrospective — do not tidy it here** and do not trust it.
- [x] **T6 — The stylesheet (AC: 1)**
  - [x] `.ipam-grid`, three cell modifiers, the legend, the panels, `.blueprint`/`.corner`,
        the two-column layout. **Static literals only** (§0h).
  - [x] The legend's three literal classes labelled as the deliberate redundancy that carries the
        stylesheet guard, and pinned by a test.
  - [x] ⚠️ Occupied cells take the mock's **blue** (`--color-accent-700`, structure), never
        `--accent-document` (amber, the gesture) — 6b.1's reservation, guarded.
  - [x] ⚠️ Check for collisions: `.grid`, `.filters`, `.cards`, `.stat`, `.empty`, `.mono` already exist.
- [x] **T7 — The mutation pass (AC: 6, 7)** — every guard observed RED before it passes; **carriers
      named per row** (assertion / panic / compiler), never one headline for a mixed set.
  - [x] 🔴 **The minimum list, corrected — the draft's own list contained a mutation measured GREEN:**
        (a) an `aria-label` replaced by `title` → must RED (§0f, M-G was green);
        (b) `subnet` threaded through and the arm ignoring it → must RED (§0e(3); Rust does not lint an
        unused parameter, and this is 6b.6's `?kind=printer` verbatim);
        (c) both witnesses set to a marker string → must RED (§0g);
        (d) a section losing `example-section`, and a second marker planted → must RED (§0g);
        (e) a key that does not exist on a new field → must RED (§0g);
        (f) `.ipam-cell-used` deleted with the legend DRY → must RED (§0h);
        (g) one octet moved between the lists → grid AND occupancy line both RED (§0d);
        (h) `aria-current="page"` on the subnet selector → must RED. ⚠️ **Measured green today**:
        `exactly_one_entry_is_current_on_each_screen` renders the shell with an EMPTY body, so a second
        `aria-current="page"` in a screen's content is invisible — and the inventory's filter bar
        already established the idiom, making `"page"` vs `"true"` a coin flip.
  - [x] ⚠️ **The driver exits non-zero when the mutation fails to apply.** The gap-hunt's driver lied
        once in this very validation (a `.replace()` no-op after `cargo fmt` reflowed the anchor) and
        eight gates stayed green over a placeholder body. **One mechanism per script** — scratchpad
        restore **or** git, never both.
- [x] **T8 — Look at the screens (AC: 8)**
  - [x] Chrome 151 is installed. **Rebuild the BINARY** — `cargo test` builds the test target, not
        `target/debug/opencmdb`, and 6b.4b screenshotted a stale binary and believed it.
  - [x] French, 1280 px, both screens. 🔴 **Start with the legend swatches** (§0h): the validation's
        build rendered them at zero width with every guard green.
  - [x] Check the three cell states are distinguishable **without colour** (§0c, WCAG 1.4.1).
- [x] **T9 — The record (AC: 7, 8)** — the live count in THIS file; both wall-clocks; eight gates; and
      the register rows, each with an OWNER by name. **Six are owed and T0 settled none of them — an
      arbitration decides what SHIPS, the register carries what stays OPEN:**
  - [x] the five nouns and the three value sets, with Guy's (c) and the reason (a) was refused as
        premature — **owner: Epic 15**, which is where (a) becomes the closure;
  - [x] the `conflit` collision, beside `Nouveau`/`undeclared` — **owner: Epic 6b's retrospective**;
  - [x] the two binding mirrors having DRIFTED (`attach` in one, not the other) — **owner: Epic 6b's
        retrospective**;
  - [x] `.0` / `.255` and whatever T1 decides — **owner: Epic 14**;
  - [x] `every_key_carries_both_locales`' three stale numbers — **owner: Epic 6b's retrospective**;
  - [x] the absence of any a11y check in the repository, measured (§0f) — **owner: story 6b.11**, with
        6b.12's sweep, since axe-core is the epic's DoD and no story has yet been given it.

---

## Dev Notes

### Baseline, measured by the validation on this tree

**653 tests** (426 bin + 161 core + 66 xtask), **0.585 s** without `DATABASE_URL`, cold build 29.8 s.
The gap-hunt's full prototype — two screens, 25 keys, 9 CSS rules, the octet lists — left all eight
gates, `cargo fmt --check` and `clippy -D warnings` green. **The story is buildable as specified**; what
it lacks is guards, not feasibility.

### The files this story touches

| File | Today (code lines, gate ceiling 2000) | This story |
|---|---|---|
| `example_data.rs` | **775** | UPDATE — apps and subnets. ⚠️ **1134 measured at the end**; the *~919* first written here was the validation prototype's figure, not this tree's, and the code review is what reconciled it. Ceiling 2000. |
| `example_screens.rs` | **393** | UPDATE — two bodies, two strings structs. |
| `screens.rs` | **462** | UPDATE — two variants, two natures, two render arms. |
| `main.rs` | — | UPDATE — two witness arms **and** the hardcoded `2`. |
| `page.rs` | **1524** | UPDATE only if §0e(4) is taken (a new function is required, not just visibility). |
| `templates/` | twelve partials, flat | NEW — `_apps_example.html`, `_ipam_example.html`. |
| `assets/app.css` | 752 lines | UPDATE — grid, legend, panels, layout. |
| `locales/app.yml` | **153** keys, both locales | UPDATE. |

⚠️ **`xtask/src/main.rs` is at 1908/2000 — the closest file to the gate**, not `page.rs`. Nothing this
story does touches it; the sentence is here because the draft got it wrong and the next story should
not inherit the error.

**Nothing else.** `opencmdb-core` untouched (no behaviour, no doc claim), no migration, no new address,
no write, **no new dependency** — askama 0.16, axum 0.8, `rust-i18n`, and hand-authored CSS (⚠️ there
is **no Tailwind chain and no `cargo xtask css`**). A CSS Grid of 256 cells needs no library. **Never
invent a version — pin from the real `Cargo.lock`.**

### The guards, sorted by what was MEASURED

**Will red** (measured with a control): the marker partition over the route table (`main.rs:862`), the
witness match (`main.rs:976`), the stylesheet guard **for static literal classes only**
(`page.rs:3218`), `every_key_carries_both_locales` (`screens.rs:622`), the amber reservation
(`ac4_the_amber_is_reserved_for_the_documenting_gesture`, `page.rs:2686`).

**Measured GREEN on these screens** — §0g and §0h: the per-section marker guard, `no_i18n_key_reaches_the_screen`,
`the_example_copy_is_translated_rather_than_typed`, the stylesheet guard for a Rust-built class, the
`aria-current` guard, and any check on the witness's distinctiveness. **`no_screen_renders_a_key_name_as_a_label`
never sees a screen body**, and `every_variant_of_a_navigated_enum_is_listed_in_all` does not cover
`ExampleContent`.

> 🔴 ***A guard placed where the defect cannot occur reads as coverage and is none.*** *Reading the
> guard cannot find it — the guard is correct about what it tests.* Epic 5's dominant class, counted in
> at least nine of its twenty stories, and now measured **six times in one story's inheritance**.

Its live specimens in this epic, all relevant here: a pure builder tested while the ROUTE does
something else (6b.4's `triage_html`; 6b.6's `?kind=printer`) — **assert on the rendered HTTP body**; a
guard reading the SOURCE while the defect lives in the RENDER (6b.4b, four times) — **assert on the
resolved string**; a needle spelled `class="…"` defeated by a second class (6b.6) — **key on the class
NAME**; a total compared where the property is per-unit (6b.5); a hardcoded floor CI cannot check
(6b.5 shipped a red suite).

### House rules this story is measured against

- **Prove-to-red**, and the mutation recorded with its carrier.
- **DRY, minus the deliberate redundancies** — §0h makes one on purpose; label it and pin it.
- **No source file over 2000 CODE lines**; **split, not grown**.
- **Document every `pub` item, and a doc comment must be TRUE.** Prefer the weaker true sentence.
- **`epics.md` and the UX spec are NOT edited by a story.**

### What the operator can DO with these two screens, asked on purpose

**Nothing.** Look, switch subnet, follow a link to the triage. No form, no write, no live gesture.
⚠️ That makes **six** well-lit dead ends. Registered for the retrospective, which already owes the
count a look, and ⚠️ **these two screens are 100 % example content** — the salience risk registered at
6b.5 (*the example half is visually dominant over the honest one*) grows by two whole screens.

### Project Structure Notes

Both screens stay on the pool-free `Router<()>`; a handler taking `State<MySqlPool>` there fails to
compile. No demo screen opens a connection and no example row is ever written. `/apps` and `/ipam` are
already in `Screen::ALL`, already in the nav, already refused 401 to an unauthenticated caller, and
already routed — serving `not_built_yet_body()` today. **This story adds no address.**

### References

⚠️ Line citations below were re-verified by the fact-check layer; the ones the draft got wrong are
corrected here.

- [`epics.md:2226-2242`] — the three ACs, verbatim. [`epics.md:2090-2110`] — the four premises and six
  constraints. [`epics.md:316`] — UX-DR71, which names the occupancy grid as a WCAG key view.
  [`epics.md:473-479`] — Epic 14 (IPAM, FR21-25) and Epic 15 (Applications, FR26-29).
- [`prd.md:906-910`] — FR21-FR25. [`prd.md:915-936`] — FR26-FR29. [`prd.md:919`] — FR27 naming
  applications, owner and criticality. [`prd.md:921`] — `hosts` **and** `exposes` in one clause.
  [`prd.md:985`] — the gesture table, **ten** rows. [`prd.md:1004-1018`] — the state axis, five rows.
  [`prd.md:988`] — *"Neither language carries two meanings for one word."* [`prd.md:1016`] — the
  `conflict` row. [`prd.md:1036`] — the retired list.
- [`ux-design-specification.md:1332`] — the canonical glossary, **eleven** gesture rows (the `attach`
  row the PRD lacks is at `:1348`). [`ux-design-specification.md:1365`] — the `conflict` row's mirror.
- [`docs/manuals/user-manual/user-manual.tex:155-168`] — the manual's seven-entry glossary appendix,
  the third location 6b.6's first draft missed.
- [`~/travail/Projets actuels/opencmdb/documents/opencmdb - maquette.html`] — `APPS` (8 rows, 7
  columns, its subtitle sentence, four criticality and four exposure values), `SUBNETS`, the
  `(i * 37) % 256` formula, the `role="img"` grid, the legend, the free-address and conflict panels.
- [`deferred-work.md:4267`] — 6b.6's registration of `criticality`/`app`/`owner`. [`:4260`] — the
  `Nouveau`/`undeclared` collision this story's §0b joins.
- [`6b-6-inventory-and-device-record.md`], [`6b-4b-action-bar-and-the-gesture-nature.md`] — the
  vocabulary method, the browser-only defects, the planned-gesture mechanism.
- [`screens.rs`] — `Screen`, `Nature`, `ExampleContent`, `router`. [`main.rs:832-1030`] — the
  partition, the witness match, the hardcoded `2`. [`page.rs:658-717`] — `Gesture`, `GestureView`,
  `action_bar`. [`example_data.rs:141`] — the criticality refusal. [`_dashboard.html:58`] —
  `spark-h{{ height }}`, epic constraint 5's shipped specimen.
- [`CLAUDE.md`] — the four engineering conventions, the eight gates, the dependency frontier.

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), 2026-08-20.

### Debug Log References

Built and measured against a live `mariadb:10.11.16` on port **13317** (⚠️ 3306 holds another
project's container and was avoided rather than discovered), and looked at in **Chrome 151** at
1280 px, in French, **on a rebuilt binary**.

### Completion Notes List

⚠️ **THE LIVE COUNT FOR THE PROJECT LIVES HERE**: **653 → 668 tests** (441 bin + 161 core + 66
xtask, after the code review). Eight gates green, `clippy -D warnings` clean, `cargo fmt --all --check` clean. **The suite
was run BOTH ways**: **0.49 s** without `DATABASE_URL` and **5.92 s** against the live MariaDB — the
clock is the tell that the database-backed tests genuinely executed. `opencmdb-core` byte-identical,
no migration, no new address, no write, no new dependency, `epics.md` and the UX spec not edited.

**What shipped.** `/apps` and `/ipam` leave `Nature::Empty` and become `Example` content: an
applications table (six columns) and a 256-cell CSS-Grid occupancy map with a subnet selector, a
legend, a next-free-address panel and an address-conflict panel. `example_data.rs` gains `ExampleApp`,
`ExampleSubnet`, `CellState` and `ExampleAddressConflict`; `example_screens.rs` gains `apps_body` and
`ipam_body`; two templates, 25 keys in both locales, ~60 lines of stylesheet.

🔑 **The two screens describe ONE network.** Every application's host names a device the inventory
ships (seven of the mock's eight named machines that exist in no dataset), the office subnet is the
`192.0.2.0/24` the devices already live in, and the conflict sits at `.41` — `vm-billing`'s address,
which the grid draws as occupied. All three are asserted, not arranged.

🔴 **A /24 carries 256 ADDRESSES and 254 HOSTS, and the mock conflates them.** It loops `0..256`,
draws `.0` and `.255` as ordinary free cells, and its *next free address* panel then names the
NETWORK address — reproduced on a real build at this story's validation. So `CellState` has a fourth
variant, `Structural`, the occupancy line counts over the **254 hosts**, and `next_free` starts at 1.
⚠️ **This story's own first draft prescribed a test asserting the counts *"sum to 256"***, which would
have pinned the defect as the expected behaviour — *a test that pins the ugly thing is a test that
requires it* (6b.4).

🔴 **AC1's `aria-label` per cell is incompatible with the mock's `role="img"`**, which is *Children
Presentational: True* and would swallow all 256 labels; and `aria-label` on a bare `<div>` maps to
`generic`, where **ARIA 1.2 prohibits it outright** — an axe-core violation, not merely a keyboard
inconvenience. The grid ships as a `<ul role="list">` laid out by CSS Grid, one `aria-label` per
`<li>`, `role="list"` explicit because `list-style: none` is known to drop it in Safari/VoiceOver.
⚠️ **The `role="img"` half is REASONED, not measured**: the validation read Chrome 151's accessibility
tree and the children still appear there, because the pruning is a platform/AT mapping. Said rather
than claimed. ⚠️ *Reserved* is a **hatched pattern** and not a flat grey: three states told apart by
colour alone is WCAG 1.4.1.

🔴 **THREE INHERITED GUARDS WERE ENUMERATIONS AND ARE MEASURED GREEN ON THESE TWO SCREENS**, each
with a control that reds. `every_example_section_is_covered_by_exactly_one_marker` looped a hardcoded
`[("/devices", …), ("/devices/{id}", …)]`; `no_i18n_key_reaches_the_screen` looped the pages
`example_screens` happened to build; `the_example_copy_is_translated_rather_than_typed` read **two**
key-bearing fields of the seven the dataset already carried.

⚠️ **AND THE HEADLINE ABOVE READ *"ARE NOW PROPERTIES"*, WHICH THE CODE REVIEW REFUTED — the
correction matters more than the patch.** What actually happened is three different things, and only
naming them separately is honest:

- **The key rule and the copy rule were genuinely converted.** The first now runs over the **real
  HTTP body of all ten screens** in the route-table guard; the second covers **87 strings** where it
  read 10.
- 🔴 **The marker rule was NOT converted. It was SUPERSEDED.** A new page-level property was added to
  the route-table loop — which does cover `/apps` and `/ipam`, verified by mutation and by two review
  layers independently — while `page.rs`'s enumeration was left standing. **The named guard is
  untouched**, and calling it *converted* was false.
- 🔑 It is kept rather than deleted, and now says why: its surviving assertion — *the body carries no
  marker of its own* — is a property the route-table guard **cannot express**, because it sees only
  the served page, where a template-side marker and the dispatch's are indistinguishable. Two markers
  stacking is what the route-table count catches; a marker that has MOVED into a template is what
  this catches. The redundancy is now labelled deliberate, as `CLAUDE.md`'s DRY rule requires.

⚠️ **And *"covers every key-bearing field"* was also too wide**: `CellState::label_key()` — four new
`ipam.state.*` keys this story introduces — is not in that loop. Its carrier is the route-level key
property over the rendered body, which is strictly stronger for the display path and weaker for a key
that is never rendered. Stated rather than tidied.

⚠️ The copy guard *reads* as strong coverage because it is the first thing to red when a `role_key`
breaks: an enumeration that catches the case you test is what *"a guard placed where the defect cannot
occur"* looks like when the defect moves one field over.

🔴 **A WITNESS IS ONLY A WITNESS IF IT IS DISTINCTIVE, and nothing said so.** `demonstration_screen`
prepends the example marker to every `Example` body, so any string taken from the marker satisfies
every witness for all time — measured with both witnesses set to the marker's text and **zero tests
red** while the two screens served each other's content. There is now a property asserting each
witness appears in **its** body and in no other.

⚠️ **Guy's arbitration of 2026-08-20 is pinned by a NEGATIVE test.** Option (c) — render owner and
criticality, not `exposure` — is a vocabulary decision living in a document, one column away from
being undone by anyone comparing the screen to the mock. `the_apps_screen_renders_no_exposure_column`
is what makes it survive. Criticality renders from an i18n **key**, owner values are **data**: the
validation measured that with the owners as literals, `/apps` shows five French words under an English
UI with the whole suite green.

🔴 **TWO MUTATIONS CAME BACK GREEN AND BOTH REFUTATIONS ARE THE DELIVERABLE.**

- **M-g** put octet `41` into the office subnet's *reserved* list — a deliberate corruption — and
  **changed nothing**: `state_of` tests `used` before `reserved`, so an octet in both is silently
  resolved as *used*. **The two lists are not orthogonal, and a priority order hides a contradiction
  in the data rather than showing it.** Closed by `no_octet_is_both_occupied_and_reserved`, after
  which M-g reds. The honest form of the mutation (`M-g'`, the octet REMOVED) reds the conflict guard.
- 🔴 **THE DRIVER ITSELF LIED, and this one is worth carrying past the story.** `shutil.copy2`
  preserves the ORIGINAL mtime, so after restoring a template cargo saw a file older than the artefact
  that embedded it and **did not rebuild**. Askama compiles templates INTO the binary, so **two full
  suite runs reported one test red over a clean `git status`** — the mutation was still in the
  compiled artefact. Caught only because a failure contradicted a just-measured green; had it appeared
  as a pass, it would have been filed as a confirmation. *The mutation driver lies*, fifth epic
  running, in a new way: not by failing to apply, but by failing to UNAPPLY. The driver now restores
  with `copy` plus an explicit `utime`, and the three template mutations were **re-verified** under it.

⚠️ **An assertion order was measured wrong and corrected.** Under M-h the selector test reddened on
*"exactly one tab is in force"* — a true failure naming the wrong cause — while the assertion written
for that exact defect was never reached. The negative now comes first. Story 5.13's assertion-order
finding, met a fifth time.

### Mutation pass — 10 mutations, 10 reds, carriers named per row

⚠️ **Carriers are MIXED and stated row by row; no headline claims one carrier for the set.** Every
red below was read from its own panic message.

| # | Mutation | Result | Carrier |
|---|---|---|---|
| M-a | every cell's `aria-label` → the mock's `title` | RED ×1 | assertion, `every_cell_of_the_grid_carries_its_own_aria_label` |
| M-b | the render arm ignores its query (`&Default::default()`) | RED ×1 | assertion, `the_subnet_selector_narrows_through_the_real_route` — 🔑 **the pure test stays green**, which is why the route one exists |
| M-c | both witnesses set to the example marker's own text | RED ×1 | assertion, the new distinctiveness property |
| M-d1 | the applications section loses `example-section` | RED ×1 | assertion, the route-table anchor property |
| M-d2 | a second marker planted on the IPAM body (6b.6's *"four banners"*) | RED ×1 | assertion, the route-table one-marker property |
| M-e | `criticality_key` names a key that does not exist | RED ×2 | assertions — the dataset property **and** the route-level key property, the pair AC6 built |
| M-f1 | `.ipam-cell-used` deleted from the sheet | RED ×1 | assertion, `every_class_a_template_names_is_defined_in_the_stylesheet` |
| M-f2 | the legend rendered from data instead of literals (**the tidy gesture**) | RED ×1 | assertion, `the_legend_names_every_cell_modifier_as_a_literal` |
| M-g | octet `41` in BOTH lists | 🔴 **GREEN first** → RED ×1 after the guard it produced | assertion, `no_octet_is_both_occupied_and_reserved` |
| M-g' | octet `41` removed from the occupied list | RED ×1 | assertion, the conflict-address guard |
| M-h | `aria-current="page"` on the subnet selector | RED ×1 | assertion — ⚠️ **first run named the wrong cause**; the order was corrected and it now reds on its own assertion |

🔑 **M-f1 and M-f2 together are the §0h chain**: the legend's literals are what let the stylesheet
guard see a modifier the cells choose in Rust, and the new test is what stops them being tidied away.
Written the DRY way — *which is what this codebase's rules ask for first* — the occupied cells ship
with no colour and nothing reds.

### The browser look (T8), and what only it could say

✅ **The legend swatches RENDER.** The validation's build had them at zero width — an empty
`<span>` with only `aspect-ratio` collapses as `inline-block` — so the sheet ships `display: block`
with an explicit `min-width`. *A guard proves a rule EXISTS, never that it renders.*

🔴 **What the look found and this story does NOT fix, registered instead: the applications table shows
a gap and does not name it.** Nextcloud reads *déclarée 28.0.4 · observée 29.0.1*, Sage 50 reads
*2024.1 · 2024.1*, and **nothing distinguishes the two** — no pill, no colour, no word — on the
product whose founding sentence is *"the gap is the product"*. The mock does the same, and AC2 asks
only that owner and criticality be shown as unobservable, so fixing it here would be inventing scope;
but the state vocabulary exists (`ObjectState`, five binding rows) and this screen is the first that
displays a divergence without using it. **Owner: Epic 15**, and it is registered.

⚠️ In the legend, *libre* and *réseau ou diffusion* are hard to tell apart at 10 px. Both mean *not
assignable to you*, so the confusion is low-harm, and the grid's corners read correctly at full size.
Registered rather than fiddled with.

### Code review — three layers, 2026-08-20, on a different model, each isolated

**Blind Hunter** (diff only, no repository) · **Edge Case Hunter** (own worktree, mutations) ·
**Acceptance Auditor** (own worktree, spec and planning artefacts). **5 patches applied, 1 finding
refuted with the check that refutes it, 0 arbitrations needed.** Suite after the review: **668**
(441 bin + 161 core + 66 xtask), eight gates green.

🔴 **THE BLIND LAYER FOUND THE HIGH, AGAIN, AND IT WAS IN THE CODE THIS STORY WROTE TO CLOSE A DEFECT
OF THE SAME FAMILY.** Rewriting `no_i18n_key_reaches_the_screen` changed its premise counter from
*text words* to `html.split_whitespace()` — **raw markup**, so every `href`, `class` and tag name
counted as an inspected word — under a message still promising *"at least two hundred rendered words
were inspected"*. The detection was intact; the floor had stopped measuring what it names, and was
then satisfied by markup whatever the page said. ⚠️ **The neighbouring guard in the same commit warns
about exactly this** (*"a floor is only a guard while it is near what is there"*). Closed by splitting
`visible_text` out, after which the floor holds on real text. **Second consecutive story where the
layer with no repository access found what two layers with the whole tree did not.**

🔴 **THE AUDITOR REFUTED THE STORY'S OWN AC6 HEADLINE** — *"three inherited guards are now
properties"* — by reading the tree rather than the record: **the named guard was never edited.** The
correction is written above; the short form is that a property was ADDED beside an enumeration that
was left standing, and *superseded* is not *converted*.

⚠️ **A task was ticked whose either/or was discharged in neither branch.** T2 required the planned
*Réserver* control **or the written decision not to render it**; the control is absent and no sentence
said why. The decision is now written at the site, with its cost stated: `/triage` shows five dead
controls and says why, this screen shows none, and the two are inconsistent until Epic 14 gives IPAM a
real subnet. *Announcing a gesture over invented data is the wider of the two errors.*

⚠️ **Two more claims narrowed rather than defended**: *"the copy rule covers every key-bearing
field"* is false for `CellState::label_key()` (its carrier is the route-level key property instead),
and a doc asserting in the PRESENT that the repository holds *"not one assertion on an `aria-*`
attribute"* was falsified by the three this same commit adds — the second time in two stories that a
sentence is refuted by its own diff.

✅ **REFUTED, with the check.** The auditor reported AC4 NOT MET — no register rows in
`deferred-work.md`. **That is an artefact of the review scope I gave it, not of the tree**: the diff
handed to the layers was `crates/` only, so the ten rows appended under *"Registered by story 6b.7"*
were invisible to it. Verified present, ten rows, each with a named owner. 🔑 **The finding is kept
rather than deleted, because the lesson is mine**: a layer asked to audit registrations must be given
the file the registrations live in, and I asked it to audit what I had excluded from its input.

✅ **Confirmed by independent measurement**, not taken on the record's word: **both terms** of
653 → 667 (`git stash -u` for the baseline, 426+161+66 = 653; with the diff, 440+161+66 = 667); the
eight gates; `largest: 1908` being `xtask/main.rs` and not `page.rs`; **87** strings reproduced by
forcing the assertion to fail; the stale locale floor at 47 for 184 keys; and three mutation rows
replayed live, M-g included — *silent before the guard exists, red after*, exactly as the GREEN-first
row states.

✅ **The mutating layer measured every guard and found NO green one** — nine plants, all red with the
right carriers — plus a battery of hostile query strings on both routes (duplicate keys answer **400**
from axum's own extractor; an unknown slug, a NUL, a traversal string, a 10 000-character value and a
`<script>` all answer 200 and **none is reflected**, the unknown-subnet page never echoing its input),
no `|safe` in either new template, and the locale decision applied consistently in **both** languages —
owner proper nouns identical in `en` and `fr`, criticality resolving per locale.

⚠️ **One gap it found is now closed and one limit is stated**: no committed subnet is full, so
`ipam.next_free_none` was **unreachable, not merely untested**. `a_full_subnet_has_no_next_free_address`
now guards the function against a synthetic subnet, with a control that the committed three are not
full — but **the RENDER of that sentence is still carried by nothing**, and putting a full subnet in
the dataset to exercise one line would be shaping the demonstration around the test. Registered for
Epic 14.

### File List

- `crates/opencmdb-bin/src/example_data.rs` — MODIFIED
- `crates/opencmdb-bin/src/example_screens.rs` — MODIFIED
- `crates/opencmdb-bin/src/screens.rs` — MODIFIED
- `crates/opencmdb-bin/src/main.rs` — MODIFIED
- `crates/opencmdb-bin/templates/_apps_example.html` — NEW
- `crates/opencmdb-bin/templates/_ipam_example.html` — NEW
- `crates/opencmdb-bin/assets/app.css` — MODIFIED
- `crates/opencmdb-bin/locales/app.yml` — MODIFIED
- `_bmad-output/implementation-artifacts/6b-7-applications-and-ipam.md` — MODIFIED (this file)
- `_bmad-output/implementation-artifacts/deferred-work.md` — MODIFIED (the register rows T9 owes)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — MODIFIED

### Change Log

- 2026-08-20 — contexted, validated by two fresh-context layers (eleven claims of the draft refuted),
  arbitrated by Guy on three points, implemented, mutated (10 mutations, 10 reds, two green-first),
  looked at in a browser. Status → `review`.
