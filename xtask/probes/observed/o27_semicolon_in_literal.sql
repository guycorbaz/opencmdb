-- The overwrite hidden behind a `;` INSIDE a single-quoted literal. `raw` is an opaque blob by
-- design, so a payload carrying a semicolon is ordinary data, not an attack.
INSERT INTO observation_record (id, raw) VALUES ('x', 'a;payload')
  ON DUPLICATE KEY UPDATE raw = VALUES(raw);
