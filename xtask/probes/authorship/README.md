# The authorship gate's evasion corpus

Thirty-eight hand-written violations of NFR5's never-overwrite invariant, one per mechanism. They are
the regression suite of `gate_declared_authorship` (`xtask/src/main.rs`): a repaired gate is
**measured** against all of them, never argued about.

Their expected verdicts are pinned in `xtask/src/main.rs`'s `AUTHORSHIP_PROBES` table — **the file
AND the line the gate must name** — and the sweep test reads these files.

🔴 The verdict was a bare `true`/`false` until a defect showed what that cannot hold: a boolean pins
THAT the gate reds, never WHERE. `normalise_sql_text` was mapping byte offsets onto a per-character
line table, so any multibyte string literal shifted every later finding (measured: **line 0** for a
write on line 2) with the whole suite green — and `e23`, the multibyte probe itself, reporting the
right line by luck. Twenty-nine booleans are now twenty-nine located verdicts. A probe whose verdict changes reds that test — including a probe that starts
being CAUGHT, since a gate that quietly widens is a gate whose stated limits have gone stale.

## Provenance

Written by story 5.12's code review against the FIRST implementation of the gate, which **16 of the
30 passed** — three of them executing successfully against MariaDB 10.11.11. Three were added during the repair: `e31`, for a mechanism the review's own sweep had missed;
`e32`, because mutation M13 came back GREEN — `e06` puts its zero-width space BEFORE the verb, where
a token boundary already exists, so it left the guard carried by nothing; and **`e33`, which is a
different AXIS from the other thirty-two**. They are all planted in an unsanctioned file and ask
*does it red*. `e33` carries the SANCTIONED NAME and asks whether the name alone lets a write
through from somewhere else — and it did, on the committed tree, with the gate green. The measurement that repair is judged
by is recorded in the story file.

## They are not compiled

`xtask/probes/` is neither `src/`, `tests/`, `examples/` nor `benches/`, so cargo never builds these
files and `cargo fmt` never rewrites them. That matters: `e04` carries hard tabs, `e05` a no-break
space and `e06` a zero-width space, and a formatter would silently repair the very thing they probe.

## `e36` is a CONTROL, and that is a distinct job

`e34` and `e35` are the same read carrying a quote inside the query — raw string and escaped
spelling. `e36` is that read **without** the quote, and it reddened all along. It earns its place
because it is what proves the quote is the cause: under M26 and M27 the two quoted forms go green
while `e36` stays red. A probe that cannot fail proves nothing; a control that never changes is what
makes its neighbours' change mean something.

## The four that are GREEN on purpose

`e02`, `e14`, `e31` and `e38` are pinned as PASSING the gate. They are not oversights — they are where the
promise stops, and each is stated in the story:

- **`e02`** — the query is assembled at runtime (`format!`). A text matcher cannot follow a table
  name that does not exist until the program runs.
- **`e38`** — `CREATE TABLE … AS SELECT` does write values, and the weakest of the four for it. It
  cannot RUN against a schema `0001` has already migrated (the table exists), and the form of the
  class that can — `CREATE OR REPLACE TABLE … AS SELECT` — already reds. A false sentence in the
  code's own comment with a narrow executable reach, pinned so the fact stays measured.
- **`e14`, `e31`** — `RENAME TABLE` and `ALTER … DROP CONSTRAINT` touch no row: they neutralise the
  guard rather than write a value under a false author. NFR5 is about AUTHORSHIP, and the closure
  for guard removal is a privilege the database refuses, not a matcher that reads source text.

⚠️ **The line between them and `e22` is not clean, and pretending otherwise would be a false
sentence.** `CREATE OR REPLACE TABLE` (`e22`) writes no row under a false author either — it
destroys the table and every declared value in it — yet it REDS. Mutation M19b showed the red is
incident: it survives removing the verb from the list, because the `REPLACE` inside the phrase
governs the same reference. It is kept because the gesture annihilates the guarded table from
inside a `.sql` migration; it is not kept because the criterion demanded it.
