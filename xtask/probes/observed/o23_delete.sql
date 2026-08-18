-- The stated exclusion: a bulk delete is data loss, a different invariant.
DELETE FROM observation_record WHERE id = 'x';
