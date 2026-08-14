# Story 6.2: The route writes a declared value, through the adapter and nowhere else

Status: ready-for-dev

<!-- ⚠️ CONTEXTED 2026-08-14, the same day story 6.1 merged (PR #89 → `664693b`, docs flipped by
     PR #90 → `3f7069a`). The tree this story extends is master at `3f7069a`: 566 tests
     (344 bin + 160 core + 62 xtask), seven gates green, trap gate RED at 26/15/11 by design.
     Validation (MANDATORY, two fresh layers) has NOT run yet — `ready-for-dev` means only what
     the vocabulary says, "a story file exists". -->

## Story

As the operator,
I want an observed value to become a declared one,
So that what the product found becomes what I documented.

**And this is the story where milestone J3's second half becomes measurable for the first
time**: J3 wants a real gap *detected AND corrected*, the detection has existed since v0.1, and
the correction has had no code path at all — five routes, all read-only, the only
`insert_declared_attribute` call outside `repo.rs` sitting inside a `#[cfg(test)]`. After this
story, POST `/document-all` writes the declared record, and a reconcile pass over the documented
entity shows the gap CLOSED (§7).

---

## What this story does NOT do

- it does **not** put a button on the page — story 6.4, after Epic 6b. A diff touching
  `crates/opencmdb-bin/templates/` is a FINDING;
- it does **not** ship NFR5's two parked assertions (bit-for-bit observation, the re-assert
  through the new path) — **story 6.3 exists for exactly that** and follows immediately. This
  story writes; 6.3 proves the write could not have overwritten. Do not preempt its
  deliverable: a basic "the observation row still exists" sanity is fine, the bit-for-bit
  measurement is 6.3's;
- it does **not** implement `document-field` — FR13(b), Epic 7's;
- it does **not** create an `entity` table, a `device`, or a `state` column — story 6.5's
  schema story. `declared_attribute.entity_id` is a plain CHAR(36) with no FK today, and a
  freshly-minted entity is N attribute rows sharing one id, exactly like the walking skeleton's
  seeded entities;
- it does **not** touch `/metrics`, `is_public`, or the Basic mechanism — story 6.1 settled the
  auth surface; this story adds ONE refusal in front of ONE route (the CSRF check, §5) and
  changes nothing else in `auth.rs`;
- it does **not** edit `epics.md` — and it must NOT, because §1 records a divergence from the
  epic's own AC wording;
- it does **not** add a crate. The lockfile is untouched.

---

## 1. 🔴 The schema already names this gesture, and the epic's AC letter is WRONG about the fn

The epic's AC says the rows are written *"through `insert_declared_attribute` and through no
other path, and `SANCTIONED_SITES` gains exactly one entry"*. **Those two clauses contradict
each other, and the schema resolves the contradiction**: `insert_declared_attribute` is ALREADY
a sanctioned site — writing through it would add ZERO entries — and it writes
`origin = 'manual'` with **no `origin_obs_id` parameter at all**, while `0001_initial.sql`
carries the documenting gesture's vocabulary since day one:

```sql
origin        VARCHAR(16) ... NOT NULL, -- manual|adopted|imported
origin_obs_id CHAR(36)    ...,          -- the adopted observation
CONSTRAINT declared_adopted_has_obs CHECK (origin <> 'adopted' OR origin_obs_id IS NOT NULL),
```

**A documented value is an ADOPTED value.** Writing it as `'manual'` would falsify the schema's
own vocabulary and throw away the one provenance link (`origin_obs_id`) that records WHICH
sighting was adopted — the very column xtask's `PROVENANCE_COLUMNS` doc calls *"the adopted
observation"*.

**The resolution, and the story's central prescription:**

- a NEW adapter fn in `repo.rs`, beside the existing one and on its exact idiom:
  **`adopt_declared_attribute(executor, entity_id, attr_key, attr_value, origin_obs_id)`** —
  static SQL, bound VALUES for the four data parameters, and **`'adopted'` and `'operator'` as
  SQL LITERALS** (story 5.12's structural constraint: there is no actor parameter to pass a
  non-human author through — NFR5's third assertion holds by the signature);
- **`SANCTIONED_SITES` gains exactly one entry**:
  `("crates/opencmdb-bin/src/repo.rs", Some("adopt_declared_attribute"))` — named, never a
  blanket exemption. The array's type is `[(&str, Option<&str>); 3]` today and becomes `; 4`;
- 🔴 **the epic divergence is REGISTERED, not edited**: `epics.md:1758`'s *"through
  `insert_declared_attribute`"* reads as *"through the repo adapter"* — the corrected sentence
  is *"through `adopt_declared_attribute`, `insert_declared_attribute`'s sibling"*. Owner of
  the record: this story's §Register row (1); `epics.md` untouched (verify-only rule).

⚠️ `actor_id` stays the literal `'operator'` — the Basic pair authenticates a CALLER, not a
person (story 6.1 §3), so inventing a per-user actor here would claim an identity the auth
layer cannot supply. Epic 19 (real sessions) owns actors; registered.

---

## 2. The port: one method, the whole gesture, one transaction

Story 6.1 shipped `SubjectLookup` (*"it can look, never write"*) with `AlwaysUnknown` as the
production wiring, and its doc PROMISED: *"story 6.2 replaces it with the store-backed impl."*
This is that replacement — and the shape matters, because 6.1's central carrier must survive:

> 🔴 **`DocumentState` still holds NO pool field, and M4 (add a `State<MySqlPool>` parameter to
> the handler) must STILL fail to compile** — re-measure it, record the actual diagnostic
> (6.1 measured `error[E0277]` on the `Handler` bound; a carrier named without re-measurement
> is how 6.1 nearly shipped a false claim).

**The trait grows into the gesture's port** (rename `SubjectLookup` → `DocumentPort`; its
"never write" doc sentence is RETIRED WITH the rename — a trait that writes under a doc saying
it never writes would be the false-doc defect this project hunts):

```rust
pub(crate) trait DocumentPort: Send + Sync {
    /// Perform the WHOLE documenting gesture for `subject`, atomically:
    /// check the subject exists → check it was not already adopted → project its facts →
    /// mint an entity → write the adopted rows. One transaction: a check that commits
    /// separately from its write is a TOCTOU hole, not a check.
    fn document_all(&self, subject: ObsId) -> BoxFuture<'_, Result<Documented, DocumentFailure>>;
}

/// What a successful gesture produced: the minted entity and how many fields were written.
pub(crate) struct Documented { pub entity_id: String, pub fields: usize }

/// Why it did not happen: a domain refusal (each mapped to its status, exhaustively,
/// no `_` arm) or a backend failure (500 — sqlx stays in bin, D47).
pub(crate) enum DocumentFailure { Refused(DocumentRefusal), Backend(sqlx::Error) }
```

- the production impl is **`StoreDocument { pool: MySqlPool }`** — the pool lives INSIDE the
  impl struct, behind `Arc<dyn DocumentPort>`, which is why the handler still cannot extract
  it (the M4 carrier). `main::app` constructs it: `document::router_with(Arc::new(
  StoreDocument { pool: pool.clone() }))` — the merge stays ABOVE `.layer()` (6.1 §2's rule,
  M9 unchanged);
- `AlwaysUnknown` is **deleted**, and its pin test with it — replaced by the store-backed
  tests. The in-memory test impl (6.1's `KnowsOne` shape) becomes an in-memory `DocumentPort`
  for the handler-level tests that need no database;
- the impl's transaction calls ONLY `repo` fns: a new read
  `repo::load_observation_facts_by_id(executor, &obs_id) -> Result<Option<Vec<Fact>>, _>`
  (none exists — `load_observation_facts` loads ALL rows), the already-adopted check (a read
  over `origin_obs_id`), then `repo::adopt_declared_attribute` once per field. **The SQL
  write exists in `repo.rs` and nowhere else** — that is what the authorship gate measures;
- the observation id is bound as **`subject.as_uuid().to_string()`** — canonical hyphenated
  lowercase, which is how `insert_observation` stored it. 🔑 **This sentence CLOSES the
  canonical-UUID-form question 6.1's review registered with this story**: a braced, urn: or
  hyphenless spelling of a REAL id parses to the same `Uuid` and is formatted canonical before
  any SQL sees it, so the wide parse is harmless BY CONSTRUCTION — carried by a test that
  documents a real subject through a braced spelling (§8).

---

## 3. The projection is core's, and it must be SHARED — or the gap never closes

`opencmdb_core::gap`'s **private** `fn project(&Observation) -> Vec<(String, String)>` is the
one place the vocabulary bridge lives: `Fact::IpV4 → "ipv4"`, `Hostname → "hostname"`,
`Mac → "mac"`, everything else (Rtt, DhcpLease, Uplink, OuiVendor) deliberately ignored.

**The documenting write MUST use the same projection.** A second mapping in bin would let the
two drift — a field documented under a key the reconcile does not compare, and the gap the
operator just closed stays open on the page. **Make `project` `pub`** with a rustdoc naming
both consumers (the reconcile and the documenting gesture) and the drift this sharing prevents.

⚠️ This is a change in `opencmdb-core` — a VISIBILITY change, no behaviour: state the promise
at that width (story 5.13b's lesson: *"a promise of non-modification protects behaviour and
shelters false sentences"* — promise "no behaviour change in core", never "core untouched").
DRY note: this is mutualisation of LOGIC (one source of truth for the field vocabulary), which
the house DRY rule commands; do not "balance" it with a copied table in a test — the right
second oracle is a test asserting the documented KEYS equal the projection of the observation's
facts through the real fn.

**Multi-value facts, decided now so the dev does not decide it mid-debug**: an observation can
carry two IPv4 facts (or two hostnames) and `declared_attribute`'s PK is `(entity_id,
attr_key)` — two rows under one key cannot both insert. **First occurrence wins, per key, and a
test pins it** — `project` preserves fact order, the adapter loop inserts the first and SKIPS
the rest of that key (never `Err`: a multi-homed device is normal, not a refusal). The count in
`Documented.fields` is the number of rows WRITTEN, not of facts seen; the skipped-duplicate
case carries its own test.

---

## 4. The refusal taxonomy, v2 — every row of 6.1's table survives, three rows are new

| refusal | status | where decided | discriminator |
|---|---|---|---|
| no/wrong credential | **401** + challenge | `auth_deny` (UNCHANGED) | the challenge header |
| 🆕 cross-origin browser request | **403** | the CSRF check, first thing in the handler (§5) | pinned body naming the origin refusal; **never** `WWW-Authenticate` |
| malformed form / non-UUID / nil UUID | **422** | handler (UNCHANGED from 6.1) | the "expected form field `subject`" body |
| unknown subject | **404** | `DocumentRefusal::UnknownSubject` — **now store-backed truth**: the id really names no `observation_record` row | the pinned body (6.1's decision (b) text kept: "unknown subject: nothing can be documented" — still true, and now for the right reason; Guy may refine wording at validation) |
| 🆕 already documented | **409** | `DocumentRefusal::AlreadyDocumented` — the subject was adopted before | pinned body; **this is the epic's title enforced**: documenting twice is counting one box twice |
| 🆕 nothing to document | **422** | `DocumentRefusal::NothingToDocument` — the projection of the subject's facts is empty (e.g. an Rtt-only observation) | pinned DOMAIN body, distinct from the shape 422's |
| backend failure | **500** | `DocumentFailure::Backend` | logged; body does not leak the SQL error |
| success | **201 Created** | the write happened | non-empty body naming the minted entity id and the field count |

- `DocumentRefusal` (core) grows two variants. **The handler's match is exhaustive with no `_`
  arm** (story 5.3's precedent, already installed in 6.1) — adding the variants without
  mapping them is `error[E0004]`, which is the design working;
- refusal ORDER inside the transaction: unknown → already-documented → nothing-to-document →
  write. Each order-dependent pair carries a test (an unknown subject that would ALSO be
  "already documented" cannot exist — but an already-documented subject with an empty
  projection CAN, and must answer 409, not 422);
- the 404 CANNOT collide with the fallback's empty 404 (6.1 §6's discriminator, unchanged) and
  the two 422s discriminate by BODY.

---

## 5. 🔴 CSRF — this story OWNS it (6.1's register row (j)), and the mechanism is the Origin check

6.1 measured and recorded: Basic is AMBIENT AUTHORITY — once the browser holds the credential
it attaches it to a cross-site form POST too, and 6.1's bench probed only same-origin
initiators. The route now has an effect to forge, so the protection lands HERE:

**Mechanism** — a pure fn in `document.rs`, called FIRST in the handler (before the form is
even parsed — a forged request must not exercise the parser):

- **`Origin` header absent → PASS.** A machine caller (`curl -u`) sends none; the threat model
  is a BROWSER carrying ambient credentials, and every current browser sends `Origin` on every
  cross-site POST. ⚠️ Stated limit, registered: a pre-2020 browser that omits `Origin` on a
  cross-site form POST is not protected — acceptable for a LAN single-operator product, and
  Epic 19's session+token closure supersedes it;
- **`Origin: null` → REFUSE** (sandboxed iframes, some redirect chains — no legitimate
  same-origin caller sends it);
- **`Origin` present → parse its authority (`host[:port]`) and compare, ASCII
  case-insensitively, against the request's `Host` header.** Match → pass; mismatch → 403.
  Behind the reverse proxy the browser's `Host` and its `Origin` authority agree by
  construction (same URL bar). ⚠️ Known edge, stated: default-port elision (an `Origin` of
  `http://nas` against a `Host` of `nas:80`) is compared literally and would refuse — pin the
  behaviour with a test and the limit in the doc rather than normalising ports speculatively;
- **the 403 carries a pinned body and NEVER `WWW-Authenticate`** — it is not an auth failure,
  and arbitration 6's discipline (no advertising the scheme where it is not the answer)
  extends here;
- **placement: in the HANDLER, not a layer.** A `route_layer` on the sub-router would work,
  but 6.1 §2 exists because layer/registration ordering was measured treacherous once already;
  a first-line call in the handler is order-proof, testable pure AND end-to-end.

**Measured, not assumed — the validation bench (register row (a) lands here too):** the
two-browser bench (Firefox AND Chromium — the Chromium half is the debt 6.1's review
registered) must record, server-side: (1) an `hx-post` from the served page carries
`Origin` equal to the page's own origin — the happy path survives the check; (2) a cross-site
form POST from an attacker page carries the CACHED Basic credential (the threat is REAL) AND an
`Origin` naming the attacker — the check refuses it 403. ⚠️ If probe (1) shows a browser
omitting `Origin` on same-origin XHR, that browser's users cannot document — the check must
then be revisited BEFORE dev, which is why this measurement belongs to validation, not to the
implementation's own test run.

---

## 6. The migration: `0005_document_guards.sql` — the race the check cannot hold

The 409 check (a `SELECT` over `origin_obs_id`) gives the friendly refusal, but two concurrent
`document-all` of one subject can both pass it and mint TWO entities — one box counted twice,
through the very gesture named after not doing that. Story 5.9's precedent (the guard above the
DDL for the message, the DDL for the invariant):

```sql
CREATE UNIQUE INDEX declared_one_adoption_per_field
    ON declared_attribute (origin_obs_id, attr_key);
```

- `origin_obs_id` is NULL on every `'manual'` and `'imported'` row, and MariaDB holds NULLs
  distinct — **D21's NULL-distinctness used DELIBERATELY a third time** (5.9's review's
  precedent): manual rows never collide, adopted rows collide exactly when one observation's
  field is adopted twice;
- the loser of the race gets `Err(Constraint("unique"))` through the adapter — map it to the
  same 409 (`AlreadyDocumented`), tested with two raw sequential adoptions of one subject
  (the *"first writer owes its guards"* rule, and the guard must be shown to BITE: a test
  inserts the second adoption through raw SQL and asserts the refusal, on 5.9's M3 lesson —
  a CHECK the adapter cannot violate is measured through raw SQL or it is measured by nothing);
- 🔴 **the raw SQL for that test MUST go through `raw_declared_write_for_ddl_test`** —
  `repo.rs`'s test module, the authorship gate's ONE sanctioned raw writer (story 5.12: a
  `#[cfg(test)]` blanket hole was measured hiding a planted write, so the exemption is one
  NAMED fn). Today it hardcodes `'hostname'/'nas'/'manual'` and has no `origin_obs_id`
  parameter: **WIDEN its signature** (attr_key, origin, origin_obs_id as parameters — same
  name, same site, so `SANCTIONED_SITES` still gains only the one production entry and the
  epic's "exactly one" holds). Its doc says the name is load-bearing; keep the name;
- ⚠️ Epic 7's `document-field` re-documents a DRIFTED field from a NEWER observation — a
  DIFFERENT `origin_obs_id`, no collision. The index does not pre-block Epic 7; say so in the
  migration comment so nobody widens it "for later".

⚠️ The DDL gates apply: binary collation grep (D64) is satisfied by an INDEX (no new column);
run `cargo xtask ci` after writing the migration, not only at the end.

---

## 7. J3's second half, measured end-to-end for the first time

One DB-backed test is this story's reason to exist, and it goes through the FULL stack:

1. seed one observation (ipv4 + hostname, the drifting pair from `index_renders_the_real_gap`)
   with NO declared record — the day-one case, FR13(a)'s own words;
2. POST `/document-all` through `app(pool, config)` with the valid credential and same-origin
   headers → **201**;
3. read the declared side: N rows, `origin = 'adopted'`, `origin_obs_id` = the subject,
   `actor_id = 'operator'`, values equal to the projection;
4. run the reconcile view over the documented entity → **the entity is present and carries NO
   divergence on the documented fields** — the gap the product would have shown is CLOSED by
   the gesture. *(The divergence computation never consults the provenance columns — xtask's
   `PROVENANCE_COLUMNS` gate holds that half already; do not re-test the gate, test the VIEW.)*

⚠️ This test is `DATABASE_URL`-gated like every DB test (it self-skips locally without the
container) — **a green local suite says nothing; run it against the live MariaDB** (13316's
container from 6.1 is still up) and CI re-proves it.

---

## 8. What must be pinned

- **the whole 6.1 surface survives** — every 6.1 test stays green UNCHANGED except: the
  `AlwaysUnknown` pin (deleted with its subject), the known-subject 501 test (the branch no
  longer exists — its replacement asserts 201 + the write), and the trait-rename mechanical
  edits. Anything else red in `document.rs`/`main.rs`/`auth.rs` tests is a FINDING;
- **M4 re-measured** on the new `DocumentState` (compile refusal, actual diagnostic recorded);
- **the authorship gate carries the new site**: (a) with the entry present, `cargo xtask ci`
  green; (b) REMOVE the entry → the gate REDS naming `adopt_declared_attribute` (the epic's
  own AC2, as a mutation); (c) move the SQL write into `document.rs` → the gate REDS
  (unsanctioned site). (b) and (c) are the story's authorship mutations;
- **the canonical-form closure**: a braced spelling (`{uuid}`) of a REAL subject documents
  successfully (201) — the wide parse is harmless because SQL only ever sees the canonical
  format (§2);
- **the multi-value rule**: two IPv4 facts → one `ipv4` row, first wins, `fields` counts rows
  written (§3);
- **every refusal reachable through the real port** (in-memory impl for handler-level, store
  impl for DB-level) — *a guard placed where the defect cannot occur reads as coverage and is
  none* (Epic 5's dominant class, nine occurrences);
- **the CSRF check**: pure tests (absent/null/match/mismatch/case/port-elision) AND end-to-end
  (a cross-site `Origin` on a fully-authenticated, well-formed request → 403, nothing written
  — the write-nothing half asserted through the port with a counting impl or the DB).

---

## Acceptance Criteria

**AC1 — the write exists, is `adopted`, and goes through the adapter and nowhere else.**
**Given** a seeded observation and an authenticated, same-origin, well-formed request naming it
**When** `POST /document-all` runs
**Then** it answers **201** with a body naming the minted entity and the field count; the
declared side carries one row per projected field with `origin='adopted'`,
`origin_obs_id=subject`, `actor_id='operator'`; and the SQL write exists ONLY in
`repo::adopt_declared_attribute`, sanctioned by exactly ONE new named `SANCTIONED_SITES` entry.
_Reddened by: M1, M2, M2b (gate), M3._

**AC2 — the refusal taxonomy is total, ordered, and each row discriminates.**
**Given** the taxonomy of §4
**When** each refusal's condition is constructed (through the in-memory port where no DB is
needed, through the store where it is)
**Then** 403/422-shape/404/409/422-domain each answer their status AND their pinned body; the
handler's match on `DocumentRefusal` has no `_` arm; the already-documented-empty-projection
order case answers 409; and the no-write half of every refusal is asserted, not assumed.
_Reddened by: M4 (compile), M5, M6, M7, M8._

**AC3 — CSRF is closed by the Origin check, at the stated strength and no higher.**
**Given** a browser holding the cached Basic credential
**When** a cross-site page POSTs to `/document-all`
**Then** the request is refused **403** before the form is parsed, with the pinned body and no
challenge header; `Origin`-absent machine callers pass; the stated limits (pre-Origin browsers,
port elision) are in the doc and the register, not silently absorbed.
_Reddened by: M9, M9b._

**AC4 — the race cannot count one box twice.**
**Given** one subject adopted once
**When** a second adoption is attempted through raw SQL (the adapter cannot construct it)
**Then** `declared_one_adoption_per_field` refuses it — the migration's guard shown to BITE —
and through the route the same condition answers the friendly 409.
_Reddened by: M10._

**AC5 — J3's correction half is measured end-to-end** (§7's four-step test, through
`app(pool, config)`, against a live MariaDB, CI-gated).
_Reddened by: M11._

**AC6 — the projection is shared, not copied.** `gap::project` becomes `pub` (no behaviour
change in core — stated at that width); the write uses it; a test asserts documented keys ==
projected keys through the REAL fn; first-occurrence-wins pinned.
_Reddened by: M12._

**AC7 — gates and tree.** `cargo xtask ci` seven gates green (authorship now over FOUR sites),
28 fixtures, trap gate still RED 26/15/11, fmt and clippy clean, no new crate, templates and
`page.rs` at zero diff.

**AC8 — the register, each row WITH ITS OWNER, re-read against THIS list**: **(1)** the
`epics.md:1758` wording divergence (*"through `insert_declared_attribute`"* → the adapter
sibling) — owner: **Epic 6's retrospective**; **(2)** row (j) CSRF — **CLOSED by this story**,
marked so in `deferred-work.md` with the residual (pre-Origin browsers) re-registered — owner
of the residual: **Epic 19**; **(3)** row (a) Chromium bench — **discharged at this story's
validation**, results recorded in the story file; **(4)** the canonical-UUID question —
**CLOSED by §2's sentence**, marked so; **(5)** `actor_id='operator'` as a literal (no real
actors) — owner: **Epic 19**; **(6)** the `SubjectLookup`→`DocumentPort` rename retires 6.1's
"never write" sentence — recorded as done WITH the rename, so no stale doc survives quoting it.

**AC9 — documents in the same commit, ONE live count in ONE place** (this story file carries
the final test count; the twins cite it by reference — 6.1's AC8 precedent, including its F2:
no count in `sprint-status.yaml`'s comments).

---

## Tasks / Subtasks

- [ ] **T1 — the domain refusals** (AC2): `AlreadyDocumented` + `NothingToDocument` in
      `opencmdb_core::document`, Display texts pinned, no `axum`, no status
- [ ] **T2 — the projection goes `pub`** (AC6): rustdoc naming both consumers; core behaviour
      unchanged (state the promise at that width)
- [ ] **T3 — the adapter** (AC1): `repo::adopt_declared_attribute` (literals `'adopted'`,
      `'operator'`; bound entity/key/value/obs_id) + `repo::load_observation_facts_by_id` +
      the already-adopted read; `SANCTIONED_SITES` + 1 named entry
- [ ] **T4 — the migration** (AC4): `0005_document_guards.sql`, the unique index, the Epic-7
      comment, DDL gates re-run; `raw_declared_write_for_ddl_test` WIDENED (same name/site)
      for the raw second-adoption probe
- [ ] **T5 — the port** (AC1, AC2): `SubjectLookup` → `DocumentPort` (doc retired with the
      rename), `Documented`/`DocumentFailure`, `StoreDocument` (pool INSIDE the impl),
      `AlwaysUnknown` deleted, one transaction, refusal order, first-occurrence-wins
- [ ] **T6 — the CSRF check** (AC3): the pure fn, first in the handler, 403 pinned, the limits
      in the doc
- [ ] **T7 — the handler** (AC1, AC2): exhaustive mapping, 201 body, 500 arm, `main::app`
      wiring (merge above the layer, unchanged)
- [ ] **T8 — the J3 test** (AC5): §7's four steps, DB-gated, run against live MariaDB
- [ ] **T9 — prove-to-red** (AC1–AC6): M1–M12, predictions FIRST, each carrier read from its
      own panic message; ⚠️ **commit the green state BEFORE the mutation pass** (Dev Notes)
- [ ] **T10 — the register and the documents** (AC8, AC9)

---

## 9. Prove-to-red

| id | mutation | predicted |
|---|---|---|
| **M1** | write `origin='manual'` in the new adapter | RED — AC1's provenance read (`origin` + `origin_obs_id` asserted on the written rows) |
| **M2** | drop the new `SANCTIONED_SITES` entry | RED — the `authorship` gate names `adopt_declared_attribute` (run `cargo xtask ci`, read the gate's own message) |
| **M2b** | move the INSERT into `document.rs` | RED — the gate, unsanctioned site (the epic's AC2 as a mutation) |
| **M3** | route the write through `insert_declared_attribute` (manual, no obs id) | RED — AC1's provenance read; and the DDL CHECK cannot carry it (manual rows legally omit the obs id), which is WHY the test reads the columns |
| **M4** | add `State<MySqlPool>` to the handler | **does not COMPILE** — re-measure the diagnostic on the new `DocumentState` |
| **M5** | answer 404 for an already-documented subject | RED — AC2's 409 pinned pair |
| **M6** | skip the `NothingToDocument` arm (write zero rows, answer 201) | RED — AC2's empty-projection test (a 201 naming zero fields is the lie the variant exists to prevent) |
| **M7** | swap the refusal ORDER (already-documented after nothing-to-document) | RED — the order case: adopted subject whose projection is empty must answer 409 |
| **M8** | let the port write OUTSIDE the transaction (commit between check and write) | RED — prediction: the two-step raw-SQL race test; ⚠️ if it comes back GREEN, record it — a single-connection test may be unable to interleave, and the honest carrier is then the index (M10) plus a stated limit, NOT a fake concurrency test |
| **M9** | drop the CSRF check | RED — the cross-site 403 test (end-to-end, credential attached) |
| **M9b** | compare Origin case-sensitively / accept `Origin: null` | RED — the pure-fn table's two rows |
| **M10** | drop the unique index from the migration | RED — AC4's raw-SQL second adoption inserts cleanly where a refusal is asserted |
| **M11** | make the documented write use a WRONG attr key (`"ip"` for `"ipv4"`) | RED — the J3 test's step 4: the gap does NOT close (this is the drift §3 exists to prevent, measured) |
| **M12** | re-introduce a private copy of the projection in bin and use it | RED — AC6's keys-equality test through the real fn (if it stays green, the test is comparing the copy to itself — rewrite it) |

⚠️ **Predict first, then measure; a divergence is a FINDING. Read each red's panic message one
by one — a mutation named for one thing and applied to another measures the other thing** (three
occurrences across 5.13/5.14/6.1).

---

## Dev Notes

### Traps, each measured on this project — 6.1's list plus its review's addition

- 🔴 **Commit the green state BEFORE the mutation pass, and revert the MUTATION, never the
  FILE — with the precondition 6.1's review paid to learn: a file-level `git checkout --
  <file>` equals a mutation revert ONLY when the baseline is COMMITTED.** 6.1's review pass
  lost every uncommitted review patch in two files exactly that way, caught only because the
  FULL suite re-ran against the live database before concluding;
- ⚠️ **`cargo test --workspace A B` passes TWO filters and silently runs nothing**;
- ⚠️ **Never read a measurement through a truncation**;
- ⚠️ **`DATABASE_URL` is unset locally** — DB tests pass by `return`ing; this story's central
  test (§7) and the migration guard (AC4) say NOTHING on a green local run. The 6.1 container
  is still up: `DATABASE_URL='mysql://root:story61@127.0.0.1:13316/opencmdb_test'`;
- ⚠️ **`cargo fmt --all --check` runs in CI before the tests** (`ci.yml:56`); rustfmt has
  invalidated python-script anchors mid-story once already (6.1's M9 driver incident) — make
  every mutation driver print `MUTATED` and assert its anchor;
- ⚠️ the trap-text privacy floor and fixture locks are untouched by this story — if
  `cargo xtask ci`'s fixture gate moves, something drifted that should not have.

### The tree this story extends (measured 2026-08-14, master `3f7069a`)

**566 tests** (344 bin + 160 core + 62 xtask), seven gates green + `views-hash ℹ STALE exit 0`,
28 fixtures, trap gate RED at 26/15/11 by design (Epic 6's L2 stories close it, 6.15 last).
Story 6.1's surface: `document.rs` (sub-router, `DocumentState`, `SubjectLookup`+`AlwaysUnknown`,
nil-UUID refused, pinned 404 body "unknown subject: nothing can be documented"), `auth.rs`
(Basic default arm, challenge discipline), `AppConfig` (pure `from_env`), `app(pool, config)`
with the merge ABOVE the layer. The known-subject branch answers 501 today and DISAPPEARS here.

### Validation obligations (for the MANDATORY validate pass, before any dev)

1. **Run the two-browser bench** (Firefox + Chromium — register row (a)) with the §5 probes:
   same-origin `hx-post` carries `Origin`?; cross-site form POST carries the cached credential
   AND its own `Origin`?. Server-side oracle, three runs, results INTO this file. 🔴 If any
   current browser omits `Origin` on same-origin XHR, §5's mechanism must be re-arbitrated
   BEFORE dev;
2. verify §1's schema reading against `0001_initial.sql` and the gate's `PROVENANCE_COLUMNS`
   (the fact-check layer re-reads, never trusts);
3. verify the §3 multi-value decision against the committed corpus (does any fixture
   observation carry two facts of one kind? — if yes, name it as the test's seed);
4. verify prescribed mutation M8 is EXECUTABLE (5.9's M4/M5 defect class: a prescribed
   mutation that cannot run); if not, rewrite it per its own ⚠️ clause.

### References

- [Source: `epics.md:1758-1776` — Story 6.2 + Epic 6 constraint (3)] — ⚠️ divergence registered §1
- [Source: `crates/opencmdb-bin/migrations/0001_initial.sql:10-21`] — the schema's `adopted` vocabulary
- [Source: `crates/opencmdb-bin/src/repo.rs:113-133`] — `insert_declared_attribute`, the idiom to copy
- [Source: `xtask/src/main.rs:1151-1162`] — `SANCTIONED_SITES`; `:1167` — `PROVENANCE_COLUMNS`
- [Source: `crates/opencmdb-core/src/gap/mod.rs:77-91`] — the private projection
- [Source: `_bmad-output/implementation-artifacts/6-1-write-route-writes-nothing.md`] — the port seam, the M4 carrier, decision (b), the register rows this story owns
- [Source: `prd.md:884-890` (FR13), `prd.md:1210-1222` (NFR5's three assertions)]
- [Source: `architecture.md:3818` — the `document` vocabulary; D21 (NULL-distinctness), D47, D48]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
