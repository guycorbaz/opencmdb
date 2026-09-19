-- opencmdb — the operator's release of an address (story 14.4b, Epic 14).
--
-- 🔑 EPIC CONSTRAINT (3): a sighting protects an address FOREVER, until the operator releases it.
-- The product cannot tell GONE from SILENT — one sweep of the reference LAN sees 46 hosts where 49
-- exist — so without this table the grid fills and never empties by itself, and story 14.3b's audit
-- is a one-way ratchet.
--
-- 🔴 A RELEASE IS A ROW, NEVER A DELETION (Guy, 2026-09-16; the binding row for `release` says so in
-- `prd.md` and `ux-design-specification.md`). Three other shapes were BUILT at the validation and
-- refused on what they measured:
--   * marking `address_sighting` — defeated by an ordinary MAC change: its key is
--     (addr, l2_domain, mac), so a replaced NIC inserts a FRESH unmarked row and the address is back;
--   * deleting the summary rows — `upsert_sighting` restores them on the next sweep with
--     `first_seen_at` RESET, so the address returns as newly discovered;
--   * deleting plus a refusal window — the same hole after the window.
-- Keyed on the ADDRESS, consulted by the audit, and THE INGEST PATH UNTOUCHED: nothing here is read
-- or written by `sighting_repo.rs`, and `address_sighting` keeps every row it had.
--
-- 🔑 WHAT THE ROW MEANS (Guy's decision 1, 2026-09-19): *everything the network showed on this
-- address up to `released_at` is forgotten by the plan*. A sighting whose `last_seen_at` is LATER
-- than `released_at` is the network answering again, and it holds the address again — the release
-- LAPSES rather than standing against a machine that answers. That is the protective direction: an
-- address something answers on is never offered.
--
-- 🔑 `released_at` IS THE LAST SIGHTING THE OPERATOR WAS SHOWN, carried by the release form — NOT
-- the instant of the press, and not any clock (story 14.4b's code review, Guy, 2026-09-19). Every
-- observation of a sweep is dated at the sweep's START, so a release dated at the press forgot a host
-- that answered in a sweep already running, and offered its address; all three review layers reached
-- it. An instant taken from the data also makes clock skew and a database in another time zone
-- irrelevant — the comparison is between two values the application wrote from the same sightings.
--
-- A second release of the same address REPLACES the instant with the later one (the adapter widens
-- with `GREATEST`): an operator re-releasing an address that answered again is saying *forget this
-- too*, never *forget less*.
--
-- ⚠️ NO FOREIGN KEY onto `ip_address` or `ip_subnet`: a release names an address the plan may not
-- define at all — most held addresses are exactly the ones no record claims. Decision 4 (a DEFINED
-- address cannot be released) is refused by the adapter, not here: a CHECK cannot read another table.
-- 🔴 And a row CAN stand on a defined address — the review measured 31 of 61 concurrent
-- define/release pairs leaving one. The audit ignores such a row, and defining or correcting an
-- address deletes its release (`ipam_repo::forget_release_of`).
--
-- 🔴 `\z` AND NOT `$`, and the octet alternation: `0007`'s pattern, byte for byte, for `0007`'s
-- reasons (a trailing newline was a second spelling; `999.999.999.999` was storable raw).
--
-- D64: MariaDB 10.11+ only; the one column that holds text carries a binary collation.
--
-- 🔴 `IF NOT EXISTS`, because MySQL DDL IS NOT TRANSACTIONAL. If this migration fails part-way,
-- `_sqlx_migrations` may hold a row with `success = 0`. Recovery, on `0003`'s idiom:
--     DELETE FROM _sqlx_migrations WHERE version = 9 AND success = 0;
-- then restart; the statement below is idempotent and the re-run completes what the first did not.

CREATE TABLE IF NOT EXISTS address_release (
  -- The address in `0007`'s canonical padded form: `192.0.2.9` is `192.000.002.009`.
  addr VARCHAR(39) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  -- The instant of the operator's latest release of this address.
  released_at DATETIME(6) NOT NULL,
  PRIMARY KEY (addr),
  CONSTRAINT address_release_addr_canonical
    CHECK (addr RLIKE '^(00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])([.](00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])){3}\\z')
) ENGINE = InnoDB;
