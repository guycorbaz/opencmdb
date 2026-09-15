-- opencmdb — the address sighting summary (story 14.3a, Epic 14).
--
-- One row per DISTINCT sighting — an IPv4 address, the L2 domain it was seen in, and the hardware
-- address seen on it — with the first and the last instant it was seen. Maintained in the SAME
-- transaction as each observation (`sighting_repo::insert_with_sightings`) and backfilled once at
-- boot from `observation_record` (`sighting_repo::backfill_at_boot`). Story 14.3b's audit reads it.
--
-- 🔑 WHY IT EXISTS, MEASURED: the only reader of every observation's facts took 3.0–3.3 s and
-- 492 MB at 1 000 000 rows (≈ 75 days of sweeping 46 hosts every five minutes), against NFR2's
-- 1.5 s. At that size there are 46 distinct addresses. This table is bounded by the NETWORK, not by
-- time — ⚠️ except for a host that randomises its MAC, which adds a pair per new MAC (registered
-- with story 14.4, which releases addresses).
--
-- 🔑 EPIC CONSTRAINT (3): a sighting protects an address until the operator releases it. PRODUCT
-- CODE NEVER DELETES A ROW HERE — release is story 14.4's. Test fixtures and the two seeds clear it
-- beside `observation_record`, which they already clear.
--
-- 🔴 THE KEY IS (addr, l2_domain, mac) AND `mac` IS NEVER NULL. MariaDB holds NULLs distinct in a
-- UNIQUE key (`0002`'s header), so a sighting with no hardware address carries the SENTINEL `-`:
-- it cannot be a MAC and it sorts before every hex MAC. ⚠️ NOT the zero MAC `00:00:00:00:00:00`,
-- which is a valid `Fact` — measured by the validation, it inserts as a separate row. The Rust
-- derivation reads a zero MAC as ABSENT; the schema does not refuse it, because under decision 1(a)
-- a refusal here would fail the OBSERVATION's own transaction.
--
-- 🔴 THE MAC CHECK IS A SHAPE CHECK, AND `interface.mac_canon`'s IS NOT A MODEL. That one
-- (`0003`, `mac_canon = LOWER(mac_canon)`) checks CASE ONLY: measured, `'zz'` and `''` pass it, and
-- `'-\n'` inserts BESIDE `'-'` as a second "no MAC" row for one address — `LENGTH = LENGTH(TRIM)`
-- does not catch it, `TRIM` removing spaces only. A 17-character MAC is safe there by accident (the
-- column width truncates); a one-character sentinel is not. The anchored pattern below was measured
-- refusing `''`, `'- '`, `'-\n'` and uppercase, and admitting `'-'` and a canonical MAC.
-- `\z` and not `$`: `$` matches before a final newline in MariaDB's RLIKE (`0007`'s header).
--
-- ⚠️ `ddl-collation` reds on a CHECK line containing ` CHAR` without `_bin` — so `LENGTH`, never
-- `CHAR_LENGTH`, anywhere in this file.
--
-- 🔑 NO FOREIGN KEY onto `observation_record`: sixteen fixture sites delete observations, and a key
-- pointing there would turn each into ERROR 1451. The summary is not a projection of the rows that
-- exist; it is a memory of what was seen.
--
-- 🔑 `first_seen_at`/`last_seen_at` are data-derived, never `NOW(6)`: the adapter writes the
-- observation's own `observed_at` through `repo::datetime_literal`, and widens with
-- `LEAST`/`GREATEST` — bookkeeping, not the value comparison D10 forbids (the precedent is
-- `repo::widen_interface_seen_window`). ⚠️ A CHECK is evaluated on the PROPOSED tuple, so an
-- `INSERT … VALUES` whose own first is after its last is refused even when the merged row would be
-- valid; the adapter always proposes `first <= last`.
--
-- The marker table records that the ONE-TIME backfill completed. It is written in the SAME
-- transaction as the backfill's flush, so a run interrupted before the flush leaves no marker and
-- the next boot redoes it; nothing else can tell a half-finished summary from a complete one.
--
-- D64: MariaDB 10.11+ only; every column that holds letters carries a binary collation.
--
-- 🔴 `IF NOT EXISTS` THROUGHOUT, because MySQL DDL IS NOT TRANSACTIONAL. If this migration fails
-- part-way, `_sqlx_migrations` may hold a row with `success = 0`. Recovery, on `0003`'s idiom:
--     DELETE FROM _sqlx_migrations WHERE version = 8 AND success = 0;
-- then restart; the statements below are idempotent and the re-run completes what the first did not.

CREATE TABLE IF NOT EXISTS address_sighting (
  -- The address in `0007`'s canonical padded form: `192.0.2.9` is `192.000.002.009`.
  addr VARCHAR(39) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  -- The observation's `scope.l2_domain` — the MAC's uniqueness space. Merging across domains is
  -- the reader's business (story 14.3b), never this table's.
  l2_domain CHAR(36) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  -- A lowercase colon MAC, or `-` when the sighting carried none.
  mac VARCHAR(17) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  first_seen_at DATETIME(6) NOT NULL,
  last_seen_at DATETIME(6) NOT NULL,
  -- The address FIRST, because story 14.3b looks up and ranges over addresses (Guy, decision 3).
  PRIMARY KEY (addr, l2_domain, mac),
  CONSTRAINT address_sighting_addr_canonical
    CHECK (addr RLIKE '^(00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])([.](00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])){3}\\z'),
  CONSTRAINT address_sighting_mac_shape
    CHECK (mac RLIKE '^([0-9a-f]{2}(:[0-9a-f]{2}){5}|-)\\z'),
  CONSTRAINT address_sighting_l2_domain_shape
    CHECK (l2_domain RLIKE '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\\z'),
  CONSTRAINT address_sighting_window CHECK (first_seen_at <= last_seen_at)
) ENGINE = InnoDB;

CREATE TABLE IF NOT EXISTS sighting_backfill (
  -- One row per one-time backfill, named. Today there is one: `address_sighting`.
  name VARCHAR(64) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  -- What the completed run read, what it could not decode, and how many pairs it flushed — the
  -- skip count is here so an operator can see that a backfill completed OVER unreadable rows.
  observations_read BIGINT UNSIGNED NOT NULL,
  observations_skipped BIGINT UNSIGNED NOT NULL,
  pairs_flushed BIGINT UNSIGNED NOT NULL,
  PRIMARY KEY (name),
  CONSTRAINT sighting_backfill_name_shape CHECK (name RLIKE '^[a-z_]+\\z')
) ENGINE = InnoDB;
