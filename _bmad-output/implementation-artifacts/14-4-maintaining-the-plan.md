# Story 14.4: Maintaining the plan — the corrections

Status: review

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
- [x] **T1** (AC1) The adapter's edit and delete with the parent-row lock and `capped!`; **widen the
      cap's guard** to cover `DELETE`/`UPDATE`, proven red before it passes. ⚠️ Write, at the site, the
      reason no new gate is owed (decision 4) — the gates' silence must not stand in for it.
      ✅ 2026-09-16. **The widened guard was proven red on a prediction written first**: adding the
      destructive verbs and `FOR UPDATE"` to its needles reddened at `left: 9, right: 5`, and the two
      range writes then took it to **`left: 15, right: 9`** — predicted before the run, and the count
      was moved only after recounting by hand what the fifteen are (3 inserts · 3 deletes · 2 updates ·
      7 locking reads, each named in the failure message). Five writes land: `delete_address`,
      `delete_subnet`, `update_address`, `delete_range` (the computed refusal under the parent-row
      lock, containment compared in Rust per D10) and `update_range`, whose sibling scan is a SECOND
      SQL literal carrying `id <> ?` rather than an `Option<&str>` on the insert's — *a parameter one
      can forget to pass is exactly where forgetting becomes the defect*. `IpamError::RangeStillHoldsAddresses`
      joins the domain (ALL 7 → 8) with the compiler naming the one non-exhaustive site, and its key
      is in both locales. ⚠️ **Two things are NOT proven at T1 and are not claimed**: the widened guard
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
      ✅ 2026-09-16. **`WriteRoute` goes 3 → 8**, so every exhaustive `match` became an `E0004` naming
      each site rather than a list someone had to remember: five POST routes (decision 1), five port
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
      two key guards now cover the modules that render them — 38 in `ipam_write.rs`, and **73 in
      `ipam_page.rs`, which had none at all before this story**, both asserting non-blank in `en` and
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
      ✅ 2026-09-16. **AC7 — THE LIVE COUNT, both store conditions, commands named.** Baseline **980
      → 987** (**697 bin + 191 core + 99 xtask**). Against a live `mariadb:10.11` on port 13450, the
      database **dropped and recreated** and one **warm run** first: `cargo test --workspace --locked`
      = 987, **22.40 s** (warm run 23.12 s). Storeless, same command with `RUSTFLAGS="-D warnings"`:
      987, **5.04 s** — *the clock is the tell*, since the counts are identical either way and only
      the wall time says the database-backed tests executed.
      ✅ **AC8 — no regression**: ten gates (`cargo xtask ci` — all green) · `cargo clippy --workspace
      --all-targets --locked -- -D warnings` · `RUSTFLAGS="-D warnings" cargo test --workspace
      --locked` · `cargo deny --manifest-path Cargo.toml check` (advisories, bans, licenses, sources
      ok — ⚠️ the flag goes BEFORE `check`, an ordering this project has already got wrong once) ·
      `cargo fmt --all` · **both manuals build** (user manual 15 pages) · **both browser gates**: axe
      **10 routes + 5 states, 0 violation nodes** under all four `AXE_REQUIRE_*` flags, kbd **52
      checks, 0 failed**.
      🔑 **Seven prove-to-red passes, every prediction written BEFORE the run**: the widened cap guard
      (`left: 9, right: 5`, then `15/9`, recounted by hand before the count moved) · **M-T2a** and
      **M-T2b**, each reding exactly one test with the OTHER as its green control · **M-T3, M-T3b,
      M-T3c**, whose first two greens are the finding · the key guard, red on a blanked French value ·
      and the two browser checks, red with a deliberate asymmetry (one fails, the other must not).
      ⚠️ **The driver reports no test NAMES — its whole log is 31 lines of summaries — so every
      carrier here was established BY HAND** (own copy, mutation verified applied, single filter,
      restore verified, never `git checkout` on a tree carrying hours of uncommitted work). *A
      matching count is not a matching carrier.*
      ✅ Documents: the user manual's *Defining the addressing plan* section now describes correcting
      and removing, the rail's two lists, and the warning before both removals; both twins and
      `sprint-status.yaml` updated in the same push.

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
  carrier the next mutation refuted. Both keep their first version visible.
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
`docs/manuals/user-manual/user-manual.tex` · `docs/project-context.md` · `CLAUDE.md` ·
`_bmad-output/implementation-artifacts/sprint-status.yaml` · this story file.
