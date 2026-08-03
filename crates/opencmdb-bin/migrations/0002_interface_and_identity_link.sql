-- opencmdb — interfaces, identity links and their candidates (story 5.9).
--
-- D14: the link is an ENTITY, not a foreign key. It is SCD2 — superseding one appends a row and
-- moves the old row's valid_to off the sentinel — and it carries the rule applied, the evidence,
-- when, by whom and the ruleset_version. A bad link is UNLINKED, never erased. An ambiguous
-- outcome is a LINK with its candidates, never an absence: the ambiguity is DATA, not a hole,
-- otherwise there is nothing to display and FR16 is vapour.
-- D21: NO unique index on the L1 key — a cloned MAC is two real interfaces sharing one address,
-- and a UNIQUE there would turn the exact case the engine must ABSTAIN on into a 500. And NO
-- NULL inside a uniqueness key: MariaDB holds NULLs distinct, so a NULL makes the constraint
-- decorative. Two sentinels close that trap here, and they are one idiom used twice — OPEN_END
-- on valid_to, and the nil UUID on link_subject.
-- D48: opaque ids are CHAR(36) ascii_bin, minted client-side.
-- D64: MariaDB 10.11+ only; every column that holds letters carries a binary collation.

-- One interface. At L1 an interface IS the scope-qualified key: the join groups observations by
-- (l2_domain, mac) and each group is one interface. Rows here are NOT purged by story 5.10's
-- re-run and their ids are stable — the re-run finds an interface by its key, and if the id were
-- re-minted every reproduced link would carry a different interface_id.
CREATE TABLE interface (
  id            CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  l2_domain     CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  mac_canon     VARCHAR(17) CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL, -- lowercase, colons
  first_seen_at DATETIME(6) NOT NULL,                                            -- from observations
  last_seen_at  DATETIME(6) NOT NULL,                                            -- never from the clock
  PRIMARY KEY (id),
  -- NOT UNIQUE, deliberately (D21). A cloned MAC is two real interfaces sharing one address; a
  -- UNIQUE here would turn the exact case the engine must ABSTAIN on into a 500, which is the
  -- corpus's cloned-mac family. This is that family's guard, in the schema.
  INDEX interface_l1_key (l2_domain, mac_canon)
) ENGINE = InnoDB;

-- One placement of an observation on an interface, versioned.
--
-- link_subject is interface_id with its NULL sentinelled to the nil UUID, so the uniqueness key
-- below never contains a NULL. It is a written column rather than a generated one because
-- MariaDB 10.11 refuses to INDEX a generated column whose expression coalesces to a string
-- literal (error 1901 — the literal's charset is session-dependent, so the expression is not
-- indexable). identity_link_subject_matches is what makes it unable to drift.
CREATE TABLE identity_link (
  id               CHAR(36)     CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  observation_id   CHAR(36)     CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  interface_id     CHAR(36)     CHARACTER SET ascii   COLLATE ascii_bin,             -- NULL iff abstained
  link_subject     CHAR(36)     CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL, -- interface_id, or nil
  outcome          VARCHAR(16)  CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  rule_id          VARCHAR(64)  CHARACTER SET ascii   COLLATE ascii_bin,             -- NULL iff abstained
  abstention_cause VARCHAR(32)  CHARACTER SET ascii   COLLATE ascii_bin,             -- set iff abstained
  evidence         LONGTEXT     CHARACTER SET utf8mb4 COLLATE utf8mb4_bin  NOT NULL, -- serialized Vec<ObsId>
  ruleset_version  INT UNSIGNED NOT NULL,
  decided_by       VARCHAR(16)  CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  valid_from       DATETIME(6)  NOT NULL,                                            -- a parameter, never NOW()
  valid_to         DATETIME(6)  NOT NULL,                                            -- OPEN_END while current
  PRIMARY KEY (id),
  -- Exactly one current link per (observation, subject). NOT (observation_id, valid_to): the join
  -- puts one observation on every key it carries, so a multi-MAC observation legitimately holds
  -- one current link per interface, and the narrower key was measured refusing that write.
  UNIQUE KEY identity_link_one_current (observation_id, link_subject, valid_to),
  INDEX identity_link_interface (interface_id),
  CONSTRAINT identity_link_interface_fk FOREIGN KEY (interface_id) REFERENCES interface (id),
  CONSTRAINT identity_link_outcome CHECK (outcome IN ('match', 'no_match', 'abstained')),
  CONSTRAINT identity_link_decided_by CHECK (decided_by IN ('ENGINE', 'OPERATOR')),
  -- Decision::rule() returns None exactly for an abstention, expressed in DDL.
  CONSTRAINT identity_link_rule_xor_cause CHECK (
    (outcome = 'abstained' AND rule_id IS NULL AND abstention_cause IS NOT NULL)
    OR (outcome <> 'abstained' AND rule_id IS NOT NULL AND abstention_cause IS NULL)
  ),
  CONSTRAINT identity_link_abstained_has_no_interface CHECK (
    (outcome = 'abstained' AND interface_id IS NULL)
    OR (outcome <> 'abstained' AND interface_id IS NOT NULL)
  ),
  -- The sentinel cannot drift from what it stands for.
  CONSTRAINT identity_link_subject_matches CHECK (
    (interface_id IS NULL AND link_subject = '00000000-0000-0000-0000-000000000000')
    OR (interface_id IS NOT NULL AND link_subject = interface_id)
  )
) ENGINE = InnoDB;

-- The candidate interfaces of an observation the engine abstained on, each with its evidence.
-- This is what FR16 renders: "present the candidate matches with their evidence". The link row
-- always exists; these hang off it.
CREATE TABLE link_candidate (
  link_id      CHAR(36) CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  interface_id CHAR(36) CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  evidence     LONGTEXT CHARACTER SET utf8mb4 COLLATE utf8mb4_bin  NOT NULL,
  PRIMARY KEY (link_id, interface_id),
  CONSTRAINT link_candidate_link_fk FOREIGN KEY (link_id) REFERENCES identity_link (id),
  CONSTRAINT link_candidate_interface_fk FOREIGN KEY (interface_id) REFERENCES interface (id)
) ENGINE = InnoDB;
