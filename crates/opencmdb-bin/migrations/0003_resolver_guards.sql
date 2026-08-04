-- opencmdb — the guards the first WRITER owes (story 5.9b).
--
-- Story 5.9 created the tables; nothing wrote a link the engine had derived, so three guards were
-- registered rather than installed, each with story 5.9b named as owner. This is that story: the
-- resolver is the first production code that writes links, from observations that exist, with rules
-- the engine chose. Each guard below is the DDL echo of a property the writer already holds — a
-- second line of defence, never the only one.
--
-- These are ALTER statements in a new file rather than edits to 0002, deliberately: sqlx checksums
-- every migration it has applied, so changing one that has already run makes every existing
-- database refuse to migrate.
--
-- 🔴 PRECONDITION, and it bites on a DEVELOPER's database rather than in production.
-- The foreign key below VALIDATES existing rows. Story 5.9's test suite minted observation ids
-- freely and never inserted the observations, so any database that ran it holds identity_link rows
-- whose observation_id matches nothing, and this migration fails ERROR 1452.
--
-- 🔴 AND THE FAILURE STICKS. _sqlx_migrations then records version 3 with success = 0, and MySQL
-- DDL is not transactional, so deleting the offending rows does NOT recover: every subsequent run
-- fails Dirty(3). Recovery is manual:
--
--     DELETE FROM link_candidate; DELETE FROM identity_link;
--     DELETE FROM _sqlx_migrations WHERE version = 3;
--     -- then check which of the three ALTERs below already went through, and drop those
--     -- constraints before re-running.
--
-- Measured end to end at this story's code review. Production is unaffected: 0002 is unreleased and
-- no deployment has ever written a link. The simplest developer fix is to drop and recreate the
-- test database before the first run that carries this file.

-- A link names an observation that exists.
--
-- Every other cross-table reference in 0002 carries a foreign key; this one did not, and a link
-- whose observation had never been inserted was measured inserting Ok(()). That is precisely the
-- silent corruption story 5.10's bit-for-bit replay would report as a mismatch with no cause.
-- No index is created: identity_link_one_current already begins with observation_id and InnoDB
-- uses it.
--
-- ⚠️ RESTRICT, not CASCADE, and the asymmetry with link_candidate is deliberate. A candidate has no
-- meaning without its link, so 0002 cascades there; an observation is an INPUT, immutable and
-- linked-never-merged (D19/FR11), and deleting one out from under a link would erase the evidence
-- the link points at.
ALTER TABLE identity_link
  ADD CONSTRAINT identity_link_observation_fk FOREIGN KEY (observation_id)
  REFERENCES observation_record (id);

-- The canonical MAC is the lowercase colon form, and nothing but a comment said so.
--
-- MacAddr's Display renders it that way, so the writer cannot emit anything else — but the L1 index
-- is deliberately non-unique (D21: a cloned MAC is two real interfaces), so a second writer using
-- uppercase would create a second interface row for one physical NIC, invisibly, and every link
-- after it would point at the wrong one.
ALTER TABLE interface
  ADD CONSTRAINT interface_mac_canon_lower CHECK (mac_canon = LOWER(mac_canon));

-- A decision names the RULE that settled it — and an empty name is not a name.
--
-- identity_link_rule_xor_cause only tests IS NOT NULL, so '' satisfied it: a link could claim a
-- rule settled it while naming none. D19 wants the rule id left behind because a rule that fires
-- without one is undebuggable in production, and '' is that same undebuggability spelled
-- differently.
ALTER TABLE identity_link
  ADD CONSTRAINT identity_link_rule_id_not_empty CHECK (rule_id IS NULL OR rule_id <> '');
