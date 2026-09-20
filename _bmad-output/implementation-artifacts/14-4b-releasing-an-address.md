# Story 14.4b: Releasing an address

Status: done

🔴 **INSERTED at story 14.4's validation, 2026-09-16 (Guy): Epic 14 goes from six stories to SEVEN**
(`epics.md` NOT edited; registered). 14.4 keeps the corrections — edit and delete — and this story
takes the RELEASE. The seam is the one the measurements support: **the two halves share nothing**;
this one alone carries a migration, a new binding word and a change to what the audit reads.

⚠️ **The validation ran on the COMBINED story before the split**, by two fresh-context layers
(fact-check; gap-hunt, which BUILT all four release shapes against a live MariaDB and refuted two).
Everything in §1 is measured, not reasoned.

🔴 **BLOCKED ON A PLANNING ACT**: the gesture has **no binding word**, and Guy takes the minting
(decision 4 of 2026-09-16) — see §1(d). This story writes the word; it does not invent it.

~~Baseline: **980 tests** (690 bin + 191 core + 99 xtask) at `82ff356`. Ten gates, axe 0 over 10 routes
+ 5 states, kbd 45/45.~~ ⚠️ **Struck at the code review**: that was 14.4's VALIDATION baseline, before 14.4
shipped. The real baseline, `master` at `031e2d7`: **995** (705 + 191 + 99), kbd 53.

## Story

As the operator,
I want to release an address I know is free,
So that the plan stops holding it against me.

🔑 **Constraint (3) is why this story exists** (`epics.md:2351`): *a sighting protects an address
FOREVER, until the operator releases it*. The product cannot tell **gone** from **silent** — one sweep
of the reference LAN sees 46 hosts where 49 exist — so **the grid fills and never empties by itself**.
Without this story, 14.3b's audit is a one-way ratchet: every sighting is permanent and the offer
shrinks for ever.

## §0 — What is settled

**Guy's six arbitrations of 2026-09-10**, the epic's **six constraints** (`epics.md:2345-2357`), and
**14.2b's five decisions** (the port-not-pool state, the form in the rail, the server-minted id with
the arriving nil refused, the label required at the route). **14.3b's audit** is what this story
changes the inputs of.

### 🔴 Guy's decision of 2026-09-16 (1 of 4): a release is a ROW IN ITS OWN TABLE

```sql
CREATE TABLE IF NOT EXISTS address_release (
  addr VARCHAR(39) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  released_at DATETIME(6) NOT NULL,
  PRIMARY KEY (addr)
) ENGINE = InnoDB;
```

plus `0007`'s anchored canonical `RLIKE` on `addr`. Keyed on the **address**, consulted by the audit,
**the ingest path untouched**. The three refused shapes and what refuted each are in §1(a) — they were
built, not argued.

## §1 — What this story inherits, each WITH the measurement that produced it

### (a) 🔴 THE THREE REFUSED SHAPES, AND WHY EACH FAILED

A release that only **DELETES** the summary rows is undone by the next sweep: `upsert_sighting`
(`sighting_repo.rs:126`) is `INSERT … ON DUPLICATE KEY UPDATE first_seen_at = LEAST(…), last_seen_at =
GREATEST(…)`, reached on every ingested observation (`scan_pass.rs:146` → `ingest_observation` `:259`
→ `insert_with_sightings` `:207` → `write_observation_and_sightings` `:232`).

| shape | what the measurement did to it |
|---|---|
| **mark `address_sighting`** | **Defeated by an ordinary MAC change.** The key is `(addr, l2_domain, mac)`, so a replaced NIC or a phone rotating its address inserts a **fresh row with `released_at = NULL`** and the address is back, no rule broken. ⚠️ MAC randomisation is already the register's one unbounded axis for this table; nobody had connected the two. |
| **delete only** | **Loses the history constraint (3) exists to keep.** Measured: `DELETE` → 0 rows; one upsert restores the pair with `first_seen_at` **reset** (2026-09-16 10:05 where the original was 2026-09-01 10:00). The address returns as *newly discovered*. |
| **delete + a refusal window** | The same hole as *delete only*, after the window, with more machinery. |

🔑 **The `ALTER` cost was expected to be the discriminator and is not**: `ALTER TABLE address_sighting
ADD COLUMN released_at DATETIME(6) NULL` costs **8.10 ms at 46 rows and 8.02 ms at 200 000**. Story
6.5 refused an `ALTER` at boot on a published product for a reason that does not reach this size, so
the shapes differ only in **what they can express**.

🔑 **And the chosen shape makes the register's own instruction unnecessary.** `deferred-work.md:5605`
tells this story to delete every pair *"sentinel row included"* — an instruction written for the delete
shape. Keyed on the address, **the `-` sentinel is not representable and needs no clause.**

### (b) ✅ THE AUDIT STAYS A READER — measured, not assumed

`the_plan_reads_the_network_only_through_the_audit` forbids four literals in the plan's modules:
`observation_record`, `identity_link`, `declared_attribute`, `address_sighting`. **`address_release` is
none of them**, and it is a PLAN table — so `ipam_write.rs` may own its write and `ipam_audit.rs` may
read it exactly as it reads `ip_range` today.

🔴 **That is what the chosen shape BUYS, and the other shapes did not.** Planting measured it: with the
release writing `address_sighting`, the only shape the guard admits is *a deleter in
`sighting_repo.rs`, re-exported by `ipam_audit.rs`, called from `ipam_write.rs`* — which makes the one
door **export a write** and falsifies its own module doc (*"the only place … that reaches outside the
plan's tables"*, a reader throughout). ⚠️ **Assert this rather than inherit it**: a test that the guard
is green with the release in `ipam_write.rs` is cheap and pins the property the split was chosen for.

### (c) What the audit must do with a release

`ipam_audit::Plan` decides the verdict and the offer from the sightings. A released address must stop
being a finding and become offerable again **under the rule this story writes**. The two questions
§2 poses are exactly the ones a join cannot answer by itself:

Measured shape of the join (built at validation):

| | last seen | released at | answered since the release |
|---|---|---|---|
| an address whose MAC changed after the release | 11:06 | 11:00 | **1** |
| an address genuinely gone | 10:00 | 11:00 | **0** |

### (d) 🔴 THE GESTURE HAS NO BINDING WORD, AND THE MINTING IS GUY'S

The gesture table is `prd.md:991-1002` — **ten rows**: observed · declared · gap · reconcile ·
document · accept-gap · snooze · exclude · triage · source. The UX spec's (`:1339-1351`) has eleven,
adding `attach`. **`release` has no row** — nor have `edit`, `delete` or even `create`. Term search
across both documents: `libérer`, `éditer`, `modifier`, `supprimer` → **zero hits**.

✅ Measured clear: `copy_vocabulary`'s RETIRED lists carry none of these words, so minting reds nothing.
🔑 **Guy's decision 4 of 2026-09-16: he mints `release` / « libérer »**, a planning act on PR #166's
precedent (he took the PLAN axis before Epic 14's first story, so that no story named a column ahead of
the table). **This story may write the word and may not invent it** — if the rows are not in
`prd.md` and `ux-design-specification.md` when T0 opens, the story waits.

### (e) The screen, and what the seed does not carry

Guy's decision 3 of 2026-09-16 puts 14.4's edit and delete controls in **lists in the rail**. Where a
RELEASE is reached is this story's own question (§2.2): the operator meets a held address in the
**findings list** 14.3b built, which is where they are already looking at the sighting that holds it.

⚠️ **`a11y/seed.sql` carries no release case.** It reaches both of 14.4's delete cases already, but a
released address must be seeded as a sighting the gate then PRESSES a control to release — new seed
work, and the gate must find the control the way 6.4 taught it (by the control, never by a translated
word).

### (f) Registered items this story answers

- `deferred-work.md:5605` — the release's removal, re-shaped by §1(a).
- `:5602-5606` — the summary grows by distinct pairs and a rotating MAC adds one per address; *"the
  only removal is the operator's"*. This story is that removal, and §1(a) says why the removal alone
  was not enough.

## §2 — Decisions this story must take at T0

1. 🔴 **What a released address BECOMES when the network answers again.** The table says *released at
   T*; a sighting says *seen at T+1*. Is it silently re-held (the release lapses), is it a FINDING of
   its own (*"you released this and something answered"*), or does the release stand until the operator
   says otherwise? ⚠️ The measurement in §1(c) makes the case distinguishable; what it means is a
   decision, and it is the one an operator will meet most often.
2. **Where the gesture lives on screen** (§1(e)): the findings list, the rail's address list, or the
   cell. 14.3b put the operator in front of the finding; 14.4 put the corrections in the rail.
3. **Whether a release can be UNDONE**, and how — deleting the row is trivial; whether the screen
   offers it is a decision. (The UX spec has an *Undo Toast* component, `:1244-1246`.)
4. **Whether the release is refused for an address the plan DEFINES** — releasing a sighting on an
   address the operator has declared is a different act from releasing a stray one.

## Acceptance Criteria

**AC1 — migration `0009` ships `address_release`** with `0007`'s canonical CHECK on `addr`, the
`IF NOT EXISTS` + `success = 0` recovery recipe its header carries, and the `ddl-collation` gate green.

**AC2 — a held address can be released, and the release SURVIVES the next sweep.** The test that proves
it ingests an observation of the released address afterwards and asserts the audit's answer is
unchanged — the property every refused shape failed (§1(a)).

**AC3 — the release is RECORDED** (`epics.md:2457`): `released_at` is written and read back.

**AC4 — the audit and the offer agree with the release**: a released address stops being a finding and
becomes offerable under decision 1's rule, and a test asserts both halves.

**AC5 — the audit stays a READER** (§1(b)): the plan's guard is green with the release's write in
`ipam_write.rs`, and a test says so rather than the split's rationale being a comment.

**AC6 — the gesture carries the binding word Guy mints** (§1(d)), in both locales, and the vocabulary
gate is green.

**AC7 — every refusal is a KEYED body in both locales.**

**AC8 — the gesture is keyboard-reachable and named to a screen reader**, PRESSED by
`a11y/kbd-probe.mjs` against a seeded release case, and clean under axe.

**AC9 — THE LIVE COUNT lives here**, both store conditions and the command named.

**AC10 — no regression**: ten gates, `clippy --all-targets`, both store conditions, `cargo deny`, both
manuals, both browser gates, documents current before the push.

## Tasks / Subtasks

- [x] **T0** Take §2's four decisions with Guy, and **check that the binding rows exist** (§1(d)) —
      the story waits on them rather than inventing the word.
- [x] **T1** (AC1) Migration `0009`, with its recovery recipe and the collation gate.
- [x] **T2** (AC2, AC3) The adapter's write and read, and the survive-the-sweep test.
- [x] **T3** (AC4) The audit's join and the offer's agreement, in decision 1's rule.
- [x] **T4** (AC5) The guard's test — green with the write where the split put it.
- [x] **T5** (AC6, AC7) The word and the refusals, both locales.
- [x] **T6** (AC8) The control where decision 2 puts it; the seed's release case; the keyboard gate's
      new checks; the axe pass.
- [x] **T7** (AC9, AC10) Measure. Prove-to-red with `cargo xtask mutate --baseline`, predictions
      first; both browser gates; the documents and both twins.

### Review Findings

Three isolated layers, 2026-09-19 (Blind Hunter: code diff only; Edge Case Hunter: own worktree and
own `mariadb:10.11.11` on :13420, measured; Acceptance Auditor: full diff incl. the register).
31 raw findings → 21 distinct: 3 decisions, 13 patches, 2 deferrals, 5 dismissed with their check.

✅ **RESOLVED and REPAIRED the same day.** Guy took the three decisions, each on the recommended
option: **(1)** a release forgets what the operator was SHOWN — the form carries the finding's last
sighting as `seen_until`, and no clock is read at all; **(2)** both locks — the audit ignores a release
on a defined address, and defining or correcting an address deletes its release; **(3)** the
confirmation rides in the URL (`&released=<address>`), rendered only for a value that parses as an
address. Then all thirteen patches were applied. The review-repair mutation pass, predictions written
first (`cargo xtask mutate --baseline`, the store dropped and recreated before each):

| id | mutation | predicted | measured |
|---|---|---|---|
| R1 | the audit stops ignoring a release on a defined address | red:1 | red 1 ✅ |
| R2 | defining an address no longer deletes its release | red:1 | red 1 ✅ |
| R3 | correcting onto a released address no longer deletes it | red:1 | red 1 ✅ |
| R4 | the release forgets five minutes past what was shown | red:1 | red 1 ✅ |
| R5 | the redirect goes to the OUTERMOST subnet | red:1 | red 1 ✅ |
| R6 | the audit names a sighting WRITER in its code | red:1 | red 1 ✅ |
| R7 | the confirmation renders for a value that is not an address | red:1 | red 1 ✅ |

⚠️ **Not executable, and said rather than skipped**: a mutation of `0009`'s CHECK — the driver's baseline
migrates the unmutated file first, so the mutated run meets a checksum mismatch and reds every store
test for that reason. The CHECK is carried by raw inserts in
`the_release_table_holds_one_spelling_and_the_reader_skips_what_went_round_it`.
⚠️ **And one guard reddened in the full run that the targeted runs never reached**: the key-count guard
saw `ipam.released_note` (76 → 77, read off its printed list). *A targeted run is not a regression run.*

- [x] [Review][Decision] **A live host's address can be offered right after a release** (all THREE
  layers) — `released_at` is the PRESS instant (`page::now_utc()`), while every observation of a sweep
  is dated at the sweep's START (`arp_ping.rs:256`). A host answering in a sweep already running when
  the operator presses is forgotten, and inside `static` its address is offered until the next sweep
  (~5 min). The same clock choice makes the release depend on clock skew (a clock stepped forward makes
  it sticky under `GREATEST`) and makes the seed compare the database clock with the app's.
- [x] [Review][Decision] **A release row survives on a DEFINED address** (blind + edge, MEASURED 31 of
  61 concurrent define/release pairs) — also reachable without a race: release, then define; nothing
  deletes the row, and when the record is later removed the old release silently comes back into
  force. It hides a « Conflit d'adresse » formed from the forgotten sightings meanwhile.
  `ipam_repo.rs:release_address`'s *"harmless by construction"* is false.
- [x] [Review][Decision] **After a release the operator sees no confirmation** (auditor) — the route
  answers `HX-Redirect`, so `ipam.done.release` — the only in-product sentence saying the release
  LAPSES when the network answers — is never displayed; with no undo and no before-the-press warning.
- [x] [Review][Patch] kbd-probe: the *"no re-seed branch is owed"* comment gives the wrong reason, and
  the documenting block's *"the only check here that writes"* is now false [a11y/kbd-probe.mjs]
- [x] [Review][Patch] kbd-probe: `waitForNavigation` is armed AFTER the response, a race [a11y/kbd-probe.mjs]
- [x] [Review][Patch] a store-test assertion is vacuous: `next_offerable == .10` holds with or without
  the release [crates/opencmdb-bin/src/ipam_audit.rs]
- [x] [Review][Patch] `unknown_record`'s dead `Release` arm pairs 404 with the backend sentence, and
  `ipam_refusal` maps `Release` to a range sentence [crates/opencmdb-bin/src/ipam_write.rs]
- [x] [Review][Patch] kbd-probe's `.61` check proves "not a finding", not "sighted then forgotten" —
  the comment claims more than the check [a11y/kbd-probe.mjs]
- [x] [Review][Patch] the INNERMOST-subnet redirect is carried by no test (edge MEASURED
  `max_by_key` GREEN) [crates/opencmdb-bin/src/ipam_repo.rs]
- [x] [Review][Patch] `0009`'s canonical CHECK and `plan_releases`' skip branch are carried by no test
  [crates/opencmdb-bin/migrations/0009_address_release.sql]
- [x] [Review][Patch] AC5: the guard pins where the write lives, not that `ipam_audit.rs` stays a
  READER of `sighting_repo` [crates/opencmdb-bin/src/ipam_page.rs]
- [x] [Review][Patch] both twins still say *"Epic 14 has SIX stories"* [CLAUDE.md, docs/project-context.md]
- [x] [Review][Patch] the story header's *"Baseline: 980"* is neither corrected nor struck
- [x] [Review][Patch] the register's recovery SQL uses the unpadded form and deletes nothing; the admin
  manual does not mention the recovery [deferred-work.md, docs/manuals/admin-manual]
- [x] [Review][Patch] the user manual promises a released address is offered again, false for an
  address DOCUMENTED in the inventory [docs/manuals/user-manual/user-manual.tex]
- [x] [Review][Patch] test cleanup deletes observations by `JSON_SEARCH` (LIKE semantics, any string
  fact) [crates/opencmdb-bin/src/ipam_audit.rs]
- [x] [Review][Defer] any well-formed IPv4 is accepted and `plan_releases` reads the table whole on
  every audit read (blind + edge) — deferred, bounded by an authenticated operator like every write
- [x] [Review][Defer] releasing from an OUTER subnet's view redirects to the INNER subnet (edge,
  measured) — deferred, the address belongs to the inner one and its finding is shown there too
- Dismissed with their check: a 4xx not swapped (refuted — `before-swap` sits on `#ipam-form-result`,
  the swap target of every `/ipam` form); a non-defined conflict-only row (refuted by the auditor —
  `verdict` is `None` only for defined or pool-only addresses); a reader bypassing the releases
  (refuted by the edge layer — every IPAM reader goes through `read_the_network`); the release button
  painting the amber (refuted — `.btn-gesture.live` is neutral, the amber is `.btn-document`'s);
  AC2's letter (already registered).

## Dev Notes

### Traps this project has paid for, and which apply here

- 🔴 **A plant that does not COMPILE measures nothing** and its exit status looks like a finding — plant
  against a green control, `--bins` (no lib target).
- 🔴 **A `contains` oracle over a translated sentence cannot measure the negative direction** (Askama
  escapes `'`; 14.3b's MP6) — anchor on a literal.
- 🔴 **A guard placed where the defect cannot occur reads as coverage and is none.**
- ⚠️ `DB_TEST_LOCK`, one CIDR per test, drop and recreate before any count with one warm run first;
  `app.yml` is invisible to incremental builds; read exit statuses from a FILE; `--all-targets`.

### What the store must answer

`address_release` (new), the plan's three tables, and — through `ipam_audit.rs` alone — story 14.3a's
summary. ⚠️ The summary is READ and never written by this story: that is what §1(b) buys.

### References

- `epics.md` — Story 14.4 `:2443-2461` (its second criterion is this story), constraint (3) `:2351`.
- `prd.md` — the gesture table `:991-1002`; `ux-design-specification.md:1339-1351`, Undo Toast
  `:1244-1246`.
- `deferred-work.md:5602-5606`.
- `14-3a-the-sightings-summary.md`, `14-3b-the-audit.md`, `14-4-maintaining-the-plan.md` (the other
  half), `14-2b-…md` §2 `:168-228`.
- `migrations/0007_addressing_plan.sql` `:91-156`; `src/sighting_repo.rs` `:126`, `:207`, `:232`,
  `:259`; `src/ipam_audit.rs`; `src/ipam_write.rs`.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), 2026-09-19.

### T0 — the four decisions, Guy, 2026-09-19 (each recommendation taken; the option refused is named)

The binding rows were checked first: `release` / « libérer » stands at `prd.md:1002` and
`ux-design-specification.md:1350`, merged by PR #186 — the story's precondition held.

1. **A sighting LATER than the release holds the address again — the release LAPSES.** Refused: a
   distinct finding (a new word and a new `FindingKind` variant), and a release that stands against a
   machine that answers (the duplicate the epic exists to prevent). Implemented as
   `ipam_audit::forget_released`, per hardware address, an instant EQUAL to the release forgotten.
2. **The control lives on each `gap` / `undeclared` finding of the selected subnet.** Refused: the
   rail's address list (it lists DEFINED addresses — exactly the ones a release barely affects) and the
   cell (a 256-cell grid that is not interactive).
3. **No undo on screen; registered.** Refused: a rail list of releases with an undo control, and the
   Undo Toast.
4. **A DEFINED address is refused, 409, with a keyed sentence naming the gesture that applies
   (removing the address).** Refused: accepting it, which could only hide a « Conflit d'adresse ».

### Implementation plan, as built

- `0009_address_release.sql` — `address_release(addr PK, released_at)`, `0007`'s anchored canonical
  `RLIKE`, `IF NOT EXISTS`, the `success = 0` recovery recipe. No foreign key (a released address is
  usually one no record names).
- `IpamError::ReleaseOfADefinedAddress` in `opencmdb-core` (`ALL` 8 → 9) — one variant, no behaviour
  change to the others.
- `ipam_repo::release_address` (defined-address read + `INSERT … ON DUPLICATE KEY UPDATE
  GREATEST(…)`, returns the INNERMOST containing subnet) and `ipam_repo::plan_releases`.
- `WriteRoute::Release` at `POST /ipam/release` (8 → 9 routes): 200 (an upsert cannot promise it
  created), the port reads the clock at the edge (`page::now_utc`, the clock that dates every
  observation — not the database's `NOW(6)`), a release outside every subnet redirects to `/ipam`.
- 🔑 **The releases are applied ONCE, in `ipam_audit::read_the_network`**, so the audit, the offer,
  the grid's seen marker and the three before-the-write warnings all read `Network::seen` already
  forgotten — agreement by construction rather than by discipline.
- `_ipam_audit.html`: a `hx-post` form per `gap`/`undeclared` finding, the address in the accessible
  name; three keys in both locales (`ipam.finding.release`, `ipam.refusal.malformed_release`,
  `ipam.refusal.release_of_defined`) plus `ipam.done.release`.
- AC5: the plan guard gained `crate::repo::datetime_literal` on its own criterion (it reads nothing)
  and a POSITIVE half — the release's write is inside `ipam_repo.rs` and reached from `ipam_write.rs`.
- `a11y/seed.sql`: `address_release` cleared; `.60` (the dedicated case the keyboard gate presses) and
  `.61` (released at `@t`, an instant equal to its sighting, so forgotten — the seed's one-instant rule
  holds). `a11y/empty-plan.sql` clears releases too. `kbd-probe.mjs`: five checks, 53 → 58.

### Debug Log References

- ⚠️ **AC2 read under decision 1.** The letter (*"ingests an observation afterwards … the audit's
  answer is unchanged"*) cannot hold for an observation dated AFTER the release, which by decision 1
  holds the address again. The test asserts what the refused shapes lost: an observation DATED BEFORE
  the release leaves the audit unchanged and the summary keeps its `first_seen_at`; then an observation
  after, on a NEW MAC, holds it again without a conflict. Registered; `epics.md` not edited.
- 🔴 **The mutation pass — predictions written to a file BEFORE any run** (every run
  `cargo xtask mutate --baseline`, store dropped and recreated first):

| id | mutation | predicted | first pass | after repair |
|---|---|---|---|---|
| M1 | `forget_released` keeps an EQUAL instant | red:1 | red 1 ✅ | — |
| M2 | `plan_releases` reads nothing | red:3 | red **4** ❌ | red 3 ✅ |
| M3 | the INSERT spelled with two spaces | red:1 | red 1 ✅ (a spelling tripwire, registered) | — |
| M4 | the defined-address read finds nothing | red:2 | red **3** ❌ | red 2 ✅ |
| M5 | a second release overwrites (no `GREATEST`) | red:1 | red **2** ❌ | red 1 ✅ |
| M6 | the template offers a release on every finding | red:1 | red 1 ✅ | — |
| M7 | a release outside every subnet → `?subnet=<addr>` | **green** | green ✅ — a GAP | M7b red:1 ✅ |
| M8 | decision 4's refusal answers 422 | red:2 | red **3** ❌ | red 2 ✅ |
| M9 | the release dated at the epoch | red:1 | red **2** ❌ | red 1 ✅ |

  🔴 **Five contradictions, every one exactly ONE red over, and the defect was in this story's own
  tests.** Replaying M5 by hand to read the names (the driver prints counts, not names — registered):
  `a_defined_address_is_refused_…` panicked before its trailing cleanup, its DEFINED `100.64.142.9`
  survived, and `ipam_repo`'s plan-wide `the_store_returns_addresses_in_numeric_order` reddened on it.
  The store-backed tests now run their body under `catch_unwind` and clean up on BOTH paths
  (`cleaned_up`); all six re-runs conform. *A carrier count inflated by collateral is not a
  measurement of carriers* — story 14.1's finding, one module over.
  M7 was predicted green and was green: nothing covered a release with no containing subnet.
  `a_release_outside_every_subnet_goes_back_to_the_plan_itself` now does, and M7b reds it.
- ⚠️ **A pre-existing namespace collision, measured and registered, not fixed**: `cargo test` run
  AFTER the browser gates on the same store reds two `ipam_repo` tests (`t-race` inserts the seed's
  own `198.51.100.128/25`; the order test reads the seed's defined `.9`/`.90`). CI runs the tests
  first and cannot see it.
- The browser look: `/ipam` in French on the seeded store, captured — seven « Libérer — 192.0.2.x »
  controls, none on the defined `.9`, `.61` absent, none on the outside list.

### Completion Notes List

- **THE LIVE COUNT (AC9)** — `cargo test --workspace --locked`, wall clock, one warm run first:
  **1 007 tests** (717 bin + 191 core + 99 xtask) after the code review's repair — **24.7 s** against a
  DROPPED-AND-RECREATED `mariadb:10.11.11` (port 13419) and **5.6 s** with `DATABASE_URL` unset; the
  clock is the tell. ~~1 003 (713 + 191 + 99), 24.1 s / 5.6 s~~ at implementation, before the review.
  kbd-probe **59** checks (58 at implementation; the review added the confirmation's).
  Baseline `master` `031e2d7`: 995 (705 + 191 + 99). The story header's *980* was 14.4's validation
  baseline, before 14.4 shipped.
- **AC10**: `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `RUSTFLAGS="-D warnings" cargo test`, `cargo xtask ci` (ten gates; `views-hash` ℹ STALE by design),
  `cargo deny check` — all green. Both manuals build. Browser gates on the final binary: axe **0**
  violation nodes on the empty plan (1 route) and on the seeded store (10 routes + 5 states, all four
  `REQUIRE` flags); kbd-probe **58** checks, 0 failed.
- ACs: AC1 ✅ · AC2 ✅ under decision 1 (read above) · AC3 ✅ · AC4 ✅ (pure test on both halves, and
  the store test on the offer) · AC5 ✅ (guard green, positive half added) · AC6 ✅ (the binding word,
  both locales, `copy-vocabulary` green) · AC7 ✅ (malformed 422 and defined 409, keyed, both locales)
  · AC8 ✅ (pressed by kbd-probe, clean under axe) · AC9 ✅ · AC10 ✅.
- Registered in `deferred-work.md` (seven rows): no undo; no release on the outside list; AC2 vs
  decision 1; AC5's spelling tripwire; the lock-free defined read (harmless, said why); the seed/test
  namespace collision; the mutation driver printing no names. The row that assigned this story the
  delete-every-pair shape is marked ANSWERED in place, with what it leaves open (the summary's growth
  under MAC randomisation is not bounded by a release).
- ⚠️ **What the operator can DO now**: release a held address from its finding, and see it leave the
  list and — inside a `static` range — return to the offer. The grid finally EMPTIES by the operator's
  hand, which is constraint (3)'s whole point.

### File List

- `crates/opencmdb-bin/migrations/0009_address_release.sql` (new)
- `crates/opencmdb-core/src/ipam/mod.rs`
- `crates/opencmdb-bin/src/ipam_repo.rs`
- `crates/opencmdb-bin/src/ipam_write.rs`
- `crates/opencmdb-bin/src/ipam_audit.rs`
- `crates/opencmdb-bin/src/ipam_page.rs`
- `crates/opencmdb-bin/src/main.rs`
- `crates/opencmdb-bin/src/sighting_repo.rs`
- `crates/opencmdb-bin/templates/_ipam_audit.html`
- `crates/opencmdb-bin/locales/app.yml`
- `a11y/seed.sql`
- `a11y/empty-plan.sql`
- `a11y/kbd-probe.mjs`
- `docs/manuals/user-manual/user-manual.tex`
- `docs/manuals/admin-manual/admin-manual.tex` _(added by story 14.4c: the code review's repair touched it and this list omitted it — found by 14.4c's validation measuring this File List against `c605ac4`)_
- `docs/project-context.md`
- `CLAUDE.md`
- `_bmad-output/implementation-artifacts/deferred-work.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/14-4b-releasing-an-address.md`

### Change Log

- 2026-09-19 — T0 decisions taken with Guy; implemented T1–T7; mutation pass (9 + 1 re-designed), a
  defect in the story's own tests found and repaired; status → `review`.
- 2026-09-19 — Code review (three isolated layers): 3 decisions taken by Guy, 16 patches applied, 2
  deferred, 5 dismissed with their check; review-repair mutations R1–R7 all conform. Stays `review`
  until the merge.
- 2026-09-19 — Merged: PR #187 squash-merged as `c605ac4`, CI green on the head `7d7771b` itself (run 35451815399). Status → `done`.
