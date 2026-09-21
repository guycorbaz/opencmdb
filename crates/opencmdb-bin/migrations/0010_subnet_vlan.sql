-- opencmdb — the segment a subnet belongs to (story 14.5, Epic 14).
--
-- 🔑 FR21's other half: *"The operator can manage subnets, VLANs, and DHCP ranges"* (`prd.md:906`).
-- The word `vlan` / « VLAN » joined the PLAN axis of the binding vocabulary on 2026-09-21 (Guy's
-- planning act, PR #192) BEFORE this migration named a column — epic constraint (5) forbids a story
-- to extend that table, and PR #166 set the precedent.
--
-- 🔴 0 MEANS NONE, AND THE ALTERNATIVE WAS BUILT AND REFUSED. 802.1Q reserves VID 0 for
-- *priority-tagged, no VLAN*, and 4095 is reserved, so the domain is {0} ∪ [1, 4094]. The obvious
-- shape — a NULLable column — was measured on mariadb:10.11.11 by this story's validation and
-- REFUSED: MariaDB holds NULLs DISTINCT in a UNIQUE key (`0002`'s header, story 5.9's trap), so with
-- NULL for *none* the same CIDR was accepted THREE times (SELECT COUNT(*) → 3) and
-- `ip_subnet_cidr_vlan` stopped being a key at all. A sentinel is what makes the key hold.
--
-- 🔑 WHAT THE WIDENED KEY BUYS: one address space in TWO segments. `192.0.2.0/24` in VLAN 10 and the
-- same CIDR in VLAN 20 are two entries of the plan — the canonical VLAN case, and the reason
-- `0007`'s *"A plan that holds one subnet twice is not a plan"* is rewritten rather than deleted: a
-- plan may hold one CIDR twice, it may not hold one CIDR twice IN ONE SEGMENT.
--
-- ⚠️ WHAT IT DOES NOT BUY, measured by this story's validation with a control, and said here because
-- the operator meets it: the AUDIT is blind to the VLAN. Ranges and defined addresses are read
-- PLAN-WIDE (Guy's decision 2 of 2026-09-10 — the most protective range decides, across nested
-- subnets), and `Subnet` is `{base, prefix_len}`, so two segments on one CIDR share one audit: a
-- `dhcp-pool` declared in the second turned the first's `gap` into `undeclared` and emptied its
-- offer. `/ipam` carries that sentence; this header carries it too, because a schema that admits a
-- shape the readers cannot tell apart owes the reason in writing.
--
-- 🔴 THE FIRST MIGRATION IN THIS REPOSITORY THAT ALTERS AN EXISTING TABLE RE-RUNNABLY, and the
-- spelling is not `0007`'s. `CREATE TABLE IF NOT EXISTS` cannot widen a key; `0003`/`0004`'s ALTERs
-- are not re-runnable at all. Measured: the naive form applied twice answers
-- `ERROR 1826 (HY000) Duplicate CHECK constraint name`. Four `IF NOT EXISTS` spellings are needed,
-- and `ADD CONSTRAINT IF NOT EXISTS` is written nowhere else here.
--
-- 🔴 THE NEW KEY IS ADDED BEFORE THE OLD ONE IS DROPPED, and the order is the whole point: MySQL DDL
-- is not transactional, so a failure between the two statements must never leave `ip_subnet` with no
-- uniqueness key. Keeping the old NAME would force drop-then-add and open exactly that window.
--
-- D64: MariaDB 10.11+ only. ⚠️ `vlan` is NUMERIC and carries no collation — `ddl-collation` matches
-- `VARCHAR`/`TEXT`/`CHAR`/`CLOB` and is silent here, which is measured rather than assumed (planting
-- a `VARCHAR` into this file reds the gate naming its line).
--
-- 🔴 `IF NOT EXISTS` THROUGHOUT, because MySQL DDL IS NOT TRANSACTIONAL. If this migration fails
-- part-way, `_sqlx_migrations` may hold a row with `success = 0`. Recovery, on `0003`'s idiom:
--     DELETE FROM _sqlx_migrations WHERE version = 10 AND success = 0;
-- then restart; every statement below is idempotent and the re-run completes what the first did not.

ALTER TABLE ip_subnet
  ADD COLUMN IF NOT EXISTS vlan SMALLINT UNSIGNED NOT NULL DEFAULT 0;

ALTER TABLE ip_subnet
  ADD UNIQUE KEY IF NOT EXISTS ip_subnet_cidr_vlan (base, prefix_len, vlan);

ALTER TABLE ip_subnet
  DROP INDEX IF EXISTS ip_subnet_cidr;

ALTER TABLE ip_subnet
  ADD CONSTRAINT IF NOT EXISTS ip_subnet_vlan_bound CHECK (vlan <= 4094);
