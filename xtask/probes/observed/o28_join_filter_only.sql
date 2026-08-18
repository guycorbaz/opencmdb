-- ⚠️ A KNOWN FALSE POSITIVE, pinned to its ACTUAL behaviour rather than to the one we would
-- prefer. The SET touches `identity_link` alone; `observation_record` is only a filter. The gate
-- reds anyway, because the verdict follows the nearest governing verb and never the SET clause.
-- Naming it here is what stops a future contributor from "fixing" it with an allowlist entry.
UPDATE identity_link il
  JOIN observation_record o ON o.id = il.observation_id
   SET il.valid_to = NOW(6)
 WHERE o.raw IS NOT NULL;
