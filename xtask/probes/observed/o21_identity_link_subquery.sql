-- LOAD-BEARING NEGATIVE: the engine's own supersede, with the guarded table named in a
-- subquery. Green because `select` governs that reference, not the UPDATE.
UPDATE identity_link
   SET valid_to = NOW(6), current_subject = NULL
 WHERE observation_id IN (SELECT id FROM observation_record);
