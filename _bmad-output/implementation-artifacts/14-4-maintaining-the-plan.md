# Story 14.4: Maintaining the plan — the corrections

Status: ready-for-dev

🔴 **SPLIT at its validation, 2026-09-16 (Guy): 14.4 keeps the CORRECTIONS — edit and delete — and
`14-4b-releasing-an-address.md` takes the RELEASE.** Epic 14 goes from six stories to SEVEN
(`epics.md` NOT edited; registered). Precedent: 5.9/5.9b, 5.14/5.14b, 6b.4/6b.4b, 14.2/14.2b,
14.3a/14.3b. The seam is the one the measurements support: **the two halves share nothing** — different
tables, a migration and a binding word on one side only.

⚠️ **The validation ran on the COMBINED story before the split**, by two fresh-context layers
(fact-check; gap-hunt, which built the options against a live MariaDB). Both halves carry its findings;
nothing below is inherited untested. **Four remaining decisions are in §2 and must be taken at T0.**

Epic 14 (IPAM). ⚠️ It does NOT close the epic: FR21's VLAN half and FR25 (IPv6) are in scope and
outside the 2026-09-10 arbitrations (`epics.md:2341`).

Baseline: **980 tests** (690 bin + 191 core + 99 xtask) at `82ff356` — verified by the validation, not
recalled. Ten gates, axe 0 over 10 routes + 5 states, kbd 45/45.

## Story

As the operator,
I want to correct what I defined,
So that the plan stays true as the network changes.

🔑 **These are the product's FIRST destructive gestures.** Every write it has ever shipped adds a row;
this story removes and rewrites them, and §1(h) measures what that does to the audit underneath.

## §0 — What is settled, and must not be re-opened

**Guy's six arbitrations of 2026-09-10** and the epic's **six measured constraints**
(`epics.md:2345-2357`) stand. **Story 14.2b's five decisions** (`14-2b-…md:168-228`) are reused, not
re-derived: the sub-router's state holds a **port and no pool** (`State<MySqlPool>` fails to compile,
`E0277`); a form lives **in the rail, collapsed**; the **server mints the id** and a **nil arriving id
is refused at the route**; a **label is required at the route**; an address inside a range is accepted
silently. **Story 14.3b's audit** is in force — and §1(h) is what this story owes it.

**Guy's decision of 2026-09-16 (3 of 4): the controls live as LISTS IN THE RAIL.** The rail already
renders the three write forms as disclosures; it gains a list of ranges and a list of defined
addresses, each row carrying its controls. **Refused: a control per cell** — 256 focusable elements is
the `role="grid"` / roving-tabindex cost 14.2 declined and `_ipam.html` records as a registered
divergence. **Refused: a detail panel on the selected cell** — it introduces a selection the screen
does not have, with its focus contract and its URL state. 🔑 One placement serves **both** branches
(the drawn grid and a subnet too large to draw), which no per-cell shape can.

## §1 — What this story inherits, each WITH the measurement that produced it

### (a) 🔴 THE STATEMENT CAP'S GUARD IS BLIND TO WHAT THIS STORY ADDS

`ipam_repo.rs`'s production half (before the first `#[cfg(test)]`, line 848) holds **ten** statements;
**five** carry `capped!`'s `SET STATEMENT innodb_lock_wait_timeout=4` — the three inserts (`:255`,
`:515`, `:609`) and the two locking reads (`:500`, `:576`). The others take no lock and are correct
uncapped (`load_subnet` `:279`, `list_subnets` `:646`, `plan_ranges` `:697`, `plan_addresses` `:755`,
`ranges_in` `:791`).

🔴 `every_plan_statement_that_can_wait_on_a_lock_is_capped` (`:1817-1845`) matches the needles
`"INSERT INTO ip_"` and `"FOR UPDATE"` and asserts `checked == 5`. **A `DELETE FROM ip_range …` or an
`UPDATE ip_range SET …` matches neither**, so the guard neither requires the cap nor reds, and the
count still holds. *A guard placed where the defect cannot occur reads as coverage and is none* — this
epic's dominant class, handed to this story as live work rather than as a warning.

### (b) 🔴 A RANGE EDIT OVERLAPS ITSELF

Measured: re-running `insert_range`'s sibling scan verbatim for a widened range (`.010–.099` →
`.010–.120`) returns **the row being edited, with `overlaps = 1`**. `range_attempt`'s scan (`:499`) has
no `id <> ?` clause, because an insert has no id in the table yet. **An edit that reuses it refuses
every legal widening.** It fails closed — loud, not silent — but *"reuse, do not re-implement"* is
**false for this one function**, and the shape a developer reaches for instead (delete-then-insert)
loses the row's identity and, for a moment, the overlap protection.

### (c) 🔴 "A RANGE THAT STILL HOLDS DEFINED ADDRESSES" IS NOT A FOREIGN KEY — a SUBNET's is

`ip_range` and `ip_address` both reference **`ip_subnet` only** (`0007:120`, `:153`); nothing relates
an address to a range. The epic's refusal is **computed containment**, and it races a concurrent
address write. Measured, inserter started 0.6 s into a 2 s window:

| | deleter counted | inserter waited | outcome |
|---|---|---|---|
| **with** `SELECT base FROM ip_subnet WHERE id = ? FOR UPDATE` | 0 | **1404.8 ms** | serialised |
| **without** it (the ordinary DRY line) | 0 | **4.5 ms** | the range was deleted and **an address landed inside it** |

🔑 **Name the mechanism, because neither function states it**: `insert_address` takes no lock at all
(*"NO LOCK HERE"*, its own doc). What blocks it is the **foreign key's shared lock on `ip_subnet`**.
The delete's safety rests on a key, not on a promise — an address write that stops touching
`ip_subnet` loses it in silence.

❌ **`insert_range_pausing` is NOT the instrument.** It is a private seam inside `insert_range`'s body,
parameterised on that function's decide-then-write; a delete needs **its own** seam. ⚠️ And its scope
was narrowed at 14.2b's review: the harness reds on the COMPOSITE, while the deciding read's lock alone
is carried by the source guard `both_reads_of_a_subnet_under_write_take_their_lock` — *two guards that
each mask the other's mutation are two guards nothing measures*.

**A SUBNET delete is cheaper**: `DELETE FROM ip_subnet` answers `ERROR 1451` on `ip_range_subnet_fk` or
`ip_address_subnet_fk` — a foreign key, no count, no lock, no race. ⚠️ But `constraint_refusal` maps
`"foreign_key"` → **404 `ipam.refusal.unknown_subnet`**, the wrong sentence for *"this subnet still
holds ranges"*; that mapping must be split by route.

### (d) The adapter can only INSERT

Every `DELETE` in `ipam_repo.rs` is **test cleanup** (eleven sites, all below the cut) and **there is no
`UPDATE` at all** — every occurrence is `FOR UPDATE` inside a SELECT. This story writes the product's
first production `UPDATE` and `DELETE`.

### (e) 🔴 A DELETE CHANGES OTHER ADDRESSES' VERDICTS, AND NOTHING WARNS BEFORE IT

Measured against `ipam_audit::Plan` (three observed addresses inside a `dhcp-pool` range):

| | before | after |
|---|---|---|
| findings, deleting that pool range | 0 | **3** (`.90 .91 .92` → `Undeclared`) |
| the offer, deleting the `static` range | `Some(192.0.2.1)` | **`None`** |
| the offer, deleting the `ip_address` row for `.1` | `Some(192.0.2.2)` | `Some(192.0.2.1)` |

🔑 14.3b built `/ipam/address-check` and `/ipam/range-check` to warn before a **create**; this story
adds the gestures that are **irreversible** and that change *other* addresses' verdicts, with no
warning before the write. The mechanism exists. Decision 3.

### (f) The machinery, and where it over-claims

- **`WriteRoute::ALL` is a hand-written array** (`ipam_write.rs:188`). `path()` and `handler()` are
  exhaustive `match`es, so a new variant reds there — **but a variant added to both and omitted from
  `ALL` compiles cleanly and is never mounted**, and nothing pins `ALL`'s completeness. The file's own
  doc over-claims this; do not inherit the claim.
- **`same_origin` passes when `Origin` is absent** (`write_guard.rs:46`) — registered to Epic 19.
- **Every write GESTURE is `hx-post`**; `hx-get` exists for reads. The write sub-router has only ever
  seen POST.
- **No gate names the plan's tables.** `observed-immutable` guards `observation_record`,
  `entity-id-immutable` guards `declared_attribute`, `authorship` guards its writes; `AUTHORSHIP_ROOTS`
  already covers the files. Story 5.12's own doc says to reopen a gate's perimeter when a new writer
  appears. Decision 4.
- ✅ **`a11y/seed.sql` already reaches BOTH delete cases**: `b001` (static, holds `.9`) and `b002`
  (dhcp-pool, holds `.90`) must be REFUSED; `b003` and `b004` hold nothing and succeed. Unlike 14.3b,
  no seed widening is owed for the deletes.

### (g) The vocabulary, and an epic that owns two of these words

**The gesture table is `prd.md:991-1002` — ten rows** (observed · declared · gap · reconcile ·
document · accept-gap · snooze · exclude · triage · source); the UX spec's (`:1339-1351`) has eleven,
adding `attach`. **`edit` and `delete` have no row — and `create` has none either** (it appears in an
MVP bullet, in FR14, and inside the `triage` row's *Meaning* column).

🔴 **`edit` and `delete` are owned BY NAME by Epic 21**: FR40 (`prd.md:961`) *"edit the declared
attributes of a record"*, FR41 (`:962`) *"decommission, archive, or delete a declared device,
**subnet**, or application"*, assigned at `epics.md:379-380` to E21 (`:502`). What separates this story
from that one is **Guy's arbitration 4 — IPAM and `declared_attribute` are two registers** — and it is
said here rather than left for a dev agent to meet. ⚠️ FR41 naming *subnet* is what makes decision 2
real.

## §2 — Decisions, ✅ ALL FOUR TAKEN 2026-09-16 (Guy)

Each is recorded **with the option refused**, so that none is re-opened in silence by a dev agent
meeting it mid-implementation.

1. ✅ **THE ROUTE SHAPE: more POST routes on the existing machinery.** `WriteRoute` gains its variants;
   the exhaustive `match` reds if one is not mounted, and the Origin check, the keyed refusals, the
   budget and `settle` are reused verbatim. **Refused: real verbs** (`hx-delete`, `hx-put`) — htmx
   2.0.4 emits them, but the write sub-router, the CSRF guard and the perimeter guard have only ever
   seen POST, and that is new machinery in the story that introduces the product's first irreversible
   gestures. ⚠️ **THE ACCEPTED COST, written rather than discovered: a path that says `delete` in a
   product whose every gesture is a POST.** Say it at the route; do not let the shape imply a verb.
2. ✅ **DELETING A SUBNET IS IN.** Guy's arbitration 4 separates the registers: `ip_subnet` is the
   PLAN, not `declared_attribute`, so the gesture belongs to this epic. **Refused: leaving it to Epic
   21**, whose FR41 (`prd.md:962`) names subnets — the register distinction is what separates them, and
   without this route creating a subnet stays a one-way act, which is the ratchet this story exists to
   end. 🔑 Measured cheap: the database refuses it alone (`ERROR 1451` on `ip_range_subnet_fk` or
   `ip_address_subnet_fk`) — one route, one refusal sentence, no count, no lock, no race. ⚠️ And
   `constraint_refusal`'s `"foreign_key"` → 404 `ipam.refusal.unknown_subnet` must be **split by
   route**: *"this subnet still holds ranges"* is not *"no such subnet"*.
3. ✅ **A DELETE WARNS BEFORE IT FIRES — for BOTH deletes.** The product already warns before a
   CREATE (14.3b's two check routes); not warning before an irreversible destruction that changes
   **other** addresses' verdicts would be that asymmetry pointed the other way. Measured (§1(e)):
   deleting a `dhcp-pool` range manufactures **three** findings at once; deleting a `static` range
   empties the offer. **Refused: the range only** — an operator would have to guess which gesture
   speaks; **refused: a refusal and a confirmation alone** — they say *this is not allowed* and *are
   you sure*, never *here is what will change*. 🔑 It WARNS and does not refuse, on
   `epics.md:2433`'s own rule.
4. ✅ **NO NEW GATE — and the reason is WRITTEN rather than left to the gates' silence.** The ten
   gates protect the OBSERVED side (`observed-immutable`) and the provenance of the DECLARED side
   (`authorship`, `entity-id-immutable`). The plan is the operator's own register, where deleting **is
   the gesture** and not a violation — so a gate refusing it would be a barrier against the product's
   own subject. **Refused: a sanctioned-site gate** on `authorship`'s model (it would close the class
   for good, at the price of an eleventh gate and its probe corpus, in a story already dense);
   **refused: deferring with a register row**, because this project has measured that *a register row
   with no named owner is not an action*. ⚠️ **What IS owed and is AC1's work: widening the lock-wait
   cap's guard**, which is blind to `DELETE`/`UPDATE` (§1(a)) — the one place where the gates' silence
   really does hide something.

⚠️ **THE ACCEPTED COST OF 2 AND 3, STATED RATHER THAN DISCOVERED: this half is now FIVE write routes
plus one check route** — edit range · edit address · delete range · delete address · delete subnet ·
`GET /ipam/delete-check`. Story 14.2b, this epic's size reference, shipped **three** write routes in 22
files and +5071 lines. The split already taken (corrections | release) is not re-opened here; the size
is written so that T6's measurement is read against it rather than against a memory of three.

## Acceptance Criteria

**AC1 — FIVE write routes on 14.2b's machinery — and WIDEN what its guard sees.** Edit a range · edit
an address · delete a range · delete an address · **delete a subnet** (decision 2), all POST (decision
1), all mounted from `WriteRoute`'s exhaustive `match`. The Origin check, the keyed refusals, the
budget and `settle` are reused, and a test asserts the reuse rather than a comment claiming it.
⚠️ **Two exceptions are MEASURED and must be BUILT rather than assumed**: the statement cap's guard is
blind to `DELETE`/`UPDATE` (§1(a)), and the sibling scan refuses a legal widening (§1(b)).
⚠️ **`WriteRoute::ALL` is a hand-written array** (§1(f)): five new variants are five chances to add one
to both `match`es and forget the list, which compiles cleanly and mounts nothing.

**AC2 — deleting a range that still holds defined addresses is REFUSED BY NAME, never cascading**, and
the refusal holds under concurrency with the parent-row lock (§1(c)). Its instrument is a **new** pause
seam. **A subnet delete is IN** (decision 2): the database refuses a non-empty one by foreign key, and
that refusal needs **its own sentence** — `constraint_refusal`'s `foreign_key` arm is split by route,
because *"this subnet still holds ranges"* is not *"no such subnet"*.

**AC2b — a delete WARNS before it fires, for both deletes** (decision 3), on 14.3b's contract: a GET
check route, a polite live region, focus left where the operator is, and **no refusal** — it says what
will change, it does not stop the gesture. It names what the measurement names: how many silent
addresses become findings, and whether the offer is emptied. ⚠️ The warning is reached only by a
gesture, so it is on no page a URL walks: the keyboard gate PRESSES it, as it does the address check.

**AC3 — every refusal is a KEYED body in both locales**, never a Rust `format!`.

**AC4 — the destructive gestures are keyboard-reachable and named to a screen reader**, measured by
`a11y/kbd-probe.mjs` (which PRESSES) and clean under axe; a confirmation, if built, carries story
6b.11's focus contract.

**AC5 — no gesture can destroy what it did not name**: a delete names one id, and the test that proves
it leaves the neighbours standing.

**AC6 — the rail's two lists** (Guy's decision 3 of 2026-09-16) render in BOTH branches — the drawn
grid and a subnet too large to draw — and carry their controls.

**AC7 — THE LIVE COUNT lives here**, both store conditions and the command named. Baseline **980**.

**AC8 — no regression**: ten gates, `clippy --all-targets -D warnings`, `RUSTFLAGS="-D warnings"` on a
dropped-and-recreated store after one warm run and storeless, `cargo deny`, both manuals, both browser
gates, documents current before the push.

## Tasks / Subtasks

- [x] **T0** Take §2's four decisions with Guy. ✅ 2026-09-16 — all four taken, each recorded with the
      option refused; §0 and §1 are settled and are not re-opened.
- [ ] **T1** (AC1) The adapter's edit and delete with the parent-row lock and `capped!`; **widen the
      cap's guard** to cover `DELETE`/`UPDATE`, proven red before it passes. ⚠️ Write, at the site, the
      reason no new gate is owed (decision 4) — the gates' silence must not stand in for it.
- [ ] **T2** (AC1) The edit's own sibling scan (`id <> ?`): a legal widening accepted, an illegal one
      still refused.
- [ ] **T3** (AC2, AC5) The computed refusal, its **own** pause seam, the concurrent-inserter
      measurement, the neighbours-stand-still test; the **subnet route**, whose refusal the database
      raises alone (`1451`); `constraint_refusal`'s `foreign_key` arm split by route, so *"this subnet
      still holds ranges"* stops being served as *"no such subnet"*.
- [ ] **T3b** (AC2b) `GET /ipam/delete-check` on 14.3b's contract — a polite live region, focus left
      where it was, **no refusal** — naming how many silent addresses a deletion turns into findings and
      whether it empties the offer. It is reached only by a gesture, so the keyboard gate PRESSES it.
- [ ] **T4** (AC3) The refusal set over what each handler can RECEIVE, keyed in both locales.
- [ ] **T5** (AC4, AC6) The rail's two lists and their controls; the keyboard gate's new checks; the axe
      pass. The seed already reaches both delete cases.
- [ ] **T6** (AC7, AC8) Measure. Prove-to-red each new guard with `cargo xtask mutate --baseline`,
      predictions written first; both browser gates; the documents and both twins.

## Dev Notes

### Traps this project has paid for, and which apply here

- 🔴 **A guard placed where the defect cannot occur reads as coverage and is none** — §1(a) is a live
  instance, not a hypothetical.
- 🔴 **A plant that does not COMPILE measures nothing**, and its exit status is indistinguishable from a
  finding — the validation hit this and caught it only because its control failed too. Plant against a
  green control, and use `--bins` (this crate has no lib target).
- 🔴 **A `contains` oracle over a TRANSLATED sentence cannot measure the negative direction** (Askama
  escapes `'` to `&#x27;`; 14.3b's MP6). Anchor a negative render assertion on a literal.
- 🔴 **A mutation named for one thing and applied to another measures the other thing.** Write the
  prediction first; `cargo xtask mutate --baseline` checks it.
- ⚠️ Store-backed tests take `DB_TEST_LOCK` and own their CIDR; drop and recreate before any count,
  with one warm run first. `app.yml` is invisible to incremental builds. Read an exit status from a
  FILE, never a pipe. `clippy --workspace -- -D warnings` does not check test targets — use
  `--all-targets`.

### Project Structure Notes

- `ipam_write.rs` owns the routes, `ipam_repo.rs` the SQL, `ipam_page.rs` the screen.
- The `file-size` ceiling is 2000 CODE lines: **`page.rs` is at 1954** — 46 from red — while
  `ipam_page.rs` is at 1357, `ipam_write.rs` 862, `ipam_repo.rs` 847. `page.rs` is the file at risk.

### References

- `epics.md` — Story 14.4 `:2443-2461`, constraints `:2345-2357`, Epic 21 `:502`, FR40/FR41 `:379-380`.
- `prd.md` — FR21-24 `:906-909`, FR40 `:961`, FR41 `:962`, Epic 21's edit `:888`; gesture table
  `:991-1002`, STATE `:1014-1018`, PLAN `:1036-1039`.
- `ux-design-specification.md` — gesture table `:1339-1351`, Occupancy Grid `:1227-1231`.
- `deferred-work.md` — `:5671` (the address warning's stated limits), `:5672-5676` (`addresses_in`
  removed; write a per-subnet read if needed rather than inherit one).
- `14-2b-the-plan-in-the-operators-hands.md` §2 `:168-228`; `14-3b-the-audit.md`;
  `14-4b-releasing-an-address.md` (the other half).
- `migrations/0007_addressing_plan.sql` `:91-156`; `src/ipam_write.rs` (`ALL` `:188`, `settle` `:443`);
  `src/ipam_repo.rs` (sibling scan `:499`, parent lock `:576`, the cap's guard `:1817-1845`).

### Two document defects the validation found, which a story may NOT fix

- `ux-design-specification.md:1345` still gives `document`'s FR label as **« Merger »** — retired by
  story 6b.10; `prd.md:997` says « Ajouter ». A stale row in a BINDING table.
- Story 6b.7's absent *Réserver* control is called *registered* in its own file and appears in **no
  register row** — *a section that says "registered" is not a registration*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
