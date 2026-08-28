-- opencmdb — the entity supertype, the device, and the lifecycle state (story 6.5).
--
-- D21: the interface/device disjunction is enforced BY THE ENGINE, not by convention. A
-- polymorphic `(entity_type, entity_id)` admits no foreign key on any engine, so it guarantees
-- orphans — found in production, never in test. The shape is a supertype `entity(id, kind)` with a
-- composite key the subtype's foreign key can point at, plus a constant `kind` on the subtype.
--
-- 🔴 THE COMPOSITE KEY DOES NOT CARRY THE DISJUNCTION. `PRIMARY KEY (id)` does, alone. Measured on
-- two parents side by side: with `PRIMARY KEY (id)` + `UNIQUE (id, kind)` a second row for one id
-- under another kind is refused (ERROR 1062); with `PRIMARY KEY (id, kind)` — the literal reading
-- of "a composite unique key on the parent" — the same insert SUCCEEDS and one id lives under both
-- kinds at once. The UNIQUE below exists ONLY to give the subtype's foreign key a parent index.
-- Do not "simplify" the two keys into one.
--
-- D64: MariaDB 10.11+ only; every column that holds letters carries a binary collation.
-- D48: opaque ids are CHAR(36) ascii_bin, minted client-side.
--
-- ⚠️ NO `interface` ADOPTION HERE, by Guy's arbitration of 2026-08-28 (option (a)). `interface`
-- predates this supertype and stays outside it; its adoption is story 6.12's, together with the
-- migration widening `identity_link` to admit a device as a placement subject and the resolver
-- change that mints an entity row per interface. Those three are one gesture: a backfill repairs
-- the rows that exist and nothing repairs the rows the next scan writes — measured, 65 tests red.
-- The cost accepted here, in writing: until 6.12 this supertype constrains nothing about
-- interfaces. `device` is born correct; `interface` is ADOPTED when something needs it.
--
-- ⚠️ AND THERE IS NO PRODUCER. Nothing in this codebase writes an entity or a device outside its
-- own tests. `epics.md`'s criterion says so and it is met literally.

-- 🔴 `IF NOT EXISTS` IS AC6'S REMEDY, AND IT WAS SPECIFIED BEFORE IT WAS SKIPPED. The first draft
-- argued that this migration cannot fail — it creates two tables and alters nothing, so there is no
-- row to trip on. The narrow claim is true; its generalisation was not, and the code review
-- reproduced the counterexample: MySQL DDL is not transactional, so a process killed BETWEEN the two
-- statements leaves `entity` committed, `_sqlx_migrations` at version 6 with `success = 0`, and the
-- next boot refusing — *"migration 6 is partially applied; fix and remove row from
-- `_sqlx_migrations`"* — where repairing the DATA does not clear it. On a published v0.2.0 that is
-- a deployment down until someone has SQL access.
--
-- ⚠️ RECOVERY, if this migration is ever found half-applied anyway (0003's header idiom): drop the
-- tables it created, in child-then-parent order, then delete the version row —
--   DROP TABLE IF EXISTS device; DROP TABLE IF EXISTS entity;
--   DELETE FROM _sqlx_migrations WHERE version = 6;
-- and restart. With `IF NOT EXISTS` a re-run heals itself and this recipe is a residual, not a
-- procedure.

-- The supertype. One id, one kind, for the life of that id — ⚠️ against a second INSERT, which is
-- what `PRIMARY KEY (id)` refuses, and (where a child exists) against the foreign key. A plain
-- `UPDATE entity SET kind = …` on a CHILDLESS row succeeds, measured at the code review. No gate
-- covers it and none is added here: the rule wanted is *this COLUMN is immutable*, which is the
-- matcher class `entity_id_immutable`'s own doc declines twice, and D15's migration mechanism is
-- the story that will legitimately write here. Registered to story 6.12.
--
-- ⚠️ For a row the ADAPTER wrote. `ascii_bin` is a PAD SPACE collation, so `'device '` satisfies
-- `kind IN (…)`, `kind = 'device'` and the composite foreign key while `VARCHAR` keeps the padding
-- — and `load_entity` then reads that row back as an unfamiliar token. Unreachable through the
-- adapter, which binds `EntityKind::as_str()`; reachable by a raw write, a backfill or `LOAD DATA`.
-- Stated rather than guarded: a `CHECK` cannot express "no trailing space" without restating the
-- domain a second time, and a second representation is what this story spent a mutation learning
-- to distrust.
CREATE TABLE IF NOT EXISTS entity (
  id    CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  kind  VARCHAR(16) CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  state VARCHAR(20) CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL DEFAULT 'active',
  -- The disjunction. See the header: this is the key that enforces it.
  PRIMARY KEY (id),
  -- Not redundant with the PK: MariaDB needs a parent INDEX on the exact columns a composite
  -- foreign key names, and `device_entity_fk` names (id, kind).
  UNIQUE KEY entity_id_kind (id, kind),
  -- `interface` is admitted now although nothing writes it until story 6.12. Widening this domain
  -- later would be an ALTER that runs at boot on a published product, which is the hazard 0003's
  -- header documents; posing the model's kinds once costs nothing today.
  CONSTRAINT entity_kind_domain CHECK (kind IN ('interface', 'device')),
  -- 🔴 SIX values, from architecture.md:1502 — NOT the two `epics.md:1826` names. A divergence,
  -- registered rather than taken silently. Shipping two would buy an ALTER at boot the day a
  -- lifecycle story needs `superseded`; the domain is posed once, here.
  -- ⚠️ `dormant` is valid only for an interface whose address is locally administered
  -- (architecture.md:1503). That scope needs a `mac_kind` column, which no table carries, and a
  -- cross-table CHECK is not portable — so it is an APPLICATION invariant owned by story 6.18,
  -- which carries FR38b's transition. This column is the domain and no behaviour: nothing in this
  -- codebase sets any value but the default.
  CONSTRAINT entity_state_domain CHECK (state IN (
    'active', 'dormant', 'superseded', 'quarantined', 'pending_migration', 'sentinel'
  )),
  -- The nil UUID is D21's sentinel value; a real entity carrying it would collide with whatever
  -- uses NIL_DEVICE the day something places on a device. Same guard as interface_id_not_nil.
  CONSTRAINT entity_id_not_nil CHECK (id <> '00000000-0000-0000-0000-000000000000')
) ENGINE = InnoDB;

-- One device. It has NO business columns, and that is a decision, not an omission.
--
-- architecture.md:1480: "Everything a device 'is' is either observed (via its interfaces) or
-- declared. A device is an identifier and nothing else. If anyone proposes adding `hostname` to it,
-- they have just restored the OBSERVED/DECLARED merge we forbade."
--
-- `kind` is here only to give the composite foreign key its second column; it is constant, it is
-- written by the adapter as a literal, and the CHECK below is what stops it drifting.
CREATE TABLE IF NOT EXISTS device (
  id   CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  kind VARCHAR(16) CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL DEFAULT 'device',
  PRIMARY KEY (id),
  CONSTRAINT device_kind_constant CHECK (kind = 'device'),
  CONSTRAINT device_id_not_nil CHECK (id <> '00000000-0000-0000-0000-000000000000'),
  CONSTRAINT device_entity_fk FOREIGN KEY (id, kind) REFERENCES entity (id, kind)
) ENGINE = InnoDB;
