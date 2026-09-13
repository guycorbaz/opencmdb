-- The addressing plan, emptied — for the axe gate's EMPTY-PLAN mode (story 14.2b).
--
-- 🔴 **THE ONE `/ipam` STATE THE BROWSER GATES WERE CONFIGURED NEVER TO REACH.** `a11y/seed.sql`
-- always seeds a subnet and `AXE_REQUIRE_PLAN=1` refuses a run that draws no cell, so the EMPTY
-- plan — the branch where story 14.2b's AC8 puts the link into the gesture — was measured by no
-- browser at all. This file makes that state reproducible instead of accidental.
--
-- ⚠️ **It must run BEFORE `a11y/seed.sql`, never instead of it.** The empty-plan pass walks one
-- route; the seeded pass walks ten plus four states. Neither replaces the other.
--
-- 🔴 **AND IT TRUNCATES RATHER THAN TRUSTING A FRESH DATABASE.** CI's `Tests` step runs against
-- the same store immediately before, and story 6b.11 measured what that costs: a run that passed
-- because the previous step's residue carried a floor. *A state this repository can reproduce, not
-- one the pipeline accumulated.*
--
-- Children first: `ip_range` and `ip_address` carry a foreign key onto `ip_subnet`.
DELETE FROM ip_range;
DELETE FROM ip_address;
DELETE FROM ip_subnet;
