# Story 5.9: The interface and its identity link are persisted, ambiguity included

Status: ready-for-dev

<!-- ✅ VALIDATED 2026-08-03 by two fresh-context agents (fact-check + gap-hunt), as this project
     requires (Guy's decision, Epic 4 retrospective 2026-07-26). The template banner saying
     "Validation is optional" does not apply here.

     🔑 THE GAP-HUNT RAN WITH A LIVE DATABASE — `mariadb:10.11.11` on host port 13306, and every
     one of the six mutations was executed against real DDL. It reached 374 green tests (2
     newtypes, the full `0002`, 7 query bodies, 9 tests), six green gates, clippy twice clean.
     That matters because it MEASURED the story's own §7 warning and confirmed it: 174/156/46
     green **identically** with and without `DATABASE_URL`.

     🔴 FOUR HIGH findings, all from the agent that COMPILED the story — the FIFTH consecutive
     story where the fact-checker produced no HIGH. All four are applied below:
       • **AC3's uniqueness key rejected a LEGITIMATE write.** `join` loops
         `for key in keys_of(observation)` [`l1.rs:174-178`], so a multi-MAC observation lands on
         N interfaces — and `l1.rs:186` says so in prose. `multi-nic` is a COMMITTED trap family.
         Measured: the second link gave `left: Err(Constraint("unique"))`.
         **Guy's arbitration: the key widens to `(observation_id, link_subject, valid_to)`** — see
         decision 9 and AC3. The abstention half of that trade-off is closed by a sentinel, the
         same idiom `valid_to` already uses.
       • **M5 was not executable as written** and did not measure D21's NULL trap: `valid_to` is
         `NOT NULL`, so binding NULL dies on error 1048 at the FIRST insert. Now a two-part
         mutation, and its red was re-aimed at an assertion that COUNTS.
       • **No prescribed test could carry that assertion.** T5 asked only for the
         `Constraint("unique")` shape, which panics at `expect_err` before any count exists. The
         counting test is now prescribed, with its measured red: `left: 2, right: 1`.
       • **M4 was a no-op against AC4** — `link_candidate`'s FK makes "candidates but no link row"
         unwritable, so the measured red was `Constraint("foreign_key")` at `.expect(…)` and
         AC4's assertion was never evaluated. Now a two-part mutation; measured red `left: 0,
         right: 1`.

     The fact-check's two HIGH are applied too: §7's `docker run` could not have worked on this
     machine (port 3306 is held by an unrelated `mariadb:11-jammy`, so the DSN would have reached
     the WRONG ENGINE VERSION silently), and §5 undercounted the register by one entry. -->

## Story

As the operator,
I want interfaces and their identity links stored as revisable records carrying their evidence,
so that *"present the candidate matches with their evidence"* (FR16) is something the product can
actually do.

**This story creates the schema and the persistence contract. It writes no link the engine derived.**
The rows it writes are written by its own tests, from `Decision`s that `identity::l1::decide_pair`
produced. The pass that walks a set of observations, calls the blocker and the join, and writes what
it derives is **story 5.9b**, inserted at this story's contexting — see §1.

**What this story does NOT do**, so the boundary is explicit and not discovered in review:

- it does **not** create `device` — Epic 6. `epics.md:1576` says so, and a table this epic would not
  populate is speculation;
- it does **not** create the `entity` supertype either. D21's disjunction
  [architecture.md:1450-1454] exists so a link can point at an interface *or* a device; `device` does
  not exist, so the disjunction has one arm. **Owner: Epic 6, with `device`** (§4, decision 4);
- it does **not** run the engine over anything. `identity::blocking::candidates` still has **no
  production caller** after this story, and `identity::l1::join` still has **no cross-crate caller
  at all** — both are story 5.9b's, and the register entry is re-owned rather than silently carried
  (§5);
- it does **not** add a `state` column to `interface`. `dormant`/`active` (D21's extended
  `entity.state`, F17's lifecycle) is read by nothing in Epic 5. **Owner: the lifecycle epic
  (FR40-42)**;
- it does **not** display anything. FR16's rendering, the abstention counter and its grouping by
  cause are **story 5.14**. This story makes the rows readable; it renders no page;
- it does **not** touch `identity::l1`, `identity::blocking` or `identity::cascade`. The only
  `opencmdb-core` change is two id newtypes (§4, decision 5).

**Nothing under `fixtures/` moves.** No artefact bytes, no `MANIFEST.toml`, no re-hash. If any step
appears to require re-authoring a committed artefact, **STOP** — that is a finding, reported rather
than absorbed.

**`architecture.md` is NOT edited** (issue #54; a milestone act).
**`architecture-views.md` is NOT regenerated** (issue #50).
**`epics.md` IS already edited** — at this story's contexting, to record the 5.9/5.9b split and the
link-subject arbitration. The dev **does not edit it again**; verify-only from here.

⚠️ **Branch from `master`.** Measured at contexting: `master` is at **`28a7f51`** and
`cargo test --workspace --locked` reports **367 tests** — **165 bin + 156 core + 46 xtask** (plus
one ignored doc-test).

⚠️ **The tree is NOT clean, and that is expected.** The contexting left three uncommitted planning
edits — `epics.md` (the SPLIT note, story 5.9b, `16 → 17`), `sprint-status.yaml` (the split, the
two keys) and this story file, untracked. **They belong on the story branch**; carry them, do not
stash them and do not commit them to `master`. _(An earlier draft of this line said the tree was
clean, three lines under a sentence saying `epics.md` had been edited — caught by the validation.)_

---

## What this story inherits, measured rather than assumed

Everything below was measured at contexting on `28a7f51`, by reading the tree. **The dev re-derives
none of it; a surprise reads as a FINDING.**

### 1. The story was SPLIT at contexting, and 5.9b was inserted

`epics.md`'s story 5.9 carried two ideas: **(a)** the schema and the persistence contract, and
**(b)** the path that runs the engine over a set of observations and WRITES the links it derives.
Guy split them on 2026-08-03, before any persistence code existed. Three reasons, all measurable:

1. **(b) is the heavier half**, and it is the first production caller of *two* organs that have
   never had one — `identity::blocking::candidates` (no production caller at all) and
   `identity::l1::join` (no *cross-crate* caller at all; `lib.rs:46-53` says so and gives the
   reason).
2. **(b) is what story 5.10 requires.** 5.10's *"the engine re-runs"* [`epics.md`] is a re-run of a
   pass that does not exist. Leaving (b) unowned would have made 5.10 carry two ideas.
3. The register has owned that residue as *"story 5.9 or Epic 6, whichever first hands the blocker
   a set of observations"* since story 5.6, re-stated by 5.7 and again by 5.8
   [`deferred-work.md`]. **This story is not it; 5.9b is.**

The letter suffix is the house idiom for an INSERTED item (D56b; stories 5.2b and 5.4b), so
5.10–5.14 keep their numbers. **Epic 5 is now 17 stories.** `epics.md` and `sprint-status.yaml`
carry it; nothing else needs to.

**No acceptance criterion moved, and two were ADDED.** `epics.md`'s four 5.9 criteria are all schema
statements and all four are still here: the three tables and not `device` → **AC1**; the SCD2 link
carrying rule/evidence/when/whom/`ruleset_version` → **AC2**; the ambiguity as a link with its
candidates → **AC4**; binary collation with the DDL gate green → **AC8**. **AC3** (one current link
per observation, held by the `OPEN_END` sentinel) and **AC5** (no unique index on the L1 key) come
from **D21**, not from `epics.md` — they are added here because both are schema decisions this
migration takes whether or not anyone writes them down, and D21 says what happens when they are
taken wrongly.

### 2. 🔴 What an `identity_link` LINKS — the arbitration this story rests on

This is the decision a reviewer will challenge first, so it is stated before anything is built.

- `identity::l1::decide_pair(&Observation, &Observation) -> Decision` judges **a PAIR of
  observations** and returns **no interface**. A row shaped `(obs_a, obs_b, verdict)` is what the
  engine literally returns — and no `interface` row follows from it without a clustering step.
- `identity::l1::join(&[Observation]) -> BTreeMap<(L2DomainId, MacAddr), BTreeSet<ObsId>>`
  [`l1.rs:172`] groups observations by the scope-qualified key. **That map IS the set of interfaces
  at L1**: one key, one interface, and the `BTreeSet<ObsId>` is the set of observations on it. D13
  says exactly this — *"L1 = pure A — a deterministic **join** on the scope-qualified key
  `(l2_domain, mac) -> interface`"* [architecture.md:984-985]. _(D13's word is `join`; an earlier
  draft quoted it as "lookup", which is this story's word and was inside quotation marks.)_
- ⚠️ **One observation can sit on SEVERAL interfaces, and the schema must permit it.** `join` is
  `for observation { for key in keys_of(observation) { … } }` [`l1.rs:174-178`] — an observation is
  inserted under **every** key it carries, and `l1.rs:186` states it: *"An observation may carry
  several MACs."* `multi-nic` is a committed trap family. This is why AC3's uniqueness key is
  `(observation_id, link_subject, valid_to)` and not `(observation_id, valid_to)` — see decision 9.

**Therefore: an `identity_link` binds one `observation_record` to one `interface`**, carrying the
rule applied, the evidence, when, by whom and `ruleset_version` (D14). `link_candidate` carries the
N candidate **interfaces** of an observation the engine abstained on — which is what FR16's
*"present the candidate matches with their evidence"* asks a page to display. The `interface ->
device` grouping is a **different relation** and arrives with `device`, in Epic 6.

⚠️ **`decide_pair` is still the source of the rule and the evidence** on a link — the pair verdict
is what justifies placing an observation on an interface. This story stores that justification; it
does not compute it (5.9b does).

### 3. The tree this story extends, measured

| what | where | size |
|---|---|---|
| the only migration | `crates/opencmdb-bin/migrations/0001_initial.sql` | 34 lines, 2 tables |
| the MariaDB adapter — the only place SQL against the domain tables is written | `crates/opencmdb-bin/src/repo.rs` | 342 lines (236 code + 106 tests; `#[cfg(test)]` at :237) |
| the abstract persistence contract, sqlx-free (D47/D49) | `crates/opencmdb-core/src/repo/mod.rs` | 66 lines |
| the id newtypes and the `uuid_newtype!` macro | `crates/opencmdb-core/src/observation/mod.rs:24-64` | 4 newtypes |
| the engine's return type | `crates/opencmdb-core/src/identity/cascade.rs` | `Decision`, `Conclusion`, `RuleVerdict`, `RulesetVersion`, `Verdict`, `IdentityAbstentionCause` |

**The existing adapter idiom, which this story follows and does not reinvent** [`repo.rs`]:

- **query bodies are free functions generic over `sqlx::Executor`** (D49), written **once**, called
  both by the read side (with the pool) and by a unit of work (with the transaction connection —
  read-your-own-writes). `count_declared_attributes`, `insert_observation`, … are the pattern;
- **static SQL, bound values** (D48) — `sqlx::query("…").bind(x)`. No `format!`, no
  `AssertSqlSafe`;
- **`classify(sqlx::Error) -> RepositoryError`** is the ONE `sqlx::Error → RepositoryError`
  translation in the crate [`repo.rs:214-235`] — `page.rs::server_error` delegates to it. _(It is
  **not** the only place a MariaDB error code is named: `dburl.rs:105` names `"1045"` for the
  connect-error explainer. And `sqlx` appears as code in four files — `repo.rs`, `main.rs`,
  `page.rs`, `dburl.rs` — so do not repeat the "only place sqlx appears" sentence; it is false.)_
  It already maps `is_unique_violation` →
  `Constraint("unique")` and `is_check_violation` → `Constraint("check")` — **which is what makes
  AC3's and AC5's guards assertable without new plumbing**;
- **ids are bound as `String`** — `observation.obs_id.to_string()` [`repo.rs:151`], because D48 is
  `CHAR(36) ascii_bin`, not `BINARY(16)`;
- **serialized domain values go through `serde_json::to_string`** and are decoded in Rust
  [`repo.rs:140-141, 205-207`] — **SQL never descends into a value comparison** (D10);
- **DB tests are gated and serialized**: `let Ok(url) = std::env::var("DATABASE_URL") else { … return }`,
  then `let _guard = crate::DB_TEST_LOCK.lock().await;` [`main.rs:32-33`], then
  `sqlx::migrate!("./migrations").run(&pool)`, then a `DELETE FROM …` to isolate the run.

### 4. 🔴 The `ddl-collation` gate, exactly how it matches — and the two ways to trip it by accident

`xtask/src/main.rs:307-367`. It walks `crates/opencmdb-bin/migrations/**/*.sql` **line by line**,
and it is a **reflex heuristic, not a parser**:

```
trim the line; skip it if it is empty or starts with "--"
uppercase it
is_text  := contains "VARCHAR" | contains "TEXT" | contains " CHAR" | starts_with "CHAR" | contains "CLOB"
has_bin  := contains "_BIN" | contains "COLLATE BINARY"
offender := is_text && !has_bin
```

**Two consequences the dev must hold in mind while writing `0002`:**

1. ⚠️ **The word `TEXT` inside a TRAILING comment on a non-text line reds the gate.** `-- context`,
   `-- the text of the rule`, `-- CHARacter` all contain a matching substring, and the line is not
   skipped because the skip only fires when the line *starts* with `--`. **Put explanatory prose on
   its own `--` line**, which is how `0001_initial.sql` is already written.
2. ⚠️ The check is **per line**. A column declaration split across two lines with the `COLLATE` on
   the second one reds. Keep one column per line, as `0001` does.

**The shape that passes — the idiom of `0001_initial.sql` (`attr_value` is its line 13; the id line
below is modelled on `observation_record.id`, line 26). Do not invent a second one:**

```sql
  id            CHAR(36)    CHARACTER SET ascii   COLLATE ascii_bin    NOT NULL,
  attr_value    TEXT        CHARACTER SET utf8mb4 COLLATE utf8mb4_bin,
```

`ascii_bin` for opaque ids and enumerated tokens; `utf8mb4_bin` for anything that could carry a
non-ASCII byte. Both contain `_BIN`, so both pass.

### 5. The registered debt this story owns — what it CLOSES, what it ANSWERS, what it RE-OWNS

**Eight** entries in `deferred-work.md` name story 5.9 as owner. _(An earlier draft said seven and
missed #8 — caught by the validation's fact-check, which is the "promise re-made" defect the
register itself records five consecutive reviews catching.)_ **Three of them are conditional
(*"if it persists a cause at all"*, *"the first story that reconstructs a `Decision`"*, *"the first
story to persist a key"*), and the honest disposition of a conditional entry whose condition is not
met is to ANSWER it with the measurement — not to close it, and not to carry it silently.** That is
the disposition story 5.7 used for the blocker and 5.6 used for `L1Key`; six consecutive code
reviews have caught the alternative.

| # | entry | disposition here |
|---|---|---|
| 1 | *"`IdentityAbstentionCause` derives no `Serialize`/`Deserialize` … Owner: story 5.9, **if it persists a cause at all**"* | **CLOSED — and by REFUSING the derive.** This story persists a cause, as an explicit exhaustive `match` in the ADAPTER (§4, decision 3). A derived name is a wire format nobody chose: renaming a variant would silently change the stored bytes. |
| 2 | *"None of the five new types derives `Serialize`/`Deserialize` … Owner: story 5.9, **if it persists a decision at all**"* | **CLOSED — same refusal, same reason.** This story persists a decision's *components* as COLUMNS, never a `Decision` as a blob. Nothing serialises `Verdict`, `RuleVerdict`, `Conclusion` or `Decision`. |
| 3 | *"`RulesetVersion` derives no `PartialOrd`/`Ord`. The first consumer that ORDERS two versions is persistence … Owner: story 5.9."* | **ANSWERED, NOT CLOSED — the prediction is refuted by measurement.** Persistence stores the version and reads it back; **nothing compares two.** *"The link decided under the current ruleset"* is an EQUALITY, not an order. No `Ord` is added. |
| 4 | *"An incoherent `Decision` is still buildable by struct literal … Owner: story 5.9, the first story that reconstructs a `Decision` from somewhere other than `decide` (persistence)."* | **ANSWERED, NOT CLOSED.** This story's read side returns **persisted rows**, not a reconstructed `Decision` — the `verdict_vector` is deliberately not stored (§4, decision 2), so a `Decision` cannot be rebuilt from a row and no constructor is written. The condition is not met. |
| 5 | *"Nothing enforces that a `RuleVerdict` built by struct literal leaves non-empty evidence … residue is story 5.9's."* | **ANSWERED, NOT CLOSED** — same reason as #4: no `RuleVerdict` is constructed by this story. |
| 6 | *"`L1Key` is a bare tuple alias … Owner: story 5.9, the first story to persist a key."* | **RE-OWNED to story 5.9b, with the measurement.** This story persists the key's two COMPONENTS as columns (`interface.l2_domain`, `interface.mac_canon`) and **never holds an `L1Key` value** — it does not call `join`. 5.9b does. |
| 7 | *"The blocker STILL has no production caller … Owner: story 5.9 or Epic 6."* | **RE-OWNED to story 5.9b**, explicitly and for the reason in §1. Unchanged in substance. |
| 8 | *"The universe is quadratic in the slice the CALLER supplies, and nothing yet bounds that slice … ↺ owner: 5.9 or Epic 6, whichever first hands the blocker a set of observations."* [`deferred-work.md:1861-1871`] | **RE-OWNED to story 5.9b.** A DISTINCT bullet from #7 — #7 is *"has no caller"*, this one is *"the caller's slice is unbounded"*. This story hands the blocker nothing, so the condition is untouched; 5.9b is the first story that hands it a set of observations and therefore the first that can measure `n`. |

⚠️ **Do not report #3, #4, #5 as CLOSED.** An answered entry says *"the condition was measured and
not met"*; a closed one says *"the thing was done"*. Reporting the first as the second is the
over-claim this project's reviews have caught in six consecutive stories.

### 6. 🔴 The purge-stability constraint story 5.10 inherits from THIS schema

5.10 asserts that deleting the engine's links (`epics.md:1627` writes it
`TRUNCATE ... WHERE decided_by = 'ENGINE'`, which is pseudo-SQL — MariaDB's `TRUNCATE` takes no
`WHERE`, so the real statement is a `DELETE`; the `decided_by` column is what it needs either way)
followed by a re-run reproduces
the engine's links **bit for bit** (D14 [architecture.md:1038-1039]). **That property is decided
here, by the schema, and it is easy to destroy without noticing.** Three constraints follow, and
they are ACs rather than notes:

1. **`interface` rows are NOT purged and their `id` is stable.** The purge deletes *links*. If an
   interface's UUID were re-minted on a re-run, every reproduced link would carry a different
   `interface_id` and 5.10 could never pass. The interface survives; the re-run finds it by its
   `(l2_domain, mac_canon)` key.
2. **`interface.first_seen_at` / `last_seen_at` are DERIVED FROM OBSERVATIONS, never from the
   clock.** They are `min`/`max` of the `observed_at` of the observations on the interface. A
   `NOW(6)` there is not reproducible, and *"the engine never touches the clock"*
   [architecture.md:3364]. **This story does not compute them** (5.9b does) — it types them, writes
   them from its parameters, and its doc says where they must come from.
3. **`identity_link.valid_from` is a parameter, never `NOW(6)`.** Same reason. `0001`'s
   `insert_declared_attribute` uses `NOW(6)` [`repo.rs:120`] — **that is a DECLARED row authored by
   a human and it is not a precedent for an engine-derived one.** Do not copy it here.

### 7. 🔴 A green suite says NOTHING about the database — and this story is nothing but database

`DATABASE_URL` is **unset** on Guy's machine (measured at contexting). Every DB-backed test in
`repo.rs` and `main.rs` begins with `let Ok(url) = std::env::var("DATABASE_URL") else { … return; }`
and **passes by returning**. CI provides the service — `.github/workflows/ci.yml` sets
`DATABASE_URL: mysql://root:opencmdb@127.0.0.1:3306/opencmdb_test` against `mariadb:10.11.11`, the
exact DSM 7 package (D64) — so the tests do run *there*. But:

> **Six of this story's seven mutations red NOTHING with `DATABASE_URL` unset.** M1, M2, M3, M4, M5
> and M7 are all schema behaviour; only M6 (a token string) is reachable without a database. A
> prove-to-red pass run without one would record six green mutations and conclude the guards are
> decorative when in fact they were never executed.
>
> **This was MEASURED at the validation pass, not reasoned**: the suite reports 174/156/46 green
> **identically** with and without `DATABASE_URL`.

**The dev MUST run a local MariaDB and record it.** The exact image is not negotiable (D64:
dev = CI = prod):

```sh
docker run --rm -d --name opencmdb-dev-db -p 13306:3306 \
  -e MARIADB_ROOT_PASSWORD=opencmdb -e MARIADB_DATABASE=opencmdb_test \
  mariadb:10.11.11
docker ps --filter name=opencmdb-dev-db      # ⚠️ CONFIRM IT IS UP BEFORE TRUSTING ANY GREEN RUN
export DATABASE_URL='mysql://root:opencmdb@127.0.0.1:13306/opencmdb_test'
cargo test --workspace --locked
```

🔴 **Host port 13306, NOT 3306 — this is a correction, not a preference.** Measured on Guy's
machine at validation: `0.0.0.0:3306` is held by **`kesh-mariadb`, an unrelated `mariadb:11-jammy`
container belonging to another project**. The `docker run` on 3306 fails with *"port is already
allocated"* — and the dangerous path is not that failure, it is the dev exporting the DSN anyway:
`127.0.0.1:3306` then reaches a **MariaDB 11** instance, the wrong engine version under D64, and
`sqlx::migrate!` applies `0001` and `0002` to **someone else's database**. Never touch
`kesh-mariadb`. If 13306 is also taken, pick another free port and change the DSN with it.

⚠️ `docker/docker-compose.yml` deliberately has **no database service** — it points at an external
MariaDB. It is not the tool for this; the `docker run` above is.

The Debug Log must state, for each mutation, **whether it was run with a database and which
assertion carried the red**. "Green" without that qualification is not a measurement.

### 8. Baseline, gates and the traps that cost an hour if they are not read

- **Baseline: 367 tests** (165 bin + 156 core + 46 xtask) at `28a7f51`, clean tree.
- **The six gates must stay green**: `cargo xtask ci` — dependency frontier (D47), **`ddl-collation`
  (§4 — this story is the first to add DDL since story 3.2)**, retired vocabulary (D65), the fixture
  corpus lock, `file-size` (D56b), `float-free` (D13). `views-hash` reports `ℹ STALE` and exits 0 —
  **by design, do not regenerate**.
- **The `float-free` gate walks `crates/opencmdb-core/src/identity/` and nothing else**
  (`IDENTITY_DIR`, `xtask/src/main.rs:868`; it currently reports 4 files). Decision 5 puts the two
  new newtypes in `observation/mod.rs`, which the gate does **not** walk — measured at validation:
  the gate stayed green and did not interact. So the rule *"no float, no float literal"* holds here
  as a house rule, **not** because a gate enforces it. `ruleset_version` is a `u32`; there is no
  confidence, no score, no ranking in this story.
- ⚠️ **D47 is a gate.** `opencmdb-core` must not gain a dependency on `sqlx`, `anyhow`, `axum` or
  `askama`. Every line of SQL in this story lives in `opencmdb-bin`.
- ⚠️ **Run clippy TWICE** before pushing (`local-gate-must-mirror-ci`): `cargo clippy --workspace
  --all-targets -- -D warnings` **and** `cargo clippy --workspace --locked -- -D warnings` (the
  second is what CI runs; an import kept alive only by a test passes the first and fails the second).
- ⚠️ **Issue #38 — unexplained local test non-determinism** recurred on `master` on 2026-08-02. If a
  test reds once and then passes 8 times on a clean tree, that is #38 and **not** a finding about
  this story. Record the observation on the issue; **do not adopt a cause without naming the check
  that would have failed if the cause were wrong.**
- ⚠️ `sqlx::migrate!` embeds the migration directory **at compile time**. After adding `0002`, a
  stale build can run the old set — if a test fails with "table doesn't exist", `touch` the crate or
  `cargo clean -p opencmdb` before believing it.

---

## Decisions taken at contexting

Nine, all with Guy or measured against the tree. They are decisions, not suggestions — a dev who
disagrees reports a FINDING rather than choosing differently. **Decision 9 was taken at the
validation pass, not at contexting**, and it changes AC3.

**1. The story is SPLIT; 5.9b is inserted.** §1. Epic 5 → 17 stories.

**2. The `verdict_vector` is NOT stored.** D14 lists what a link carries [architecture.md:1015-1016],
and `epics.md`'s AC2 restates it as *"the rule applied, the evidence, when, by whom, and
`ruleset_version`"* — **the vector is in neither list.** D18's *"the harness records the COMPLETE VERDICT VECTOR"* [architecture.md:1396-1399]
is a requirement on the **trap harness**, which is `ScoredRecord`, in `score.rs`, and is a different
sentence about a different object. Storing the vector would mean deriving a wire format for four
domain types (§5, #2) to serve no reader. **Consequence, stated rather than discovered: a persisted
link cannot be turned back into a `Decision`**, which is exactly why §5 #4 is answered and not
closed. If Epic 6 or 5.14 needs the vector, that is a schema addition with a named consumer.

**3. The cause and the conclusion are persisted as ASCII tokens by an explicit exhaustive `match` in
the ADAPTER — no serde derive.** Two reasons: (a) a derived variant name is a wire format nobody
chose, and a rename would silently rewrite stored bytes — the *"silent data migration, the worst
kind"* D14 names about `ruleset_version`; (b) `IdentityAbstentionCause` is deliberately **not**
`#[non_exhaustive]` [`cascade.rs`], precisely so a new variant produces `error[E0004]` in every
downstream crate. An exhaustive `match` in `opencmdb-bin` **uses** that mechanism; a `#[derive]`
would bypass it. **No `_` arm** — the `_` is what turns the compile error into a silent
mis-classification, which is the defect story 5.8's `TrapError::RuleMalformed` patch fixed.

**4. No `entity` supertype table, no `device`, no `state` column.** D21's supertype
[architecture.md:1450-1454] exists to make the interface/device disjunction structural; with
`device` absent the disjunction has one arm, and a supertype over one subtype is the speculation the
*"create tables only when the story needs them"* rule refuses. `state` (`active|dormant|…`,
D21:1477-1479) is read by nothing before the lifecycle epic. **Owners: Epic 6 for `entity` and
`device`; the lifecycle epic (FR40-42) for `state`.** Register all three.

**5. `InterfaceId` and `LinkId` are new id newtypes in `crates/opencmdb-core/src/observation/mod.rs`**,
declared with the existing `uuid_newtype!` macro beside `ObsId`/`ConnectorId`/`L2DomainId`/
`VantageId`, and re-exported from `lib.rs`. The macro is a plain `macro_rules!` with no
`#[macro_export]` and no `pub(crate) use`, so it is **not reachable from `identity/`**; a
`crates/opencmdb-core/src/identity/ids.rs` would need macro plumbing added to buy a folder move,
and *"the folder is not the frontier — visibility is"* (D54, quoted by `identity/mod.rs` itself).
`L2DomainId` (*"the MAC's uniqueness space"*) and `VantageId` (*"WHO saw it"*) are already not
observations, so the module is where the id newtypes live, and the doc says so. **They are minted
client-side and no function returns one** (D48; architecture.md:2906).

**6. There is no `decided_at` column — `valid_from` IS the "when".** SCD2 needs `valid_from`; a
second timestamp that always equals it at this story is the speculation decision 4 refuses.
`decided_by` DOES exist and is not optional: story 5.10 runs
`TRUNCATE … WHERE decided_by='ENGINE'` literally.

**7. 🔴 There is NO `confidence` column — and the two source documents disagree about that, which is
why it is a decision and not an omission.** D14's own sentence says a link carries *"the rule
applied, the evidence, **the confidence**, when, by whom"* [architecture.md:1015]; `epics.md`'s
AC2 for this story omits `confidence` and lists `ruleset_version` in its place. **The omission is
the later and the correct one**, on D13's authority: *"REFUSED: `rule -> confidence: f64` … if the
output is a float, B has won in disguise"* [architecture.md:956-958], and its corollary
*"`confidence` is an INTEGER in milli-units (0..1000), never `REAL`/`DOUBLE`"* [architecture.md:991-993]
**binds the day a ranking value appears** — the register already owns that entry with **story 5.14**
as owner, *"the first story with a ranking surface"*. L1 is a deterministic lookup with nothing to
rank, so a `confidence` column here would be a value asserting that a ranking exists. **Record the
discrepancy in `deferred-work.md` with story 5.14 as owner; do not patch `architecture.md`** (issue
#54's precedent — a correction to a decision body is a milestone act, never a story's).

**8. `OPEN_END` is a MariaDB `DATETIME(6)` sentinel, not the architecture's ISO-8601 string.** D21
writes `OPEN_END = '9999-12-31T23:59:59.999Z'` [architecture.md:1467] — an **ISO-8601 TEXT** literal
from the two-engine era, when dates were TEXT. D64 made MariaDB the only engine and the columns are
`DATETIME(6)`, so the same instant is written `9999-12-31 23:59:59.999999`. **This is a
transposition, not a contradiction, and the constant's doc must say so** — otherwise the next
reader finds a document and a constant that disagree and reports a defect. One constant, in the
adapter, used by every query; core has no reason to name it.

**9. 🔴 The link's uniqueness key is `(observation_id, link_subject, valid_to)`, and `link_subject`
is a GENERATED column that sentinels the NULL away.** Taken by Guy at the validation pass, on a
measurement: an earlier draft wrote `UNIQUE (observation_id, valid_to)` — *"exactly one current link
per observation"* — and the gap-hunt agent showed that **refuses a legitimate write**. `join` is
`for observation { for key in keys_of(observation) { … } }` [`l1.rs:174-178`]: a multi-MAC
observation lands on **N interfaces at once**, `l1.rs:186` says so in prose, and `multi-nic` is a
committed trap family. Measured against a live 10.11.11, the second link came back
`Err(Constraint("unique"))`.

Widening the key to include `interface_id` fixes the multi-NIC case — **and re-opens D21's NULL trap
on the other side**, because `interface_id` is NULL for an abstention and MariaDB holds NULLs
distinct, so two current abstentions for one observation would both insert and AC3 would be
decorative for exactly the half FR16 exists to display. Guy accepted the trade-off knowing this; the
guard closing it is **the idiom this story already committed to for `valid_to`** — a sentinel, not a
NULL:

```sql
  link_subject  CHAR(36)  CHARACTER SET ascii  COLLATE ascii_bin  AS (COALESCE(interface_id, ABSTAINED_SUBJECT)) STORED,
```

with `ABSTAINED_SUBJECT` the nil UUID `00000000-0000-0000-0000-000000000000`, a second adapter
constant documented beside `OPEN_END` and for the same stated reason. **`interface_id` itself stays
nullable and keeps its FK to `interface(id)`** — the nil UUID is not an interface and must never be
one; only the derived column sees it. One index then carries both halves: multi-NIC inserts, two
current abstentions do not.

⚠️ **The generated column is a text column and the `ddl-collation` gate walks it** — give it
`CHARACTER SET ascii COLLATE ascii_bin` on its own single line like every other (§4).

## Acceptance Criteria

**AC1 — the three tables exist, and only those three.**
`crates/opencmdb-bin/migrations/0002_interface_and_identity_link.sql` creates `interface`,
`identity_link` and `link_candidate`. It creates **no** `device`, **no** `entity`, and adds **no**
column to the two tables `0001` created. `0001_initial.sql` is **not edited** — sqlx checksums an
applied migration, and editing one breaks every existing database.

**AC2 — the link is an ENTITY, SCD2, carrying its evidence (D14).**
`identity_link` carries, for one `(observation_id, interface_id)` placement: the **rule applied**,
the **evidence**, **when** (`valid_from`/`valid_to`), **by whom** (`decided_by ∈ {ENGINE, OPERATOR}`)
and **`ruleset_version`**. Superseding a link is an **append plus a single closing stamp** — the old
row's `valid_to` moves off `OPEN_END` and stays readable. **A test reads a superseded link back
after it was superseded**: *"a bad link is UNLINKED, never erased"* [architecture.md:1016-1017].

**AC3 — exactly one link is current per `(observation, subject)`, and TWO sentinels are what make
that true.** `valid_to` is `NOT NULL` and carries `OPEN_END` while the row is current; `link_subject`
is the generated column of decision 9, sentinelling `interface_id`'s NULL to the nil UUID; and the
index is `UNIQUE (observation_id, link_subject, valid_to)`. D21's NULL trap
[architecture.md:1462-1468] is why both sentinels exist: with a `NULL` in the key, MariaDB treats
NULLs as distinct and the constraint **never fires — it is decorative**. The criterion has **three**
assertable halves, and each has its own test:

1. **the constraint fires** — opening a second current link for the same `(observation, interface)`
   without closing the first gives `RepositoryError::Constraint("unique")`, asserted;
2. **it does not over-fire** — a multi-MAC observation legitimately holds **two** current links, one
   per interface, and both insert (decision 9; the `multi-nic` family is the case);
3. **exactly one is current** — after an attempted double-open, a **COUNT** of current links for
   that `(observation, interface)` is `1`.

⚠️ Half 3 needs its own test shape and it is not optional: the `Constraint("unique")` shape panics
at `expect_err` **before any count exists**, so it cannot carry that assertion. Attempt the second
insert with `let _ = …`, then count. Measured at validation under the corrected M5:
`assertion left: 2, right: 1 — "exactly one link is current per observation"`.

**AC4 — an ambiguous outcome is a LINK with its candidates, never an absence (D14/FR16).**
An abstained `Decision` persists as **an `identity_link` row** carrying its abstention cause, plus N
`link_candidate` rows each naming a candidate `interface_id` and its evidence. A test reads both
back and **asserts the link row is present by counting it** — not by `.expect()`ing the write.
*"The ambiguity is DATA, not a hole; otherwise there is nothing to display and FR16 is vapour"*
[architecture.md:1032-1034].

⚠️ **M4 is a TWO-PART mutation and the one-part version measures nothing.** `link_candidate` has an
FK to `identity_link(id)`, so "candidates but no link row" is simply unwritable: measured at
validation, the one-part mutation red on `write abstention: Constraint("foreign_key")` at an
`.expect(…)` — the FK carried it and AC4's assertion **was never evaluated**. Drop
`link_candidate_link_fk` **and** skip the link row; the measured red is then the real one,
`left: 0, right: 1`.

**AC5 — `interface (l2_domain, mac_canon)` is indexed and NOT unique (D21).**
*"NO unique index on `interface.mac_canon`. Deliberate. A cloned MAC = two real interfaces, same
MAC. A UNIQUE would turn the exact case we must ABSTAIN on into a 500"* [architecture.md:1470-1473].
Two `interface` rows sharing one `(l2_domain, mac_canon)` insert successfully, **asserted**, and
mutation **M2** (make it `UNIQUE`) reds that test. The corpus's `cloned-mac` family is the case;
this is its guard **in the schema**.

⚠️ Write the second insert as `.map_err(classify)` + `assert_eq!(…, Ok(()))`, **not** `.expect(…)`.
The natural `.expect()` gives a panic-carried red; the assertion form is what produced the clean
measured red at validation, `left: Err(Constraint("unique"))` against `right: Ok(())`.

**AC6 — the adapter follows D49's shape, and `opencmdb-core` gains no SQL.**
Every new query is a free function generic over `sqlx::Executor` in `crates/opencmdb-bin/src/repo.rs`,
static SQL with bound values (D48), errors classified by the existing `classify` (D47). The write
path is reached through `WriteRepository::transact`, so an identity decision cannot be split across
two transactions. `opencmdb-core` gains **only** the two id newtypes of decision 5 — no `sqlx`, no
`anyhow`; `cargo xtask ci`'s frontier gate stays green.

**AC7 — the EIGHT registered entries are dispositioned exactly as §5 says.**
`deferred-work.md` gains a section for this story: **2 closed** (both by REFUSING a serde derive,
with the reason), **3 answered-not-closed** with the measurement that says why the condition was not
met, **3 re-owned to story 5.9b** with the reason. **Append, never rewrite a bullet.** Reporting an
answered entry as closed is a defect. _(The count was seven until the validation's fact-check found
entry #8 at `deferred-work.md:1861-1871` — a distinct bullet from #7, still saying "5.9 or Epic 6".)_

**AC8 — the DDL gate and the other five stay green, and the gate was *shown* to bite.**
`cargo xtask ci` is green on the finished tree. In addition, **a text column written without a
binary collation was observed to red `ddl-collation`, and the observation is recorded** — the gate
has never been exercised by a second migration and *"a gate that cannot fall is decoration"* (D18).

**AC9 — the mutations are run WITH a database, and each red is recorded with what CARRIED it.**
Seven mutations (§ Tasks). For each, the Debug Log records: the mutation, **whether `DATABASE_URL`
was set**, which test(s) red, and **what carried the red — an assertion, an `expect`/`expect_err`
panic, or the compiler**. A compiler-carried red does not count. **A mutation reported green without
a database is reported as NOT RUN.**

⚠️ *"Assertion-carried"* was measured at validation to be unachievable for M1 and M3 as prescribed:
the `Constraint(…)` shape reds at `expect_err`, which is a panic. That is acceptable by story 5.7's
precedent, but **the Debug Log must say `expect_err` rather than "assertion"** — the distinction is
the whole point of this AC, and M2, M4, M5 and M6 do reach a real assertion.

**AC10 — the doc twins are updated in the same commit, and they agree.**
`docs/project-context.md` and `CLAUDE.md` both state what this story shipped, with the **same
numbers**: the new test count, the three tables, the 5.9/5.9b split, and that the blocker **still**
has no production caller. ⚠️ **Four of story 5.8's nineteen review defects were violations of its
own equivalent of this AC**, and the recurring shape is one twin corrected and the other missed —
so **grep both files for every sentence you change**. The pre-edit greps that are actually
load-bearing, measured at validation: **`16 stories` → 1 hit in each twin** (`CLAUDE.md:7`,
`docs/project-context.md:45`; both must become 17) and **`no production caller` → 1 in `CLAUDE.md`,
2 in `project-context.md`**. `sixteen` and `story 5.9` return **0/0** in both files — useful only as
*post*-edit checks, not as pre-checks. Also re-grep `sprint-status.yaml`, which the contexting left
half-updated once already.

---

## Tasks / Subtasks

- [ ] **T1 — branch and baseline (AC9)**
  - [ ] Branch from `master` at `28a7f51`: `story-5.9-persist-interface-and-identity-link`.
  - [ ] Start `mariadb:10.11.11` and export `DATABASE_URL` (§7). Record `cargo test --workspace
        --locked` **with** the DB set — the baseline is 367 and **the DB-backed tests now actually
        run**; note any that were silently skipping before.
  - [ ] **Commit the clean baseline before the mutation pass** (`mutation-pass-needs-committed-baseline`:
        `git checkout <file>` restores to HEAD, not to uncommitted work).

- [ ] **T2 — the two id newtypes (AC6, decision 5)**
  - [ ] `InterfaceId` and `LinkId` in `crates/opencmdb-core/src/observation/mod.rs`, via
        `uuid_newtype!`, each with a `///` doc saying what it identifies and that it is minted
        client-side (D48).
  - [ ] Re-export both from `lib.rs`'s `pub use observation::{…}` list.

- [ ] **T3 — the migration (AC1, AC2, AC3, AC5, AC8)**
  - [ ] Write `crates/opencmdb-bin/migrations/0002_interface_and_identity_link.sql`. One column per
        line; prose on its own `--` lines (§4). Header comment in the shape of `0001`'s, naming
        D14, D21 and D64.
  - [ ] `interface`: `id` PK; `l2_domain`; `mac_canon` (the lowercase colon form — `MacAddr`'s
        `Display`); `first_seen_at`/`last_seen_at` `DATETIME(6)`; a **non-unique** index on
        `(l2_domain, mac_canon)` with the `-- NOT UNIQUE, deliberately (D21)` comment and the
        reason.
  - [ ] `identity_link`: `id` PK; `observation_id`; `interface_id` (**NULL iff abstained**, FK to
        `interface(id)`); `outcome`; `rule_id`; `abstention_cause`; `evidence`;
        `ruleset_version INT UNSIGNED`; `decided_by`; `valid_from`; `valid_to NOT NULL`;
        `link_subject` **STORED generated** `= COALESCE(interface_id, ABSTAINED_SUBJECT)`, ascii_bin,
        on its own line (decision 9); and `UNIQUE (observation_id, link_subject, valid_to)`.
        ⚠️ **NOT `UNIQUE (observation_id, valid_to)`** — that key refuses the multi-NIC write, and
        it was measured refusing it (decision 9).
  - [ ] The CHECKs, each one the DDL-level echo of a type-level property:
        `outcome IN ('match','no_match','abstained')` · `decided_by IN ('ENGINE','OPERATOR')` ·
        **rule XOR cause** (`abstained` ⇒ `rule_id IS NULL AND abstention_cause IS NOT NULL`;
        otherwise the reverse) — this is `Decision::rule()` returning `None` exactly for an
        abstention, expressed in the schema · **`interface_id IS NULL` iff `outcome='abstained'`**.
  - [ ] `link_candidate`: `(link_id, interface_id)` PK, `evidence`, FK to `identity_link(id)` and
        to `interface(id)`.
  - [ ] Run `cargo xtask ci` and confirm `ddl-collation` green; then **temporarily strip one
        `COLLATE`, observe the gate name the offending line, restore** (AC8).

- [ ] **T4 — the adapter (AC2, AC4, AC6, decisions 3, 6, 8)**
  - [ ] `OPEN_END` constant with the doc of decision 8 (the ISO→`DATETIME(6)` transposition), and
        `ABSTAINED_SUBJECT` (the nil UUID) beside it with the doc of decision 9 — **both are
        sentinels closing D21's NULL trap, and their docs must say so in the same words**, because
        a reader who meets only one of them will read it as an oddity rather than an idiom.
  - [ ] Token mappings: `Conclusion → outcome` and `IdentityAbstentionCause → abstention_cause`,
        **exhaustive `match`, no `_` arm**, each with a unit test that pins every token string.
  - [ ] Query bodies, generic over `Executor`: `insert_interface`, `insert_identity_link`,
        `close_identity_link`, `insert_link_candidate`, **`load_current_links_for_observation`**
        (PLURAL — decision 9: a multi-MAC observation has several current links, and a singular
        name would encode the constraint the arbitration removed), `load_link_candidates`,
        `count_identity_links`.
  - [ ] `insert_identity_link` takes the `Decision` and derives `outcome`/`rule_id`/
        `abstention_cause`/`evidence`/`ruleset_version` from it — **one call site cannot get the
        rule-XOR-cause pairing wrong**, which is what makes the DDL CHECK a second line of defence
        rather than the only one.
  - [ ] ⚠️ `valid_from` is a **parameter**, never `NOW(6)` (§6.3). `first_seen_at`/`last_seen_at`
        likewise, with the doc saying they must be derived from observations.

- [ ] **T5 — the tests (AC2–AC5), all `DATABASE_URL`-gated and under `DB_TEST_LOCK`**
  - [ ] a match link round-trips: interface + link written in one `transact`, read back current.
  - [ ] **SCD2**: supersede a link; exactly ONE current row; the superseded row is still readable
        with its old `valid_to` (AC2).
  - [ ] **the unique index fires**: a second current link for the same `(observation, interface)` →
        `Constraint("unique")` (AC3, half 1). Red carried by `expect_err` — record it as such.
  - [ ] 🔴 **exactly ONE is current, by COUNTING** (AC3, half 3) — the test T5 did not have. Attempt
        the second insert with `let _ = …` (never `expect_err`, which panics before any count),
        then `assert_eq!(current_links(obs, iface).len(), 1, "exactly one link is current per
        observation")`. **Without this test M5 has nothing to red**, which is what the validation
        measured.
  - [ ] 🔴 **the index does NOT over-fire** (AC3, half 2): one observation, two MACs, two interfaces
        → **two** current links, both insert. `assert_eq!(second, Ok(()))`, not `.expect()`. This is
        the `multi-nic` case and the reason for decision 9.
  - [ ] 🔴 **two current abstentions for one observation are refused** — the other half of decision
        9. Without `link_subject` the widened key holds two NULLs distinct and both insert.
  - [ ] **ambiguity is a LINK**: abstained link + 2 `link_candidate` rows; both read back with their
        evidence; **the link row is asserted present by COUNT**, not by `.expect()`ing its write
        (AC4) — the `.expect()` form lets the FK carry M4's red instead of the assertion.
  - [ ] **the CHECKs fire**: a `match` with `interface_id` NULL → `Constraint("check")`; an
        `abstained` carrying a `rule_id` → `Constraint("check")`; `decided_by='SCANNER'` →
        `Constraint("check")`.
  - [ ] **no unique on the L1 key**: two interfaces, same `(l2_domain, mac_canon)`, both insert
        (AC5) — `.map_err(classify)` + `assert_eq!(…, Ok(()))`, not `.expect()`.
  - [ ] evidence round-trips as `Vec<ObsId>` byte-identically.
  - [ ] the token mappings are exhaustive and every token is pinned (no DB needed).

- [ ] **T6 — prove-to-red (AC9). Every mutation run WITH `DATABASE_URL` set.**
      ⚠️ **Five of these seven were run at the validation pass against a live 10.11.11, and three
      of them did not behave as the story first prescribed.** The corrected forms are below; the
      measured reds are in the Debug Log table as PREDICTIONS the dev confirms or refutes.
  - [ ] **M1** drop `UNIQUE (observation_id, link_subject, valid_to)` → the second-current-link test
        must red. *Measured: reds at `expect_err`, panic-carried. Record it as `expect_err`.*
  - [ ] **M2** make `interface (l2_domain, mac_canon)` `UNIQUE` → the cloned-MAC test must red (AC5).
        *Measured: assertion-carried, `left: Err(Constraint("unique"))`.*
  - [ ] **M3** drop the rule-XOR-cause CHECK → the CHECK test must red (AC2). *Measured: reds at
        `expect_err`, panic-carried.*
  - [ ] 🔴 **M4 — TWO PARTS, and the one-part version is a NO-OP.** Drop `link_candidate_link_fk`
        **and** write the abstention as an absence (candidates, no link row) → the ambiguity test
        must red on its COUNT. *Measured: the one-part version reds on
        `Constraint("foreign_key")` at `.expect(…)` — the FK carries it and AC4's assertion is never
        evaluated. The two-part version reds `left: 0, right: 1`.* **If the two-part version does
        not red, FR16 is still vapour and that is the finding.**
  - [ ] 🔴 **M5 — TWO PARTS, and the one-part version is not executable.** Replace the `OPEN_END`
        sentinel with `NULL` **and** drop `valid_to`'s `NOT NULL` → the COUNTING test of AC3 half 3
        must red. *Measured: with only the sentinel changed, error **1048 `Column 'valid_to' cannot
        be null`** kills the FIRST insert and the "exactly one current" question is never reached.
        With both edits, D21's trap appears exactly as written — the second current insert
        SUCCEEDS — and the count assertion reds `left: 2, right: 1`.*
  - [ ] **M6** change one persisted token string (e.g. `abstained` → `abstain`) → the token test must
        red. *Measured: assertion-carried, `left: "abstain" / right: "abstained"`.*
  - [ ] 🔴 **M7 — NEW, decision 9's other half.** Drop the `link_subject` generated column and key
        the index on `(observation_id, interface_id, valid_to)` directly → the "two current
        abstentions are refused" test must red, because MariaDB holds the two NULL `interface_id`s
        distinct. **If it stays green, the abstention half of AC3 is decorative** — which is the
        exact trade-off Guy accepted decision 9's sentinel to close.
  - [ ] Record for each: DB set yes/no, tests red, **and what CARRIED the red — assertion,
        `expect`/`expect_err` panic, or compiler**.

- [ ] **T7 — register and docs (AC7, AC10)**
  - [ ] Append this story's section to `deferred-work.md`: 2 closed, 3 answered, **3 re-owned**
        (§5 #6, #7 and #8), plus the three new entries from decision 4 (`entity`, `device`,
        `state`), decision 2 (the vector is not stored) and decision 7 (the `confidence`
        discrepancy between `architecture.md:1015` and `epics.md`, **owner story 5.14**).
  - [ ] Update `docs/project-context.md` **and** `CLAUDE.md` with the same numbers (AC10), then grep
        both for the phrases AC10 names.

- [ ] **T8 — the full local gate, then PR**
  - [ ] `cargo fmt --all` · clippy **twice** (§8) · `cargo test --workspace --locked` **with the DB
        running** · `cargo xtask ci`.
  - [ ] Push the branch, open the PR, wait for green CI, **squash merge**. Never push to `master`.

---

## Dev Notes

### Existing shapes to follow, not reinvent

- **The whole adapter idiom** is in §3. Read `crates/opencmdb-bin/src/repo.rs` end to end before
  writing a line — it is 342 lines and it already answers every structural question this story
  raises.
- **The DDL idiom** is `0001_initial.sql`, 34 lines. Copy its column layout and its comment
  placement exactly; §4 explains why the placement is load-bearing.
- **`MacAddr`'s `Display`** [`observation/mod.rs:83`] is lowercase colon-separated. That IS
  `mac_canon`; do not write a second canonicalisation. `MacAddr` is `[u8; 6]` deliberately —
  *"device identity is compared byte-exact"* — and the column is `ascii_bin`, which is the same rule
  one layer down (D64).
- **`Decision::rule()`** [`cascade.rs`] returns `None` exactly for an abstention. The rule-XOR-cause
  CHECK is that property expressed in DDL; write the insert so it derives both from the same
  `match`, and the CHECK becomes a second line of defence instead of the only one.

### Compile-level facts — each costs an hour if it is discovered under `rustc`

- `uuid_newtype!` is a bare `macro_rules!` in `observation/mod.rs:24` with **no** `#[macro_export]`
  and **no** `pub(crate) use`. It is not reachable from `identity/`. That is why decision 5 puts the
  two newtypes in `observation/mod.rs`.
- `RuleId` is `pub struct RuleId(pub String)` [`trap.rs:42`], `#[serde(transparent)]`. `.0` is the
  `String`; bind it directly.
- `ObsId` and friends already derive `Serialize`/`Deserialize`, so `serde_json::to_string(&Vec<ObsId>)`
  works with no new derive — **that is the whole reason `evidence` needs none of the four types
  §5 #2 refuses derives for.**
- `RulesetVersion` is `pub struct RulesetVersion(pub u32)`; bind `.0`. sqlx maps `u32` to
  `INT UNSIGNED`.
- `sqlx::query_as` returns tuples; `Option<String>` for a nullable column. `interface_id`,
  `rule_id` and `abstention_cause` are all nullable — a non-`Option` binding panics at decode.
- `chrono::DateTime<Utc>` formats for MariaDB as `"%Y-%m-%d %H:%M:%S%.6f"` [`repo.rs:142-145`] —
  reuse that, do not invent a second format.
- `MariaUnit::executor()` is `pub(crate)` [`repo.rs:33`]; the query bodies take
  `E: Executor<'e, Database = MySql>` and both the pool and `unit.executor()` satisfy it.
- ⚠️ **sqlx 0.9 REJECTS `sqlx::query(&format!(…))` at compile time** — *"dynamic SQL strings should
  be audited for possible injections"*. Met under `rustc` at the validation pass. It bites on the
  per-table `DELETE` cleanup T5's tests need: write one static `DELETE` per table, not a loop over
  table names. (This is D48 enforced by the library rather than by review.)
- **All the other compile-level facts above were re-verified under a compiler at validation and
  hold** — `RuleId(pub String).0`, `RulesetVersion(pub u32).0`, sqlx `u32` ↔ `INT UNSIGNED`, the
  chrono format string, `MariaUnit::executor()`, `Option<String>` decode for nullable columns, and
  decision 5's claim: injecting `uuid_newtype!` into `identity/l1.rs` gives
  `error: cannot find macro uuid_newtype in this scope`.

### What a reviewer will challenge, and the answer that is already measured

| challenge | answer |
|---|---|
| *"Why is there no `device` / `entity`?"* | Decision 4 + `epics.md:1576`. Owner named: Epic 6. |
| *"D21 says the supertype is enforced by the engine, not by convention — you dropped it."* | It is **deferred with `device`, not dropped**. A supertype over one subtype enforces nothing. |
| *"You store no `verdict_vector`, but D18 requires the complete vector."* | D18's sentence is about the **trap harness** (`ScoredRecord`), not the link. D14's list of what a link carries does not include it. Decision 2. |
| *"`RulesetVersion` still has no `Ord` and you owned that entry."* | §5 #3: the prediction was that persistence orders versions. **Measured: it does not** — it stores and reads one. Answered with the measurement, not closed. |
| *"`decided_at` is missing."* | Decision 6: `valid_from` is the "when"; a second always-equal timestamp is speculation. |
| *"D14 says a link carries the CONFIDENCE and you have no such column."* | Decision 7: D13 refuses `rule -> confidence: f64` outright, its milli-unit corollary binds *"the day a ranking value appears"*, and L1 has nothing to rank. `epics.md`'s AC2 already omits it. Registered with story 5.14. |
| *"The `OPEN_END` constant does not match `architecture.md`."* | Decision 8: ISO-8601 TEXT is the two-engine era; D64 made the column `DATETIME(6)`. Same instant, transposed, and the constant's doc says so. |
| *"The blocker still has no production caller."* | True, by decision. §1 and §5 #7 — story 5.9b. |
| *"`UNIQUE (observation_id, link_subject, valid_to)` is over-engineered — one observation, one link."* | Decision 9, and it is **measured**: `join` inserts an observation under every key it carries [`l1.rs:174-178`], `l1.rs:186` says an observation may carry several MACs, `multi-nic` is a committed trap family, and the narrow key was observed returning `Err(Constraint("unique"))` on a legitimate second link. |
| *"A generated column is a clever trick."* | It is the **same** idiom as `OPEN_END`, applied to the same trap on the other column. D21's NULL trap is about NULLs in a unique key; `valid_to` closes it with a sentinel and `interface_id` closes it with a sentinel. One idiom, twice, both documented in the same words (T4). |

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 5.9`] — the four ACs, the SPLIT note and
  the link-subject arbitration added at this contexting.
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 5.9b`] — what this story deliberately
  leaves to the next one.
- [Source: `_bmad-output/planning-artifacts/architecture.md:1013-1049`] — **D14**, the link as an
  SCD2 entity, `AMBIGUOUS` as a link, the purge test, `ruleset_version` mandatory.
- [Source: `_bmad-output/planning-artifacts/architecture.md:1450-1479`] — **D21**: the supertype,
  the NULL trap and `OPEN_END`, the refusal of a unique index on `interface.mac_canon`, the extended
  `entity.state`.
- [Source: `_bmad-output/planning-artifacts/architecture.md:1434-1446`] — read-your-own-writes, the
  transaction unit, *"an identity decision is NEVER split across two transactions"* (5.9b's, quoted
  here because AC6's `transact` is what will make it possible).
- [Source: `_bmad-output/planning-artifacts/architecture.md:2658`] — **D48**, opaque ids
  `CHAR(36) ascii_bin`.
- [Source: `_bmad-output/planning-artifacts/architecture.md:4377`] — **D64**, MariaDB only, binary
  collation on every text column.
- [Source: `_bmad-output/planning-artifacts/prd.md:896-897`] — **FR16 / FR16b**.
- [Source: `crates/opencmdb-bin/src/repo.rs`] — the adapter idiom, `classify`, the DB-test shape.
- [Source: `crates/opencmdb-bin/migrations/0001_initial.sql`] — the DDL idiom.
- [Source: `xtask/src/main.rs:307-367`] — the `ddl-collation` gate, verbatim.
- [Source: `crates/opencmdb-core/src/identity/cascade.rs`] — `Decision`, `Conclusion`,
  `IdentityAbstentionCause`, and the three registered residues §5 dispositions.
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md`] — the **eight** entries naming
  story 5.9; #8 is at `:1861-1871`.

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

#### The database run (AC9) — filled in by the dev

The **prediction** column is what the validation pass measured against a live `mariadb:10.11.11`
on an independent implementation. It is there to be **confirmed or refuted**, not copied: a dev who
records a prediction without running the mutation has recorded nothing, and a refutation is a
finding worth more than a confirmation.

| mutation | DB set? | tests red | carried by (assertion / `expect_err` / compiler) | prediction from validation |
|---|---|---|---|---|
| M1 | | | | reds; `expect_err`, panic-carried |
| M2 | | | | reds; **assertion**, `left: Err(Constraint("unique"))` |
| M3 | | | | reds; `expect_err`, panic-carried |
| M4 (two parts) | | | | reds; **assertion**, `left: 0, right: 1`. One-part version = NO-OP (FK carries it) |
| M5 (two parts) | | | | reds; **assertion**, `left: 2, right: 1`. One-part version = not executable (err 1048) |
| M6 | | | | reds; **assertion**, `left: "abstain" / right: "abstained"` |
| M7 | | | | untested at validation — decision 9 postdates the run |

#### The `ddl-collation` gate, shown to bite (AC8)

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-08-03 | Story contexted. **SPLIT with Guy: 5.9b INSERTED** (Epic 5 → 17 stories) — the resolver that runs the engine over a set of observations and writes the links is its own story, because it is the first production caller of `join`/`candidates` and it is what story 5.10 re-runs. **Second arbitration: an `identity_link` binds `observation → interface`** (`join` forms an interface at L1; `decide_pair` judges a pair and returns none). `epics.md` and `sprint-status.yaml` updated. |
| 2026-08-03 | **VALIDATED** by two fresh-context agents, the gap-hunt one against a live `mariadb:10.11.11` (374 green tests, six green gates, all six mutations executed against real DDL). **4 HIGH + 2 HIGH applied.** 🔴 **Third arbitration, Guy's, at the validation pass: the link's uniqueness key widens to `(observation_id, link_subject, valid_to)`** — the narrow `(observation_id, valid_to)` was measured REFUSING a legitimate multi-NIC write, and its abstention half is closed by a generated-column sentinel (decision 9, new AC3, new M7). Also: M4 and M5 were both no-ops as first written and are now two-part mutations with their measured reds; AC3 gained the COUNTING test that alone can carry M5's red; §5 undercounted the register by one entry (AC7: seven → **eight**); §7's `docker run` would have migrated an unrelated project's MariaDB 11 (port 3306 → **13306**); and five smaller claims were corrected against measurement (`0001` is 34 lines not 35, `repo.rs` is 236+106, `sqlx` appears in four files not one, D13's word is `join` not `lookup`, the tree was never clean). |
