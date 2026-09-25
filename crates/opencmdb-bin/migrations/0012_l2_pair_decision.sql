-- opencmdb — the L2 decision about a PAIR of interfaces (story 6.12).
--
-- L1 asks whether two observations are one INTERFACE and writes one `identity_link` per observation.
-- L2 asks whether two interfaces are one DEVICE, and its decision is about a PAIR. This table is the
-- L2 sibling of `identity_link` — same SCD2 shape, same `decided_by`, same `ruleset_version` — and it
-- is a SIBLING rather than a widening by Guy's arbitration of 2026-09-24 (story 6.12, §0.6 B):
-- `identity_link.observation_id` is NOT NULL with a foreign key, its uniqueness key is per
-- observation, and `count_engine_reach` counts its rows by observation with no level column, so an L2
-- row there is either unwritable or silently counted on four screens.
--
-- 🔴 KEYED ON THE PAIR, not on one interface (§0.6 E). The validation built the per-interface shape
-- and measured it rolling a whole sweep back: an abstention or an L2 `NoMatch` has no device to name,
-- so every such row landed in one nil slot, and three NICs sharing a name — or one VRRP address on the
-- network — collided on the uniqueness key. The pair IS the candidate set of an ambiguity, so no
-- separate candidate table exists (`link_candidate` references `identity_link` and could not hold one).
--
-- ⚠️ WHAT IS PERSISTED (§0.6 G): `Ambiguous` and `NoMatch`, never `AbsenceOfProof` — a 46-interface
-- sweep judges ~1000 pairs about which nothing can be said, and writing them would break the
-- transaction cap (architecture.md:1498) to record silence. And never `Match`: no L2 rule emits
-- `Decisive`, and a `Match` would need a device this story does not mint. Both refusals are in DDL
-- below as well as in the adapter, so a row written around the adapter meets them.
--
-- ⚠️ NO EVIDENCE COLUMN OF OBSERVATION IDS (§0.6 F). The shipped connector mints a fresh `ObsId` per
-- sighting, so evidence keyed on observations changes at every sweep and an unchanged network would
-- close and re-append every current row every five minutes. `verdicts` carries the rule/verdict
-- vector instead — stable while the network is — and the pair's interfaces are the evidence's subject.
--
-- 🔴 BORN WITHOUT `identity_link`'s UNKNOWN DEFECT (deferred-work.md, "current_subject IS NOT NULL is
-- NOT equivalent"). There a NULL `current_subject` makes the coupling CHECK evaluate to UNKNOWN, which
-- a CHECK accepts. Here the coupling compares two expressions that are NEVER NULL — `valid_to` is
-- NOT NULL and `IS NOT NULL` is never NULL — so it evaluates to TRUE or FALSE and nothing between.
--
-- D64: every column holding letters carries a binary collation. D48: ids are CHAR(36) ascii_bin.
--
-- ⚠️ RECOVERY, if ever found half-applied (0006's idiom): `DROP TABLE IF EXISTS l2_pair_decision;
-- DELETE FROM _sqlx_migrations WHERE version = 12;` and restart. One statement, `IF NOT EXISTS`, so a
-- re-run heals itself and this is a residual rather than a procedure.
CREATE TABLE IF NOT EXISTS l2_pair_decision (
  id               CHAR(36)     CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  -- The pair, ordered, so one unordered pair has one spelling.
  interface_low    CHAR(36)     CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  interface_high   CHAR(36)     CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  outcome          VARCHAR(16)  CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  rule_id          VARCHAR(64)  CHARACTER SET ascii COLLATE ascii_bin,           -- NULL iff abstained
  abstention_cause VARCHAR(32)  CHARACTER SET ascii COLLATE ascii_bin,           -- set iff abstained
  -- The verdict vector, `rule=verdict` joined by `;`, in the order `l2::decide_pair` builds it.
  verdicts         VARCHAR(512) CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  ruleset_version  INT UNSIGNED NOT NULL,
  decided_by       VARCHAR(16)  CHARACTER SET ascii COLLATE ascii_bin NOT NULL,
  valid_from       DATETIME(6)  NOT NULL,                                          -- derived, never NOW()
  valid_to         DATETIME(6)  NOT NULL,                                          -- OPEN_END while current
  -- 1 while current, NULL once closed. NULLs are distinct in a UNIQUE (story 5.9's trap, used
  -- DELIBERATELY here as `current_subject` uses it): closed versions drop out of the key.
  is_current       TINYINT UNSIGNED,
  PRIMARY KEY (id),
  UNIQUE KEY l2_pair_decision_one_current (interface_low, interface_high, is_current),
  INDEX l2_pair_decision_high (interface_high),
  -- CASCADE, and the choice is stated: a decision about a pair means nothing once an interface is
  -- gone, production never deletes an interface, and RESTRICT would make every one of the twenty
  -- `DELETE FROM interface` test cleanups fail ERROR 1451 the moment an L2 row exists (measured at
  -- the validation). `link_candidate` chose RESTRICT for its interface FK; this table has no row whose
  -- meaning survives its interface, which is the difference.
  CONSTRAINT l2_pair_decision_low_fk FOREIGN KEY (interface_low) REFERENCES interface (id)
    ON DELETE CASCADE,
  CONSTRAINT l2_pair_decision_high_fk FOREIGN KEY (interface_high) REFERENCES interface (id)
    ON DELETE CASCADE,
  CONSTRAINT l2_pair_decision_ordered CHECK (interface_low < interface_high),
  -- `match` is refused: no L2 rule is `Decisive`, and a match would need a device (§0.6).
  CONSTRAINT l2_pair_decision_outcome CHECK (outcome IN ('no_match', 'abstained')),
  -- `absence_of_proof` is refused: it is never persisted (§0.6 G).
  CONSTRAINT l2_pair_decision_rule_xor_cause CHECK (
    (outcome = 'abstained' AND rule_id IS NULL AND abstention_cause = 'ambiguous')
    OR (outcome = 'no_match' AND rule_id IS NOT NULL AND rule_id <> '' AND abstention_cause IS NULL)
  ),
  CONSTRAINT l2_pair_decision_verdicts_not_empty CHECK (verdicts <> ''),
  CONSTRAINT l2_pair_decision_decided_by CHECK (decided_by IN ('ENGINE', 'OPERATOR')),
  CONSTRAINT l2_pair_decision_interval CHECK (
    (valid_to =  '9999-12-31 23:59:59.999999' AND valid_from <  valid_to)
    OR (valid_to <> '9999-12-31 23:59:59.999999' AND valid_from <= valid_to)
  ),
  -- The coupling, with no NULL on either side of the `=` — see the header.
  CONSTRAINT l2_pair_decision_current CHECK (
    (valid_to = '9999-12-31 23:59:59.999999') = (is_current IS NOT NULL)
  ),
  -- `IS NULL OR` first, so a closed row is TRUE here and never UNKNOWN.
  CONSTRAINT l2_pair_decision_current_value CHECK (is_current IS NULL OR is_current = 1)
) ENGINE = InnoDB;
