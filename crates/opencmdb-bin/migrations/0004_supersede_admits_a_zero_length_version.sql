-- opencmdb — a superseded version may be zero-length (story 5.11).
--
-- Story 5.11 is the first that SUPERSEDES. 0002 wrote identity_link_interval as a strict
-- `valid_from < valid_to` with the comment "a version covers a half-open interval, so it can never
-- be zero-length or inverted", and that comment was written before anything superseded. It is now
-- measured refusing a legitimate write.
--
-- The mechanism. Both versions of one placement are versions of ONE observation's placement, and
-- valid_from is that observation's own observed_at. So when the caller supplies each observation
-- with a stable instant — the ordinary case, and the only one the engine controls — the two
-- versions carry the SAME valid_from, and closing the old one at the new one's valid_from means
-- closing it at its own:
--
--     UPDATE identity_link SET valid_to = <its own valid_from>, current_subject = NULL
--       -> ERROR 4025 (23000): CONSTRAINT `identity_link_interval` failed
--
-- The engine may not read the clock (architecture.md:3364), and story 5.10's purge-and-replay test
-- is what HOLDS that: a clock-derived instant would make the replay produce a different one and red
-- the comparison. So the close instant must come from the data, and no data-derived instant is
-- strictly greater than the old version's valid_from.
--
-- The reading, and it is the point: the first belief never held over any interval the data can
-- distinguish. The engine dates a link by the OBSERVATION, not by when it came to believe it, so
-- an engine link's history is ordered by insertion rather than by time. Pretending otherwise would
-- take either a clock (forbidden) or an invented microsecond (a duration that never happened).
--
-- ⚠️ That stability is a CALLER'S DISCIPLINE, not a structural property, and this comment may not
-- overstate it. valid_from comes from the IN-MEMORY Observation; 0003's foreign key checks only
-- that observation_id EXISTS; and nothing in the workspace reads observation_record.observed_at
-- back as a value. Hand the pass the same obs_id with a LATER observed_at and a non-zero-length
-- supersede is produced — which this constraint also admits, and which is why the relaxation is
-- `<=` rather than `=`.
--
-- What stays refused, both measured on this exact form:
--   · an INVERTED closed interval (valid_to < valid_from) — still ERROR 4025. An observed_at moving
--     BACKWARDS reaches it through the pass, as Constraint("check") and a full rollback, so the
--     constraint is load-bearing at both ends rather than only at the one this file relaxes;
--   · a CURRENT row that would be zero-length (valid_from at the sentinel) — still ERROR 4025.
--
-- An ALTER in a new file rather than an edit to 0002, for 0003's reason: sqlx checksums every
-- migration it has applied, so changing one that has already run makes every existing database
-- refuse to migrate with VersionMismatch.
ALTER TABLE identity_link DROP CONSTRAINT identity_link_interval;

ALTER TABLE identity_link
  ADD CONSTRAINT identity_link_interval CHECK (
    (valid_to =  '9999-12-31 23:59:59.999999' AND valid_from <  valid_to)
    OR (valid_to <> '9999-12-31 23:59:59.999999' AND valid_from <= valid_to)
  );
