-- Story 6.2: the documenting gesture writes an ADOPTED declared value, and one observation's
-- field may be adopted only ONCE — documenting twice would count one box twice, the gesture's
-- own name refusing. The friendly 409 comes from a check the application cannot hold under a
-- race (two concurrent document-all of one subject both pass a pre-read and mint two entities),
-- so the invariant is a database guard, on story 5.9's precedent (the guard above the DDL is for
-- the message; the DDL is for the invariant).
--
-- `origin_obs_id` is NULL on every 'manual' and 'imported' row, and MariaDB holds NULLs DISTINCT
-- in a unique index (D21) — so 'manual'/'imported' rows never collide here; only two 'adopted'
-- rows for the SAME observation's SAME field collide. The loser gets ERROR 1062 naming this
-- index, which the adapter's caller maps to AlreadyDocumented (409).
--
-- ⚠️ This index does NOT pre-block Epic 7's `document-field`: re-documenting a DRIFTED field from
-- a NEWER observation carries a DIFFERENT origin_obs_id, so it does not collide HERE. The wall
-- Epic 7 must negotiate is the PRIMARY KEY (entity_id, attr_key) — re-documenting one entity's
-- same field reds ERROR 1062 on PRIMARY — with an ON DUPLICATE KEY UPDATE or a supersede, not by
-- widening this index.
CREATE UNIQUE INDEX declared_one_adoption_per_field
    ON declared_attribute (origin_obs_id, attr_key);
