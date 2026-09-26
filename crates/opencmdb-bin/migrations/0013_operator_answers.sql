-- opencmdb — the operator's answer to an L2 question (story 6.14b).
--
-- Story 6.12's `0012` refused `match` outright: no L2 rule is `Decisive`, and an ENGINE match would be
-- a device nobody mints. Story 6.14b gives the question an answer, and the operator's positive answer
-- — *the same machine* — IS a `match`. So `0013` widens the two CHECKs that refused it, and widens
-- them FOR `OPERATOR` ONLY: an ENGINE `match` stays refused, by the same constraint name as before.
--
-- What an answer row is (Guy's arbitration of 2026-09-25, story 6.14b §0.8 B): `decided_by =
-- 'OPERATOR'`, `rule_id = 'operator'` (a token outside both `l1-` and `l2-`, so no trap runner can
-- mistake it for a rule), no abstention cause, and the verdict vector and ruleset version COPIED from
-- the ENGINE row it supersedes — the evidence the operator was shown is kept with the answer.
--
-- The three arms of `rule_xor_cause`, after this migration:
--   - an ENGINE abstention (`abstained`/`ambiguous`), no rule;
--   - an ENGINE `no_match`, with a rule that is neither empty nor the operator's token;
--   - an OPERATOR `match` or `no_match`, with `rule_id = 'operator'` and no cause.
-- So an OPERATOR `abstained` is refused (an answer that answers nothing), and an ENGINE row may not
-- claim the operator's token.
--
-- 🔴 THE PADDING IDIOM (story 14.1's `LENGTH(x) = LENGTH(TRIM(x))`). `ascii_bin` is a PAD SPACE
-- collation: `'OPERATOR '` satisfies `l2_pair_decision_decided_by` and `= 'OPERATOR'` alike, and then
-- reads as the ENGINE's in `CurrentL2Decision::is_operators`, which compares in Rust. `'operator '`
-- would be a second spelling of the token. Both are refused here, on the OPERATOR arms. On the ENGINE
-- arm `<> 'operator'` already refuses `'operator '` under the same padding, so no idiom is owed there.
-- ⚠️ `ascii_bin` is case-sensitive, so an ENGINE `rule_id = 'Operator'` passes: it is not the token,
-- `is_operators` reads `decided_by` and never `rule_id`, and no runner routes a token outside
-- `l1-`/`l2-`.
--
-- 🔴 THE RE-RUNNABLE SPELLING, and both obvious ones are silent defects (story 6.14b §0.9 A, measured
-- by the validation on a populated `0012` store):
--   - `ADD CONSTRAINT IF NOT EXISTS x` on a store holding the narrow `x` keeps the OLD body, with only
--     `Note 1826`;
--   - `DROP CONSTRAINT IF EXISTS x, ADD CONSTRAINT IF NOT EXISTS x …` exits 0 and DELETES the
--     constraint, the `IF NOT EXISTS` being evaluated before the drop.
-- What works: ONE statement, `DROP CONSTRAINT IF EXISTS x, ADD CONSTRAINT x CHECK (…)`, with NO
-- `IF NOT EXISTS` on the ADD. It is atomic — a failing one (`ERROR 4025`) left the old constraint
-- intact — and running it twice re-adds the same bodies.
--
-- 🔴 ORDER: `outcome` FIRST, then `rule_xor_cause`. MariaDB names the FIRST failing CHECK in
-- declaration order, and a re-added constraint moves to the END of the table's list. Re-adding
-- `outcome` alone would make an ENGINE `match` name `rule_xor_cause`, and
-- `a_match_or_an_absence_of_proof_is_refused_by_the_schema` would red for no defect.
--
-- ⚠️ `0012`'s HEADER now says `match` is refused outright, and it is NOT edited: sqlx checksums every
-- applied migration, and changing a comment in `0012` would stop every existing store booting with a
-- version mismatch. This header is where the correction lives.
--
-- ⚠️ RECOVERY. A store holding an OPERATOR row this migration refuses — `abstained`, or a rule id other
-- than `operator` — fails here with `ERROR 4025` and boots `Dirty(13)`. Only tests ever wrote such rows
-- (two forged OPERATOR `abstained` rows by `UPDATE`, before this story gave them a real producer). The
-- recipe: delete them, then clear the failed record, and restart:
--     DELETE FROM l2_pair_decision WHERE decided_by = 'OPERATOR' AND outcome = 'abstained';
--     DELETE FROM _sqlx_migrations WHERE version = 13 AND success = 0;
ALTER TABLE l2_pair_decision
  DROP CONSTRAINT IF EXISTS l2_pair_decision_outcome,
  ADD CONSTRAINT l2_pair_decision_outcome CHECK (
    outcome IN ('no_match', 'abstained')
    OR (outcome = 'match' AND decided_by = 'OPERATOR'
        AND LENGTH(decided_by) = LENGTH(TRIM(decided_by)))
  ),
  DROP CONSTRAINT IF EXISTS l2_pair_decision_rule_xor_cause,
  ADD CONSTRAINT l2_pair_decision_rule_xor_cause CHECK (
    (outcome = 'abstained' AND decided_by = 'ENGINE'
     AND rule_id IS NULL AND abstention_cause = 'ambiguous')
    OR (outcome = 'no_match' AND decided_by = 'ENGINE'
        AND rule_id IS NOT NULL AND rule_id <> '' AND rule_id <> 'operator'
        AND abstention_cause IS NULL)
    OR (outcome IN ('match', 'no_match') AND decided_by = 'OPERATOR'
        AND LENGTH(decided_by) = LENGTH(TRIM(decided_by))
        AND rule_id = 'operator' AND LENGTH(rule_id) = LENGTH(TRIM(rule_id))
        AND abstention_cause IS NULL)
  );
