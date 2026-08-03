-- opencmdb — interfaces, identity links and their candidates (story 5.9).
--
-- D14: the link is an ENTITY, not a foreign key. It is SCD2 — superseding one appends a row and
-- moves the old row's valid_to off the sentinel — and it carries the rule applied, the evidence,
-- when, by whom and the ruleset_version. A bad link is UNLINKED, never erased. An ambiguous
-- outcome is a LINK with its candidates, never an absence: the ambiguity is DATA, not a hole,
-- otherwise there is nothing to display and FR16 is vapour.
-- D21: NO unique index on the L1 key — a cloned MAC is two real interfaces sharing one address,
-- and a UNIQUE there would turn the exact case the engine must ABSTAIN on into a 500. And NO
-- NULL inside a uniqueness key WHERE ONE WOULD BE ACCIDENTAL: MariaDB holds NULLs distinct, so
-- an accidental NULL makes a constraint decorative. D21 names both sentinels that follow from
-- that: OPEN_END, and NIL_INTERFACE/NIL_DEVICE [architecture.md:1462-1468].
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
  INDEX interface_l1_key (l2_domain, mac_canon),
  -- The nil UUID is D21's NIL_INTERFACE, which identity_link.current_subject uses to mean "placed
  -- on no interface". A real interface carrying it would occupy an abstention's slot in the
  -- uniqueness key below and make a legitimate abstention unwritable — measured, ERROR 1062.
  CONSTRAINT interface_id_not_nil CHECK (id <> '00000000-0000-0000-0000-000000000000'),
  CONSTRAINT interface_seen_window CHECK (first_seen_at <= last_seen_at)
) ENGINE = InnoDB;

-- One placement of an observation on an interface, versioned.
--
-- current_subject is what carries "exactly one current link per (observation, interface)". While
-- the row is current it holds interface_id, or D21's NIL_INTERFACE when the link places the
-- observation nowhere (an abstention); once the row is superseded it becomes NULL and leaves the
-- uniqueness key. Both halves are deliberate:
--
--   · the nil sentinel keeps an ACCIDENTAL NULL out of the key — without it two current
--     abstentions for one observation would both insert, since MariaDB holds NULLs distinct, and
--     the constraint would be decorative for exactly the half FR16 exists to display;
--   · the NULL on a closed row is that same NULL-distinctness used DELIBERATELY, so the key
--     constrains only what is current. valid_to must NOT appear in the key: it is NOT NULL on
--     closed rows too, so a key containing it constrains HISTORY — and since every instant here is
--     data-derived and never the clock, a replay reproduces instants, so two versions of one
--     placement closed at the same instant collided (measured, ERROR 1062) and the second close
--     was refused while the link silently stayed current. That is story 5.10's purge-and-replay
--     and story 5.11's "no new version for an unchanged decision", i.e. the normal path.
--
-- It is a written column, not a generated one: MariaDB 10.11 refuses to INDEX a generated column
-- whose expression coalesces to a string literal (error 1901 — the literal's charset is
-- session-dependent, so the expression is not indexable). identity_link_current_subject is what
-- makes it unable to drift from what it stands for.
CREATE TABLE identity_link (
  id               CHAR(36)     CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  observation_id   CHAR(36)     CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  interface_id     CHAR(36)     CHARACTER SET ascii   COLLATE ascii_bin,             -- NULL iff abstained
  current_subject  CHAR(36)     CHARACTER SET ascii   COLLATE ascii_bin,             -- NULL once superseded
  outcome          VARCHAR(16)  CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  rule_id          VARCHAR(64)  CHARACTER SET ascii   COLLATE ascii_bin,             -- NULL iff abstained
  abstention_cause VARCHAR(32)  CHARACTER SET ascii   COLLATE ascii_bin,             -- set iff abstained
  evidence         LONGTEXT     CHARACTER SET utf8mb4 COLLATE utf8mb4_bin  NOT NULL, -- serialized Vec<ObsId>
  ruleset_version  INT UNSIGNED NOT NULL,
  decided_by       VARCHAR(16)  CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  valid_from       DATETIME(6)  NOT NULL,                                            -- a parameter, never NOW()
  valid_to         DATETIME(6)  NOT NULL,                                            -- OPEN_END while current
  PRIMARY KEY (id),
  -- Exactly one CURRENT link per (observation, subject). NOT (observation_id, valid_to): the join
  -- puts one observation on every key it carries, so a multi-MAC observation legitimately holds
  -- one current link per interface, and that narrower key was measured refusing the write.
  UNIQUE KEY identity_link_one_current (observation_id, current_subject),
  INDEX identity_link_interface (interface_id),
  CONSTRAINT identity_link_interface_fk FOREIGN KEY (interface_id) REFERENCES interface (id),
  CONSTRAINT identity_link_outcome CHECK (outcome IN ('match', 'no_match', 'abstained')),
  CONSTRAINT identity_link_decided_by CHECK (decided_by IN ('ENGINE', 'OPERATOR')),
  -- A version covers a half-open interval, so it can never be zero-length or inverted.
  CONSTRAINT identity_link_interval CHECK (valid_from < valid_to),
  -- Decision::rule() returns None exactly for an abstention, expressed in DDL.
  CONSTRAINT identity_link_rule_xor_cause CHECK (
    (outcome = 'abstained' AND rule_id IS NULL AND abstention_cause IS NOT NULL)
    OR (outcome <> 'abstained' AND rule_id IS NOT NULL AND abstention_cause IS NULL)
  ),
  CONSTRAINT identity_link_abstained_has_no_interface CHECK (
    (outcome = 'abstained' AND interface_id IS NULL)
    OR (outcome <> 'abstained' AND interface_id IS NOT NULL)
  ),
  -- current_subject cannot drift from what it stands for, and cannot disagree with valid_to about
  -- whether this row is current. One constraint, because they are one property.
  CONSTRAINT identity_link_current_subject CHECK (
    (valid_to = '9999-12-31 23:59:59.999999'
     AND ((interface_id IS NULL AND current_subject = '00000000-0000-0000-0000-000000000000')
          OR (interface_id IS NOT NULL AND current_subject = interface_id)))
    OR (valid_to <> '9999-12-31 23:59:59.999999' AND current_subject IS NULL)
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
  -- CASCADE, because a candidate has no meaning without its link and story 5.10's purge deletes
  -- engine links wholesale — measured: with RESTRICT that DELETE fails ERROR 1451 the moment any
  -- engine link carries a candidate, which is exactly the ambiguity case this table exists for.
  CONSTRAINT link_candidate_link_fk FOREIGN KEY (link_id) REFERENCES identity_link (id)
    ON DELETE CASCADE,
  -- RESTRICT here, deliberately: interfaces are never purged, so a candidate must not outlive one.
  CONSTRAINT link_candidate_interface_fk FOREIGN KEY (interface_id) REFERENCES interface (id)
) ENGINE = InnoDB;
