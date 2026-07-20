-- opencmdb — initial schema (walking skeleton).
--
-- Two tables: the declared side (attributes-per-row, field-level provenance, D3) and the
-- observed side (immutable, D19). Linked-never-merged: `origin_obs_id` points at the exact
-- adopted observation; the observation itself never moves. Opaque ids are CHAR(36) ascii_bin
-- (D48); EVERY text column carries a binary collation so identity comparison is byte-exact
-- and never depends on the DB locale (D64). MariaDB 10.11+ only.

-- The declared record, one row per (entity, field), so each field carries its own provenance.
CREATE TABLE declared_attribute (
  entity_id     CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  attr_key      VARCHAR(64) CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  attr_value    TEXT        CHARACTER SET utf8mb4 COLLATE utf8mb4_bin,
  origin        VARCHAR(16) CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL, -- manual|adopted|imported
  origin_obs_id CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin,             -- the adopted observation
  actor_id      CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL, -- a human; never 'scanner'
  updated_at    DATETIME(6) NOT NULL,
  PRIMARY KEY (entity_id, attr_key),
  CONSTRAINT declared_adopted_has_obs CHECK (origin <> 'adopted' OR origin_obs_id IS NOT NULL),
  CONSTRAINT declared_actor_not_scanner CHECK (actor_id <> 'scanner')
) ENGINE = InnoDB;

-- The observed record, immutable. Its facts are stored serialized; the engine reads them into
-- Rust and compares there — SQL never descends into a value comparison (D10).
CREATE TABLE observation_record (
  id           CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  connector_id CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  observed_at  DATETIME(6) NOT NULL,                                            -- dated by the source
  l2_domain    CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL, -- Scope (D19)
  vantage      CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  facts        LONGTEXT    CHARACTER SET utf8mb4 COLLATE utf8mb4_bin  NOT NULL, -- serialized Vec<Fact>
  raw          LONGTEXT    CHARACTER SET utf8mb4 COLLATE utf8mb4_bin,           -- opaque provenance
  PRIMARY KEY (id)
) ENGINE = InnoDB;
