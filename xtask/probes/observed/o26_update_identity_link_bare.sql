-- ⚠️ VACUITY MARKER, and labelled as one: this file does not contain the guarded table name at
-- all, so a table-anchored matcher never enters its loop. It is GREEN under any implementation,
-- a deliberately broken one included, and must NOT be counted as evidence that the engine's
-- supersede survives. o21 is the probe that really measures that.
UPDATE identity_link SET valid_to = NOW(6) WHERE id = 'x';
