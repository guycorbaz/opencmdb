# The authorship gate's evasion corpus

Thirty-one hand-written violations of NFR5's never-overwrite invariant, one per mechanism. They are
the regression suite of `gate_declared_authorship` (`xtask/src/main.rs`): a repaired gate is
**measured** against all of them, never argued about.

Their expected verdicts are pinned in `xtask/src/main.rs`'s `AUTHORSHIP_PROBES` table, and the sweep
test reads these files. A probe whose verdict changes reds that test — including a probe that starts
being CAUGHT, since a gate that quietly widens is a gate whose stated limits have gone stale.

## Provenance

Written by story 5.12's code review against the FIRST implementation of the gate, which **16 of the
30 passed** — three of them executing successfully against MariaDB 10.11.11. `e31` was added during
the repair, for a mechanism the review's own sweep had missed. The measurement that repair is judged
by is recorded in the story file.

## They are not compiled

`xtask/probes/` is neither `src/`, `tests/`, `examples/` nor `benches/`, so cargo never builds these
files and `cargo fmt` never rewrites them. That matters: `e04` carries hard tabs, `e05` a no-break
space and `e06` a zero-width space, and a formatter would silently repair the very thing they probe.

## The three that are GREEN on purpose

`e02`, `e14` and `e31` are pinned as PASSING the gate. They are not oversights — they are where the
promise stops, and each is stated in the story:

- **`e02`** — the query is assembled at runtime (`format!`). A text matcher cannot follow a table
  name that does not exist until the program runs.
- **`e14`, `e31`** — `RENAME TABLE` and `ALTER … DROP CONSTRAINT` neutralise the guard rather than
  write a value under a false author. NFR5 is about AUTHORSHIP, and the closure for guard removal is
  a privilege the database refuses, not a matcher that reads source text.
