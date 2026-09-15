# Story 14.3a: The sightings summary

Status: ready-for-dev

⚠️ **`ready-for-dev` is the workflow's status, not a statement that nothing is open.** §2 poses
decisions that are Guy's, and the mandatory validation (two fresh-context agents) has not run.

Epic 14 (IPAM). 🔴 **INSERTED 2026-09-15 at story 14.3's VALIDATION (Guy: split)** — Epic 14 goes from
five stories to SIX (`epics.md` not edited; registered). Story 14.3b (the audit) is decided and blocked
on this one. Baseline: `02d15f2` (master), **924 tests** (634 bin + 191 core + 99 xtask), ten gates.

## Story

As the operator,
I want the product to remember, cheaply and for ever, every address it has seen and the hardware
addresses seen on it,
so that the audit can tell me an address is in use without re-reading the whole history of the
network on every page.

🔑 **Why this story exists, measured and not argued.** Story 14.3's gap-hunt timed the only reader
that returns every observation's facts, `repo::load_observation_facts`, plus a Rust fold, on rows
shaped like the ARP/ping connector's (46 hosts, a sweep every 5 min, a 32-core box — the NAS is
slower): **3 004–3 325 ms and 492 MB peak at 1 000 000 rows**, which is ≈ 75 days of sweeping; a year
is ≈ 4.8 M rows. NFR2's p95 is 1.5 s. The fold costs 18–49 ms; the READ is the whole cost. And epic
constraint 3 — *a sighting protects an address FOREVER, until the operator releases it* — needs every
past sighting. **At 1 M rows there are 46 distinct addresses.** A summary keyed by the distinct pair is
bounded by the network, not by time.

## §0 — What is settled, and must not be re-opened

- **Guy's split (2026-09-15, story 14.3 decision 5(d))**: a bounded SIGHTING SUMMARY — one row per
  distinct observed (IPv4, hardware address), with the instant it was last seen — **maintained at
  ingest and backfilled once from `observation_record`**; **no screen**. Refused: reusing
  `load_observation_facts` (fails NFR2 after ≈ 2 months), a narrower reader (at the limit at ≈ 75
  days), waiting for a retention decision.
- **Epic constraint (3)** (`epics.md:2351`): a sighting holds an address until the operator releases
  it; RELEASE is story 14.4's. This story therefore **never deletes a summary row** of its own accord.
- **The architecture's comparison rule** (`architecture.md:239`; D10, `0001_initial.sql:22-23`): *all
  value comparison and normalization happen in Rust, never in SQL.* Canonicalising an address or a MAC
  is Rust's; `GREATEST(last_seen_at, ?)` is BOOKKEEPING, not a value comparison — the precedent is
  `repo::widen_interface_seen_window` (`repo.rs:864-884`), whose doc says exactly that.
- **Observations are immutable** (NFR5, story 6.3): the `observed-immutable` gate reds on any
  overwriting verb governing `observation_record` (`xtask/src/observed_immutable.rs:154-158`); the
  summary is a DIFFERENT table and an upsert into it does not trip the gate.
- **MariaDB 10.11 only; `ascii_bin` text; `CHECK` never an `ENUM`; NULLs are distinct in a UNIQUE key,
  so a key column that can be "absent" carries a SENTINEL** (`0002:8-12`, `current_subject`,
  `OPEN_END`, `ABSTAINED_SUBJECT`); `CREATE TABLE IF NOT EXISTS` with a recovery recipe, because MySQL
  DDL is not transactional (`0006:210-224`, `0007:362-367`).

## §1 — What this story inherits, each WITH the measurement that produced it

⚠️ Line numbers were read on `a21c889` (code identical to `02d15f2`) by a research pass — re-derive
before citing.

### (a) The production writer is ONE seam, and the other writers bypass it

`scan_pass::poll_ingest_resolve` (`scan_pass.rs:89`) polls into a `VecSink`, then writes **one
transaction per observation** (`:142-161`): `repo.transact(|unit| insert_observation(unit.executor(),
&observation).map_err(classify))`; a refused row is counted in `failed` and logged, not fatal; a second
unit resolves what landed (`:182-197`). Production calls it from `main.rs:875-877` with
`SystemClock.now()` read ONCE per sweep; `arp_ping.rs:253-259` stamps every observation of the sweep
with that instant; `obs_id` is `Uuid::now_v7()` (`arp_ping.rs:254`).

`repo::insert_observation(executor: E, observation) -> Result<(), sqlx::Error>` (`repo.rs:434`) is a
plain `INSERT INTO observation_record … VALUES (…)` (`:448-450`), `facts` as `serde_json`, `observed_at`
through `datetime_literal` (`:892`, the single formatting site; sqlx has no `chrono` feature). ⚠️ `E` is
consumed by the first `.execute`, so a second statement inside this function means a signature change
(decision 1). **Other writers**: `insert_observation` from test code (`resolver.rs:836`, `:2066`,
`fault_injection.rs:1375`, `page.rs:6201`, `main.rs` test module, `repo.rs:2604`, `:2707`); raw SQL in
`diagnostic.rs:1606` (test); and **the two seeds** — `a11y/seed.sql:39`/`:54` and
`docker/seed-example.sql:24`/`:35` — which DELETE and INSERT observations with `NOW(6)`, bypassing
ingest entirely. **A summary maintained only at ingest, or backfilled only once, does not see seed
rows** — and story 14.3b's AC10 adds `Mac` sightings to `a11y/seed.sql`.

### (b) What a sighting is made of

`Fact` (`opencmdb-core/src/observation/mod.rs:293`, `#[non_exhaustive]`, `deny_unknown_fields`):
`Mac { addr: MacAddr, locally_administered }`, `IpV4 { addr: Ipv4Addr }`, `Hostname`,
`DhcpLease { ip, expires_at }` (also an IPv4), `Uplink`, `OuiVendor`, `Rtt`. `MacAddr(pub [u8; 6])`
(`:219`) compares by bytes, `Display` lowercase colons (`:230-238`); JSON serializes it as an
**integer array** (`{"Mac":{"addr":[2,0,0,0,0,1],…}}`, test `:568`). `Observation { obs_id,
connector_id, observed_at, scope: Scope { l2_domain, vantage }, facts, raw }` (`:392-428`): `l2_domain`
is *the MAC's uniqueness space*. **An observation may carry several `Mac` facts** (exercised,
`resolver.rs:1672`; L1 `keys_of` collects every one, `identity/l1.rs:140-165`) and nothing restricts
several `IpV4`. The shipped connector emits one `IpV4`, one `Rtt`, an optional `Hostname`, at most one
`Mac` (`arp_ping.rs:442-467`). The neighbour table holds one MAC per address per sweep
(`neighbour.rs:83`).

### (c) Conventions the migration must follow

Next number **`0008`**. Opaque ids `CHAR(36) … ascii_bin`; instants `DATETIME(6) NOT NULL`, never from
the clock (`0002:24-25`). The MAC column precedent: `interface.mac_canon VARCHAR(17) … ascii_bin` with
`CHECK (mac_canon = LOWER(mac_canon))` (`0003:171-172`). The canonical IPv4 precedent:
`ip_address.addr VARCHAR(39) … ascii_bin` with the octet-VALUE `RLIKE … \z` CHECK (`0007:429-440`;
`$` matches before a trailing newline, hence `\z`, `0007:323-333`) and the PAD SPACE carrier
`LENGTH(x) = LENGTH(TRIM(x))` (`0007:344-348`); Rust `ipam_repo::canonical` / `from_canonical`
(`ipam_repo.rs:63`, `:76`). `current_subject` is a WRITTEN column, not generated — MariaDB refuses to
index a generated column coalescing to a literal (error 1901, `0002:56-59`). Migrations are
checksummed — never edit an applied one (`0003:9-11`); they run at boot (`main.rs:629`), and
`diagnostic.rs:313` renders their count (check whether a test pins 7). ⚠️ A developer store that ran
the withdrawn `0007` meets `VersionMismatch(7)` (`0007:369-373`).

### (d) Concurrency: the precedent is a race, and the remedy is a key

`repo.rs:3240-3268` (`the_mint_is_not_atomic_and_two_passes_can_duplicate_one_key`) measures two
passes minting two interfaces for one MAC: read-then-insert on a NON-unique index. A UNIQUE there was
written and withdrawn (issue #161, OPEN) because a cloned MAC is two real interfaces (story 5.9 AC5).
**A summary is not an identity**: a `UNIQUE KEY` over its whole key does not collide with D21, and
`INSERT … ON DUPLICATE KEY UPDATE last_seen_at = GREATEST(last_seen_at, VALUE(last_seen_at))` is atomic
per row in InnoDB — no read-then-insert window; every key column NOT NULL (sentinel idiom); concurrent
upserts can DEADLOCK, surfacing as `RepositoryError::Contention` (`repo.rs:1629-1633`). Reachability
today is low — `spawn_scan_loop` runs one pass at a time — which is exactly the shield story 5.14 warned
a later change removes.

### (e) Gates the new code meets

`ddl-collation` (`xtask/src/main.rs:382`): every migration line with `VARCHAR`/`TEXT`/`CHAR`/`CLOB` must
carry `_BIN` or `COLLATE BINARY` (it also fires on a column named `…context…` or a comment containing
"text"). `observed-immutable`: empty allowlist, walks `crates`, `docker`, `a11y`; an upsert into another
table passes, but `INSERT INTO … SELECT … FROM observation_record … ON DUPLICATE KEY UPDATE` must be
checked against its probes. `authorship` and `entity-id-immutable` cover `declared_attribute` only.
`file-size`: `repo.rs` is **1 777 true code lines** (the gate reads 183, stopping at a test struct at
`:184`) — the adapter belongs in a NEW module; `scan_pass.rs` 268, `resolver.rs` 751, `ipam_repo.rs`
752.

### (f) Readers, fixtures, purge-and-replay

Callers of `load_observation_facts`: `page.rs:1363` (`reconcile_view`), `:1411` (`triage_view`),
`:1699` (inventory). Inventory needs only the freshest `observed_at` per address
(`inventory_view.rs:174-190`); triage's `Nouveau` needs the whole newest batch (hostname,
`connector_id`, `id` for the documenting gesture) — **a summary serves inventory's need, not triage's**
(issue #150, OPEN, stays open). Store fixtures (`scan_pass.rs:341`, `repo.rs:2656`, `resolver.rs:816`,
`fault_injection.rs:1360-1380`) delete children before parents and end with
`DELETE FROM observation_record` — **none cleans a new table**, and a foreign key from it onto
`observation_record` would break those DELETEs (ERROR 1451). Purge-and-replay (story 5.10,
`repo::purge_engine_links` `repo.rs:1483`, test `resolver.rs:1689`) re-runs `resolve` only, never
ingest: the summary sits outside it, and what to pin is that `resolve` never touches it.

## §2 — Decisions this story must take, and which are Guy's

1. **Where the summary is maintained.** Options: **(a) inside `insert_observation`** — every writer of
   an observation keeps the summary in step, in the same statement batch; the signature changes from a
   consumed `Executor` to a connection or transaction, and every caller (mostly tests) is adapted;
   (b) in the ingest seam only, beside the call — production-exact, tests and other writers diverge.
   *Recommendation: (a)* — an invariant held by one function cannot drift from a second call site.
2. **How the existing history is backfilled** — a SQL migration cannot: `facts` is JSON with MACs as
   byte arrays, and D10 keeps the parsing in Rust. Options: **(a) once, at boot, in Rust, tracked by a
   marker** (a row stating the backfill completed), STREAMING `observation_record` in `id` order in
   chunks so memory stays bounded, idempotent by the upsert if interrupted; (b) at every boot,
   incrementally from the summary's latest `last_seen_at` — self-heals rows a seed wrote with a newer
   instant, misses older ones, costs a query per boot; (c) no backfill — history before this story is
   forgotten, against constraint 3. *Recommendation: (a)*, with the SEEDS writing the summary
   themselves (they bypass ingest either way).
3. **The key, and a sighting with no hardware address.** Options: **(a) (`l2_domain`, address, MAC)**,
   the MAC column carrying a SENTINEL when the observation had no `Mac` fact — an address seen without
   a MAC is still IN USE, which is what the offer must exclude; (b) (address, MAC) without `l2_domain`;
   (c) no row for a MAC-less sighting. *Recommendation: (a)* — `l2_domain` is the MAC's uniqueness
   space and dropping it cannot be undone later.
4. **What the row stores** — defaults the developer applies unless Guy refuses: the address in the
   plan's CANONICAL padded form with the same `RLIKE … \z` and PAD SPACE CHECKs (one spelling shared
   with the plan, numeric order in text); the MAC lowercase with the `interface.mac_canon` CHECK, the
   sentinel admitted by the CHECK; `first_seen_at` (`LEAST`) and `last_seen_at` (`GREATEST`); **one row
   per IPv4 × MAC pair of an observation** (every `IpV4` fact with every `Mac` fact, or with the sentinel
   when there is none); `DhcpLease.ip` NOT counted yet (no producer), registered; no foreign key onto
   `observation_record` (the summary outlives nothing and must not break the fixtures' deletes).
5. **Growth and release** — default: the summary grows by distinct pairs, and a randomised MAC (19 of 64
   reference neighbours are locally administered) adds pairs over time; RELEASE deleting an address's
   pairs is story 14.4's, registered with it.
6. **The reader until 14.3b exists** — default: the reader returns every row decoded to `Ipv4Addr` /
   `Option<MacAddr>` / instants, carries an item-level `#[allow(dead_code, reason = …)]` naming story
   14.3b, and 14.3b removes it (story 14.1/14.2b's precedent); no screen reads it here.

## Acceptance Criteria

⚠️ **The shapes below assume the recommendations.** T0 rewrites any AC a decision changes.

**AC1 — the table** (migration `0008`): per decisions 3–4, every text column `ascii_bin`, every column
NOT NULL, the address and MAC CHECKs as the plan's and `interface`'s (canonical `RLIKE … \z`, PAD
SPACE `LENGTH = LENGTH(TRIM)`, lowercase MAC, the sentinel admitted explicitly), a UNIQUE key over the
whole identity, `first_seen_at <= last_seen_at` CHECKed, no `ENUM`, `CREATE TABLE IF NOT EXISTS` with
the recovery recipe in its header. Raw-SQL tests prove each CHECK refuses what the adapter can never
send (story 5.9's M3: *a guard the adapter cannot violate is measured by raw SQL or by nothing*),
including a trailing newline and a trailing space.

**AC2 — maintained with the observation, atomically** (decision 1): an observation landing writes or
updates its summary rows in the SAME unit; a refused observation (a duplicate `obs_id`, a malformed row)
leaves the summary untouched — tested by planting the refusal.

**AC3 — the upsert's semantics**: a new pair inserts with `first_seen_at = last_seen_at = observed_at`;
a known pair's `last_seen_at` becomes the later instant and `first_seen_at` the earlier; an observation
arriving OUT OF ORDER never regresses either (tested both orders); the cross product of decision 4 and
the sentinel are tested (two MACs, no MAC, no IPv4 → no row, `DhcpLease` → no row).

**AC4 — concurrent ingest leaves one row per pair**: two connections upserting the same pair,
interleaved by hand on story 5.14's precedent (`repo.rs:3240`), leave exactly one row with the later
instant; a deadlock, if provoked, surfaces as `Contention` and the seam counts it as a failed row.

**AC5 — the backfill** (decision 2): run at boot after `migrate!`, it produces EXACTLY the summary a
fold over `load_observation_facts` would — a test compares the two on a corpus containing several MACs,
MAC-less rows and out-of-order instants; it is idempotent (run twice, same table), resumable (interrupted
halfway, then run, same table), and marked done so a later boot does not re-read the history. **Measured
at ≥ 1 M observation rows: wall-clock and PEAK MEMORY with the command** — the memory staying bounded by
the chunk, not by the table.

**AC6 — the reader** (decision 6): returns every row decoded, timed at the summary's size AND at the size
a year of reference sweeping produces (distinct pairs, not rows), recorded with its command.

**AC7 — nothing else changes behaviour**: `resolve` and purge-and-replay never touch the summary
(pinned by the purge-and-replay test); every store fixture cleans the new table; no screen, route or
template changes — **and saying so is the criterion** (story 14.1's precedent: the browser gates are not
claimed because nothing renders).

**AC8 — the seeds** (decision 2): `a11y/seed.sql` and `docker/seed-example.sql` clear and populate the
summary consistently with the observations they write; a test (or a gate check) asserts that, for each
seed, the summary equals the fold over its observations — so a seed that forgets one reds.

**AC9 — ingest overhead measured**: the time an observation takes to land, before and after, at the
reference rate, recorded with its command.

**AC10 — THE LIVE COUNT lives here**, both store conditions named (a virgin store after one warm run;
`env -u DATABASE_URL`, never `DATABASE_URL=`). Baseline **924** (634 + 191 + 99) at `02d15f2`.

**AC11 — no regression**: ten gates (the new migration green under `ddl-collation`; `observed-immutable`
green with the upsert, checked against its probes), `clippy --workspace --all-targets -D warnings`,
`RUSTFLAGS="-D warnings"`, fmt, `cargo deny`; no file over 2000 code lines (`repo.rs` does not grow —
the adapter lives in a new module); docs current before push — the administrator manual (a one-time
backfill at first boot after upgrading, its duration and memory at a stated scale), both twins, and the
register (issue #150 updated: the summary serves inventory's need and not triage's).

## §3 — What the validation measured that this story must carry

_(Empty until the mandatory validation runs.)_

## Tasks / Subtasks

- [ ] **T0** Take §2's decisions 1–3 with Guy; confirm or refuse defaults 4–6; rewrite affected ACs.
- [ ] **T1** (AC1) Migration `0008`, raw-SQL CHECK tests first.
- [ ] **T2** (AC3) The pure part: from an `Observation`, the pairs it yields (cross product, sentinel,
  canonical forms) — tested without a store.
- [ ] **T3** (AC2, AC4) The upsert, maintained with the observation per decision 1; concurrency test.
- [ ] **T4** (AC5) The backfill at boot, streaming, marked, idempotent, resumable; equality with the fold;
  the 1 M-row measurement.
- [ ] **T5** (AC6, AC7) The reader; fixtures; the purge-and-replay pin.
- [ ] **T6** (AC8) The seeds and their consistency check.
- [ ] **T7** (AC9, AC10, AC11) Measure; docs; the live count.

## Dev Notes

### Traps this project has paid for, and which apply here

- 🔴 **NULLs are distinct in a UNIQUE key** — a MAC-less sighting needs a sentinel, or the key admits
  duplicates (story 5.9, D21).
- 🔴 **`$` matches before a trailing newline in MariaDB `RLIKE`; `ascii_bin` is PAD SPACE** — `\z` and
  `LENGTH = LENGTH(TRIM)` (story 14.1's review).
- 🔴 **MySQL DDL is not transactional** — `IF NOT EXISTS` and the `success = 0` recovery recipe.
- 🔴 **A read-then-insert on a non-unique index races** (story 5.14, issue #161) — the key is the remedy.
- 🔴 **A measurement at a flattering scale measures nothing** — story 14.3's first draft measured one
  week of production; measure in days of sweeping, with memory.
- 🔴 **A guard the adapter cannot violate is measured by raw SQL or by nothing** (story 5.9's M3).
- ⚠️ A virgin store races on `migrate!` — warm once; drop and recreate before counting;
  `DB_TEST_LOCK`; `cargo xtask mutate` reports counts, never names; `diagnostic.rs` renders the migration
  count — check whether a test pins it; "registered" means a row exists; a cause needs a check.

### Project Structure Notes

- A new `crates/opencmdb-bin/src/` module for the summary adapter (`repo.rs` is at 1 777 true code lines);
  the pure pair derivation may sit beside it or in `opencmdb-core` (no sqlx in core, D47).
- The name of the table and module: avoid `ip_subnet` / `ip_range` / `ip_address`, which pull a file into
  the plan guard's perimeter (story 14.3b §1(a)); e.g. `address_sighting` / `sighting_repo.rs`.
- No new dependency.

### References

- Story `14-3b-the-audit.md` §1(b) (the measurement), decision 5.
- `_bmad-output/planning-artifacts/epics.md:2351` (constraint 3); `architecture.md:239`.
- Migrations `0001`–`0007`; `repo.rs` (`insert_observation`, `load_observation_facts`,
  `widen_interface_seen_window`, the mint race test); `scan_pass.rs`; `observation/mod.rs`;
  `xtask/src/observed_immutable.rs`.
- `deferred-work.md` — the unbounded read `:5383-5401`; issue #150; issue #161.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Contexted 2026-09-15 from a research pass on the ingest path and migration conventions, on Guy's split
  of story 14.3. Not yet validated.

### File List

### Change Log

- 2026-09-15 — contexted, inserted at story 14.3's validation.
