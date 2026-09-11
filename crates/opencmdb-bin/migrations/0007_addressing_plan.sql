-- opencmdb — the addressing plan (story 14.1, Epic 14).
--
-- Three tables and NO PRODUCER. Nothing in this codebase writes an `ip_subnet`, an `ip_range` or an
-- `ip_address` outside story 14.1's own tests; story 14.2 opens the write routes. Precedent: `0006`.
--
-- 🔑 AN ADDRESS IS A ZERO-PADDED DOTTED QUAD — `192.0.2.9` is stored `192.000.002.009`.
-- Guy's arbitration, 2026-09-11, taken after the validation refuted the first answer. Three reasons,
-- and the third is what a binary column could not give:
--
--   1. LEXICOGRAPHIC ORDER IS NUMERIC ORDER, which is the property this store exists for. It is
--      D10's own precedent applied to addresses — `architecture.md:564`: "timestamps stored as
--      ISO-8601 UTC TEXT so lexicographic order == chronological order". The registered defect at
--      `inventory_view.rs:261` ("192.0.2.9 follows 192.0.2.10") is caused by the ABSENCE of padding,
--      not by text.
--   2. D48's live reason, "the 3 a.m. dump": with a binary column you write a HEX() per query and
--      THE GREP DOES NOT WORK. D48 is scoped to opaque identifiers and does not bind here, but for a
--      product whose subject IS addresses it weighs.
--   3. FIXED WIDTH MAKES THE FAMILY A SAME-ROW CHECK. `LENGTH()` is 15 for IPv4 and, when FR25
--      brings IPv6, 39 for an expanded form. `ip_range_same_family` below is that check, and the
--      validation measured why it is needed: with the refused `VARBINARY(16)` it built a range
--      running FROM an IPv4 address TO an IPv6 one that passed every constraint, and saw an IPv6
--      address RETURNED by a containment query over IPv4 bounds, sorted between .9 and .10.
--
-- ⚠️ THE COLUMN IS VARCHAR(39), NOT CHAR(15), AND THE STORY SAID CHAR(15). Found while building:
-- reason 3 above requires a column that can hold 39 characters, so `CHAR(15)` contradicts the very
-- argument that chose the representation. The width is posed ONCE rather than ALTERed at boot on a
-- published product — story 6.5's precedent, which is what `INT UNSIGNED` was refused for.
-- Measured on mariadb:10.11.11 before choosing: CHAR strips trailing padding on retrieval, so
-- LENGTH() stays semantic under either type; VARCHAR is chosen for the width alone.
--
-- 🔴 ONE ADDRESS HAS ONE SPELLING, AND THE PATTERN BELOW IMPOSES IT — VALUE AS WELL AS SHAPE.
-- `ascii_bin` is a PAD SPACE collation — measured, `'192.000.002.009' = '192.000.002.009 '` is TRUE
-- under it — so without an anchored pattern a trailing space is a second spelling the UNIQUE key
-- does not refuse. A canonical form that is not imposed is not a canonical form.
--
-- 🔴 THE FIRST VERSION OF THIS PATTERN WAS `^[0-9]{3}[.]…[0-9]{3}$` AND IT HAD TWO HOLES, BOTH
-- INSIDE THE PROMISE ABOVE. Two review layers found them independently:
--
--   `$` IS NOT END-OF-STRING IN MariaDB's RLIKE — it matches before a final NEWLINE. Measured:
--   `'192.000.002.009\n'` RLIKE the old pattern → 1, `'…\r'` → 0, `'…\n x'` → 0. So a trailing
--   line terminator was a SECOND accepted spelling of every address, and neither
--   `ip_address_in_subnet` nor `ip_subnet_cidr` refused the pair — measured end to end: two rows,
--   one address, lengths 15 and 16. Worse, it INVERTED the ordering this whole representation was
--   chosen for (`0x0A` sorts before the PAD SPACE `0x20`), and ONE poisoned row makes
--   `addresses_in` return `Err` for the whole subnet, losing the good rows with the bad one.
--   ⚠️ A trailing newline is how a bulk import, a `LOAD DATA INFILE` or a shell `$(…)` poisons a
--   text column — the ordinary gesture, not an adversary's probe, and the very population this
--   CHECK exists for ("a value can reach here from a backfill that went around the adapter").
--   Closed by `\z`, which is end-of-string (measured: newline → 0, canonical → 1).
--
--   `[0-9]{3}` BOUNDS THE SHAPE AND NOT THE VALUE, so `999.999.999.999` and `256.000.000.000` were
--   storable raw. The Rust side refused them, which is precisely why they were dangerous: a raw row
--   nothing can read back is the `addresses_in` blinding above. Closed by the octet alternation.
--
-- 🔑 One pattern closes both, plus the trailing space and the unpadded form. Measured on
-- mariadb:10.11.11 before it was written: canonical 1 · trailing newline 0 · trailing space 0 ·
-- `999.999.999.999` 0 · `256.000.000.000` 0 · `255.255.255.255` 1 · `000.000.000.000` 1 ·
-- `192.0.2.9` 0.
--
-- ⚠️ THE SAME TRAP ON `policy`, and it needs a DIFFERENT instrument. `policy IN (...)` compares
-- under PAD SPACE too, so `'static '` satisfies it; and `policy = TRIM(policy)` is ALSO a PAD SPACE
-- comparison and is therefore always true. The carrier is a comparison of INTEGERS —
-- `LENGTH(policy) = LENGTH(TRIM(policy))` — which is why it looks redundant and is not.
-- Story 6.5 registered this trap on `entity.kind` and did not close it; here it is closed.
--
-- 🔑 WHAT IS **NOT** HERE, AND WHY. Four refusals live in the adapter (`ipam_repo.rs`) rather than
-- in this file, all for one reason: they compare two TABLES, and a CHECK that references another
-- table is ERROR 1901 on MariaDB 10.11 — measured, not supposed. They are: a range outside its
-- subnet, a range overlapping a sibling, an address outside its subnet, and a base that is not the
-- network address of its own prefix. ⚠️ RAW SQL DOES NOT MEASURE THOSE GUARDS — IT BYPASSES THEM.
-- Story 5.9's M3 ("a guard the adapter cannot violate is measured by raw SQL or by nothing") governs
-- the CHECKs in this file and NOT those four; applying it to them produces a test whose raw INSERT
-- succeeds, which proves nothing.
--
-- D64: MariaDB 10.11+ only; every column that holds letters carries a binary collation.
-- D48: opaque ids are CHAR(36) ascii_bin, minted client-side.
--
-- 🔴 `IF NOT EXISTS` THROUGHOUT, because MySQL DDL IS NOT TRANSACTIONAL. Story 6.5's AC6 first
-- claimed there was "no success = 0 to recover from" and that claim was FALSE — it displaced a
-- remedy that had already been specified. If this migration fails part-way, `_sqlx_migrations` may
-- hold a row with `success = 0`. Recovery, on `0003`'s idiom:
--     DELETE FROM _sqlx_migrations WHERE version = 7 AND success = 0;
-- then restart; the statements below are idempotent and the re-run completes what the first did not.
--
-- ⚠️ A STORE THAT RAN THE **WITHDRAWN** `0007` IS POISONED AND SAYS SO BADLY. The MAC-connector
-- story wrote a different `0007` (a UNIQUE index on `interface_l1_key`), ran it and withdrew it
-- (issue #161). No artefact remains in the repository — verified by `git rev-list --all --objects`.
-- But a developer store that applied it meets `VersionMismatch(7)` here, and the error names no
-- remedy: drop that database rather than debugging it.

CREATE TABLE IF NOT EXISTS ip_subnet (
  id CHAR(36) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  -- The network address of the subnet, canonical form. NOT any address inside it: containment is
  -- arithmetic on this value, and a base that is not the network address makes every answer wrong
  -- in a way nothing downstream can detect. Refused by the adapter (it needs `prefix_len`).
  base VARCHAR(39) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  prefix_len TINYINT UNSIGNED NOT NULL,
  label VARCHAR(120) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  PRIMARY KEY (id),
  -- A plan that holds one subnet twice is not a plan.
  UNIQUE KEY ip_subnet_cidr (base, prefix_len),
  CONSTRAINT ip_subnet_base_canonical
    CHECK (base RLIKE '^(00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])([.](00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])){3}\\z'),
  -- 128 is the only bound this shape admits; binding the prefix to the FAMILY needs the base's
  -- length and lives in the adapter. The validation measured why it is a refusal and not a lint: a
  -- /64 on an IPv4 base makes the natural containment compute 32 - 64 and PANIC on subtract-with-
  -- overflow. A schema that admits an impossible pair hands the arithmetic an impossible input.
  CONSTRAINT ip_subnet_prefix_bound CHECK (prefix_len <= 128)
) ENGINE = InnoDB;

CREATE TABLE IF NOT EXISTS ip_range (
  id CHAR(36) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  subnet_id CHAR(36) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  first_addr VARCHAR(39) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  last_addr VARCHAR(39) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  policy VARCHAR(16) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  label VARCHAR(120) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  PRIMARY KEY (id),
  KEY ip_range_subnet (subnet_id),
  CONSTRAINT ip_range_subnet_fk FOREIGN KEY (subnet_id) REFERENCES ip_subnet (id),
  CONSTRAINT ip_range_bounds_ordered CHECK (last_addr >= first_addr),
  -- ⚠️ **VACUOUS TODAY BY CONSTRUCTION, AND THAT IS WRITTEN RATHER THAN LEFT TO BE DISCOVERED.**
  -- All three review layers reached it: the two canonical CHECKs below admit exactly ONE width, so
  -- every row they accept already satisfies this one, and no mutation can be built that reds it —
  -- *a guard placed where the defect cannot occur reads as coverage and is none*, this epic's
  -- dominant class, committed in the migration written to avoid it. 🔑 Its ONE live effect under
  -- the first pattern was to partially mask the newline hole above, catching a MISMATCHED pair of
  -- poisoned bounds and not a matched one; `\z` removes even that.
  -- It is KEPT rather than deleted because the family rule is real the day FR25 adds a 39-character
  -- alternative to the pattern — and `the_family_check_is_implied_until_a_second_width_exists` pins
  -- the implication, so the day it stops being vacuous, a test says so instead of nobody noticing.
  CONSTRAINT ip_range_same_family CHECK (LENGTH(first_addr) = LENGTH(last_addr)),
  CONSTRAINT ip_range_first_canonical
    CHECK (first_addr RLIKE '^(00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])([.](00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])){3}\\z'),
  CONSTRAINT ip_range_last_canonical
    CHECK (last_addr RLIKE '^(00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])([.](00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])){3}\\z'),
  -- The PLAN axis of the binding vocabulary, ratified 2026-09-11 (PR #166). No story may extend it.
  CONSTRAINT ip_range_policy_domain
    CHECK (policy IN ('static', 'dhcp-pool', 'reserved', 'infrastructure')),
  -- See the header: an INTEGER comparison, because both `IN` and `= TRIM(...)` compare under PAD
  -- SPACE and accept `'static '`.
  CONSTRAINT ip_range_policy_exact CHECK (LENGTH(policy) = LENGTH(TRIM(policy)))
) ENGINE = InnoDB;

CREATE TABLE IF NOT EXISTS ip_address (
  id CHAR(36) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  subnet_id CHAR(36) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  addr VARCHAR(39) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  label VARCHAR(120) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL,
  PRIMARY KEY (id),
  -- One address is defined at most once within a subnet.
  UNIQUE KEY ip_address_in_subnet (subnet_id, addr),
  CONSTRAINT ip_address_subnet_fk FOREIGN KEY (subnet_id) REFERENCES ip_subnet (id),
  CONSTRAINT ip_address_canonical
    CHECK (addr RLIKE '^(00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])([.](00[0-9]|0[0-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])){3}\\z')
) ENGINE = InnoDB;
