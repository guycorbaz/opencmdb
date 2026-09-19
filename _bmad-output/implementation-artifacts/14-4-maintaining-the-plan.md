# Story 14.4: Maintaining the plan — the corrections

Status: done

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
(*"NO LOCK HERE"*, its own doc). ~~What blocks it is the **foreign key's shared lock on `ip_subnet`**.
The delete's safety rests on a key, not on a promise — an address write that stops touching
`ip_subnet` loses it in silence.~~

🔴 **STRUCK, AND THE STRIKE IS THE POINT — that sentence was REFUTED at T6 by three mutations, and
the code review found it still standing here without a marker.** The section it sits in is headed
*each WITH the measurement that produced it*, so a reader who stops at §1 takes a refuted attribution
as measured fact. What the mutations established: the serialisation has **two independent carriers** —
the parent row's lock AND the address scan's next-key/gap locks — and **removing either one alone reds
nothing** (M-T3, M-T3b). Only removing BOTH collapses the wait, to **2.86 ms** (M-T3c). ⚠️ So the
table above separates *locked* from *unlocked*, never one lock from the other, and the *"stops
touching `ip_subnet` loses it in silence"* clause is false while the scan stands. *A correction that
adds the true text without removing the false one leaves the record carrying both* — hence the strike
rather than a quiet deletion.

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

**AC2b — a delete WARNS before it fires, for ALL THREE deletes** (decision 3), on 14.3b's contract: a GET
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
- [x] **T1** (AC1) The adapter's edit and delete with the parent-row lock and `capped!`; **widen the
      cap's guard** to cover `DELETE`/`UPDATE`, proven red before it passes. ⚠️ Write, at the site, the
      reason no new gate is owed (decision 4) — the gates' silence must not stand in for it.
      ✅ 2026-09-16. **The widened guard was proven red on a prediction written first**: adding the
      **two destructive verbs** to its needles reddened at `left: 9, right: 5`, and the two range
      writes then took it to **`left: 15, right: 9`** — predicted before the run, and the count was
      moved only after recounting by hand. ⚠️ **This read *"adding the destructive verbs AND
      `FOR UPDATE"`"*, which the code review refuted from the diff: `FOR UPDATE"` was ALREADY a
      needle** — the removed line is `for needle in ["\"INSERT INTO ip_", "FOR UPDATE\""]`, and §1(a)
      of this very file says so. Only two needles were added, and the `left: 9` arithmetic works
      *because* the fourth was already there (5 + 2 deletes + 1 update + the address edit's locking
      read = 9). ⚠️ The count is **16** on the reviewed tree, not 15: the abandonment rule added a
      locking read, and the figure was again read off the guard rather than written ahead.
      Five writes land: `delete_address`, `delete_subnet`, `update_address`, `delete_range` (the
      computed refusal, containment compared in Rust per D10) and `update_range`, whose sibling scan
      is a SECOND SQL literal carrying `id <> ?` rather than an `Option<&str>` on the insert's — *a
      parameter one can forget to pass is exactly where forgetting becomes the defect*.
      `IpamError::RangeStillHoldsAddresses` joins the domain (ALL 7 → 8) with the compiler naming
      **two** non-exhaustive sites — `Display` in the domain crate and `ipam_refusal` in
      `ipam_write.rs`, an exhaustive `match` with no `_` arm — where this line said *"the one"*, a
      miscount of the compiler's own help caught by the review. Its key is in both locales. ⚠️ **Two things are NOT proven at T1 and are not claimed**: the widened guard
      has reddened on a COUNT only, never yet on a `DELETE` that lost its cap — that is a T6 mutation —
      and the five writes have **no store-backed test at this point**, only the compiler and the source
      guards. ⚠️ The tree carries **seven `never used` warnings** (the five writes and two helpers) and
      is therefore **not push-ready until T3 wires the routes**: they are the compiler naming work not
      yet done, and silencing them with an `allow` would delete that signal and re-open the module-wide
      allow story 14.1 registered for narrowing. Storeless: **691 + 191 + 99**.
- [x] **T2** (AC1) The edit's own sibling scan (`id <> ?`): a legal widening accepted, an illegal one
      still refused. ✅ 2026-09-16 — two store-backed tests, each on its own `/24`
      (`100.66.10.0`, `100.66.11.0`), **9.67 s against a live store where the storeless run is 0.00 s**,
      which is the tell that they executed rather than returning early. The widening test carries the
      exclusion clause on THREE assertions, and the third is the load-bearing one: **re-applying a
      range's own current bounds must be accepted** — the self-overlap case in its purest form, where
      a widening could still be argued about. The refusal test reads the neighbours BACK and compares
      the plan whole, because *an `Err` proves the decision and never the rollback*, and it re-asks the
      insert's other refusals on the edit path (`RangeOutsideSubnet`, `RangeBoundsInverted`, plus
      `NotFound` for a stale id) — a rule held on one path and not on its twin is the shape story 6.3
      named.
      🔴 **THE PROVE-TO-RED COULD NOT RUN, AND THE REFUSAL IS THE MEASUREMENT.** The planned pair was
      written first — **M-T2a** neuter `id <> ?` (predicted `red:1`, the widening test alone, with the
      refusal test GREEN as the control, since without the exclusion an overlap is still refused) and
      **M-T2b** neuter the overlap decision (the mirror). ⚠️ Note the correction made before firing:
      DELETING ` AND id <> ?` leaves **two binds against one placeholder**, so the query fails on arity
      and the test reds for the wrong reason — this project's own *"a mutation named for one thing and
      applied to another"*. The mutation therefore preserves arity (`id <> ?` → `? IS NOT NULL`).
      `cargo xtask mutate --baseline` then answered **exit 2**: `🔴 THE BASELINE IS NOT CLEAN:
      Red { tests: 0, clippy: true, gates: false } — nothing measured after this could be attributed
      to the mutation`, clippy being red on the seven `never used` writes. 🔑 **So the dead-code
      warnings do not merely block a push: they disable the instrument.** The pair was NOT dropped and
      NOT hand-rolled around a correct refusal: it waited for **T3** to give the writes their callers,
      which is what made the baseline clean.
      ✅ **M-T2a then RAN and conformed** (baseline clean: clippy green over `--all-targets`,
      695 + 191 + 99): `PREDICTED: Red(Some(1))` · `MEASURED: Red { tests: 1 }` · exit 0, applied at
      `ipam_repo.rs:917`, one site, the diff printed. 🔴 **But the driver reports no test NAMES — its
      whole log is 31 lines of summaries — so a matching COUNT is not a matching CARRIER**, which is
      this project's four-occurrence class *a red credited to the wrong cause*. The carrier was
      therefore established by hand (own copy, mutation verified applied, restore verified, never
      `git checkout`): **`a_range_edit_widens_into_free_space_and_onto_its_own_ground` FAILED** and
      **`a_range_edit_that_would_overlap_a_neighbour_is_refused_and_moves_nothing` stayed ok** — the
      green control, which is what proves the exclusion clause is carried by the first test
      SPECIFICALLY rather than by the pair between them. ⚠️ A correction made before firing is
      recorded rather than smoothed away: DELETING ` AND id <> ?` leaves two binds against one
      placeholder, so the query fails on arity and the test reds for the wrong reason — the mutation
      preserves the arity (`id <> ?` → `? IS NOT NULL`) so that what it measures is the RULE.
      ✅ **M-T2b, the mirror, also RAN and conformed**: neutering the overlap refusal
      (`return Err(RangeOverlapsAnother)` → a binding-consuming no-op, so clippy stays green and the
      mutation measures the RULE rather than a compile break) gave `PREDICTED: Red(Some(1))` ·
      `MEASURED: Red { tests: 1 }` · exit 0 at `ipam_repo.rs:929`, one site. Carrier established by
      hand as above: **the refusal test FAILED and the widening test stayed ok**. 🔑 **So each of the
      two tests is the OTHER's green control**, which is what makes the pair a measurement rather
      than two passes: M-T2a reds only the widening test, M-T2b reds only the refusal test, and
      neither mutation reds both. *A guard covered only by a disjunction is a guard nothing covers*
      (story 6b.10's finding) — this is its opposite, and it is measured rather than asserted.
      🔴 **M-T3 CAME BACK GREEN, AND THE REFUTATION IS THE DELIVERABLE.** Removing the parent-row
      lock from `delete_range_pausing` — *"the ordinary DRY line"* §1(c) says brings the race back —
      reds **nothing**: the mutation verifiably applied (`load_subnet_locked` calls 7 → 6, restored
      to 7) and `an_address_cannot_land_inside_a_range_while_it_is_being_deleted` stayed **ok**.
      🔑 **The cause took THREE mutations to establish, and the first two each measured nothing.**
      **M-T3b** (drop the address scan's `FOR UPDATE`, leaving the parent lock): the seam test again
      **ok**, while the cap guard reddened at `left: 14, right: 15` — proving the mutation applied
      and refuting the *"probable carrier"* this record named one draft earlier. **M-T3c** (drop
      BOTH): the seam test finally **FAILED**, the inserter returning in **2.86 ms** where the pair
      took **406.87 ms** — the deleter's 400 ms pause running alone, and the validation's
      without-lock figure of 4.5 ms reproduced to the same order against its 1404.8 ms with.
      🔴 **So the serialisation has TWO INDEPENDENT CARRIERS and either alone suffices**: the parent
      row's lock, and the address scan's own next-key/gap locks over that subnet's index range,
      which block an `INSERT` there with no foreign key involved. *A property with two carriers is a
      property a mutation of either measures nothing about* — story 6.5's M6 and 6.4b's P3, met on a
      third axis. ⚠️ The sentence at the site said ONE mechanism (*"the delete's safety rests on a
      key, not on a promise"*); it is INCOMPLETE rather than false, and is corrected to what the
      three mutations support, not explained away.
      ⚠️ **M-T3c produced a THIRD red that is COLLATERAL and is not counted as a carrier**: the seam
      test panicked, so its trailing `forget_subnet` never ran, and its interloper `100.66.12.15`
      survived into a plan-wide address test. That is exactly the hazard this module's own doc
      records — *a test that panics skips its cleanup, and the next test dies on something other
      than the thing under test* — and counting it would be the inflated-carrier-count error that
      doc exists to warn about. The database was dropped and recreated before the next count.
- [x] **T3** (AC2, AC5) The computed refusal, its **own** pause seam, the concurrent-inserter
      measurement, the neighbours-stand-still test; the **subnet route**, whose refusal the database
      raises alone (`1451`); `constraint_refusal`'s `foreign_key` arm split by route, so *"this subnet
      still holds ranges"* stops being served as *"no such subnet"*.
      ✅ 2026-09-16. **`WriteRoute` goes 3 → 8**, so ~~every exhaustive `match` became an `E0004`
      naming each site rather than a list someone had to remember~~ — 🔴 **REFUTED by the group-B
      review, and the strike is the correction**: every `match` is exhaustive, but `ALL` is a
      HAND-WRITTEN array and `router_with`, `paths()`, `main.rs`'s premise and the coverage assertion
      all derive from it, so they agree because none has a second opinion. A variant left out of
      `ALL` is mounted by nothing and no `E0004` says so. What carries it is a second hand-written
      list pinned by an equality, **a tripwire and not a barrier**. ⚠️ This sentence stood unstruck
      here, and both twins published it as a property, until the slice-D review — *a correction that
      adds the true text without removing the false one leaves the record carrying both*, which is
      this story's own rule, applied twice elsewhere in this very file and not here. The routes are:
      five POST routes (decision 1), five port
      methods with three implementors, five handlers, five request structs. 🔑 **Giving the writes
      their callers is what cleared the seven `never used` warnings and un-blocked the mutation
      driver** — the T1 note that they merely blocked a push was HALF TRUE: they disabled the
      instrument. **`delete_range_pausing` is the delete's OWN seam**, not `insert_range_pausing`,
      which is parameterised on the insert's decide-then-write; the window sits after the addresses
      are counted and before the range goes, which is exactly where §1(c) measured one landing.
      🔴 **Both refusal mappings split by route**: `1451` on a subnet delete means *it still holds
      ranges* (409) where it meant *no such subnet* (404) — the operator was told the subnet did not
      exist while looking at it — and `NotFound` now names the RECORD the id meant, where one
      sentence had reported a missing SUBNET for a range the plan no longer held.
      🔑 **Three decisions taken at the site and recorded as mine, reversible**: a correction answers
      **200, never 201** (the shared guard demanded `CREATED` for every route, and a deletion claiming
      it created something is a false statement in the protocol that only the guard kept true — story
      6b.4's raw UUID and 14.2's `offerable == 256` are the same shape); a **removed subnet redirects
      to `/ipam` bare**, since `?subnet=<the id just deleted>` would render the unknown-subnet branch
      one instant after the operator removed it at their request; and a **`unique` violation on a
      DELETE is a fault, not a re-entry** — the three deletes answer the backend sentence on this
      file's own reasoning for `check`, rather than being given re-entry copy no operator can reach.
      ⚠️ **A guard's property was corrected, not satisfied**: the refusal-set test demanded a distinct
      re-entry sentence per ROUTE, exact only while routes and records stood in bijection. They no
      longer do — `EditAddress` and `Address` concern the SAME record and must answer the SAME
      sentence — so it now asserts the property 14.2b actually found (distinct per RECORD, equal
      within one) plus a coverage check that a ninth route cannot be silently unclassified.
      Counts: **695 + 191 + 99**, clippy clean over `--all-targets`, 23.11 s against a virgin store
      versus 5.02 s storeless — the clock is the tell. ⚠️ **T3b is NOT in this** and remains open.
- [x] **T3b** (AC2b) `GET /ipam/delete-check` on 14.3b's contract — a polite live region, focus left
      where it was, **no refusal** — naming how many silent addresses a deletion turns into findings and
      whether it empties the offer. It is reached only by a gesture, so the keyboard gate PRESSES it.
      ✅ 2026-09-16 — **axe: 10 routes + 5 states, 0 violation nodes; kbd: 52 checks, 0 failed**, the
      floor moved 50 → 52 for the two checks that PRESS the warning in a browser. The route reuses the
      `AddressCheck` fragment rather than minting a parallel one, answers **200 with a keyed sentence**
      on a store failure (htmx does not swap a 5xx, so the region would otherwise keep the previous
      answer under a new question — 14.3b's measured defect), and computes from the audit's own API
      (`seen_inside`, `has_static_range`) rather than restating either rule.
      🔴 **A DESIGN FLAW CAUGHT WHILE WIRING THE CONTROL, not after**: `?subnet=<id>` alone means the
      SUBNET's own removal, so the address row's control would have been answered *"this subnet still
      holds 5 records"* — **a true sentence about the wrong gesture**, which is worse than none. The
      query gained `addr`, the renderer an arm that answers it FIRST, and the test asserts the subnet
      sentence is ABSENT there, so the confusion cannot come back silently.
      🔴 **AND THE MODULE HAD NO KEY GUARD AT ALL.** `ipam_write.rs` has guarded its keys since 14.2b;
      `ipam_page.rs` — which renders more of them — was covered by nothing, so this story's thirteen
      new keys went in under a green suite, ten gates and a clean clippy. *The silent pass was not
      evidence they were fine; it was evidence nobody was looking.* A guard now scans this file
      (needle assembled at RUNTIME, or it finds itself — its sibling hit that twice in five minutes)
      and asserts **73 keys** resolve non-blank in both locales. ⚠️ Its first list carried
      `ipam.policy.`, a NAMESPACE PREFIX from another guard's `starts_with`; skipped as a property
      (*a key never ends in its separator*), never by naming that one string.
      ⚠️ **The count assertion fires BEFORE the resolution loop** (story 5.13's family), so until the
      count was set from the printed list, not one translation had ever been checked. ✅ Then proven
      red for its REAL property: blanking `ipam.rail.delete`'s French value reds it naming that key —
      with a `.rs` touched to force a rebuild, because `app.yml` is invisible to Cargo's incremental
      build and the old string stays embedded otherwise (story 6b.9's trap, neutralised rather than
      walked into). Counts: **697 + 191 + 99**, clippy clean over `--all-targets`.
      ✅ **And the two new browser checks were PROVEN RED, prediction written first**: removing the
      `hx-get` from the range removal control gives **52 checks run, exactly 1 failed** — the *warns
      before the write* check, its region measured `""` — while the *focus stays on the control*
      check keeps PASSING, because focus is untouched by that mutation. 🔑 **The asymmetry is the
      point**: a mutation that reddened both would have meant one property measured twice, and this
      one shows them measuring different things. Anchor matched once, build clean, restore verified.
      ⚠️ Neither check is reachable from source: the warning exists only once a browser focuses a
      control, which is why it is the GATE that presses it and not a Rust assertion.
- [x] **T4** (AC3) The refusal set over what each handler can RECEIVE, keyed in both locales.
      ✅ 2026-09-16 — verified in BOTH halves of the criterion rather than the half that happened to
      be covered. **Keyed, in both locales**: `every_refusal_the_handler_can_receive_names_a_rule`
      drives **8 routes × 7 receivable errors × 2 locales** and asserts none renders its own key name;
      two key guards now cover the modules that render them — ~~38~~ **39** in `ipam_write.rs`, and
      ~~73~~ **75** in **`ipam_page.rs`, which had none at all before this story**, both asserting
      non-blank in `en` and
      `fr`. **Never a Rust `format!`**: measured across the write module, every production
      `Refusal::new` takes a key literal and `into_response` renders `t!(key)`; the only `format!`
      calls are the redirect URL and test helpers.
      🔑 **And that half is stronger than the criterion asks, which is worth saying precisely**:
      `Refusal` carries a `&'static str` key, so a formatted body is **unrepresentable** there rather
      than merely absent — a property held by the TYPE, where a guard would be held by vigilance.
      ⚠️ The refusal SET itself moved under this story and is recorded with T3: `NotFound` now names
      the record the id meant instead of reporting a missing subnet for a range, and `1451` on a
      subnet delete says *it still holds ranges* instead of *no such subnet* — two sentences that
      were previously false on five routes and on one route respectively.
- [x] **T5** (AC4, AC6) The rail's two lists and their controls; the keyboard gate's new checks; the axe
      pass. The seed already reaches both delete cases.
      ✅ 2026-09-16 — **axe: 10 routes + 5 states, 0 violation nodes** (a fifth state, the rail's own);
      **kbd: 50 checks, 0 failed**, the floor moved 45 → 50 to EQUAL what is there rather than sitting
      under it. 🔴 **A blocker found before any markup: no reader carried a record id.** `ranges_in`
      returns bounds, policy and label; `plan_addresses` returns addresses — and a control that cannot
      name its record cannot exist. Hence `correctable_ranges_in`/`correctable_addresses_in`, named for
      their purpose, with the note at the site that these are NOT the `addresses_in` story 14.3b
      removed: that one fed the AUDIT, this pair feeds the RAIL, and the removal's warning stands.
      🔑 The correction forms arrive **pre-filled**, because an edit the operator must retype is a
      delete-and-redefine wearing another word, and it loses the row identity `update_range` exists to keep.
      🔴 **THE BROWSER FOUND THREE THINGS NO RUST TEST COULD, and they are three different kinds.**
      (1) A *stale premise*: the pre-existing check *"`/ipam` offers the three write gestures as
      disclosures"* counted **8**, the rail having added five — scoped to `.ipam-forms` rather than
      loosened to `>= 3`, since a floor that tolerates losing one of the three gestures is the shape
      this project has caught twice. (2) **A PRODUCT DEFECT of mine**: every correction form's submit
      button reused the definition forms' label, so four controls read *"Define"* — a false word on a
      form that CORRECTS, and four identical accessible names on one page. The gate's own output is
      the record: `["Define","Remove — 192.0.2.1 – 192.0.2.40","Define",…]`. (3) **A defect in my own
      check**: it grabbed the first `.ipam-rail-lists button`, which is a submit inside a COLLAPSED
      `<details>` — content in a closed disclosure is correctly unfocusable, so it reddened over a page
      behaving properly. *A check aimed at the wrong element measures the wrong thing in both
      directions*; it now targets the removal control, which is what an operator meets without opening
      anything. ⚠️ **And a defect of mine caught before it shipped**: the first CSS used
      `--color-border` and `--color-text-muted`, **neither of which exists** — valid syntax painting
      nothing, invisible to any source guard, which is *a source guard cannot see a cascade* one step
      earlier. Replaced with the tokens the sheet actually defines.
      🔴 **AND THIS ENTRY WAS WRONG WHEN FIRST WRITTEN, which is recorded rather than quietly fixed.**
      It said a subnet *too large to draw* renders no rail, *"registered rather than smuggled in"* —
      but **AC6 says the two lists render in BOTH branches**, the drawn grid and the too-large one.
      That is not a cost a story may accept on its own: a criterion is not a default to be explained
      away, and *a story may not re-scope its own AC*. ⚠️ The justification did not even hold on its
      own terms — the grid is skipped there because materialising 2 GB of cells costs gigabytes,
      while the rail's lists are bounded by what the operator DECLARED and cost nothing at any subnet
      size. Found by reading the criteria back against the record, which is the check that should have
      run before the entry was written. The branch now renders the rail; this line stays so the next
      reader sees that the first version of it claimed a criterion met that was not.
      🔴 **AND THE REPAIR WAS ITSELF HALF DONE, WHICH THE NEW ASSERTION CAUGHT.** Having built the
      rail, passed it and set `rail: Some(rail)` on the too-large body, the served page still carried
      neither `/ipam/range/delete` nor `/ipam/range/edit`: the partial is included from the branch
      that draws the GRID, so the Rust half of AC6 was met and the TEMPLATE half was not.
      🔑 **What made that visible is a choice made one minute earlier**: the test could have passed
      `no_rail()` — enough for the compiler, enough for a green suite — and AC6 would have shipped
      implemented and measured by nothing. Passing the POPULATED rail and asserting a control is
      present is the whole difference between a criterion met and a criterion claimed. *A guard
      placed where the defect cannot occur reads as coverage and is none* — this is the same rule
      used in the other direction, to put the guard where the defect actually was.
      Counts: **695 + 191 + 99**, clippy clean over `--all-targets`.
- [x] **T6** (AC7, AC8) Measure. Prove-to-red each new guard with `cargo xtask mutate --baseline`,
      predictions written first; both browser gates; the documents and both twins.
      ✅ 2026-09-16, **re-measured after the code review's repairs**. **AC7 — THE LIVE COUNT, both
      store conditions, commands named.**
      ⚠️ **LIVE AFTER THREE REVIEW ROUNDS: 995** (**705 bin + 191 core + 99 xtask**), 24.55 s against
      a store dropped and recreated, 5.03 s without one, kbd-probe **53** checks. 🔴 The figures below
      are round A's and are KEPT as the measurement they were rather than overwritten — but AC7 is
      the one place this story says the live count lives, and it read **990** while the tree shipped
      995 until the slice-D review found it. *A designated home that is five short is worse than no
      designated home, because a reader who goes there stops looking.*
      Baseline **980 → 990** (**700 bin + 191 core + 99 xtask**;
      it read 987 before the review, which added three store-backed tests). Against a live
      `mariadb:10.11` on port 13450, the database **dropped and recreated** and one **warm run**
      first: `cargo test --workspace --locked` = 990, **27.17 s** (warm run 22.65 s). Storeless, same
      command with `RUSTFLAGS="-D warnings"`: 990, **5.03 s** — *the clock is the tell*, since the
      counts are identical either way and only the wall time says the database-backed tests executed.
      ✅ **AC8 — no regression**: ten gates (`cargo xtask ci` — all green) · `cargo clippy --workspace
      --all-targets --locked -- -D warnings` · `RUSTFLAGS="-D warnings" cargo test --workspace
      --locked` · `cargo deny --manifest-path Cargo.toml check` (advisories, bans, licenses, sources
      ok — ⚠️ the flag goes BEFORE `check`, an ordering this project has already got wrong once) ·
      `cargo fmt --all` · **both manuals build** (user manual 15 pages) · **both browser gates**: axe
      **10 routes + 5 states, 0 violation nodes** under all four `AXE_REQUIRE_*` flags, kbd **52
      checks, 0 failed**.
      🔴 **THIS LINE READ *"Seven prove-to-red passes"* OVER A LIST OF NINE IDS, TWO OF THEM GREEN —
      story 6b.10's recorded defect, recurring in the story that cites it.** The Acceptance layer
      counted; the corrected record classifies instead of totalling, because a summary that can drift
      from its own table is the thing being guarded against:
      **Prove-to-red passes (predicted red, measured red):** the widened cap guard, twice on a COUNT
      (`left: 9, right: 5`, then `15/9`, recounted by hand each time) · **M-T2a** and **M-T2b**, each
      reding exactly one test with the OTHER as its green control · **M-T3c**, both locks removed ·
      the key guard, red on a blanked French value · the two browser checks, red with a deliberate
      asymmetry (one fails, the other must not) · **M-T6a** and **M-T6b**, added by this review below.
      **NOT prove-to-red, and the finding precisely because they are green:** **M-T3** and **M-T3b**,
      each removing one lock and reddening nothing — which is how the two-carrier property was found.
      ✅ **M-T6a — the mutation T1 PROMISED T6 AND T6 DID NOT CONTAIN.** T1 wrote that the widened
      guard *"has reddened on a COUNT only, never yet on a `DELETE` that lost its cap — that is a T6
      mutation"*, and no such row existed. Run now: `capped!` removed from `DELETE FROM ip_address`,
      `PREDICTED: Red(Some(1))` · `MEASURED: Red { tests: 1 }`, exit 0. 🔑 **Carrier established by
      hand and it is the ASSERTION, not the count** — *"a plan statement that can wait on a lock is
      not capped — `"DELETE FROM ip_`"* — which is the distinction the promise was about: the count
      catches a statement REMOVED, the assertion one DECAPPED.
      🔴 **M-T6b — the arbitration's own refusal, and the prediction was CONTRADICTED.** Neutering the
      abandonment comparison predicted `red:1`; the driver measured **2** and exited 1. The second red
      is **collateral, not a carrier**: the abandonment test panics at its first `expect_err` before
      its trailing `forget_subnet`, so `100.66.14.15` survives into `the_store_returns_addresses_in_
      numeric_order`, whose failure prints that very address. The same run's baseline is 700/191/99
      green, so the leak is mutation-induced. ⚠️ Re-measured with the test ISOLATED (one filter,
      never two): **exactly 1 red**, on its own assertion. Recorded as *1 carrier + 1 collateral* and
      never as *two guards carry it* — this module's own documented hazard, met in its own story.
      ⚠️ **The driver reports no test NAMES — its whole log is 31 lines of summaries — so every
      carrier here was established BY HAND** (own copy, mutation verified applied, single filter,
      restore verified, never `git checkout` on a tree carrying hours of uncommitted work). *A
      matching count is not a matching carrier.*
      ✅ Documents: the user manual's *Defining the addressing plan* section now describes correcting
      and removing, the rail's two lists, and the warning before all three removals; both twins and
      `sprint-status.yaml` updated in the same push.

### Review Findings

Three isolated layers on the group-A slice (`ipam_repo.rs`, `ipam/mod.rs`, the two twins), 2026-09-16.
None failed. 🔑 **Three findings were reached by all three layers independently**, and the Edge layer
turned two of them from readings into measurements.

- [x] [Review][Decision] **A range EDIT can orphan the addresses a range DELETE refuses to orphan, and it carries neither a refusal nor a warning** — `delete_range` refuses by name while the range holds a defined address; `update_range` re-asks the insert's three refusals and not this one. MEASURED by probe: control `delete_range` → `Ipam(RangeStillHoldsAddresses)`; `update_range .10-.20 → .30-.40` → `Ok`; the address then stands under no range, and the delete the product refused two statements earlier now succeeds. Shrinking or flipping the policy reaches the same state. ⚠️ The three deletes all warn through `delete-check`; the two edit forms carry no `hx-get` at all, so a verdict-changing gesture has no guard of either kind. Both precedents exist in this project — `delete_range` REFUSES, and the 2026-09-10 arbitration says *the form warns and still writes*. Guy's call. [`ipam_repo.rs:279-340`]
- [x] [Review][Patch] `update_address`'s containment re-validation — the function's stated reason for existing — is carried by no test; the mutation leaves 987 tests, clippy and ten gates green [`ipam_repo.rs:112`]
- [x] [Review][Patch] The refuted single-carrier attribution survives at four code sites, one of them the assertion message a future debugger reads [`ipam_repo.rs` ×3, `ipam/mod.rs` ×1]
- [x] [Review][Patch] The gate-absence test reads 2 of 6 `xtask` sources; a planted `ip_subnet` in `copy_vocabulary.rs` leaves it green (measured) [`ipam_repo.rs:2708`]
- [x] [Review][Patch] `delete_subnet`, `delete_address` and `update_address` have no adapter test; both `NotFound` guards measured carried by nothing [`ipam_repo.rs:638,670,702`]
- [x] [Review][Patch] The widening test's "third assertion" is unreachable under the mutation it claims to carry, and its decision inputs are identical to the first call's [`ipam_repo.rs:459-552`]
- [x] [Review][Patch] `update_address`'s `# Errors` names `RepositoryError::Backend` where the code yields `Ipam` [`ipam_repo.rs:98-101`]
- [x] [Review][Patch] "The parent row is locked FIRST" is false in three sites — the child row is locked first — and the order inverts against `insert_range`'s parent-then-siblings [`ipam_repo.rs` ×3]
- [x] [Review][Patch] The concurrency test's oracle cannot tell "waited for the lock" from "was slow", and its name claims prevention where the body measures ordering [`ipam_repo.rs:679-744`]
- [x] [Review][Patch] `settle_plan_write`'s commit-failure branch issues no rollback, on the one path its doc does not mention [`ipam_repo.rs:355-376`]
- [x] [Review][Patch] The two new readers are neither capped nor counted, and that decision is unwritten [`ipam_repo.rs:384,421`]
- [x] [Review][Patch] The widened cap guard is an enumeration presented as a closed property; its residual is unstated [`ipam_repo.rs:2633`]
- [x] [Review][Patch] Both twins still promise a "release" route among 14.4's five, which 14.4b now owns [`CLAUDE.md`, `docs/project-context.md`]
- [x] [Review][Patch] The twins disagree on AC6's end state (*half met* / *met*), and `project-context` drops the "range" qualifier, making the sentence false for `delete_subnet` [same]
- [x] [Review][Patch] §1(c) still carries the refuted mechanism with no correction marker, in the section headed *each WITH the measurement that produced it* [story `:83-86`]
- [x] [Review][Patch] The Completion Notes claim both corrections keep their first version visible — true for T5, **false for M-T3**, whose first version is only alluded to [story `:553-556`]
- [x] [Review][Patch] "Seven prove-to-red passes" lists **nine** ids and counts **two greens** among them — story 6b.10's recorded defect, recurring [story `:476-480`]
- [x] [Review][Patch] T1 says `FOR UPDATE"` was added as a needle; it pre-existed. And "the one non-exhaustive site" was **two** [story `:243,251`]
- [x] [Review][Patch] `screens.rs`'s `IpamError` floor is 7 where the enum declares 8 — the rule this very story applied at T5 [`screens.rs:1307`]
- [x] [Review][Patch] T1 promised a DELETE-loses-its-cap mutation to T6, and T6 does not contain it [story `:253,477`]
- [x] [Review][Patch] `range_attempt`'s savepoint precondition is not inherited by the three new transaction-owning writes [`ipam_repo.rs:108,220,300`]
- [x] [Review][Patch] One unreadable address anywhere in the subnet aborts a RANGE delete with `MalformedAddress` — the refusal names a rule about an address the operator never touched [`ipam_repo.rs:235`]
- [x] [Review][Patch] `delete_address`'s doc contradicts itself four lines apart — *needs no lock* above *takes an exclusive lock* [`ipam_repo.rs:22-37`]
- [x] [Review][Patch] The two readers' `# Errors` omit the backend failure, naming only the less likely cause [`ipam_repo.rs:394,431`]
- [x] [Review][Patch] The re-added per-subnet reader is held off the audit path by prose stated as though it constrained something [`ipam_repo.rs:424-429`]
- [x] [Review][Patch] `ascii_bin` is PAD SPACE, so the five id-addressed statements match ids differing by trailing spaces; the doc sentence is stronger than the behaviour [`ipam_repo.rs`, 5 sites]
- [x] [Review][Defer] The two rail readers are unbounded in the one branch written for hugeness [`ipam_repo.rs:384,421`] — deferred: bounded by what the operator DECLARED rather than by subnet size, and a `LIMIT` would silently truncate their own list
- [x] [Review][Defer] Positional tuples with `id` and `label` both `String` at opposite ends — a swap compiles and renders the label as the id a DELETE control names [`ipam_repo.rs:389,426`] — deferred: the fix is a struct or newtype, and it ripples into `ipam_page.rs`
- [x] [Review][Defer] `delete_range`'s deciding read X-locks every address row of the SUBNET, so deleting one range blocks address writes in unrelated ranges [`ipam_repo.rs:227-239`] — deferred: it is one of the two serialisation carriers, so it cannot be narrowed without re-measuring that claim

✅ **Refuted by the layers themselves, with the check, so nobody re-chases them**: `delete_subnet` does NOT cascade (`0007:120,153` declare both foreign keys with no `ON DELETE`, so `ERROR 1451` really is the mechanism); a deadlock on any statement answers `Contention`, not a 500 (`repo.rs:1641-1646` maps `1213|1205`); the three new transaction-owning writes do NOT meet the savepoint hazard, verified at their call sites; and an over-long label errors rather than truncating (`STRICT_TRANS_TABLES`).

### Review Findings — group B (the routes)

Three isolated layers on `ipam_write.rs` and `main.rs`, 2026-09-16. None failed. **29 raw findings, 22
distinct** — two reached by all three layers and four more by two, *so a count over three isolated
reports measures the isolation and not the product*. 🔑 **The layer with no repository access found a
HIGH the two sighted layers did not, for the ninth story running.** ⚠️ The layers ran at this
session's capability, isolated by context, worktree and database — **not on a different model, and
that half is not claimed**.

- [x] [Review][Decision] **`subnet_id` is validated for SHAPE and never compared with the record's real parent** — the four record-addressed forms carried it, the adapter addresses the record by its own id, and the value served one purpose: the redirect. MEASURED on the running binary: `id=<address in subnet A>&subnet_id=<subnet B>` removed the row from A, answered 200 and sent the browser to **B's** plan under *"The address is no longer defined"*, over a page where nothing had changed. ⚠️ And the mirror: a malformed or nil `subnet_id` answered 422 and deleted nothing — **a gesture refused because of a field it takes no part in**. ✅ **Guy, option (a)**: the adapter READS the parent off the row and the field leaves the four forms — *the only remedy that removes the question instead of answering it*, and it dissolves both halves at once. Refused: comparing and refusing a mismatch (keeps the field and its unjustified refusal), and registering it (reachable from a stale tab, a second window or a hand-made request). [`ipam_write.rs`, `ipam_repo.rs`, `_ipam_rail_lists.html`]
- [x] [Review][Decision] **The one refusal this story mints serves the DELETE's sentence on an EDIT** — the review's own repair made an edit that abandons an address earn `RangeStillHoldsAddresses`, and `ipam_refusal` takes no route where both its neighbours do, so an operator who had SHRUNK a range read *"deleting the range would leave them with nothing saying what that space is meant for"*. **That is this story's own recorded class — *a true sentence about the wrong gesture, which is worse than none* — reintroduced by the repair, in the story whose headline is that a refusal must name the right record.** ✅ **Guy, option (a)**: `ipam_refusal` takes the route and the edit earns `ipam.refusal.range_edit_would_abandon`. The rule is one; the sentence is two, because the remedy differs — a delete must be given up, an edit can simply be widened. [`ipam_write.rs:1418`, `app.yml`]
- [x] [Review][Decision] **Two refusals that both say *this container is not empty* answered two different statuses** — 409 for a subnet (the database's key) and 422 for a range (computed) — and nobody had decided that: it fell out of which layer raised them. ✅ **Guy: align on 409.** The seven other `IpamError`s are about the CONTENT of the request and keep 422; this one says the request is fine and the PLAN's state opposes it, which is what 409 means. [`ipam_write.rs:1418`]
- [x] [Review][Patch] **The story's headline repair is carried by NO test** — `"ipam.refusal.subnet_still_holds"` occurs once in all of `crates/`, the mapper itself, and the only test feeding `Constraint("foreign_key")` asserts the key RESOLVES, never which key or which status. Measured: turning the repaired 409 back into the 404 left **700 tests, clippy `--all-targets` and ten gates green** [`ipam_write.rs:1321`]
- [x] [Review][Patch] **The label branch measures a request that carries no label** — three passages claim serde DROPS an unknown field at a delete route, and `a_valid_body`'s three delete arms interpolated none, so twelve byte-identical label-free requests were driven through a branch whose own comment says a `continue` *"would read exactly like a passing check"*. *A `continue` wearing an assertion.* Reached by all three layers [`ipam_write.rs:2283`]
- [x] [Review][Patch] `constraint_refusal`'s `foreign_key` arm is a `_`, three lines below the `unique` arm whose comment forbids one — compile probe: a ninth variant gives **seven `E0004`s and this arm is in none of them**, so a route added tomorrow inherits *that subnet is not in the plan*. ⚠️ Deflated with its bound: the four new routes cannot reach it today (a child `DELETE` violates no key, neither `UPDATE` touches `subnet_id`), so what was live is the hole and not the sentence [`ipam_write.rs:1321`]
- [x] [Review][Patch] **The module doc promises an `error[E0004]` that does not exist** — `ALL` is a hand-written array, and `router_with`, `paths()`, `main.rs`'s nine-path premise and this story's own coverage assertion ALL derive from it: *they agree because none of them has a second opinion to disagree with*. Closed by a second hand-written list pinned by an equality, **with its limit written: a tripwire, not a barrier** [`ipam_write.rs:168`]
- [x] [Review][Patch] The record-group completeness check is a **count, not a set** — listing one route twice and omitting another also sums to eight; and `sentences.windows(2)` is vacuously true on the one-element group. Story 6.5's *a count is not a set*, one file over [`ipam_write.rs:2025`]
- [x] [Review][Patch] The success-status guard compares `response.status()` against `route.success_status()` — **the function that produced it** — so a mapper answering `OK`, or `IM_A_TEAPOT`, for every route left it green under a message naming the split [`ipam_write.rs:2336`]
- [x] [Review][Patch] `already_defined` maps the three deletes to *already defined* while `constraint_refusal` routed them to `backend()` — **two statements of one delete's sentence, disagreeing**, the doc describing copy the product never emits. One statement now [`ipam_write.rs:343`]
- [x] [Review][Patch] *"false on five of them"* is **four**: the count of routes this story ADDS, reused for a different population; `DeleteSubnet` is one where the old sentence was right [`ipam_write.rs:235`]
- [x] [Review][Patch] *"The three definitions receive a `subnet_id`"* is false of `WriteRoute::Subnet`, which carries a CIDR, and silent about `DeleteSubnet` — refuted twice inside the same diff [`ipam_write.rs:243`]
- [x] [Review][Patch] *"a fourth route cannot be added"* stands in a doc block this change edits, over a file with **eight** [`ipam_write.rs:2181`]
- [x] [Review][Patch] The redirect expectation ends in a `_` under a comment arguing against a different loosening [`ipam_write.rs:2344`]
- [x] [Review][Patch] The three delete fakes push the identical marker `"delete"`, and two of them shared a body shape, a status and a redirect — **wiring `/ipam/range/delete` to the address handler was invisible to every guard in the file** [`ipam_write.rs:1530`]
- [x] [Review][Patch] `edit_range`'s field-order rationale claims for the whole routine a rule that governs only its operator-filled tail [`ipam_write.rs:1024`]
- [x] [Review][Patch] **The delete check answers the SUBNET's sentence for an unreadable qualifier** — both reads used `.ok()` and dropped the failure, so `?subnet=…&addr=nonsense` was answered *"this subnet still holds 2 records"*, which is what `DeleteCheckQuery::addr`'s own doc says the route was shaped to prevent. ⚠️ **`192.000.002.015` — the store's own canonical spelling — is one of the values that reopens it.** Latent, not live: the rail renders dotted [`ipam_page.rs:539`]
- [x] [Review][Patch] The delete check served `Failed to deserialize query string: duplicate field 'subnet'` at **400**, and htmx swaps a 4xx — English framework text in the `aria-live` region of a French page, at an `/ipam` address [`ipam_page.rs:526`]
- [x] [Review][Patch] The four `malformed_*` sentences name a subnet field the forms no longer carry [`app.yml`]
- [x] [Review][Patch] AC1's *"`settle` is reused"* holds on two of the five new routes; the exception is sound and was written only in code comments [`ipam_write.rs:669`]
- [x] [Review][Defer] `GET` on a write route answers **405 with an empty body** — pre-existing (story 6.1 scoped the claim to the POST pair) and unreachable through the shipped forms; recorded because this story takes the surface from three routes to eight
- [x] [Review][Defer] `subnet_id`'s removal makes the four forms un-parented at the ROUTE; a range and its addresses are still reachable only through their own ids, so a future *move this record to another subnet* gesture must re-open the question rather than inherit this answer

**Prove-to-red, eight mutations — and TWO REFUSALS that are results.** ⚠️ The driver reports counts and
never test NAMES, so **every carrier below was established by hand**, by applying the mutation from a
scratchpad copy and reading the failure list (never `git checkout --`).

| id | mutation | measured | carriers, named by hand |
|----|----------|----------|--------------------------|
| M-B1 | the repaired 409 becomes the 404 this story removes | 🔴 1 | `a_populated_subnet_is_refused_by_name_rather_than_as_a_missing_one` |
| M-B2 | the edit is given the delete's sentence again | 🔴 2 | `the_edit_and_the_delete_do_not_share_a_refusal_sentence`; ⚠️ and `every_key_this_module_can_render_resolves_in_both_locales` **incidentally**, the key count falling 39 → 38 |
| M-B3 | a variant exists and is left out of `ALL` | 🔴 3 | `every_variant_is_in_the_route_list`, `every_refusal_the_handler_can_receive_names_a_rule`, and `main.rs`'s `every_write_route_is_refused_without_a_credential_and_exists` — 🔑 which reds on a SHRINK and, as the auditor said, could never red on an omission |
| M-B4 | one record classified twice, another not at all | 🔴 1 | `every_refusal_the_handler_can_receive_names_a_rule` — the set check; a count would not have seen it |
| M-B5 | a definition answers 200 | 🔴 4 | `every_route_reuses_the_shared_machinery`; ⚠️ plus three incidental (`a_label_at_the_column…`, `a_well_formed_definition…`, `only_the_four_binding_policies…`) which assert 201 directly |
| M-B6 | the fake port names a different parent | 🔴 1 | `every_route_reuses_the_shared_machinery` — decision 1's carrier: the redirect now comes from the PORT, where the old assertion was satisfied by the form echoing its own hidden field back |
| M-B7 | the unreadable qualifier is swallowed again | 🔴 1 | `an_unreadable_qualifier_answers_nothing_rather_than_the_subnets_sentence` |
| M-B8 | the framework answers for us again, in English | 🔴 1 | `a_query_the_extractor_refuses_says_nothing_in_the_frameworks_words` |

🔑 **M-B5's first run was REFUSED — `ANCHOR MATCHED 2 TIMES` — and the refusal is the deliverable**:
the second site is the guard's own hand-written table, so replacing both would have **repaired the
guard the mutation was meant to red**. *The refusal is what establishes that the two representations
are genuinely independent*, which is exactly what the mutation set out to show. Re-run anchored on the
producer alone, it reds.

⚠️ **The driver refused a second time, over a snapshot left by that interrupted run**, and named the
hazard rather than guessing: *the file may still be MUTATED — compare, restore by hand, do NOT use
`git checkout`*. Compared: identical, so the refused run never applied anything; snapshot deleted.

⚠️ **Three instrument defects of mine, recorded because two of them nearly became findings**: the
database reset ran `mariadb`, **which is not installed on this host** (`exit 127`), so a run I was
about to report as *virgin store* was nothing of the kind — and it explains the 4-then-2 failure drift
of that pass, the known non-determinism against a REUSED database. And **twice I read a log while it
was still being written** and concluded four gate steps had not run, when all ten had. *A measurement
read through a filter, or taken with an instrument that cannot answer, is not a measurement* — met
three times in one afternoon.

**990 → 995** (705 bin + 191 core + 99 xtask), ten gates, clippy `--all-targets`, `cargo fmt`;
**25,03 s** with a store dropped and recreated against **5,03 s** without one — the clock is the tell.
**Both browser gates re-run and green**, which this slice owed because the rail lost four hidden
fields: axe **1 route / 0 nodes** on the empty plan and **10 routes + 5 states / 0 nodes** seeded,
kbd-probe **52 checks, 0 failed** — including, in a browser, a correction form arriving as
`{id, first, last, policy}` with **no `subnet_id`**, eleven rail controls with distinct accessible
names, and the removal warning announced without stealing the focus.

⚠️ **Slice C is reviewed below. Slice D is NOT** — `kbd-probe.mjs`, the manual and the status files —
🔴 **and neither is `screens.rs`, which the partition LEAKED**: it sits in no slice and no layer was
ever handed it. 🔴 **And the partition is not where the leak STARTS: the File List omitted the file**,
which is the artefact a partition is cut from — so the diagnosis *"found by checking the partition"*
named the symptom and the record's own manifest was the cause, still uncorrected in the commit that
narrated it. `deferred-work.md` was omitted on the same footing, with no note at all. Its whole change
is ~~+7/−2~~ **+6/−1** (the `IpamError` floor 7 → 8 and its comment) and it IS slice A's own repair,
so the exposure is
small — but small is not reviewed, and slice A's finding *about* that file was reached from
`ipam/mod.rs` by a layer that could not see it. It goes into slice D.

### Review Findings — group C (the screen)

Three isolated layers on `ipam_page.rs`, the two templates, `app.yml` and `app.css`, 2026-09-17/18.
None failed. **43 raw findings, 27 distinct** — three reached by all three layers and four more by
two. 🔑 **The layer with no repository access again found HIGHs the two sighted ones did not**, and
the Edge layer measured the three worst on a booted binary in both locales. ⚠️ The layers ran at this
session's capability, isolated by context, worktree and database — **not on a different model**.

🔴 **SIX OF THE WORST FINDINGS HAVE ONE CAUSE**: `render_delete_check` never consulted the rules that
actually refuse a removal, trusted the bounds the query handed it, and appended *"the removal is
still possible — not a refusal"* to **every** non-empty answer.

- [x] [Review][Decision — MINE, delegated 2026-09-18] **The check's contract.** ✅ Option (a): it asks
  the same rules the write path asks, in the same scope, and the closing sentence is appended only
  when no refusal is coming. Refused and named: deleting that sentence outright (it is true and
  useful for every gesture that really will go through), and patching only the contradiction (which
  leaves the range arm silent about its own refusal). ⚠️ **Recorded as MINE rather than Guy's so it
  can be reversed at the right cost.** 🔑 Cheaper than estimated: `delete_check_data` already read
  this subnet's ranges and addresses for its `held` count, so **no change to `plan_ranges`' signature
  was needed** — I said it would be and that estimate was wrong.
- [x] [Review][Decision — MINE] **Silence or sentence.** ✅ The keyed sentence. `ipam.check.unavailable`
  reads *"The plan could not be checked just now. The write does not depend on this check"* — worded
  about the CHECK, not the store, so it is as true of a request this build cannot read as of a store
  it cannot reach, and reusing it states nothing false. ⚠️ An empty region made *I could not ask*
  indistinguishable from *nothing here is worth saying*, immediately before an irreversible gesture.
  Refused: minting a second sentence, which buys a distinction the operator cannot act on.
  ⚠️ **It cost my own slice-B oracles**: they read *empty = declined before the store*, and with both
  cases now rendering one sentence that stops separating anything — replaced by the oracle aimed at
  the defect itself, *a malformed qualifier never answers the SUBNET's sentence*.
- [x] [Review][Decision — MINE] **The amber.** ✅ Removed from this story's two correction controls.
  🔴 The stylesheet claimed the token *"cannot leak by construction"*; measured, **`master` already
  carried four other sites** (three in `_ipam_forms.html`, one in `_ipam.html`), all landed at 14.2b —
  so this story WIDENS an older false sentence rather than creating it. The four are REGISTERED, not
  swept up here: widening a story's scope to repair its predecessor's is how a defect stops having an
  owner.
- [x] [Review][Patch] **The subnet warning contradicted itself in one live region** — *"removing it
  will be refused"* then *"the removal is still possible — not a refusal"*, in both languages, with
  the 409 settling which was false (three layers; Edge measured it)
- [x] [Review][Patch] **The range check never mentioned the refusal this story minted**, then promised
  the removal: `.1–.40` holds defined `.9`, the warning said *still possible*, the POST answered 409
- [x] [Review][Patch] **A warning fabricated for bounds naming no range** — `0.0.0.0–255.255.255.255`
  reported ten addresses; reachable from a stale tab, not only by hostility (Edge, measured)
- [x] [Review][Patch] **The named subnet was never checked against the bounds** — Workshop's id with
  Office's bounds answered Office's five addresses (Edge, measured)
- [x] [Review][Patch] The offer computation filtered a **plan-wide** set **by value** while the overlap
  rule is scoped per subnet, so a same-bounded range in another subnet was struck too
- [x] [Review][Patch] **The subnet's Remove control names a generic noun** — *"Remove — Subnet"*,
  byte-identical across two subnets (Edge), on the one gesture that takes everything with it. ⚠️ The
  keyboard gate cannot see it: it tests DISTINCTNESS, and that name is distinct
- [x] [Review][Patch] **AC6's guard covered one of its two lists** — the addresses list could have
  vanished from the too-large branch with the test green, on the criterion already corrected twice
- [x] [Review][Patch] The key guard's name and doc claimed *"every key this module can render"* while
  scanning `"ipam.` literals in one file; `triage.kind.ecart` and `state.undeclared` are rendered in
  production code and invisible to it — renamed, narrowed, and the two closed by name
- [x] [Review][Patch] The refusal justified itself by a cascade the foreign key makes **impossible**,
  and the success sentence promised that same cascade
- [x] [Review][Patch] *"Five per-row controls"* is four, in two comments — the fifth names no row and
  renders over an empty rail, so the justification was false precisely for the route it counted
- [x] [Review][Patch] The too-large include's comment credited a protection the unknown-subnet arm
  never exercises; `no_rail()` is a `Some`, not the `None` case
- [x] [Review][Patch] Both `<ul>`s carry `list-style: none` with no `role="list"` — story 6b.7's own
  lesson, one screen over. ⚠️ No guard in this project can see it and axe reports 0 either way
- [x] [Review][Patch] The rail's CSS comment gave a reason inapplicable in one of AC6's two branches;
  two selectors carried byte-identical declarations; a dead `Default` derive; a paragraph explaining
  `unknown_range`/`unknown_address` sat fifty lines away over the `ipam.rail.*` keys
- [x] [Review][Patch] **The structure note's figures were stale and every one understated** —
  `ipam_page.rs` **1740** (read 1357), `ipam_write.rs` **1504** (862), `ipam_repo.rs` **1409** (847);
  only `page.rs` 1954 was right. `ipam_page.rs` is now second at **87 % of the ceiling**
- [x] [Review][Defer] The `<summary>` and its submit button carry identical accessible names — the
  honest fix needs a new copy key, which is scope
- [x] [Review][Defer] `"Correct"` as an English button label is an adjective/verb homograph — a naming
  judgement, not a defect
- [x] [Review][Defer] Six store reads per focus with no debounce — ⚠️ **Edge measured 3–9 ms at seed
  scale and refused to inflate it into a finding**; recorded because 14.3b debounced the address
  field for this same shape

**Prove-to-red, six mutations — one refuted a prediction and one is a CONTROL.** ⚠️ Carriers
established BY HAND: the driver reports counts and never names.

| id | mutation | measured | carriers |
|----|----------|----------|----------|
| M-C1 | the rail's addresses half empties | 🔴 2 | the AC6 guard (too-large branch) + `every_form_posts_where_its_route_is_mounted` (GRID branch) — so the widening is the sole carrier **in its own branch only** |
| M-C2 | the closing sentence unconditional again | 🔴 1 ⚠️ MIXED | `a_removal_warns…`; `clippy: true` too — neutering the flag leaves `refused` unread, so this is not an assertion-only red |
| M-C3 | the range arm stops asking about abandonment | 🔴 1 | `a_removal_warns…`, clean |
| M-C4 | bounds no longer checked against this subnet's ranges | **GREEN**, then 🔴 1 | 🔴 my assertion used `.50–.60`, an interval that speaks under neither tree — *a guard placed where the defect cannot occur*, committed inside the repair for a review that found that class. Re-aimed at `.14–.16`, which holds the seen `.15` |
| M-C5 | an unreadable address back to silence | 🔴 1 | `an_unreadable_qualifier_never_answers_the_subnets_sentence`, clean |
| M-C6 | the offer filter back to the plan-wide set | **GREEN by prediction** | a CONTROL: every fixture uses ONE subnet, so the two sets are identical there — **the scoping repair is carried by no unit test**, and that is a statement about coverage rather than a failed search |

⚠️ **Two defects of my own, both caught by a guard rather than by reading, and both the same law.**
Repairing the stylesheet's false sentence, I closed its comment nineteen lines early — freeing prose
into the code half, where a documenting-token read was counted as a sixth declaration. The repair of
*that* closed it again, because the sentence explaining the stray close **quoted the two characters
that end a comment**. `ac4_the_amber_is_reserved_for_the_documenting_gesture` caught both (6, then 7).
*A guard that greps a file greps its prose, and the better the prose explains the defect, the more
reliably it reproduces it.* ⚠️ And a verification grep of mine counted `btn-document` **mentions** as
call sites — *an unbounded needle cannot tell an entry from a mention of one* — while checking a
figure I had just corrected for being wrong.

✅ **Refuted with the check, so nobody re-chases them**: the rail is included exactly twice, in two
mutually exclusive arms, so no page duplicates its ids; `/ipam/delete-check` IS behind both perimeter
guards; `--color-divider` is defined; the hostile query battery answers 200 with nothing reflected and
no framework English; AC6 holds on a real `/8`; the unknown-subnet and empty-plan pages render no rail.

**995 tests** (705 + 191 + 99), ten gates, clippy `--all-targets`, `cargo fmt`; both browser gates
re-run because the rail template changed again — axe **1 route / 0 nodes** empty-plan and **10 routes
+ 5 states / 0 nodes** seeded, kbd-probe **52 checks, 0 failed** (that figure is slice C's own
measurement; slice D added a check and the gate now runs **53**).

### Review Findings — group D (the record, the gate, `screens.rs`, the manual)

Three isolated layers, 2026-09-18. **~41 raw findings, 26 distinct.** 🔴 **THE CODE CAME THROUGH
INTACT — no acceptance criterion is unmet, and `screens.rs`, read by a layer for the first and only
time, passes on its merits (its floor of 8 equals the 8 variants `IpamError` declares, and the guard
reds assertion-carried). EVERY defect in this slice is in the RECORD, and every one is mine**,
committed in pushes whose own messages assert the opposite.

- [x] [Review][Patch] 🔴 **The File List omitted `screens.rs` and `deferred-work.md`** — and *that* is
  why the partition leaked: a slice is cut from the File List, so a file absent from it can reach no
  layer. I had diagnosed the leak as a partition error; the manifest was the cause, still uncorrected
  in the commit that narrated it. Reached by two layers.
- [x] [Review][Patch] 🔴 **The gate converted a real failure into *"could not run"*.** Measured: break
  the address warning's triage link and check 41 reds, check 42 (its dependant) never runs, `executed`
  falls under the floor, and the floor **overwrites the verdict with 2** — the gate printing the red
  and then telling the reader to fix the harness. A failure now decides before the floor does.
  🔑 **M-D1, the prove-to-red for this repair**: the same defect replanted on the repaired tree answers
  `kbd gate: 52 check(s) run, 1 failed — the keyboard layer has regressed`, **exit 1**, where it
  answered **2** before. The 52 is the evidence that the mechanism is real and not theoretical — the
  dependant check genuinely was skipped and `executed` genuinely did fall under the floor of 53; what
  changed is that the floor no longer gets to speak over a detected regression.
  ⚠️ **The run that measured it was killed mid-way and left the planted defect in the working tree**,
  because the script had no `trap … EXIT`. Caught by checking the tree rather than by noticing, and
  restored from a scratchpad copy after verifying that copy was the original — never `git checkout --`,
  the gesture this project has recorded destroying uncommitted work four times. The re-run carries the
  trap.
- [x] [Review][Patch] 🔴 **The manual contradicted itself 45 lines apart**: new present-tense text
  shipping *Correct* and *Remove*, and an untouched `\planned` block saying editing and deleting are
  not implemented. Only releasing, VLANs and IPv6 are still owed.
- [x] [Review][Patch] The manual said corrections obey the definition rules — false since round A's
  arbitration; and its warning paragraph ended *"the removal stays available"* eleven lines after
  opening *"Removing is refused when…"*. *Refuses nothing* is true of the WARNING, not the removal.
- [x] [Review][Patch] AC7 — *"THE LIVE COUNT lives here"* — read **990** where the tree shipped 995
- [x] [Review][Patch] A `[Review][Defer]` the record claimed was registered appeared in NO row, while
  three rows existed that were never on the defer list: the mapping checked in neither direction.
  Story 6b.9's finding verbatim, in a push asserting the deferrals were written
- [x] [Review][Patch] T3's `E0004` claim stood **unstruck** and both twins published it as a compiler
  guarantee, 230 lines above the group-B finding that refuted it — *a correction that adds the true
  text without removing the false one leaves the record carrying both*, this story's own rule
- [x] [Review][Patch] Three patch counts overstated (26/19/16 against a recount of 25/17/14), and
  round C's 20 rows against *27 distinct* left unreconciled. **I recounted rather than adopt the
  layer's arithmetic**; it agreed exactly
- [x] [Review][Patch] *« either removal »* in four documents where **three** controls warn — the
  address removal's warning shipped and documented nowhere
- [x] [Review][Patch] `kbd-probe.mjs` carried *"five per-row controls"*, the sentence round C corrected
  in two other places, surviving in the slice the record itself flagged as unreviewed
- [x] [Review][Patch] Four of five rail control terms were `> 0` — sixty lines below a comment
  arguing against exactly that — now the seed's exact counts; the distinctness evidence printed
  `.slice(0, 8)` over eleven names; the disclosure check could not red for any product change and
  threw (exit 2) on an ordinary class rename
- [x] [Review][Patch] `MIN_CHECKS`'s justifying comment said *twenty* (story 6b.11's figure) beside a
  constant of 52 — the one place a reader verifies *the floor equals what is there*
- [x] [Review][Patch] `screens.rs` cited the gate floor as `45 → 50`, an intermediate superseded by 52
- [x] [Review][Patch] `+7/−2` where the tree says `+6/−1`; `last_updated` two days behind its own
  commit; the sprint entry recording the story as planned; decimal separators diverging between twins
- [x] [Review][Patch] 🔴 **`CLAUDE.md` stated the sprint-status note convention BACKWARDS** — *"writes
  each note ABOVE its key"* — **in the paragraph narrating that this misreading gave story 6.6 a false
  premise.** Measured: `14-4-maintaining-the-plan:` carries its note below it. *The lesson was written
  down inverted, so the next reader was set up to repeat it.*
- [x] [Review][Patch] Both twins' gate bullet said **forty-five** checks and **FOUR** axe states where
  the tree ships 53 and five — the bullet documenting the two gates this story moved
- [x] [Review][Defer] The removal warning re-reads the whole plan and network per focus event —
  ⚠️ **measured at 3–9 ms and explicitly NOT inflated into a finding** by the layer that measured it

⚠️ **Three instrument defects of mine in this slice alone**: an unbounded grep counted *mentions* of
`btn-document` as call sites; a second unbounded alternation matched **story 5.8's** record and I
nearly "corrected" it; and the floor comment I repaired for being stale was stale again on save,
because the same review added a check — **and a floor is a MINIMUM, so that drift reds nothing.** The
number now comes off a live run rather than a hand count.

✅ **Refuted with the check**: `MIN_CHECKS` equals what is there (48 static sites + 4 loop iterations
= 52, then 53); the three new rail checks all red assertion-carried; no check is aimed at a control
inside a collapsed `<details>`; the manual's labels match the shipped ones; `screens.rs`'s floor is
right and reds.

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
- The `file-size` ceiling is 2000 CODE lines — everything before the FIRST `#[cfg(test)]`, which is
  the gate's own rule. Re-measured at the slice-C review on 2026-09-17, because **three of the four
  figures this line carried were stale and every one of them understated**: `page.rs` **1954** (right,
  46 from red), `ipam_page.rs` **1740** (read 1357 — short by 383), `ipam_write.rs` **1504** (read 862
  — short by 642), `ipam_repo.rs` **1409** (read 847 — short by 562). They were true when the story
  was contexted and were never re-taken while the story grew all three.
- ⚠️ **`page.rs` is still the file at risk, and `ipam_page.rs` is now second at 87 % of the ceiling** —
  it grew ~383 lines in this story, which is the one this slice touches. Nothing is red and no gate
  complains; the house rule is *split, not grown*, and it asks to be honoured **before** the gate
  asks, which no story in this epic has yet managed.

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

Claude Opus 5 (1M context).

### Debug Log References

Every measurement in this file was written to a log and its exit status read from that file, never
through a pipe (story 6b.10's defect: `cargo test | grep` takes the pipeline's status, and a commit
went in over `484 passed; 1 failed`). Logs under this session's scratchpad: `t1-cap`, `t2-store`,
`m-t2a` / `m-t2a-carrier`, `m-t2b` / `m-t2b-carrier`, `m-t3-carrier`, `m-t3b-carrier`, `m-t3c-both`,
`page-keys`, `m-keys-carrier`, `m-kbd-carrier`, `gates`/`gates2`/`gates3`, `t6-final`.

### Completion Notes List

- **The operator can correct and remove what they declared.** `WriteRoute` 3 → 8, five POST routes,
  the rail's two lists with a control per record, and a delete warning that refuses nothing.
- 🔴 **Three findings that cost the most, each measured rather than reasoned**: the delete's
  serialisation has **two independent carriers** (removing either reds nothing; removing both takes
  the inserter from over a second to **2.86 ms**); **`ipam_page.rs` had no key guard at all**, so
  thirteen keys landed under a green suite and ten gates — 73 are now asserted in both locales; and
  **AC6 was explained away, then met in Rust only**, caught solely because the test passed a
  populated rail instead of the empty one the compiler would have accepted.
- ⚠️ **Two of my own records were wrong when written and are corrected in place rather than
  replaced**: T5 claimed AC6 met while the too-large branch had no rail, and the M-T3 entry named a
  carrier the next mutation refuted. 🔴 **And this bullet claimed *"both keep their first version
  visible"*, which the code review measured FALSE for one of the two.** T5's false line really is
  preserved, with *"this line stays so the next reader sees…"* beside it; **M-T3's first version was
  REPLACED** and survived only as an allusion — *"the probable carrier this record named one draft
  earlier"* — while §1(c) went on carrying the refuted mechanism with no marker at all. *A correction
  that describes the text it removed is not that text.* Both are struck-through in place now, and
  this bullet says which of the two it was ever true of.
- ⚠️ **The driver reports no test names**, so every mutation carrier here was established by hand —
  own copy, mutation verified applied, single filter, restore verified, never `git checkout`.
- ⚠️ **What is NOT in this story**: the release (`14-4b`, blocked on Guy minting « release » /
  « libérer » in `prd.md` and the UX spec), FR21's VLAN half and FR25.
- ⚠️ **`a11y/seed.sql` is unchanged**, as T5 predicted: the default subnet already carries three
  ranges and two addresses, which the keyboard gate confirms by finding 3 + 2 + 1 removal controls
  and 3 + 2 correction controls on the page it opens.

### File List

**New** — `crates/opencmdb-bin/templates/_ipam_rail_lists.html`

**Modified** — `crates/opencmdb-bin/src/ipam_repo.rs` · `crates/opencmdb-bin/src/ipam_write.rs` ·
`crates/opencmdb-bin/src/ipam_page.rs` · `crates/opencmdb-bin/src/main.rs` ·
`crates/opencmdb-core/src/ipam/mod.rs` · `crates/opencmdb-bin/templates/_ipam.html` ·
`crates/opencmdb-bin/locales/app.yml` · `crates/opencmdb-bin/assets/app.css` · `a11y/kbd-probe.mjs` ·
`crates/opencmdb-bin/src/screens.rs` · `docs/manuals/user-manual/user-manual.tex` ·
`docs/project-context.md` · `CLAUDE.md` ·
`_bmad-output/implementation-artifacts/sprint-status.yaml` ·
`_bmad-output/implementation-artifacts/deferred-work.md` · this story file.

🔴 **`screens.rs` and `deferred-work.md` were both MISSING from this list until the slice-D review,
and that omission is why the review partition leaked.** A slice is cut from the File List; a file
absent from it is a file no slice can contain, and `screens.rs` was consequently handed to no layer
across three rounds. `deferred-work.md` is the artefact this story cites as *"REGISTERED"* eleven
times. *The manifest is load-bearing, and nothing in this project checks it against the diff.*
