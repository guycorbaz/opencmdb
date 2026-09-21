-- opencmdb — the plan may hold IPv6 (story 14.6, Epic 14, FR25).
--
-- 🔑 FR25: *"The operator can document IPv6 subnets and addresses (observation-only; active IPv6
-- scanning is out of MVP scope)"* (`prd.md:910`). `0007` reserved the WIDTH — `VARCHAR(39)`, which is
-- exactly the expanded IPv6 form's length, and `prefix_len <= 128` — and left the GRAMMAR IPv4-only,
-- so no IPv6 value could be stored at all. Measured before this file was written: three refusals,
-- each naming its own constraint (`ip_subnet_base_canonical`, `ip_address_canonical`,
-- `ip_range_first_canonical`).
--
-- 🔴 FOUR COLUMNS, AND EXACTLY FOUR. `address_sighting.addr` (`0008:72`) and `address_release.addr`
-- (`0009:59`) carry the same IPv4 pattern and **STAY NARROW BY DECISION** (Guy, 2026-09-21). They are
-- the OBSERVED side: the connector emits IPv4 only, and `Fact` — the domain type — has no IPv6
-- variant at all, so an IPv6 row there is a state nothing can produce. Widening them would build the
-- path this project's dominant defect class is named for: *a guard placed where the defect cannot
-- occur*, written into the schema. `the_observed_side_stays_narrow` reds if either is ever widened.
--
-- 🔑 THE CANONICAL FORM: fully expanded, zero-padded, LOWER-CASE, 39 characters —
-- `2001:0db8:0000:0000:0000:0000:0000:0001`. D-14.1(b) chose zero-padding for IPv4 so that
-- lexicographic order is numeric order, and the story's validation measured that the property extends
-- for free WITHIN a family: `::1 < ::a < ::10 < ::ff < ::1:0`, read back from an `ascii_bin` column.
-- ⚠️ ACROSS families it INTERLEAVES — `0000:…` sorts before `009.0.0.1` sorts before `2001:db8:…`
-- sorts before `255.255.255.255` — so the plan-wide order is per-family and NOT global. Nothing
-- depends on a global one today; saying so beats letting the next reader discover it.
--
-- ⚠️ RFC 4291's IPv4-embedded text form (`ffff:…:255.255.255.255`) is FORTY-FIVE characters and this
-- column cannot hold it. It is excluded by the pattern rather than truncated — a refusal the operator
-- can read beats a silent truncation, which is `0007`'s own rule for the label.
--
-- 🔴 DROP-THEN-ADD, KEEPING THE FOUR NAMES, AND THAT IS A DECISION RATHER THAN A LAPSE.
-- Story 14.5's `0010` added its new key BEFORE dropping the old one, because MySQL DDL is not
-- transactional and a failure between the two statements must never leave a table unprotected. The
-- same reasoning would say: add the widened CHECK under a NEW name, then drop the old. It is refused
-- here, on two measurements taken by this story's validation:
--
--   1. **The window is real at SQL level and unreachable through the product.** Between the DROP and
--      the ADD, `192.0.2.9` and `999.999.999.999` both insert. But an interrupted `0011` leaves
--      `_sqlx_migrations` carrying `version 11, success 0`, and the binary then REFUSES TO BOOT,
--      naming its own remedy: `migration 11 is partially applied; fix and remove row from
--      _sqlx_migrations table`. Nothing the product does can write into that window.
--   2. **A rename costs story 14.1's inherited prove-to-red outright.**
--      `the_family_check_is_implied_until_a_second_width_exists` reads the constraint BY THE NAME
--      `ip_range_first_canonical` and `.expect()`s a row. Under a rename it panics on `RowNotFound`
--      and never reaches EITHER of the two sentences written to guide this story.
--      *A guard first seen red by an absent row has not been seen red.*
--
-- 🔴 EVERY PROBE OF THIS PATTERN RUNS THROUGH THE COLUMN, NEVER AGAINST A LITERAL, and the reason is
-- a near-miss this story's validation recorded: `SELECT 'literal' RLIKE pattern` over two utf8mb4
-- literals is CASE-INSENSITIVE and answers 1 for `2001:0DB8:…`, which the `ascii_bin` column refuses
-- with `ERROR 4025`. ⚠️ `0007`'s own header records its probe results in that literal form — sound for
-- IPv4, which has no letters, and WRONG here. The verdicts below were taken by INSERT.
--
-- ⚠️ `\z`, NEVER `$`: story 14.1's measured trap. In MariaDB's `RLIKE`, `$` matches before a final
-- newline, so `$` would admit a second spelling of every address.
--
-- D64: MariaDB 10.11+ only. The columns are untouched, so `ddl-collation` has nothing to say here —
-- measured rather than assumed.
--
-- 🔴 `IF EXISTS` / `IF NOT EXISTS` THROUGHOUT, because MySQL DDL IS NOT TRANSACTIONAL. Recovery, on
-- `0003`'s idiom and `0010`'s:
--     DELETE FROM _sqlx_migrations WHERE version = 11 AND success = 0;
-- then restart; every statement below is idempotent and the re-run completes what the first did not.

ALTER TABLE ip_subnet DROP CONSTRAINT IF EXISTS ip_subnet_base_canonical;
ALTER TABLE ip_subnet
  ADD CONSTRAINT IF NOT EXISTS ip_subnet_base_canonical
    CHECK (base RLIKE '^((00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])([.](00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])){3}|[0-9a-f]{4}(:[0-9a-f]{4}){7})\\z');

ALTER TABLE ip_range DROP CONSTRAINT IF EXISTS ip_range_first_canonical;
ALTER TABLE ip_range
  ADD CONSTRAINT IF NOT EXISTS ip_range_first_canonical
    CHECK (first_addr RLIKE '^((00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])([.](00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])){3}|[0-9a-f]{4}(:[0-9a-f]{4}){7})\\z');

ALTER TABLE ip_range DROP CONSTRAINT IF EXISTS ip_range_last_canonical;
ALTER TABLE ip_range
  ADD CONSTRAINT IF NOT EXISTS ip_range_last_canonical
    CHECK (last_addr RLIKE '^((00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])([.](00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])){3}|[0-9a-f]{4}(:[0-9a-f]{4}){7})\\z');

ALTER TABLE ip_address DROP CONSTRAINT IF EXISTS ip_address_canonical;
ALTER TABLE ip_address
  ADD CONSTRAINT IF NOT EXISTS ip_address_canonical
    CHECK (addr RLIKE '^((00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])([.](00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])){3}|[0-9a-f]{4}(:[0-9a-f]{4}){7})\\z');
