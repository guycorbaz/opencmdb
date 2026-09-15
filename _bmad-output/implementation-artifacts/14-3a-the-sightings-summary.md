# Story 14.3a: The sightings summary

Status: ready-for-dev

⚠️ **`ready-for-dev` is the workflow's status, not a statement that nothing is open.** Contexted and
VALIDATED 2026-09-15 by two fresh-context layers (fact-check; gap-hunt, which built a prototype
migration and measured it); §2's decisions are Guy's and are not taken yet — **T0 takes them before
any code**.

Epic 14 (IPAM). 🔴 **INSERTED 2026-09-15 at story 14.3's VALIDATION (Guy: split)** — Epic 14 goes from
five stories to SIX (`epics.md` not edited; registered). Story 14.3b (the audit) is decided and blocked
on this one. Baseline: `02d15f2` (master), **924 tests** (634 bin + 191 core + 99 xtask), ten gates —
re-measured by the gap-hunt on a virgin store after one warm run.

## Story

As the operator,
I want the product to remember, cheaply and for ever, every address it has seen and the hardware
addresses seen on it,
so that the audit can tell me an address is in use without re-reading the whole history of the
network on every page.

🔑 **Why this story exists, measured and not argued.** Story 14.3's gap-hunt timed the only production
reader returning every observation's facts, `repo::load_observation_facts`, plus a Rust fold, on rows
shaped like the ARP/ping connector's (46 hosts, a sweep every 5 min, `cargo test --release` on a 32-core
box — the NAS is slower): **3 004–3 325 ms and 492 MB peak at 1 000 000 rows** (≈ 75 days of sweeping;
a year is ≈ 4.8 M rows); this story's gap-hunt re-measured 3 472 ms / 475 MB. NFR2's p95 is 1.5 s. The
fold costs 18–49 ms; the READ is the whole cost. Epic constraint 3 — *a sighting protects an address
FOREVER, until the operator releases it* — needs every past sighting. **At 1 M rows there are 46
distinct addresses.** A summary keyed by the distinct pair is bounded by the network, not by time.

## §0 — What is settled, and must not be re-opened

- **Guy's split (2026-09-15, story 14.3 decision 5(d))**: a bounded SIGHTING SUMMARY — one row per
  distinct observed sighting, keyed by the IPv4 address and the hardware address (the exact key is
  decision 3), with its first and last sighting — **maintained with ingest and backfilled once from
  `observation_record`**; **no screen**.
- **Epic constraint (3)** (`epics.md:2351`): a sighting holds an address until the operator releases
  it; RELEASE is story 14.4's. **Product code in this story never deletes a summary row.** (Test
  fixtures and the two seeds clear it beside `observation_record`, which they already clear — AC8.)
- **The comparison rule** (`architecture.md:239`; D10, `0001_initial.sql:23-24`): *all value comparison
  and normalization happen in Rust, never in SQL.* Canonicalising an address or a MAC is Rust's;
  `LEAST`/`GREATEST` on the instants is BOOKKEEPING — the precedent `repo::widen_interface_seen_window`
  (`repo.rs:864-884`, doc `:851-852`) says exactly that. A SQL backfill parsing the JSON `facts` with
  `JSON_TABLE` is possible on MariaDB 10.11 and **must not** be written: it is the comparison D10
  forbids, and sqlx migrations are SQL-only.
- **Observations are immutable** (NFR5, story 6.3): the `observed-immutable` gate reds on an
  overwriting verb governing `observation_record` (`xtask/src/observed_immutable.rs:154-158`); an
  upsert into ANOTHER table passes — measured green by the gap-hunt, including a literal
  `INSERT INTO … SELECT … FROM observation_record … ON DUPLICATE KEY UPDATE` (`select` governs,
  `sql_text.rs:465-486`).
- **MariaDB 10.11 only; `ascii_bin` text; `CHECK` never an `ENUM`; NULLs are distinct in a UNIQUE key,
  so a key column that can be "absent" carries a SENTINEL** (`0002:8-12`); `CREATE TABLE IF NOT EXISTS`
  with a recovery recipe, MySQL DDL not being transactional (`0006:29-45`, `0007:78-83`).

## §1 — What this story inherits, each WITH the measurement that produced it

⚠️ Line numbers were re-checked on `c533221` by the fact-check layer — re-derive before citing in code.

### (a) The production writer is ONE seam, and the other writers bypass it

`scan_pass::poll_ingest_resolve` (`scan_pass.rs:89`) polls into a `VecSink`, then writes **one
transaction per observation** (`:142-161`), `insert_observation(unit.executor(), …).map_err(classify)`;
a refused row is counted in `failed` and logged, not fatal, with NO retry; a second unit resolves what
landed (`:182-197`). Production calls it from `main.rs:875-877`, `SystemClock.now()` read ONCE per
sweep; every observation of a sweep carries that instant (`arp_ping.rs:253-259`); `obs_id` is
`Uuid::now_v7()` (`:254`).

`repo::insert_observation(executor: E, observation)` (`repo.rs:434`) is a plain INSERT (`:448-450`)
consuming its executor in one `.execute`. ⚠️ **It formats `observed_at` with its OWN
`.format("%Y-%m-%d %H:%M:%S%.6f")` (`:443-446`)** — the second format string `datetime_literal`
(`:892`, doc `:886-887`) says must not exist; the new upsert calls `datetime_literal`, and this story
routes `insert_observation` through it too. Other writers: test callers of `insert_observation`
(`fault_injection.rs:1375`, `main.rs` test module ×6, `page.rs:6201`, `repo.rs:2704`, `resolver.rs:836`,
`:2066`), raw SQL in `diagnostic.rs:1606` (test), and **the two seeds** — `a11y/seed.sql` (`:39`, `:54`)
and `docker/seed-example.sql` (`:24`, `:35`), which DELETE and INSERT observations with `NOW(6)`,
bypassing ingest.

### (b) What a sighting is made of

`Fact` (`opencmdb-core/src/observation/mod.rs:287-293`, `#[non_exhaustive]`, `deny_unknown_fields`):
`Mac { addr: MacAddr, locally_administered }`, `IpV4 { addr: Ipv4Addr }`, `Hostname`,
`DhcpLease { ip, expires_at }`, `Uplink`, `OuiVendor`, `Rtt`. `MacAddr(pub [u8; 6])` (`:219`), lowercase
colon `Display` (`:230-238`), JSON as an integer array (test `:568`). `Observation { …, scope: Scope {
l2_domain, vantage }, facts, … }` (`:392-428`); `l2_domain` is *the MAC's uniqueness space*. **Several
`Mac` facts per observation are legal and exercised** (`resolver.rs:1671`; `identity/l1.rs:140-165`
collects every one); nothing restricts several `IpV4`. The shipped connector emits one `IpV4`, one `Rtt`,
an optional `Hostname`, at most one `Mac` (`arp_ping.rs:442-467`, the "19 of 64" note at `:461`); the
neighbour table holds one MAC per address per sweep (`neighbour.rs:83`) and drops the zero MAC
(`neighbour.rs:178-180`) — ⚠️ **but `MacAddr([0; 6])` is a valid `Fact`**, so a summary must not use it
as a sentinel (measured: it inserts as a separate row).

`repo::ObservedBatch` (`repo.rs:593-609`) carries `id`, `connector_id`, `observed_at`, `facts` — **no
`l2_domain`**, and `load_observation_facts` (`:628-634`) does not select it. sqlx has no `chrono`
feature (`Cargo.toml:51`): instants are written through `datetime_literal` and read through
`DATE_FORMAT` + parsing, as that reader does.

### (c) Conventions — and the MAC precedent is NOT a model

Next migration **`0008`** (seven exist). Ids `CHAR(36) … ascii_bin`; instants `DATETIME(6) NOT NULL`,
never from the clock (`0002:24-25`). The canonical IPv4 precedent is sound and measured again here:
`ip_address.addr VARCHAR(39) … ascii_bin` with the octet-value `RLIKE … \\z` CHECK (`0007:145-156`;
`$` matches before a trailing newline, hence `\z`, `0007:39-49`) and the PAD SPACE carrier
`LENGTH(x) = LENGTH(TRIM(x))` (`0007:60-63`) — a trailing newline and a trailing space are refused
(4025), `NULL` is refused (1048). 🔴 **`interface.mac_canon`'s CHECK (`0003:54`, `mac_canon =
LOWER(mac_canon)`) checks CASE ONLY**: measured, `'zz'` and `''` are accepted, and `'-\n'` inserts beside
`'-'` as a second "no MAC" row for one address (`LENGTH = LENGTH(TRIM)` does not catch it — `TRIM` removes
spaces only). A full 17-character MAC is safe by accident (the `VARCHAR(17)` width truncates the trailing
character, Note 1265); a short sentinel is not. **The shape CHECK `RLIKE
'^([0-9a-f]{2}(:[0-9a-f]{2}){5}|-)\\z'`** was measured refusing `''`, `'- '` and uppercase, and admitting
`'-'` and a canonical MAC. ⚠️ **`ddl-collation` trap, measured**: a CHECK line containing ` CHAR`
(`CHAR_LENGTH`, `CAST(… AS CHAR)`) without `_bin` reds (`xtask/src/main.rs:422-441`) — use `LENGTH`; a
whole-line `--` comment is skipped (`:424`), an inline comment containing `text` fires. Migrations are
checksummed (`0003:9-11`); they run at boot (`main.rs:629`); `diagnostic.rs:313` derives the count (no
test pins 7). A store that ran the withdrawn `0007` meets `VersionMismatch(7)` (`0007:85-89`).

### (d) Concurrency: the key closes the race, and ORDER closes the deadlock — measured

The race precedent (`repo.rs:3240-3266`, fn `:3268`; the MAC-connector repair's hand-interleaved test,
issue #161 OPEN): read-then-insert on a non-unique index. **A summary is not an identity**: a UNIQUE key
over its whole key does not collide with D21, and `INSERT … ON DUPLICATE KEY UPDATE` is one statement —
no read-then-insert window. Measured on MariaDB 10.11.11 through sqlx prepared statements: `VALUE()`
and `VALUES()` both work; out-of-order arrivals leave `first = LEAST`, `last = GREATEST`; `rows_affected`
is 1 for an insert, 2 for an update; ⚠️ a CHECK is evaluated on the PROPOSED tuple, so `first > last` in
`VALUES` is refused (4025) even when the updated row would be valid — harmless while the adapter sends
`first = last`.

Two OS threads released by a `Barrier`:

| shape | transactions | result |
|---|---|---|
| each inserts one observation + upserts the same pair (ingest) | 400 + 400 | 800 ok, 1 summary row |
| ingest with a new MAC each vs a 46-pair ascending flush | 900 vs 300 | 0 deadlocks |
| **two 46-pair transactions in OPPOSITE orders** | 200 per thread | 🔴 **200 of 400 deadlock victims (1213)** |

**The risk is a transaction upserting SEVERAL pairs in a non-canonical order** — an observation with two
MACs or two IPv4 upserted in fact order, or a backfill flushed in hash order. ⚠️ **Maintained in the same
unit as the observation, a deadlock rolls the OBSERVATION back too** (`scan_pass.rs:142-161`, no retry),
where the plain insert had no such failure mode.

### (e) The backfill, measured at 1 M rows

Release build, 32-core box, 1 000 000 rows generated with `seq_1_to_1000000`:

| variant | wall-clock | peak memory (`VmHWM`) |
|---|---|---|
| `fetch_all` | 1 932 ms | 397 884 kB |
| keyset chunks of 10 000 by `id` | 3 376 / 3 242 ms | 13 184 / 13 720 kB |
| **one streaming `SELECT` (`fetch`), folded into a map** | **1 176 ms** | **9 700 kB** |

🔴 **Chunking costs 2.8× and buys nothing**: interrupted at 500 000 rows and rerun, it re-read all
1 000 000 (no cursor is saved), and the half-finished table already held **all 46 pairs** with older
`last_seen_at` — **nothing but a marker tells an incomplete backfill from a complete one**. 🔴 **One
unreadable row aborts it after all the work**: a planted `{"Lldp":{…}}` fact failed the streaming
backfill at 1.19 s (`unknown variant 'Lldp'`) — at boot that either stops the product or never writes the
marker and re-reads the history at every boot. Realistic: a row written by a newer binary, or a seed. At
4.8 M rows the streaming read extrapolates (linearly, not measured) to ≈ 5–6 s here; the NAS is slower.
`main.rs:629` migrates, then `spawn_scan_loop` starts, then the server serves; neither the `Dockerfile`
nor `docker/docker-compose.yml` has a `HEALTHCHECK`.

### (f) The seeds run AFTER boot, and `NOW(6)` differs per statement

`ci.yml:111` starts the binary and `a11y/seed.sql` runs at `:146`; `docker/seed-example.sql` tells the
operator to run it after the container has started once. **A backfill that runs once at boot never sees
seed rows** — and in CI it backfills whatever the `Tests` step left behind (fixtures delete at the start
of a test, not the end): story 6b.11's *green on residue*. A seed CAN write summary rows in pure SQL as
canonical literals — but `NOW(6)` is evaluated per statement (measured 2.3 ms apart between two
back-to-back INSERTs; rows of ONE multi-row statement share a value); `SET @t = NOW(6)` used by both
statements gives equal instants. No Rust test executes either seed today (`ci.yml:146` and `mutate.rs:915`
help text are their only consumers). Re-running `docker/seed-example.sql` deletes and re-inserts its
observation with a newer instant, while `LEAST` would keep an old `first_seen_at` unless the seed clears
the summary too.

### (g) Readers, fixtures, purge-and-replay, the consumer

`load_observation_facts` is called at `page.rs:1363` (reconcile), `:1411` (triage), `:1699` (inventory),
and `:6243` (test). Inventory needs only the freshest `observed_at` per address
(`inventory_view.rs:174-190`, `:247-252`); triage needs the whole newest batch — **a summary serves
inventory's need, not triage's** (issue #150 stays open). **`DELETE FROM observation_record` appears at 16
fixture sites**, and every one must clear the summary too: `scan_pass.rs:355`, `fault_injection.rs:1370`,
`repo.rs:2577`, `:2679`, `resolver.rs:831`, `main.rs:2990`, `:3113`, `:3258`, `:3789`, `:3888`, `:4042`,
`:4286`, `page.rs:2516`, `:2693`, `:6173`, `diagnostic.rs:1583` — grep for it rather than trusting the
list. No foreign key from the summary onto `observation_record` (it would break those deletes, ERROR
1451). Purge-and-replay (`repo::purge_engine_links` `repo.rs:1483`, test `resolver.rs:1689`) re-runs
`resolve` only, never ingest: pin that `resolve` never touches the summary. ⚠️ A dropped sqlx
`Transaction` rolls back only when its connection is next used — the gap-hunt's prototype held a lock
for 50 s that way; always `rollback().await` (story 14.2b's second review). Story 14.3b's plan guard
matches `code_only(half).contains("ip_address")` (`ipam_page.rs:1205-1211`): the names
`address_sighting` / `sighting_repo.rs` stay outside its perimeter; only a SQL literal naming a plan
table would pull a file in. An address seen both with and without a MAC (a stale neighbour entry) gets a
sentinel row beside its MAC row: 14.3b's two-MAC conflict must not count the sentinel.

### (h) Cost at ingest, and gates, measured

4 600 observations (100 sweeps × 46), each in its own `transact`, release build: **3.111 / 3.234 ms per
observation without the summary, 3.726 / 3.729 ms with it** — ≈ +0.5 ms (+16 %), ≈ +23 ms per sweep. All
ten gates green with the table in place. Changing `insert_observation` to take a connection gave **11
compile errors, all at test call sites**; the production seam and `repo.rs:2604` compile unchanged; and
`conn.begin()` inside the ingest unit becomes a SAVEPOINT — failing the unit afterwards left
observations = 0 and summary = 0 (atomicity measured), a duplicate `obs_id` returns `Constraint("unique")`
with the summary untouched. File sizes (the gate's rule, raw lines before the first line-start
`#[cfg(test)]`): `repo.rs` reads 183 (stopping at a test struct at `:184`) with **1 777 raw lines** before
its real test module — the adapter belongs in a NEW module; `scan_pass.rs` 268, `resolver.rs` 751.

## §2 — Decisions this story must take, and which are Guy's

1. **Where the summary is maintained, given that a deadlock now can fail an observation.** Options:
   **(a) inside `insert_observation`, in the SAME transaction** — every writer keeps the summary in step,
   atomicity measured; the pairs of one observation are upserted in KEY ORDER (which removes the measured
   deadlock shape), and a deadlock is replayed ONCE (`repo::is_deadlock`, story 14.2b's precedent) before
   the observation is counted failed; (b) **after** the observation's transaction, a separate upsert —
   an observation is never lost to the summary, but a failed upsert leaves the summary behind until the
   next backfill, which under decision 2(a) never runs again. *Recommendation: (a)*, with the ordering
   and the replay written as its conditions.
2. **Where and when the backfill runs.** Measured: one streaming read ≈ 1.2 s and ≈ 10 MB at 1 M rows,
   extrapolated ≈ 5–6 s at a year. Options: **(a) at boot, BEFORE serving** — one streaming read folded
   into a map bounded by distinct pairs, ONE flush transaction in key order that writes the "done" marker
   in the same transaction; a row that cannot be decoded is SKIPPED AND NAMED in the log (14.2's
   precedent) and the marker still written with the skipped count; (b) **in the background after
   serving** — the page serves immediately and reads a partial summary until the flush lands; it races
   ingest (0 deadlocks measured, with the flush in key order). *Recommendation: (a)* — a boot delay of
   seconds once, against a screen that would under-report sightings during the first minute of an
   upgraded install.
3. **The key, its column order and its sentinel.** Options: **(a) (address, `l2_domain`, MAC)** — the
   address FIRST, because story 14.3b looks up and ranges over addresses; MAC `'-'` for a sighting with
   no hardware address (it cannot be a MAC and sorts before every hex MAC); (b) (`l2_domain`, address,
   MAC) — the first draft's order, whose first column is always nil for the shipped connector; (c)
   without `l2_domain`. *Recommendation: (a).*
4. **Defaults the developer applies unless Guy refuses them:**
   - the address in the plan's CANONICAL padded form with `0007`'s `RLIKE … \z` and PAD SPACE CHECKs; the
     MAC with the anchored SHAPE CHECK of §1(c) admitting `'-'`; `l2_domain` with a UUID shape CHECK; the
     zero MAC `00:00:00:00:00:00` treated as ABSENT by the Rust derivation (→ `'-'`), not refused by the
     schema — a refusal there would fail the observation under decision 1(a);
   - `first_seen_at` (`LEAST`), `last_seen_at` (`GREATEST`), `CHECK (first_seen_at <= last_seen_at)`;
   - one row per IPv4 × MAC pair of an observation, sorted by key before upserting; no `IpV4` → no row;
     `DhcpLease.ip` NOT counted (no producer), registered;
   - the backfill marker in a small table of its own in `0008`, keyed by name;
   - no foreign key onto `observation_record`;
   - growth by distinct pairs (a randomised MAC adds pairs over time) and RELEASE deleting an address's
     pairs are story 14.4's, registered with it;
   - the reader returns every row decoded to `Ipv4Addr` / `L2DomainId` / `Option<MacAddr>` / instants;
     **merging across `l2_domain`s is 14.3b's**, said in both stories; it carries an item-level
     `#[allow(dead_code, reason = …)]` naming 14.3b, which removes it.

## Acceptance Criteria

⚠️ **The shapes below assume the recommendations.** T0 rewrites any AC a decision changes.

**AC1 — the table and the marker** (migration `0008`, decisions 3–4): every text column `ascii_bin`,
every column NOT NULL, the key UNIQUE over (address, `l2_domain`, MAC); the address CHECKs as `0007`'s;
the MAC's anchored shape CHECK admitting `'-'`; the `l2_domain` UUID shape CHECK; `first_seen_at <=
last_seen_at`; `LENGTH`, never `CHAR_LENGTH`; no `ENUM`; `CREATE TABLE IF NOT EXISTS` with the recovery
recipe in the header; the marker table. **Raw-SQL tests prove each CHECK refuses what the adapter can
never send** (story 5.9's M3): for the MAC `''`, `'-\n'`, `'- '`, `'zz'`, uppercase; for the address a
trailing newline and a trailing space; a duplicate key refused (1062). Green under `ddl-collation`.

**AC2 — maintained with the observation, atomically** (decision 1): an observation landing writes or
updates its summary rows in the SAME transaction; a refused observation (a duplicate `obs_id`) leaves the
summary untouched; a unit failing after the upsert leaves neither — tested by planting each refusal, with
an explicit `rollback().await`.

**AC3 — the upsert's semantics**: a new pair inserts with `first = last = observed_at`; a known pair's
instants widen (`LEAST`/`GREATEST`); OUT-OF-ORDER arrival never regresses either (both orders tested);
the cross product, the `'-'` sentinel, the zero MAC treated as absent, no `IpV4` → no row, `DhcpLease` →
no row, and an address seen both with and without a MAC (two rows) — each tested; instants through
`datetime_literal`, and `insert_observation` routed through it too.

**AC4 — concurrency leaves one row per pair and no ordering deadlock**: the pairs of an observation are
upserted in key order (a pure test of the derivation's order); two OS threads released by a `Barrier`,
each landing observations whose facts list several MACs in OPPOSITE orders, produce **no deadlock** where
unsorted pairs measured 200 of 400 — the order being the carrier, proven red by removing the sort; a
deadlock, if one still occurs, is replayed once before the observation counts as failed.

**AC5 — the backfill** (decision 2): at boot, BEFORE serving; one streaming read of `observation_record`
folded into a map bounded by distinct pairs; one flush transaction in key order writing the marker;
**it produces exactly the summary ingest maintenance would** — tested on a corpus with **at least two
`l2_domain`s**, an observation with two MACs, one with no MAC and two IPv4, one with only a `DhcpLease`,
out-of-order instants, compared against a reference fold over a reader that RETURNS `l2_domain` (not
`load_observation_facts`, which cannot); idempotent (run twice → same table); a run interrupted before its
flush leaves NO marker and the next boot completes it; **an undecodable row is skipped and named, the
backfill completes, and the marker records the skip count** — tested with a planted unknown fact; a later
boot with the marker present does not re-read the history. **Measured at ≥ 1 M observation rows: wall-clock
and PEAK MEMORY with the command**, memory bounded by distinct pairs. ⚠️ Said, not hidden: in CI the boot
backfill runs over the `Tests` step's leftovers.

**AC6 — the reader** (decision 4): every row decoded, timed at the summary's size AND at the distinct-pair
count a year of reference sweeping produces, recorded with its command.

**AC7 — nothing else changes behaviour**: `resolve` and purge-and-replay never touch the summary (pinned);
**all 16 fixture sites** (§1(g), re-grepped) clear the summary beside `observation_record`; no screen,
route or template changes — **and saying so is the criterion** (story 14.1's precedent: the browser gates
are not claimed because nothing renders).

**AC8 — the seeds write their own summary**: `a11y/seed.sql` and `docker/seed-example.sql` clear the
summary beside `observation_record` and write their summary rows as canonical literals **from the one
`@t` their observations use**; a Rust test executes each seed against the test store (e.g.
`sqlx::raw_sql` over `include_str!` — verify multi-statement support first; if unsupported, say what
carries it instead) and asserts the summary equals the reference fold over the observations it wrote.

**AC9 — ingest overhead measured**: time per observation before and after, at the reference rate, with its
command (validation: ≈ +0.5 ms, +16 %, on a 32-core box).

**AC10 — THE LIVE COUNT lives here**, both store conditions named (a virgin store after one warm run;
`env -u DATABASE_URL`, never `DATABASE_URL=`). Baseline **924** (634 + 191 + 99) at `02d15f2`.

**AC11 — no regression**: ten gates, `clippy --workspace --all-targets -D warnings`, `RUSTFLAGS="-D
warnings"`, fmt, `cargo deny`; no file over 2000 lines by the gate's rule, and `repo.rs` does not grow
(the adapter lives in a new module); docs current before push — the administrator manual (a one-time
backfill at the first boot after upgrading, its duration and memory at a stated scale, and that serving
waits for it), both twins, and the register (issue #150 updated: the summary serves inventory's need, not
triage's).

## §3 — What the validation measured that this story carries

Two fresh-context layers on `c533221`, 2026-09-15. **Fact-check** (read-only, plus a scratch SQL check):
2 HIGH, 6 MED, several LOW. **Gap-hunt** (a prototype `0008`, release-build measurements at 1 M rows,
Barrier concurrency): 2 HIGH, 4 MED, 6 LOW. Applied in place above; the ones that changed the story's
shape:

- 🔴 **AC5's oracle could not produce the key** (both layers): `load_observation_facts` returns no
  `l2_domain`. → a reference reader that returns it, a corpus with two `l2_domain`s.
- 🔴 **The MAC CHECK copied from `interface` refuses nothing but case** (both layers, measured `'zz'`,
  `''`, `'-\n'` beside `'-'`) — story 14.1's headline defect, one column over. → an anchored shape CHECK,
  the `'-'` sentinel, the zero MAC treated as absent in Rust.
- 🔴 **A deadlock now can lose an observation, and unsorted multi-pair upserts measured 200 of 400
  deadlocks** → key order, one replay, decision 1 re-posed with the cost.
- 🔴 **Chunking cost 2.8× and bought nothing; one undecodable row aborted the whole backfill after the
  work** → one streaming read, one flush with the marker, skip-and-name.
- ⚠️ **The seeds run after boot, and `NOW(6)` differs per statement** → the seeds write their own summary
  from one `@t`, and a test executes them; story 14.3b's alternative *"or the backfill must run over it"*
  was impossible and is struck there.
- ⚠️ **§0 said "never deletes a summary row" while AC8 had the seeds clear it** → scoped to product code.
- ⚠️ Facts corrected: every citation into `0003`/`0006`/`0007` pointed past the end of its file; the
  fixture list was 4 of 16; `insert_observation` does not go through `datetime_literal`; the race test is
  the MAC-connector repair's, not 5.14's; the reader drops `l2_domain` (merging is 14.3b's); *"a SQL
  migration cannot"* is *must not*; `ddl-collation`'s trap is ` CHAR` in a CHECK line, not a whole-line
  comment.

## Tasks / Subtasks

- [ ] **T0** Take §2's decisions 1–3 with Guy; confirm or refuse the defaults of decision 4; rewrite
  affected ACs.
- [ ] **T1** (AC1) Migration `0008` and the marker table; raw-SQL CHECK tests first.
- [ ] **T2** (AC3, AC4) The pure derivation: from an `Observation`, its pairs in key order (cross product,
  sentinel, zero MAC, canonical forms) — tested without a store.
- [ ] **T3** (AC2, AC4) The upsert inside `insert_observation` (decision 1), `datetime_literal`, the
  replay, the Barrier test with its prove-to-red.
- [ ] **T4** (AC5) The backfill at boot: streaming, one flush with the marker, skip-and-name; the reference
  fold; the 1 M-row measurement.
- [ ] **T5** (AC6, AC7) The reader; all fixture sites; the purge-and-replay pin.
- [ ] **T6** (AC8) The seeds from one `@t`, and the test that executes them.
- [ ] **T7** (AC9, AC10, AC11) Measure; docs; the live count.

## Dev Notes

### Traps this project has paid for, and which apply here

- 🔴 **NULLs are distinct in a UNIQUE key** — a MAC-less sighting needs a sentinel (story 5.9, D21); and a
  sentinel needs a SHAPE CHECK, or its second spelling is a second row (measured here).
- 🔴 **`$` matches before a trailing newline in `RLIKE`; `ascii_bin` is PAD SPACE; `TRIM` removes spaces
  only** — `\z`, `LENGTH = LENGTH(TRIM)`, and a shape CHECK for anything short.
- 🔴 **A CHECK line containing ` CHAR` reds `ddl-collation`** — `LENGTH`, not `CHAR_LENGTH`.
- 🔴 **MySQL DDL is not transactional** — `IF NOT EXISTS` and the `success = 0` recovery recipe.
- 🔴 **Several rows locked in different orders deadlock** — sort by key (measured 200 of 400 unsorted).
- 🔴 **A dropped sqlx transaction rolls back only when its connection is next used** — always
  `rollback().await`.
- 🔴 **A guard the adapter cannot violate is measured by raw SQL or by nothing** (story 5.9's M3).
- 🔴 **A measurement at a flattering scale measures nothing** — measure in days of sweeping, with memory.
- 🔴 **Green on residue** (story 6b.11) — a boot backfill in CI runs over test leftovers; the seeds carry
  their own summary.
- ⚠️ A virgin store races on `migrate!` — warm once; drop and recreate before counting; `DB_TEST_LOCK`;
  `cargo xtask mutate` reports counts, never names; "registered" means a row exists; a cause needs a check.

### Project Structure Notes

- A new `crates/opencmdb-bin/src/sighting_repo.rs` (or similar) for the adapter and the backfill; the pure
  pair derivation beside it or in `opencmdb-core` (no sqlx in core, D47). The boot call next to
  `migrate!` in `main.rs`.
- Names: `address_sighting` / `sighting_repo.rs` stay outside story 14.3b's plan-guard perimeter (measured).
- No new dependency.

### References

- Story `14-3b-the-audit.md` §1(b) (the read measurement), decision 5; `sprint-status.yaml`.
- `_bmad-output/planning-artifacts/epics.md:2351`; `architecture.md:239`.
- Migrations `0001`–`0007`; `repo.rs` (`insert_observation`, `datetime_literal`, `load_observation_facts`,
  `widen_interface_seen_window`, `is_deadlock`, the mint race test); `scan_pass.rs`; `observation/mod.rs`;
  `neighbour.rs`; `xtask/src/observed_immutable.rs`; `xtask/src/main.rs` (`ddl-collation`); `ci.yml`.
- `deferred-work.md` — the unbounded read (`:5389-5406`); issue #150; issue #161.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Contexted 2026-09-15 from a research pass on the ingest path and migration conventions, on Guy's split of
  story 14.3; validated the same day by a fact-check layer and a gap-hunt layer that built a prototype and
  measured it. Three decisions and a set of defaults posed; none taken.

### File List

### Change Log

- 2026-09-15 — **validated and rewritten whole.** The fold AC5 prescribed could not produce the key; the MAC
  CHECK copied from `interface` admitted `'zz'`, `''` and a second spelling of the sentinel; unsorted
  multi-pair upserts deadlocked 200 times in 400 and, in the observation's transaction, a deadlock loses the
  observation; chunking cost 2.8× for nothing and one undecodable row aborted the backfill; the seeds run
  after boot. See §3.
- 2026-09-15 — contexted, inserted at story 14.3's validation.
