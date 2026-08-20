# Story 6b.8: Sources and alerts

Status: done

Epic: 6b — *L'interface de la maquette*. **Eighth numbered slot, ninth story file.** It takes the
`Empty` screen count from four to two, and it is **the first screen of this epic whose most important
section is REAL rather than example**.

## Story

As the operator,
I want the source screen to tell me what my sources cannot see,
so that reduced reach reads as a capability to unlock rather than a fault to repair.

## Acceptance Criteria

Transcribed from `epics.md:2252-2260`, **unmodified** — divergences are raised in §0 (a story may not
edit an AC; only a retrospective or a correct-course may).

1. **Given** the shipped ARP/ping connector, **when** the source screen renders, **then** *"what this
   source cannot see"* is **REAL, not example**: story 5.14 measured that it emits `IpV4` and `Rtt`
   and **no MAC, ever** — the product genuinely knows the boundary of its own sight, and that section
   is the most honest thing on the screen.
2. **Given** the two axes (liveness × capability), **when** a source is shown, **then** the spec's
   rule holds: **blind is an incident and gets a colour; reduced is a property and gets a sentence** —
   never two amber pills meaning opposite things.
3. **And** the alert list is an example surface (Epic 16), marked as such.

**Added by contexting** (numbered from 4 so the three above keep the epic's numbering):

4. **Given** AC2's liveness axis, **when** it is rendered, **then** it shows only what the product can
   actually establish, and **what it cannot establish is said rather than fabricated** — see §0b, which
   is this story's centre.
5. **Given** the source's identity, **when** it is displayed, **then** the register row story 6b.4
   assigned to this story by name is **closed or re-owned with a measurement**, never left silently.
6. **Given** the whole delivery, **when** `cargo xtask ci`, `clippy -D warnings` and `cargo fmt
   --check` run, **then** eight gates are green and the suite is run **both ways** with both
   wall-clocks recorded.
7. **And** the live count lives in THIS file, and the screens are **looked at in a browser** before
   the story is called done.

---

⚠️ **VALIDATED 2026-08-20 BY TWO FRESH-CONTEXT LAYERS, AND THIS FILE IS THE CORRECTED VERSION.** One
checked every claim against its sources; one **built the design in a scratch copy and measured it
against a live MariaDB and four real boots of the binary**. Between them they refuted **twelve** claims
of the first draft, including its central premise about why the liveness axis has no producer, the
distance to the epic that owns a source registry, and the availability of the fallback its own task
list offered. **Each refutation is kept in place rather than overwritten** — the corrected findings are
sharper than the ones they replace, and a reader in six months must be able to re-derive the decisions
rather than only read them.

## §0 — What contexting found

🔑 **This story is NOT another example screen, and treating it as one is the failure mode.** The three
before it (6b.5, 6b.6, 6b.7) put fabricated content behind a marker. **This one has an AC that
forbids that for its central section**, and the product can honour it — but only on ONE of the two
axes AC2 names. What follows is measured, not read off the epic.

### §0a. ✅ THE CAPABILITY AXIS IS REAL, AND HERE IS EXACTLY HOW REAL

Measured in the code, not inferred:

- `crates/opencmdb-bin/src/arp_ping.rs:207` — `declared_kinds() -> BTreeSet<FactKind>` returns
  **`{IpV4, Rtt}`**, and `:255` is a test named `the_ping_sweep_declares_no_mac` pinning it.
- `crates/opencmdb-core/src/observation/mod.rs:201` — `FactKind` has **seven** variants: `Mac`,
  `IpV4`, `Hostname`, `DhcpLease`, `Uplink`, `OuiVendor`, `Rtt`.
- 🔑 **So *"what this source cannot see"* is the complement, and it is FIVE kinds** — `Mac`,
  `Hostname`, `DhcpLease`, `Uplink`, `OuiVendor` — **derivable at runtime from the binary, with no
  database, no fixture and no invention.** That is AC1, and it is satisfiable exactly as written.

🔴 **But `FactKind` has no `ALL`**, so the complement cannot be computed today. Adding one puts a
change in **`opencmdb-core`** — the domain crate — and that is a decision, not a detail:

- It is **D47-legal**: a `const ALL` adds no dependency and the frontier gate will stay green.
- ⚠️ It **breaks the *"`opencmdb-core` byte-identical"* line** that six consecutive stories have
  carried. 🔑 Story 5.13b's finding applies directly and must not be repeated: *a promise of
  non-modification protects behaviour and shelters false sentences.* Narrow the claim to **no
  BEHAVIOUR change in `opencmdb-core`** rather than defending a byte count.
- ⚠️ `every_variant_of_a_navigated_enum_is_listed_in_all` (`screens.rs`) scans **`opencmdb-bin`'s**
  source for enums with an `ALL`. A `FactKind::ALL` in the domain crate is **outside its perimeter**,
  so a new variant added later would silently drop out of the complement — and the screen would then
  under-report what a source cannot see, which is a lie in the safe-looking direction. **Decide the
  guard in the same commit as the constant**, and prefer an exhaustive `match` over a list wherever
  the compiler can carry it.

### §0b. 🔴 THE LIVENESS AXIS HAS NO PRODUCER — and the first draft was WRONG THREE WAYS about why

⚠️ **The draft wrote: *"`blind` exists nowhere as a state. No table, no column, no type, no function."*
Both validation layers refuted it, and the refutation is kept because the corrected picture is what
makes the arbitration re-derivable.**

| The draft said | What is measured |
|---|---|
| "no type" | `opencmdb-core/src/score.rs:420` — `pub enum SourceState {}`, **public, documented, deliberately uninhabited**, its doc naming **Epic 13** as its builder, and a test `source_state_cannot_be_populated_in_epic_4` pinning the emptiness. |
| "no function" | `opencmdb-core/src/connector/mod.rs:85` — `pub fn is_blinding(&self) -> bool`, whose doc reads *"every cause **blinds the source** EXCEPT a clean cancellation"*. |
| "a whole-tree search for `liveness` returns only `/healthz`" | **27 hits across seven files.** |

🔴 **AND THE THIRD IS THE ONE THAT CHANGES THE DESIGN SPACE: the value both axes need is COMPUTED ON
EVERY SCAN AND THROWN AWAY.**

```rust
// opencmdb-core/src/connector/mod.rs:116
pub struct PollSummary {
    pub capabilities: Capabilities,   // FR7's DATED descriptor
    pub scopes_covered: Vec<Scope>,   // FR5's liveness unit
}
```

`scan_pass.rs:80` calls `if let Err(error) = connector.poll(...)`. **The `Ok(PollSummary)` is dropped
at the seam** — no binding, no log, no row — and *the compiler says nothing, because `if let Err`
legitimately discards the `Ok` payload.*

🔑 **So what survives is the CONCLUSION and not the reason**: a request-time screen cannot read a value
nobody persists, and *"no table, no column"* is measured true (zero `capabilit*` matches across all
**five** migrations). ⚠️ But *a decision explained by a false premise is one nobody can re-derive*
(story 6b.5's own lesson), and the false premise hid the cheapest closure: **one `let` and one row.**

#### D32 ALREADY SPECIFIES THIS, AND THE DRAFT CITED NO ARCHITECTURE DECISION AT ALL

`architecture.md:1815` — **D32, `source_state` is TWO ORTHOGONAL AXES, not three states**:

```rust
struct SourceState {
    liveness: Liveness,          // Live { last_ok } | Blind { since, cause: BlindCause }
    capabilities: Capabilities,  // the CURRENT descriptor — not a level
}
```

🔑 **`Live { last_ok }` IS the fact the arbitration renders.** *"Last heard from: 3 minutes ago"* is not
a fact outside the model — it is the payload of D32's `Live` variant. **The arbitration is a SUBSET of
D32, not an alternative to it**, and saying so is what lets a reader re-derive it. `architecture.md:1411`
also fixes the granularity — *"per `(connector, subnet)`, the real sweep unit"* — which is what §0c
re-derived independently without citing it.

✅ **GUY'S ARBITRATION, 2026-08-20 — taken as recommended, and now anchored on D32:**

> **The liveness axis renders the FACT and withholds the VERDICT.** The screen shows `Live`'s
> `last_ok` **with no colour**, and says in one sentence that the incident axis arrives with
> **Epic 13** — the owner three artefacts already name (`score.rs:409`, `connector/mod.rs:27`,
> `epics.md:469`), and which the draft asked T2 to state while supplying it nowhere.

**Refused, with the reason**: rendering the axis as example data behind the marker. AC1's whole point
is that this screen is where the product stops pretending, and a fabricated incident beside a real
capability statement teaches the operator that neither is trustworthy. **A screen may be honest about
one axis and silent about the other; it may not be honest about one and theatrical about the other.**

⚠️ **A DIVERGENCE TO REGISTER RATHER THAN TICK**: the UX spec (`:1113`) gives liveness **a colour for
BOTH values** — `live` = the calm token, `blind` = neutral and desaturated — and paints `live` green in
its mock. Shipping **no colour at all** is defensible (the product cannot compute `blind`, so it cannot
assert `live` either, and a green dot would be the fabricated verdict) but it **is a divergence**, and
AC2 must not read as met in letter. **AC2 ships MET on its *never two amber pills* half and NOT MET on
its *blind gets a colour* half.**

### §0b-ter. ✅ GUY'S THIRD ARBITRATION, 2026-08-20 — the discarded `PollSummary` is NOT persisted

**Taken: do not persist. Bind it, log it, and register the persistence with its PRECONDITION named.**

🔑 **The argument that decides it is not scope, and that matters — it is checkable rather than
preferred**: *a per-source descriptor cannot be persisted without a stable source identity*, and
§0c has just established that the stable identity is **Epic 11's**. A capability table keyed on
`connector_id` would inherit the boot-minted defect immediately — one more row at every restart,
which is exactly what §0c exists to prevent at the screen, reproduced in a fresh table.

⚠️ **And a second decision is hidden inside it**: one row per SCAN (which accumulates) or one CURRENT
row superseded? That is D32's and Epic 13's to make, with FR7's *"a capability reduction is a notifiable
event"* attached to it. Taking it in passing, inside a screen story, is taking it badly.

**What ships instead — three gestures, no migration:**

1. **Stop discarding the value.** Bind the `Ok(PollSummary)` at `scan_pass.rs:80` and trace it, so the
   dated descriptor is at least observable today. ⚠️ **A mild overrun on a screen story, flagged as
   one** — but discovering that the real measurement is thrown away, shipping the screen that displays
   its substitute, and leaving the discard in place is what a review would name.
2. **AC1 renders `declared_kinds()` and SAYS WHAT IT IS** — *"what this source is built to observe"*,
   never *"what it observed at the last scan"*. 🔑 The same principle as §0b's arbitration: render what
   is known, name what is not. The honest sentence is short and true; the dated one would be false.
3. **Register the persistence with its precondition**, not as a to-do: *stable identity first
   (Epic 11), accumulation decision next (D32 / Epic 13)*. **A register row that names its precondition
   is actionable; one that says "later" is not.**

🔴 **THE COST, STATED AND NOT MET**: AC1 delivers **FR7's STATIC half only**, while FR7 says in so many
words that the descriptor *"is not a static property of the source"* and that *"observations are always
interpreted under the descriptor in force when they were collected"*. **This story does not close
FR7**, and the sentence must read that way wherever the story is summarised — on the precedent of
NFR5, carried on one assertion of three for two whole epics and said as such every time.

**Refused, with the reason**: capturing the descriptor now. It is defensible — it would force the
stable identity and therefore pull Epic 11 forward — **but that is a plan change, and a plan change is
a `correct-course`, not a screen story.**

### §0b-bis. 🔴 THE PRESCRIBED READER CANNOT TELL FOUR STATES APART, AND ONE IS A MISCONFIGURATION THE PRODUCT ALREADY CAUGHT

Measured by the gap-hunt on **four live boots against four fresh databases**, real binary:

| case | `OPENCMDB_SCAN_CIDR` | what the log says | observations / connectors / `MAX(observed_at)` |
|---|---|---|---|
| never scanned | *unset* | (no scan line) | 0 / 0 / **NULL** |
| scanned, nobody home | `198.51.100.0/24` | `ingested=0 failed=0` | 0 / 0 / **NULL** |
| **invalid perimeter** | `nonsense` | `ERROR invalid OPENCMDB_SCAN_CIDR — skipping scan` | 0 / 0 / **NULL** |
| blank perimeter | `"   "` | (no scan line) | 0 / 0 / **NULL** |

`repo::last_observed_at` — the only reader T2 names — returns `None` in all four. **The screen would
say the same thing about a source never asked, a source that answered nothing, and a perimeter the
product itself refused.** That is FR8's own distinction failing at boot level, on the screen whose AC2
is about that axis.

🔴 **Case three is the sharpest and the draft never mentioned it**: `AppConfig::from_env` does not
validate the CIDR (its own doc says so at `main.rs:113`); only `ArpPingConnector::from_cidr` does,
inside a **detached thread**, where the failure becomes a `tracing::error!` nobody reads. §0c's
arbitration — *list the sources the product is CONFIGURED with* — makes `/sources` **the natural home
for that error**, and without it `/sources` would list a source the product refused to build, reading
*last heard from: never*.

⚠️ **And with the perimeter unset there are ZERO configured sources.** The draft assumed one
throughout. The zero case needs copy and has none.

### §0c. 🔴 THE SOURCE HAS NO NAME, AND THIS STORY OWNS THAT ROW BY NAME

`deferred-work.md:3975`, registered by story 6b.4 and assigning this story as owner:

> 🔴 **THE PRODUCT HAS NO CONNECTOR REGISTRY** […] `observation_record.connector_id` is a bare
> `CHAR(36)`; there is **no table, no name, nothing** behind it […] **Owner: story 6b.8 (*Sources and
> alerts*)**, the screen that owns sources — and the day it lands, `page::source_label` is where the
> name arrives.

⚠️ **A registry is a MIGRATION and a write path**, which is a large thing to hide inside a screen
story whose epic ships no schema. The honest options, for Guy:

- **(a) A name derived from the connector's own identity in code** — the ARP/ping connector knows what
  it is; a `name()` beside `declared_kinds()` costs nothing, needs no table, and makes the screen true
  for the one source that exists. ⚠️ It does not survive a second source configured at runtime, and
  that limit must be written.
- **(b) A real `source` table** — correct, and it is FR1/FR6's territory (Epic 17), not a screen
  story's.
- **(c) Keep the short labelled id** and re-own the row with a measurement.

#### 🔴 AND THE ID IS MINTED FRESH AT EVERY BOOT, WHICH MAKES THIS AN IDENTITY PROBLEM, NOT A NAMING ONE

`main.rs:479` — `let connector_id = ConnectorId::from_uuid(Uuid::now_v7());`, **inside the startup
scan thread**. Measured, not read: every restart writes observations under a **new** source identity.

Two consequences, and the second is the one that decides the design:

- A *Sources* screen built from `SELECT DISTINCT connector_id` shows **one row per restart**, growing
  forever — the UX spec's very first hard ban (*"no badge, no growing counter"*) reproduced on a new
  screen, and story 5.14's accumulation defect in a new place.
- 🔴 **Worse, and it collides head-on with §0b's arbitration**: every previous boot's source would show
  *last heard from: three days ago* and would then look **blind**. The data would MANUFACTURE the
  appearance of dead sources — the fabricated incident Guy has just refused, arriving not from a clock
  but from the identity scheme.

🔑 **So the screen must list the sources the product is CONFIGURED with, never the `connector_id`s it
has written**, and the freshness must be `MAX(observed_at)` **not keyed on `connector_id`** — correct
today precisely because there is exactly one source, and false the day there are two.

✅ **GUY'S ARBITRATION, 2026-08-20 — option (a), refined.** ⚠️ Recorded honestly: he asked for the
recommendation and endorsed it rather than arriving at it independently, and the finding below is what
changed the recommendation while it was being written. **(a) refined**: a **type** name from the connector's own code (*"Balayage
ARP/ping"*), because the product genuinely has one connector of one kind and **no notion of a source
INSTANCE at all** — naming the type is the true sentence, and a table would invent instance identity
before anything can create instances. 🔑 *That is the same principle as §0b: render what the product
knows, withhold what it does not.*

🔴 **AND `page::source_label` — the site the register names as *"where the name arrives"* — RENDERS A
TRUNCATED TIMESTAMP.** It takes the first 8 characters of the connector UUID, which are the top 32 bits
of a v7's millisecond field. Measured over three live boots: two boots six seconds apart got
**different** labels, two others six seconds apart got the **same** one — the label rolls every ≈65 s.
**The operator-visible source id is a clock reading, and two genuinely different sources can share
it.** ⚠️ And that function feeds **`/triage`**, not `/sources`: closing AC5 on `/sources` alone would
mark the register row closed while the site it names is untouched. **T3 touches it or says in writing
that it does not.**

⚠️ **The boot-minted id is registered as a defect in its own
right** — it is a write-path change and does not belong in a screen story — with the limit written
beside the query: the day a second source exists, this construction is wrong and a stable source
identity is the answer.

### §0d-bis. 🔴 THE UX SPEC BINDS VOCABULARY AND LAYOUT FOR THIS EXACT SCREEN, AND THE DRAFT CHECKED ONLY THE ALERT HALF

⚠️ **The draft cited `ux-design-specification.md:1086-1115`. The section runs to `:1143`, and
everything the truncation cut off is what the REAL half needs.** Six things, none of them named:

1. 🔴 **`not evaluated`** (`:1130`) — *"Out-of-capability fields are `not evaluated` — never 'in
   default', never blank, never a dash… **the same concept as `exclude`, and deliberately the same
   words**"*. **The capability half already HAS bound vocabulary**, and the draft worried about alert
   severities while missing it.
2. **`ping-only`** (`:1113`) — *"A scope label beside the name — neutral, descriptive, no judgement.
   **Never a colour, never a severity**."* ⚠️ This is a **different slot** from the source's NAME:
   §0c arbitrated a type name (*"Balayage ARP/ping"*), and the scope label is still undecided. The
   draft conflated the two.
3. The mock pairs **`Observes: reachability`** with **`Cannot see: …`**. AC1 quotes the negative half
   only, and the draft delivers the negative half only.
4. 🔴 **The spec's own example list is `addresses · ports · OS` — and it does not map onto `FactKind`.**
   `OS` is not a variant at all; `Hostname`, `DhcpLease` and `OuiVendor` are absent from the spec's
   list. **The spec's illustration and the computable truth disagree**: a divergence to raise, not to
   resolve in silence.
5. The copy direction is the **unlock**, not the consequence (`:1135`): *"Not 'fix your source' but 'I
   could grant it the raw-socket privilege and it would read addresses.'"* — and the sentence is
   already in the code, `arp_ping.rs:5`: *"MAC facts (ARP) are the `NET_RAW` upgrade"*. That is also
   the story's own user-story line: **reduced scope is a capability to UNLOCK, not a fault to repair.**
6. ⚠️ **Nothing guards any of it.** `every_state_word_served_is_a_term_of_the_binding_glossary` reads
   the inventory's and the record's state pills only; `/sources` and `/alerts` are outside its
   population entirely, and the `vocabulary` gate is four retired pairs none of which is touched.

🔴 **And T1's *"one sentence per unseen kind"* mints display copy for FIVE nouns that are in no binding
table either** — `Mac`, `Hostname`, `DhcpLease`, `Uplink`, `OuiVendor` — which is exactly the shape
story 6b.7 arbitrated one story earlier (five nouns, owner Epic 15). **The REAL half deserves §0d's
warning more than the example half does.**

### §0d. ⚠️ THE ALERT LIST IS EXAMPLE, AND ITS VOCABULARY IS UNCHECKED

AC3 makes the alert list an example surface. The mock's alerts carry severities and kinds — **another
value set with no glossary row**, exactly the shape story 6b.7 arbitrated. 🔑 **Do not mint alert
severities without asking**: the register already carries five nouns and three value sets owed to
Epic 15, and FR30's own text (*"an unknown device appearing, a documented IP unseen for N days, and an
IP conflict"*) is a better source for the example rows than the mock's invented severities, because it
names things the plan already binds.

### §0e. THE MECHANISM THIS STORY INHERITS, ALREADY BUILT

`Nature::Mixed` (story 6b.5) is the shape this screen needs: **a real capability section beside an
example alert list**, marker per example SECTION, not per screen. ⚠️ **A `Mixed` screen leaves the
pool-free router** — its real half reads the store — so `/sources` joins `/triage` and `/dashboard` on
the main router and the compile-time refusal of `State<MySqlPool>` no longer holds for it. That cost
was accepted once at 6b.5 and is accepted again here; **narrow it in writing rather than let it pass
in silence.**

⚠️ `/alerts` is a SECOND screen in this story and is example all the way through, so it stays
`Nature::Example` on the pool-free router with a new `ExampleContent` variant. **Two screens, two
natures, and the difference is load-bearing** — and the gap-hunt measured that the difference decides
which guards can see them.

#### 🔴 A SECOND `Mixed` SCREEN HAS NO PER-SECTION MARKER COVERAGE — MEASURED GREEN *WITH* A DATABASE

Built: a real `Mixed` `/sources` with two `example-section` blocks, **the second carrying no marker at
all**. `DATABASE_URL` set, full suite: **668/668 green.** Control — the marker removed from *both* —
reds the route-table partition. **So coverage is exactly *"at least one marker somewhere"*.** Both
per-section guards start from `rendered_dashboard(…)` and split on a class only `_dashboard.html`
carries; and the route-table loop's `screen-section == example-section` assertion sits inside
`if let Nature::Example(content)`, so **a `Mixed` screen gets no anchor assertion either**. Epic 5's
dominant class, **sixth consecutive story**, landing on the story that creates the second `Mixed`
screen. *Neither guard is wrong about what it tests.*

#### 🔴 THE ORDERING HAZARD: THE TWO HALVES OF THE EDIT FAIL ASYMMETRICALLY

| mistake | what reds |
|---|---|
| **nature changed, route forgotten** | **0 locally** — 668/668 green, no error, no warning — **1 in CI**. A silent 404. |
| route added, nature forgotten | **18 red**, all `Overlapping method route` |

Because the loop `continue`s on `Mixed` when no database is reachable, **every guard in it — marker,
*not-built-yet*, i18n-key, witness — is `DATABASE_URL`-gated for a `Mixed` screen.** Confirmed
separately: a visible i18n key planted on a `Mixed` body leaves **668/668 green locally** and reds only
with a database. ⚠️ **T1 introduces five new keys on a `Mixed` screen** — precisely that combination.
And `every_screen_is_refused_without_a_credential` cannot help: it asserts **401**, which `auth_deny`
returns above routing, so a 404 route passes it.

🔑 **REGISTER THE ROUTE FIRST, THEN CHANGE THE NATURE.** The wrong order is silent; the right order
fails loudly.

⚠️ **And `Mixed` is bought by T2, not by T1**: `declared_kinds()` is a pure function, so the capability
half reads nothing. Only the freshness needs the pool. Dropping T2's freshness would let `/sources`
keep the compile-time refusal of `State<MySqlPool>` — a trade worth naming before it is made by
accident.

---

## Tasks / Subtasks

- [x] **T0 — ✅ DISCHARGED 2026-08-20. Both arbitrations taken; development is UNBLOCKED (AC: 2, 4, 5)**
  - [x] §0b: **the liveness axis shows the FACT, with NO COLOUR, and says the incident axis is not
        built.** Example data behind the marker refused, with the reason.
  - [x] §0c: **option (a) refined** — a TYPE name from the connector's code; the screen lists
        CONFIGURED sources, never written `connector_id`s; the freshness is not keyed on
        `connector_id`; the boot-minted id is registered rather than fixed here.
- [x] **T1 — The capability half, REAL (AC: 1)**
  - [x] `FactKind::ALL` in **`opencmdb-core`** — ⚠️ *not* an exhaustive `match` from `opencmdb-bin`,
        which `#[non_exhaustive]` forbids (§0a) — plus **the one-line extension of
        `every_variant_of_a_navigated_enum_is_listed_in_all`**, measured red by the validation.
  - [x] The *byte-identical* claim narrowed to **no BEHAVIOUR change in `opencmdb-core`** (5.13b).
  - [x] The complement computed from `arp_ping::declared_kinds()`, never a literal list, and asserted
        as a **partition**: `unseen ∪ declared == ALL`, `unseen ∩ declared == ∅`, with a floor on
        `ALL.len()`. ⚠️ The draft prescribed *"a test plants an eighth variant"* — **a `#[test]` cannot
        add a variant**; that is a T7 mutation, not a test.
  - [x] The **positive half too** — *Observes: …* beside *Cannot see: …* (§0d-bis(3)).
  - [x] 🔴 **The copy says *what this source is BUILT to observe*, never *what it observed*** (§0b-ter):
        `declared_kinds()` is a compile-time constant and the screen must not dress it as a
        measurement. **FR7's static half only, and the story says so.**
  - [x] Bind and trace the `Ok(PollSummary)` at `scan_pass.rs:80` (§0b-ter, gesture 1) — **three lines,
        no schema** — and register the persistence with its precondition named.
  - [x] Copy in the **unlock** framing, not the consequence framing (§0d-bis(5)), in both locales.
  - [ ] 🔴 **NOT DONE, and the box was ticked — the code review measured the term absent from the
        whole tree**, in both languages, before and after this diff. `not evaluated` / *« non
        évalué »* is the spec's binding word for an **out-of-capability FIELD** (`:1130`), which is a
        device-record slot rather than a sources-screen one, so the omission is defensible — **but
        the task claimed delivery and delivered nothing**, which is the defect. Re-owned to the story
        that renders a field a source cannot observe. ⚠️ The five nouns and the spec-vs-enum
        divergence (§0d-bis(4)) ARE registered, and that half stands.
- [x] **T2 — The liveness half, per T0's arbitration (AC: 2, 4)**
  - [x] 🔴 **`repo::last_observed_at` AS IT IS — unkeyed.** It already is (`repo.rs:472`, no `GROUP
        BY`), so **T2 needs no new SQL**. ⚠️ The draft's *"per connector, as the dashboard already
        reads it"* was **false twice over** and contradicted §0c; a dev following it literally would
        reproduce the growing-row defect §0c exists to prevent.
  - [x] 🔴 **No colour on this axis**, and a test that reds if one appears.
  - [x] The sentence naming **Epic 13** as the incident axis's owner.
  - [x] 🔴 **§0b-bis's four states**: either distinguish them, or **state the ambiguity in writing**.
        The invalid-perimeter case is a misconfiguration the product already detected and buried in a
        detached thread's log — `/sources` is its natural home.
  - [x] The **zero configured sources** case needs copy and has none.
- [x] **T3 — The source's identity (AC: 5)** — per T0, with the limit written; and **`page::source_label`
      is touched or the story says in writing that it is not** (§0c), or AC5 reads closed over the very
      site the register names.
- [x] **T4 — `/alerts`, example and marked (AC: 3)** — a new `ExampleContent` variant with a
      **distinctive** witness, rows drawn from **FR30's three named alert kinds** rather than invented
      severities (§0d). ✅ The validation measured that the distinctiveness property already covers a
      fifth variant the day it exists.
- [x] **T5 — Copy, both locales; T6 — the stylesheet; T7 — the mutation pass; T8 — the browser look;
      T9 — the record.** Same shape as story 6b.7. ⚠️ **T7 must run its `Mixed`-screen mutations WITH a
      database** (§0e), and T6 must not build a class name in Rust (the stylesheet guard was measured
      blind to a class emitted from Rust, and to any `class="…"` carrying an expression).

---

## Dev Notes

### What the previous story leaves you, and it is mostly about guards

Story 6b.7's record is the reference; four of its lessons bear directly here.

- 🔴 **Assert on the REAL HTTP body, over `Screen::ALL`.** Three inherited guards were enumerations
  measured green on new screens; the marker rule and the i18n-key rule now live in `main.rs`'s
  route-table loop. **Add to that loop, do not add a screen-specific test beside it.**
- 🔴 **A witness must be DISTINCTIVE.** `demonstration_screen` prepends the marker to every example
  body, so a witness taken from the marker satisfies every screen forever. The distinctiveness
  property exists now; a new `ExampleContent` variant needs a witness that reds under it.
- 🔴 **The mutation driver can lie by failing to UNapply**: `copy2` preserves mtime and askama compiles
  templates into the binary, so a restored template left two full runs reporting a red test over a
  clean `git status`. Restore with `copy` + `os.utime`.
- ⚠️ **Rust does not lint an unused function parameter**, and *a guard that reads the source measures
  what was written, never what was served*.

### The house rules and the frontier

`opencmdb-core` is the domain and **must not** depend on `anyhow`, `axum`, `sqlx` or `askama`; a
`const ALL` is fine, an i18n key there is not — the SENTENCES for each unseen kind belong in
`opencmdb-bin`'s locale file, exactly as `state_vocabulary.rs`'s doc argues for the state axis.
No file over 2000 CODE lines. ⚠️ `xtask/src/main.rs` is the closest at **1908**, but `page.rs` at
**1524** is the file THIS story grows, and 476 lines of headroom for a `Mixed` screen, five capability
sentences and an alerts surface is a decision rather than a reassurance — story 6b.6 extracted
`example_screens.rs` pre-emptively for exactly this. Every
`pub` item documented, and **every doc comment TRUE**. Prove-to-red, with the mutation recorded and
its carrier named per row.

### What the operator will be able to DO with this screen

**Nothing, again** — and this story is where that stops being neutral. It makes **eight** well-lit
dead ends — ⚠️ the draft said seven; the count is per SCREEN and this story ships two, from a base
of six. ⚠️ But it is also the first screen that tells the operator something *true and useful they
did not know*: that their scanner cannot see hardware addresses, and therefore why the product
abstains. 🔑 **That is not a gesture, but it is not decoration either** — it is the first honest answer
the product gives to *"why is it not doing more?"*, and it is worth saying so in the retrospective.

### References

- [`epics.md:2244-2260`] — the three ACs. [`epics.md:2090-2110`] — Epic 6b's premises and constraints.
- [`architecture.md:1815`] — **D32**, `source_state` as two orthogonal axes, with `Live { last_ok }`
  and the `Blind → offline · Live+Reduced → degraded · Live+Full → full` projection surviving only as
  a UI projection. [`architecture.md:1411`] — the granularity, **per `(connector, subnet)`**.
- [`prd.md:867`] — **FR5**, the two axes, per (source, scope), with `full/degraded/offline` a DERIVED
  presentation and not a stored state. [`prd.md:869`] — **FR7**, the capability descriptor, dated and
  travelling with each batch; ⚠️ **its named trap is at `:871`, not `:869`** (*a field present,
  well-typed and wrong is more dangerous than one that is missing*). [`prd.md:872`] — **FR8** —
  ⚠️ the draft cited `:871` for it, which is the trap. [`prd.md:941`] — **FR30**'s three alert kinds.
- [`epics.md:449`] — **Epic 11**, *Source UniFi complète* (v0.9), covering **FR1, FR2, FR5, FR6, FR7** —
  the registry's real home. [`epics.md:469`] — **Epic 13**, the two-axis model made real (v0.11).
- [`opencmdb-core/src/connector/mod.rs:116`] — `PollSummary`, computed every scan and discarded at
  `scan_pass.rs:80`. [`score.rs:420`] — the uninhabited `SourceState`.
- [`ux-design-specification.md:1086-1115`] — *Source State: Two Axes, and Only One of Them Is a
  Colour*, including the rule AC2 quotes and the *credibility of amber* argument.
- [`crates/opencmdb-bin/src/arp_ping.rs:207`] — `declared_kinds()`, and `:255` its pin.
- [`crates/opencmdb-core/src/observation/mod.rs:201`] — `FactKind`'s seven variants; `:244` —
  `Capabilities`.
- [`deferred-work.md:3975`] — the connector-registry row this story owns by name.
- [`_bmad-output/implementation-artifacts/6b-7-applications-and-ipam.md`] — the guard lessons and the
  mutation-driver defect.

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), 2026-08-20.

### Debug Log References

Built and measured against a live `mariadb:10.11.16` on port **13321**; looked at in **Chrome 151**,
1280 px, in French, **on a rebuilt binary** — and the first capture was of a STALE one (see below).

### Completion Notes List

⚠️ **THE LIVE COUNT FOR THE PROJECT LIVES HERE**: **668 → 676 tests** (449 bin + 161 core + 66 xtask, after the code review).
Eight gates green, clippy and fmt clean, suite run **both ways** — **0.39 s** without `DATABASE_URL`
and **~5.8 s** against the live MariaDB. No migration, no new dependency, no new address.

**What shipped.** `/sources` leaves `Nature::Empty` for a screen whose **whole content is real**, and
`/alerts` for an example one. `FactKind::ALL` lands in `opencmdb-core`; `arp_ping` gains
`observes_and_cannot_see()` and a translated type name; `page.rs` gains the `/sources` handler and
view; two templates, ~30 keys in both locales, ~20 lines of stylesheet.

🔴 **THE STORY SAID `Mixed` AND THE CODE IS `Fed` — corrected while building, not after.** §0e reasoned
that `/sources` would hold a real section beside an example alert list. It does not: AC3 puts the alert
list on its **own** screen, which `Screen::ALL` has carried since 6b.2, so **every section of
`/sources` is real and it owes no marker at all**. Declaring it `Mixed` would have made the partition
demand a marker for example content that does not exist. 🔑 *A nature is a statement about what the
screen SHOWS, and the screen turned out simpler than the plan for it.*

🔑 **AC1 is REAL and DERIVED**: `FactKind::ALL` minus the connector's own declaration — `{Mac,
Hostname, DhcpLease, Uplink, OuiVendor}`, five kinds, no database, no fixture, no invention. The guard
that keeps `ALL` complete is a **cross-crate `include_str!` row** in
`every_variant_of_a_navigated_enum_is_listed_in_all`, measured red on a planted eighth variant and
measured **not to disturb the frontier gate** — `include_str!` is a compile-time read, not a
dependency. ⚠️ `FactKind` is `#[non_exhaustive]`, so the compiler could not carry it: a `match` from
`opencmdb-bin` demands a `_` arm that is then permanently silent.

⚠️ **AC1 delivers FR7's STATIC half only, and the story says so.** The screen shows *what the source is
BUILT to observe*, never *what it observed*. Per Guy's third arbitration the real dated descriptor is
**bound and traced rather than persisted** — `scan_pass.rs` now logs `capabilities={IpV4, Rtt}
as_of=… scopes_covered=1`, verified in a live boot's log — because persisting it needs a stable source
identity (**Epic 11**) and an accumulation decision (**D32 / Epic 13**).

🔴 **AC2 ships MET on one half and NOT MET on the other, and that is written rather than ticked.** The
liveness axis carries **no colour**: the product can establish *an observation arrived at T* and cannot
establish *this source is blind*. ⚠️ **The UX spec prescribes a colour for BOTH liveness values**, so
this is a **divergence**, registered as one. The incident axis is named to **Epic 13**, the owner three
artefacts already carry and which the draft asked for while supplying nowhere.

🔴 **`None` is FOUR states of the world and the screen says so** — never scanned, scanned and nobody
answered, an **invalid perimeter the product refused**, a blank one; `MAX(observed_at)` is NULL in all
four, measured on four live boots at the validation. And **zero configured sources** is a fifth case
the draft assumed away; it has its own copy now.

### What the browser found, and no test could

🔴 **The source's name shipped as an English literal under a French interface** — `"ARP/ping sweep"` —
**while the story's own §0c had written it in French** (« Balayage ARP/ping »). Story 6b.6's `role_key`
defect verbatim, **in the story that quotes it**: a literal is not a key, and the locale guard can only
see keys. Now `sources.name.arp_ping`. 🔑 And it is a KEY rather than data by story 6b.7's own rule: an
owner is a proper noun, **a source's TYPE is a classification**.

⚠️ **AND THE FIRST FIX'S SCREENSHOT WAS A LIE, because the binary was stale** — `cargo test` builds the
test target, not `target/debug/opencmdb`, so the served page still said *"ARP/ping sweep"* after the
key had landed. Story 6b.4b's lesson, met again: **grep the artefact you are about to believe, not the
source you just edited.** The served body was re-read after an explicit `cargo build`.

### The route-level key guard fired — for the WRONG REASON, and that was a defect of mine

🔴 With a database, `/sources` reddened the guard on `["example.No"]`. The copy is correct; the
**helper I extracted this morning joins across tags with nothing between them**, so
`…from an example.</p><p>No source is configured…` becomes `example.No`, which reads as a dotted key.
**A false positive manufactured by the guard.** Closed by pushing a space at every tag boundary.
🔑 *A check that fails for the wrong reason is worth nothing — and one that fails often enough for the
wrong reason gets deleted by whoever meets it next.*

### Mutation pass — 10 mutations, 10 reds, carriers named per row (two added by the review)

| # | Mutation | Result | Carrier |
|---|---|---|---|
| T1-red | an eighth `FactKind`, declared and absent from `ALL` | RED ×1 | assertion, the cross-crate guard row |
| M1 | the complement from a literal instead of the declaration | RED ×1 | assertion, the derivation guard |
| M2 | a state pill on the liveness axis | RED ×1 | assertion, the no-colour guard |
| M3 | the ambiguity sentence dropped | RED ×1 | assertion, the four-states guard |
| M4 | a source listed with no perimeter configured | RED ×1 | assertion, the no-source guard |
| M5 | the *Observes* half dropped | RED ×1 | assertion, the both-halves guard |
| M6 | `/sources` declared `Example` instead of `Fed` | RED ×**18 without a database, ×30 with one** | panic, all `Overlapping method route` — 🔑 **the loud half of the ordering hazard, confirmed live** |
| M7 | the alert witness taken from the nav (`alerts.title`) | RED ×1 | assertion, the distinctiveness property from 6b.7 |
| M8 | a kind loses its key pair and falls into the `_` arm | RED ×1 | assertion, `every_fact_kind_has_its_own_sentence` — **the hole the blind layer deduced from the diff alone** |
| M9 | a refused perimeter shown as if it were in force | RED ×1 | assertion, `a_refused_perimeter_says_so_rather_than_reading_as_configured` |

⚠️ **M6's figure was recorded as a bare *"30"* and the code review caught that it disagreed with
§0e's *"18"* — for one mutation, with no condition attached.** Both are true: **18** without
`DATABASE_URL`, **30** with a live database, re-measured both ways at the review. §0e's number was the
no-database one and this table's was the with-database one, and neither said so. *A count that omits
the condition it was taken under is two numbers pretending to be one.*

🔑 **M6 is the measurement the story was written around**: the route lives on the main router, so
declaring the screen `Example` makes the demo loop register it a second time and **eighteen-to-thirty
tests shout**. The mirror mistake — nature changed, route forgotten — is the silent one, which is why
the route was registered first and the order is documented at both sites.

### Code review — three layers, 2026-08-20, on a different model, each isolated

**Blind Hunter** (code diff only) · **Edge Case Hunter** (own worktree, mutations, four live boots) ·
**Acceptance Auditor** (own worktree, both diffs, the planning artefacts). **8 patches, 0
arbitrations.** ⚠️ **The cadrage error of the previous review was corrected**: the layers were handed
TWO diffs — the code and the documents — because last time the auditor was asked to audit register
rows that had been excluded from its input. Its brief named the incident.

🔴 **THE BLIND LAYER FOUND THE TWO HIGHS AGAIN — three stories running — and both were sentences of
mine.** A module doc said story 6b.8 *"made `Sources` `Mixed`"* **twenty lines below the arm that
sets it to `Fed`**, in the commit whose own headline corrects that same slip elsewhere. And
`kind_keys`' doc claimed the `_` arm *"falls back to the kind's `Debug` name"* when it returns a fixed
generic pair — 🔑 **from which the layer deduced, with no repository access, a hole neither sighted
layer named**: `FactKind::ALL`'s cross-crate guard pins the CONSTANT against the enum and says nothing
about this map, so **an eighth kind correctly added to `ALL` satisfies the guard and still renders
*"Unrecognised kind"***. *A guard placed where the defect cannot occur, one field over.* Closed by
`every_fact_kind_has_its_own_sentence`, which also pins distinctness and resolution; M8 reds it.

🔴 **THE MUTATING LAYER FOUND THE ONE DEFECT ONLY A BOOT COULD SHOW.** With
`OPENCMDB_SCAN_CIDR=nonsense`, the log carries `ERROR invalid OPENCMDB_SCAN_CIDR — skipping scan` and
`/sources` rendered a full source card reading *"Périmètre nonsense"* under the generic four-state
sentence — **the rejected string presented as an in-force value**. ⚠️ *That is stronger than the
ambiguity this story registered*: *we cannot tell which of four* is one thing; *we are showing a
configuration we already refused as though it were live* is another. Closed with the connector's OWN
parser, so the screen and the scan cannot disagree; M9 reds it. ⚠️ The perimeter is **still shown** —
the operator must see what was rejected — it is simply no longer shown as live.

🔴 **THE AUDITOR CAUGHT TWO NUMBERS PRETENDING TO BE ONE.** M6 was recorded as a bare **30** while
§0e said **18**, for one mutation with no condition attached. Re-measured both ways: **18 without
`DATABASE_URL`, 30 with a live database**. Both true, neither said so.

⚠️ **And a ticked task that delivered nothing**: T1's *"`not evaluated` used as the spec binds it"* —
the term is **absent from the whole tree**, in both languages, measured by two layers independently.
The omission is defensible (it is the spec's word for an out-of-capability FIELD, a device-record slot)
but **the box was ticked**, which is the defect. Unticked with its reason and re-owned.

⚠️ **A register row left silently false at its origin.** Story 6b.4's row still promised *"the day it
lands, `page::source_label` is where the name arrives"* — and 6b.8 landed without touching it. AC5
says *closed or re-owned, never left silently*. Struck through rather than deleted, with the
measurement that makes it doubly false (it renders a v7 UUID's millisecond field, rolling every ≈65 s)
and the owner moved to Epic 11.

⚠️ **A guard that fired with the wrong explanation.** The cross-crate row reused a message written for
`Screen` — *"it vanishes from the navigation, the routing"* — which is meaningless for `FactKind`. The
layer that planted the eighth variant read the red and found it describing another enum's world. *A
guard that fires with the wrong explanation sends its reader looking in the wrong place.*

✅ **EVERY GUARD THE STORY CLAIMS WAS MEASURED AND FIRED**, restored with `cp` + `os.utime` and never
`git checkout --`: the no-colour rule, the cross-crate `FactKind` row (with all eight gates still green
— confirming `include_str!` is not a dependency), both halves of the ordering hazard (**0 red locally
and exactly 1 in CI** for the silent one; **18** for the loud one), the witness distinctiveness, the
stylesheet class guard. Four boot configurations driven against four fresh databases; both locales
resolving fully; hostile queries reflecting nothing; **both terms of the test delta** re-derived by
`git stash`.

### File List

- `crates/opencmdb-core/src/observation/mod.rs` — MODIFIED (`FactKind::ALL`; ⚠️ **no behaviour change**)
- `crates/opencmdb-bin/src/arp_ping.rs` — MODIFIED
- `crates/opencmdb-bin/src/scan_pass.rs` — MODIFIED
- `crates/opencmdb-bin/src/page.rs` — MODIFIED
- `crates/opencmdb-bin/src/screens.rs` — MODIFIED
- `crates/opencmdb-bin/src/example_data.rs` — MODIFIED
- `crates/opencmdb-bin/src/example_screens.rs` — MODIFIED
- `crates/opencmdb-bin/src/main.rs` — MODIFIED
- `crates/opencmdb-bin/templates/_sources.html` — NEW
- `crates/opencmdb-bin/templates/_alerts_example.html` — NEW
- `crates/opencmdb-bin/assets/app.css` — MODIFIED
- `crates/opencmdb-bin/locales/app.yml` — MODIFIED
- `_bmad-output/implementation-artifacts/6b-8-sources-and-alerts.md` — MODIFIED (this file)
- `_bmad-output/implementation-artifacts/deferred-work.md` — MODIFIED
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — MODIFIED

### Change Log

- 2026-08-20 — contexted, validated by two fresh-context layers (twelve claims refuted), arbitrated by
  Guy on three points, implemented, mutated (8 mutations, 8 reds), looked at in a browser. Status →
  `review`.
