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
-- ⚠️ **AND `NOT NULL` IS THE SECOND CARRIER, NOT THE FIRST — measured, where this header first
-- implied otherwise.** Mutation D2 relaxed this column to `NULL DEFAULT NULL` and the store tests
-- stayed GREEN: the adapter's `vlan` is a `u16` at every site, so no code path in this product can
-- bind NULL at all, and the trap the validation reproduced needed raw SQL to reach. The property is
-- held by the TYPE; this clause is what stops a future writer going round it, and stating which is
-- which is the difference between a guard and a habit.
--
-- 🔑 WHAT THE WIDENED KEY BUYS: one address space in TWO segments. `192.0.2.0/24` in VLAN 10 and the
-- same CIDR in VLAN 20 are two entries of the plan — the canonical VLAN case. It supersedes
-- `0007:100`'s *"A plan that holds one subnet twice is not a plan"*: a plan MAY hold one CIDR twice,
-- and may not hold one CIDR twice IN ONE SEGMENT.
--
-- 🔴 **AND `0007` IS NOT CORRECTED IN PLACE, WHICH IS A CONSTRAINT RATHER THAN AN OVERSIGHT.** This
-- header first said that sentence *"is rewritten"*, and the code review measured that it is not —
-- `0007` is byte-identical on this branch. It cannot be otherwise: `sqlx` CHECKSUMS an applied
-- migration, so editing one byte of `0007` makes every store that already ran it refuse to boot
-- (story 14.1 met exactly that as `VersionMismatch(7)`). **A shipped migration's prose is as
-- immutable as its SQL**, and the correction therefore lives HERE, in the migration that changed
-- the rule, where a reader of `0007` arrives by following the version order. The story's own
-- *"rewrite the sentences a changed world falsified"* sweep has this one exception, and it is the
-- only file in that list that could not be touched.
--
-- ⚠️ WHAT IT DOES NOT BUY, measured by this story's validation with a control, and said here because
-- the operator meets it: the AUDIT is blind to the VLAN. Ranges and defined addresses are read
-- PLAN-WIDE (Guy's decision 2 of 2026-09-10 — the most protective range decides, across nested
-- subnets), and `Subnet` is `{base, prefix_len}`, so two segments on one CIDR share one audit: a
-- `dhcp-pool` declared in the second turned the first's `gap` into `undeclared` and emptied its
-- offer. `/ipam` carries that sentence; this header carries it too, because a schema that admits a
-- shape the readers cannot tell apart owes the reason in writing.
--
-- ⚠️ **AND IT REACHES THE GRID AND THE COUNTS, which the first version of both sentences left out.**
-- The code review's edge layer measured a page that contradicts itself: with an address defined in
-- segment A only, segment B's grid paints a cell named *defined* and its occupancy line reads
-- *1 defined*, while B's own RAIL lists nothing defined and offers no control for it — because the
-- grid is plan-wide by decision and the rail is per-subnet by decision. Both halves are deliberate
-- and their MEETING was described by nothing. The note now says *nothing below this line uses it*
-- rather than naming three readers of five.
--
-- 🔴 THE FIRST MIGRATION IN THIS REPOSITORY THAT ALTERS AN EXISTING TABLE RE-RUNNABLY, and the
-- spelling is not `0007`'s. `CREATE TABLE IF NOT EXISTS` cannot widen a key; `0003`/`0004`'s ALTERs
-- are not re-runnable at all. Measured: the naive form applied twice answers
-- `ERROR 1826 (HY000) Duplicate CHECK constraint name`. Four CONDITIONAL spellings are needed — and
-- they are not all the same word: `ADD COLUMN IF NOT EXISTS`, `ADD UNIQUE KEY IF NOT EXISTS`,
-- `ADD CONSTRAINT IF NOT EXISTS` and `DROP INDEX **IF EXISTS**`, the last being a removal and taking
-- the other form. ⚠️ This header said *"four `IF NOT EXISTS` spellings"* until the code review, which
-- is one word wrong about the one statement that differs. `ADD CONSTRAINT IF NOT EXISTS` is written
-- nowhere else in this repository.
--
-- 🔴 THE NEW KEY IS ADDED BEFORE THE OLD ONE IS DROPPED, and the order is the whole point: MySQL DDL
-- is not transactional, so a failure between the two statements must never leave `ip_subnet` with no
-- uniqueness key. Keeping the old NAME would force drop-then-add and open exactly that window.
--
-- ⚠️ **NO MUTATION CAN CARRY THIS, and it is recorded rather than guarded.** D4 swapped the two
-- statements and every test stayed green — correctly: a run that SUCCEEDS is indistinguishable
-- either way, and what the order protects is a run that fails BETWEEN them. Green was the
-- prediction and green is the measurement; the property lives in this sentence and in review.
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
