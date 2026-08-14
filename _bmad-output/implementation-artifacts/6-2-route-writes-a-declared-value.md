# Story 6.2: The route writes a declared value, through the adapter and nowhere else

Status: review

<!-- ⚠️ CONTEXTED 2026-08-14, the same day story 6.1 merged (PR #89 → `664693b`, docs flipped by
     PR #90 → `3f7069a`). The tree this story extends is master at `3f7069a`: 566 tests
     (344 bin + 160 core + 62 xtask), seven gates green, trap gate RED at 26/15/11 by design.
     ✅ VALIDATED 2026-08-14, FOUR passes: two fact-check layers + TWO gap-hunt layers that each
     BUILT the whole prescribed design against the live mariadb:10.11.11 — 1 HIGH arbitration
     taken by Guy, and ~15 text/mutation corrections, ALL APPLIED, plus the two-browser CSRF
     bench (§4). 🔴 Guy's arbitration (option A): the authorship gate gains a NAMED READ-SANCTION,
     FR13-framed — see §6.5. Confirmed by construction (both gap-hunts, independently): M4 does
     not compile (no side door — no `FromRef`, no downcast; the transaction shape
     `pool.begin()` → repo free fns over `&mut *tx` → commit runs green), the index gives ERROR
     1062, the end-to-end J3 answers 201, `page.rs` byte-identical. Refuted/corrected with their
     measurement: M8 GREEN by the index, M11's oracle is abstention-not-gap, M12 needs a source
     scan, M2's gate message, plus a fourth broken 6.1 test, the M10 checksum-panic driver step,
     the multi-value reconcile abstentions, the scheme-blind Origin compare, the PK (not the
     index) blocking Epic 7, and the J3 view path (build_view's `preferred`, never the env var). -->

## Story

As the operator,
I want an observed value to become a declared one,
So that what the product found becomes what I documented.

**And this is the story where milestone J3's second half becomes measurable for the first
time**: J3 wants a real gap *detected AND corrected*, the detection has existed since v0.1, and
the correction has had no code path at all — at Epic 5's close the product had five read-only
routes and not one production writer of `declared_attribute` (the calls outside `repo.rs` all
sit inside `#[cfg(test)]`). Story 6.1 added `POST /document-all` as a SHAPE that writes nothing;
after THIS story it writes the declared record, and a reconcile pass over the documented
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
- 🔴 **the epic divergence is REGISTERED, not edited**: `epics.md:1768`'s *"through
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
    /// check the subject exists → project its facts (empty → NothingToDocument) →
    /// mint an entity → write the adopted rows, the unique index turning a re-adoption
    /// into AlreadyDocumented (no pre-read — §4/§6.5). One transaction: a check that
    /// commits separately from its write is a TOCTOU hole, not a check.
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
  **`repo::load_observation_by_id(executor, &obs_id) -> Result<Option<Observation>, _>`** — it
  returns the whole `Observation`, NOT just its facts, so `gap::project(&obs)` composes
  unchanged (validation M1: `project` takes `&Observation`, and a facts-only reader would force
  a synthetic observation a reviewer reds on sight — `load_observation_facts` loads ALL rows and
  is the wrong shape here). Then `repo::adopt_declared_attribute` once per projected field.
  **The SQL write exists in `repo.rs` and nowhere else** — that is what the authorship gate's
  write half measures. 🔴 **There is NO pre-write "already-adopted" SELECT** — the 409 rides the
  unique index (§4/§6), so production reads no provenance column at all (validation H1);
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
case carries its own test. ⚠️ **No committed fixture carries two facts of one kind** (validation:
the `multi-nic` family is N single-MAC observations), so the test's seed is SYNTHETIC — name it
so. 🔴 **And the pin must assert what the reconcile then does with it** (second gap-hunt, M-c,
measured): a `{ipv4 a, ipv4 b, hostname}` observation documents 201 / 2 rows, then `reconcile`
over the entity returns NO gap but TWO abstentions — `ConflictingObservations` (the engine reads
two same-key facts in one observation as a conflict and drops the field) and `NoObservedValue`
on the freshly-declared `ipv4`. This is not a defect to "fix" — assert the two abstentions in the
multi-value test so a later story does not turn them into a false gap.

---

## 4. The refusal taxonomy, v2 — every row of 6.1's table survives, three rows are new

| refusal | status | where decided | discriminator |
|---|---|---|---|
| no/wrong credential | **401** + challenge | `auth_deny` (UNCHANGED) | the challenge header |
| 🆕 cross-origin browser request | **403** | the CSRF check, first thing in the handler (§5) | pinned body naming the origin refusal; **never** `WWW-Authenticate` |
| malformed form / non-UUID / nil UUID | **422** | handler (UNCHANGED from 6.1) | the "expected form field `subject`" body |
| unknown subject | **404** | `DocumentRefusal::UnknownSubject` — **now store-backed truth**: the id really names no `observation_record` row | the pinned body (6.1's decision (b) text kept: "unknown subject: nothing can be documented" — still true, and now for the right reason; Guy may refine wording at validation) |
| 🆕 already documented | **409** | `DocumentRefusal::AlreadyDocumented` — **detected by the unique index, not a pre-read** (validation H1): the first projected field's INSERT hits `ERROR 1062` (the projection is identical — facts are immutable — so at least one key always collides), mapped by the index NAME | pinned body; **this is the epic's title enforced**: documenting twice is counting one box twice |
| 🆕 nothing to document | **422** | `DocumentRefusal::NothingToDocument` — the projection of the subject's facts is empty (e.g. an Rtt-only observation) | pinned DOMAIN body, distinct from the shape 422's |
| backend failure | **500** | `DocumentFailure::Backend` | logged; body does not leak the SQL error |
| success | **201 Created** | the write happened | non-empty body naming the minted entity id and the field count |

- `DocumentRefusal` (core) grows two variants. **The handler's match is exhaustive with no `_`
  arm** (story 5.3's precedent, already installed in 6.1) — adding the variants without
  mapping them is `error[E0004]`, which is the design working;
- refusal ORDER inside the transaction: unknown → nothing-to-document → attempt the write
  (already-documented surfaces HERE, as the index 1062). ⚠️ **The "adopted subject with an
  empty projection" order case is UNCONSTRUCTIBLE through the route** (validation M-2/M5: adoption
  needs a non-empty projection and facts are immutable, so an adopted subject's projection can
  never later be empty) — it exists only if a test PLANTS the adopted row via raw SQL. Its test
  therefore lives in `repo.rs`'s test module through the widened `raw_declared_write_for_ddl_test`
  (§6), never through the route, and the story says so rather than leaving the dev to discover
  the state cannot arise;
- the 404 CANNOT collide with the fallback's empty 404 (6.1 §6's discriminator, unchanged) and
  the two 422s discriminate by BODY.

---

## 5. 🔴 CSRF — this story OWNS it (6.1's register row (j)), and the mechanism is the Origin check

6.1 RECORDED (validation finding F3, replacing a false sentence it nearly shipped): Basic is
AMBIENT AUTHORITY — once the browser holds the credential it attaches it to a cross-site form
POST too — and 6.1's bench *"probed only same-origin initiators and must not be quoted wider"*.
The cross-site attach is a recorded threat-model fact, not a 6.1 measurement. The route now has
an effect to forge, so the protection lands HERE:

**Mechanism** — a pure fn in `document.rs`, called FIRST in the handler. ⚠️ **It cannot run
"before the form is parsed"** (validation M3, measured on axum's `Handler`): the handler keeps
6.1's `form: Result<Form<…>, FormRejection>` extractor argument, and axum runs ALL extractors —
the body parse included — before the handler body. The true, weaker property, which is what AC3
asserts: **the 403 is decided FIRST and no refusal path consults the parsed form** — a cross-site
POST with a malformed body answers 403, not 422 (measured). Taking `Request` to parse manually
was considered and refused: it would rewrite 6.1's whole handler for a property the ordering
already gives.

The check reads two headers (`Origin`, `Host`) and decides:

- **`Origin` header absent → PASS.** A machine caller (`curl -u`) sends none; the threat model
  is a BROWSER carrying ambient credentials, and every current browser sends `Origin` on every
  cross-site POST. ⚠️ Stated limit, registered: a pre-2020 browser that omits `Origin` on a
  cross-site form POST is not protected — acceptable for a LAN single-operator product, and
  Epic 19's session+token closure supersedes it;
- **`Origin: null` → REFUSE** (sandboxed iframes, some redirect chains — no legitimate
  same-origin caller sends it);
- **`Origin` present → parse its authority (`host[:port]`) and compare, ASCII
  case-insensitively, against the request's `Host` header.** Match → pass; mismatch → 403.
  ⚠️ **This holds only when the reverse proxy FORWARDS the original `Host`** (validation M-3):
  nginx's default `proxy_pass` REWRITES `Host` to the upstream address, which would refuse every
  browser POST (same-origin included). **State the deployment requirement in the doc AND the
  admin manual: `proxy_set_header Host $host;`** — this is a stated limit, not a mechanism flaw;
- ⚠️ **`Host` ABSENT** (HTTP/2 sends `:authority`, not a `Host` header) → REFUSE (measured: the
  prototype cannot compare against a missing `Host`; a proxy terminating HTTP/2 re-adds `Host`
  upstream, so this bites only a direct-HTTP/2 client, which this LAN product does not serve) —
  pin it and state it;
- ⚠️ **Known edge, stated: default-port elision** (an `Origin` of `http://nas` against a `Host`
  of `nas:80`) is compared literally and would refuse — pin the behaviour with a test and the
  limit in the doc rather than normalising ports speculatively;
- ⚠️ **Two `Origin` headers** → REFUSE outright (6.1's `Authorization` precedent: `HeaderMap`
  first-value semantics let right-then-wrong through, so more than one is refused, tested);
- ⚠️ **The comparison is authority-only, SCHEME-BLIND** (second gap-hunt L-a, measured):
  `Origin: https://nas:8080` passes against `Host: nas:8080`, so a same-authority page on the
  wrong scheme is not distinguished. A stated limit (this LAN product terminates TLS at the
  proxy and serves one scheme), registered, not silently fixed;
- **the 403 carries a pinned body and NEVER `WWW-Authenticate`** — it is not an auth failure,
  and arbitration 6's discipline (no advertising the scheme where it is not the answer)
  extends here;
- **placement: in the HANDLER, not a layer.** A `route_layer` on the sub-router would work,
  but 6.1 §2 exists because layer/registration ordering was measured treacherous once already;
  a first-line call in the handler is order-proof, testable pure AND end-to-end.

**✅ Measured at validation, 2026-08-14 — the two-browser bench (register row (a)), real
vendored htmx 2.0.4, two origins, server-side oracle:**

| engine | same-origin `hx-post` | cross-site `<form>` POST |
|---|---|---|
| **Chrome / Blink 151.0.7922.137** | `Origin` = page's own origin `http://127.0.0.1:18080`; carries `Authorization` (on the 401 retry — Blink primes XHR auth on challenge, not preemptively) | `Origin` = ATTACKER `http://127.0.0.1:18081` ≠ `Host` `127.0.0.1:18080`; carries cached `Authorization: Basic …` PREEMPTIVELY (200, one request) |
| **Firefox / Gecko 153.0.1** | `Origin` = page's own origin `http://127.0.0.1:18080`; carries `Authorization: Basic …` PREEMPTIVELY (200, one request) | `Origin` = ATTACKER `http://127.0.0.1:18081` ≠ `Host` `127.0.0.1:18080`; carries cached `Authorization: Basic …` PREEMPTIVELY (200, one request) |

**🔧 The Gecko cell was measured after all** (2026-08-14, second bench pass): Firefox headless made
zero requests because the **snap** `firefox` wrapper refuses a second instance while the operator's
own Firefox is running (`--no-remote`/`MOZ_NO_REMOTE`/`--new-instance` all ignored by the wrapper —
stderr: *"Firefox is already running… use a different profile"*). Invoking the raw binary
`/snap/firefox/current/usr/lib/firefox/firefox` directly bypasses the single-instance lock and
Firefox runs headless normally. **Three runs per engine, all identical**; server-side oracle in
`scratchpad/bench-6.2/{A,B}-{CHR,FFX}_r{1,2,3}.log` (A = protected origin, B = attacker).

🔑 **The decision-critical result, on BOTH engines**: htmx 2.0.4's XHR SENDS `Origin` on a
same-origin POST (value = the page's own origin), so the happy path passes through
*present → compare → match* (not through *absent → pass*, which is for `curl` only); and the
cross-site `<form>` POST carries the ATTACKER's `Origin`, so the check REFUSES it. **§5's mechanism
is NOT re-arbitrated** — the risk this bench existed to catch (a browser omitting `Origin` on
same-origin XHR) is answered **NO for BOTH Blink and Gecko**. `deferred-work.md` row (a) is
DISCHARGED for **both engines**, not only Chromium.

🔴 **The cross-site CACHED-Basic-credential attachment (the threat §5 exists to stop) was FRESHLY
MEASURED on both engines, not merely inherited from 6.1's F3**: with the credential primed for
origin A (a real 401→credential exchange), the auto-submitted cross-origin `<form>` POST to A
carried `Authorization: Basic …` **preemptively** — Chrome and Firefox alike, all three runs — so
ambient authority is confirmed live, and the ONLY thing standing between the attacker page and a
write is the `Origin` check. 🔑 **A third probe (P3) was added**: a cross-origin `fetch` POST
(form-encoded, default credentials mode) from B to A. Both engines SEND the request (it reaches
the server and would be `Origin`-refused), and it carries **NO** credential (401, no retry) — so
the `fetch`/XHR path is not itself a credential-bearing vector, but the `<form>` path is, which is
exactly why §5 must not depend on `Sec-Fetch-*` or on the request being an XHR.

⚠️ **One nuance §5 must not lean on**: here A and B differ only by PORT (`:18080` vs `:18081`), so
`Sec-Fetch-Site` reads **`same-site`** on the cross-origin POST, NOT `cross-site` — a header-based
CSRF filter keyed on `Sec-Fetch-Site: cross-site` would have MISSED this attacker. §5's literal
`Origin`-authority-vs-`Host` comparison refuses it correctly (`127.0.0.1:18081` ≠ `127.0.0.1:18080`)
because it compares authorities, not site. Ports here are explicit and non-default, so the
default-port-elision edge (§5's `http://nas` vs `nas:80`) was not exercised — that limit stands as
written.

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
- the loser gets `Err(Constraint("unique"))` through the adapter — ⚠️ `classify` folds EVERY
  constraint (the CHECKs included) into `Constraint`, so **key the 409 on the index NAME**
  (`declared_one_adoption_per_field`), never on the bare `Constraint` variant (validation M6):
  the adapter's literals cannot violate the CHECKs today, but a bare mapping would answer
  `AlreadyDocumented` for any future constraint. Tested with two raw sequential adoptions of one
  subject
  (the *"first writer owes its guards"* rule, and the guard must be shown to BITE: a test
  inserts the second adoption through raw SQL and asserts the refusal, on 5.9's M3 lesson —
  a CHECK the adapter cannot violate is measured through raw SQL or it is measured by nothing);
- 🔴 **the raw SQL for that test MUST go through `raw_declared_write_for_ddl_test`, AND the
  raw-plant tests MUST live in `repo.rs`'s test module** (validation M5, measured): that fn is
  the authorship gate's ONE sanctioned raw writer, and placing a raw adopted-row INSERT beside
  the port's tests in `document.rs` REDS the gate's write half (measured at `document.rs:661/689`
  in the prototype — an unsanctioned site). Today the fn hardcodes `'hostname'/'nas'/'manual'`
  with no `origin_obs_id`: **WIDEN its signature** (attr_key, origin, origin_obs_id as
  parameters — same name, same file, same site, so the gate keys on `(path, fn)` and
  `SANCTIONED_SITES` still gains only the one production entry; the epic's "exactly one" holds,
  measured). Its doc says the name is load-bearing; keep the name;
- ⚠️ Epic 7's `document-field` re-documents a DRIFTED field from a NEWER observation — a
  DIFFERENT `origin_obs_id`, so THIS index does not collide. 🔴 But the **PRIMARY KEY
  `(entity_id, attr_key)` DOES** (second gap-hunt L-c, measured: re-documenting one entity's
  same field reds `1062 … PRIMARY`) — so "the index does not pre-block Epic 7" must NOT read as
  "Epic 7 will just work". The migration comment names the PK as what Epic 7 must negotiate (an
  `ON DUPLICATE KEY UPDATE` or a supersede), not this index.

⚠️ The DDL gates apply: binary collation grep (D64) is satisfied by an INDEX (no new column);
run `cargo xtask ci` after writing the migration, not only at the end.

---

## 6.5. 🔴 The authorship gate gains a NAMED read-sanction — Guy's arbitration (validation H1)

**The finding, measured by the gap-hunt against a live build:** story 5.12's authorship gate has
a READ half (`authorship_findings`, `xtask/src/main.rs:1619-1636`) that reds on ANY `SELECT` of
`declared_attribute` naming a `PROVENANCE_COLUMNS` entry (`origin_obs_id`, `origin`, `actor_id`),
in the projection OR in a `WHERE` (probe `e36`, 5.12's repair) — and it **never consults
`SANCTIONED_SITES`**: its doc says a site *"short-circuits the write half only"*. So a test that
verifies the write's provenance (`SELECT origin, origin_obs_id … WHERE entity_id = ?` — AC1's
central carrier) REDS `cargo xtask ci`. Measured: the 4th write-site entry present, the gate
still reds at the test's SELECT.

**Guy's arbitration (2026-08-14, option A): give the gate a NAMED read-sanction, FR13-framed.**

- **the FR13 nuance is the whole justification, and it goes in the gate's own doc**: FR13/NFR5's
  invariant is that *"the divergence computation never consults how a declared value was
  obtained"* — NOT that no code may ever read provenance. The gate has been over-approximating to
  *"no `.rs` under `crates/` may read a provenance column"*, which was free until this story; 6.2
  is the first legitimate reader;
- **add `SANCTIONED_READS: [(&str, Option<&str>); N]`** beside `SANCTIONED_SITES`, applied in the
  READ half exactly as the write half applies its list — a `(path, enclosing_fn)` allowlist, as
  narrow and as named. **It admits exactly ONE site**: the test verifier that reads the adopted
  rows to assert their provenance (a named fn in `repo.rs`'s test module, e.g.
  `read_declared_provenance_for_test`). Production admits ZERO reads — the 409 rides the index
  (§4/§6), so `count_declared_attributes` and `load_declared_attributes` (which select no
  provenance column) stay the only production reads and need no sanction;
- **the gate's `authorship` line count grows** (it now walks the same files for a second,
  smaller allowlist) and **its module doc's "N gates / what each does" enumeration must be
  updated** — 5.12's review caught that exact self-description drifting twice; do not let it;
- 🔴 **prove the read-sanction BITES, both directions** (5.12's own idiom): with the entry
  present, the verifier test's SELECT is green under `cargo xtask ci`; REMOVE the entry → the
  gate REDS naming `repo.rs` and the provenance column; put an UNSANCTIONED provenance SELECT in
  `document.rs` → the gate REDS. These are this story's authorship-READ mutations (M13, M13b),
  added to §9;
- ⚠️ **the narrowed promise, on 5.12's precedent**: a read-sanction is a TRIPWIRE against a
  future story reading provenance into a divergence path by accident, never a barrier — the same
  strength 5.12 stated for the write half, stated again here and no higher. Register it.

This is the ONE change to `xtask/` this story makes, and it is a DELIBERATE widening of the
gate's trusted surface — recorded as such, arbitrated, not slipped in under `; 3 → ; 4`.

---

## 7. J3's second half, measured end-to-end for the first time

One DB-backed test is this story's reason to exist, and it goes through the FULL stack:

1. seed one observation (ipv4 + hostname, the drifting pair from `index_renders_the_real_gap`)
   with NO declared record — the day-one case, FR13(a)'s own words;
2. POST `/document-all` through `app(pool, config)` with the valid credential and same-origin
   headers → **201**;
3. read the declared side: N rows, `origin = 'adopted'`, `origin_obs_id` = the subject,
   `actor_id = 'operator'`, values equal to the projection;
4. run the reconcile view over the documented entity → **the entity is present, carries NO
   divergence AND NO abstention on the documented fields** — the gap the product would have
   shown is CLOSED by the gesture. 🔴 **Both halves are load-bearing** (validation H2, measured):
   a wrong attr key does NOT produce a `Gap` — it produces a `NoObservedValue` ABSTENTION (or the
   field simply vanishes from `build_view`), which a `gaps.is_empty()` oracle cannot see. So the
   assertion is `gaps.is_empty()` **AND** `abstention_count == 0` on the documented entity — M11
   reds THIS, not a gap count. *(The divergence computation never consults the provenance columns
   — xtask's `PROVENANCE_COLUMNS` gate holds that half; do not re-test the gate, test the VIEW.)*

🔴 **The path INTO the view is prescribed, not left to the dev** (second gap-hunt L-d): step 4
selects the entity through **`build_view`'s `preferred` parameter** (or a cleaned table), NEVER
through `reconcile_view`'s `OPENCMDB_ENTITY_IPV4` env read — 6.1's no-env-mutation norm holds
(that env var is exactly the kind of hidden coupling the norm exists to kill, and
`index_renders_the_real_gap` already `remove_var`s it). ⚠️ **Corollary the dev must know**:
`build_view` selects the entity BY its declared `ipv4` (`page.rs:361-372`), so the J3 seed must
carry an ipv4 — a **hostname-only** subject documents 201 but mints an entity the view can never
select (a 201 invisible on the page). That is not this story's bug to fix (the entity model is
6.5's), but the J3 test must seed an ipv4 subject or it measures nothing, and the invisible-entity
case is registered for 6.5.

⚠️ This test is `DATABASE_URL`-gated like every DB test (it self-skips locally without the
container) — **a green local suite says nothing; run it against the live MariaDB** (13316's
container from 6.1 is still up) and CI re-proves it.

---

## 8. What must be pinned

- **the whole 6.1 surface survives — with FOUR named breaks, not three** (validation M2,
  measured on the prototype): (a) the `AlwaysUnknown` pin, deleted with its subject; (b) the
  known-subject 501 test, whose branch no longer exists (replacement asserts 201 + the write);
  (c) the `SubjectLookup`→`DocumentPort` rename, which is NOT merely mechanical — the trait's
  METHOD changes (`check(…) -> Result<(), DocumentRefusal>` becomes `document_all(…) ->
  Result<Documented, DocumentFailure>`), so `KnowsOne` and every in-memory impl are rewritten,
  not renamed; and 🔴 (d) **`the_production_route_answers_the_pinned_unknown_subject_body`
  (`main.rs`)** — it drives an unknown subject through `app(lazy_pool(), …)` and asserts 404,
  but `lazy_pool()` (wrong credentials by design) can never reach a store, so under the
  store-backed port it answers `Backend → 500`, locally AND in CI. **It must become DB-gated**
  (an unknown subject through the real empty store answers 404) OR move to an in-memory
  `DocumentPort` that answers `UnknownSubject` with no DB. Prescribed: DB-gated, so it pins the
  production truth; a green LOCAL run then says nothing about it. Anything red BEYOND these four
  is a FINDING;
- **M4 re-measured** on the new `DocumentState` (compile refusal, actual diagnostic recorded);
- **the authorship gate carries the new WRITE site AND the new READ site (§6.5)**: (a) with
  the write entry present, `cargo xtask ci` green; (b) REMOVE it → the gate REDS **naming
  `repo.rs` and the insert keyword** (validation M4: the write half prints `file:line` +
  `insert into declared_attribute`, NOT the fn name — do not assert on `adopt_declared_attribute`
  in the message); (c) move the SQL write into `document.rs` → the gate REDS (unsanctioned site);
  (d) the read-sanction mutations M13/M13b (§6.5). These are the story's authorship mutations;
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
_Reddened by: M1, M2, M2b (gate), M3, M4 (compile — the no-pool carrier)._

**AC2 — the refusal taxonomy is total, ordered, and each row discriminates.**
**Given** the taxonomy of §4
**When** each refusal's condition is constructed (through the in-memory port where no DB is
needed, through the store where it is)
**Then** 403/422-shape/404/409/422-domain each answer their status AND their pinned body; the
handler's match on `DocumentRefusal` has no `_` arm; the already-documented-empty-projection
order case answers 409 (constructed by a RAW adopted-row plant in `repo.rs`'s test module — it
cannot arise through the route, §4/M5); and the no-write half of every refusal is asserted.
_Reddened by: M5, M6, M7 (order), M8 (GREEN by structural argument — see §9)._

**AC3 — CSRF is closed by the Origin check, at the stated strength and no higher.**
**Given** a browser holding the cached Basic credential
**When** a cross-site page POSTs to `/document-all`
**Then** the request is refused **403** with the 403 decided FIRST (no refusal path consults
the parsed form — NOT literally before the parser runs, which axum's extractor order forbids,
§5/M3), with the pinned body and no challenge header; `Origin`-absent machine callers pass; the
stated limits (pre-Origin browsers, port elision, `Host` ABSENT, and the `proxy_set_header Host`
deployment requirement) are in the doc and the register, not silently absorbed.
_Reddened by: M9, M9b._

**AC4 — the race cannot count one box twice.**
**Given** one subject adopted once
**When** a second adoption is attempted through raw SQL (the adapter cannot construct it)
**Then** `declared_one_adoption_per_field` refuses it — the migration's guard shown to BITE —
and through the route the same condition answers the friendly 409.
_Reddened by: M10._

**AC5 — J3's correction half is measured end-to-end** (§7's four-step test, through
`app(pool, config)`, against a live MariaDB, CI-gated — the oracle is `gaps.is_empty()` AND
`abstention_count == 0`, both load-bearing: a wrong key yields an ABSTENTION, not a gap, H2).
_Reddened by: M11 (co-carried with AC6's keys-equality test — a wrong key reds both)._

**AC6 — the projection is shared, not copied.** `gap::project` becomes `pub` (no behaviour
change in core — stated at that width); the write uses it through `load_observation_by_id`'s
`Observation` (§2/M1); a test asserts documented keys == `gap::project(&obs)` through the REAL
fn; first-occurrence-wins pinned. ⚠️ The no-COPY property is a SOURCE tripwire, not a
behavioural test (validation H4: a faithful private copy passes the keys-equality test — only a
DRIFTED copy reds it, and that is M11): a test scans `document.rs`/the port file for a local
`"ipv4"`/`"hostname"`/`"mac"` key table and asserts it calls `gap::project`.
_Reddened by: M11 (drift) + M12 (the source-scan tripwire — a planted local key table reds it)._

**AC7 — gates and tree.** `cargo xtask ci` seven gates green (authorship now over FOUR write
sites AND one named READ site, §6.5; its module-doc enumeration updated), 28 fixtures, trap gate
still RED 26/15/11, fmt and clippy clean, no new crate, templates and `page.rs` at zero diff.

**AC8 — the register, each row WITH ITS OWNER, re-read against THIS list**: **(1)** the
`epics.md:1768` wording divergence (*"through `insert_declared_attribute`"* → the adapter
sibling) — owner: **Epic 6's retrospective**; **(2)** row (j) CSRF — **CLOSED by this story**,
with residuals re-registered (pre-Origin browsers; `Host`-forwarding proxy requirement) — owner:
**Epic 19**; **(3)** row (a) two-browser bench — **FULLY DISCHARGED at this story's validation on
BOTH Blink AND Gecko** (three runs each, all three probes), results in §5; no Gecko cell remains
open; **(4)** the canonical-UUID question — **CLOSED by §2's sentence**, marked so;
**(5)** `actor_id='operator'` as a literal (no real actors) — owner: **Epic 19**; **(6)** the
`SubjectLookup`→`DocumentPort` rename retires 6.1's "never write" sentence — recorded as done
WITH the rename; **(7)** 🔴 the authorship gate's READ-sanction (§6.5) is a TRIPWIRE against a
future story reading provenance into a divergence path, never a barrier — the narrowed promise,
owner: **Epic 6's retrospective** (it is a widening of 5.12's apparatus).

**AC9 — documents in the same commit, ONE live count in ONE place** (this story file carries
the final test count; the twins cite it by reference — 6.1's AC8 precedent, including its F2:
no count in `sprint-status.yaml`'s comments).

---

## Tasks / Subtasks

- [x] **T1 — the domain refusals** (AC2): `AlreadyDocumented` + `NothingToDocument` in
      `opencmdb_core::document`, Display texts pinned, no `axum`, no status
- [x] **T2 — the projection goes `pub`** (AC6): rustdoc naming both consumers; core behaviour
      unchanged (state the promise at that width)
- [x] **T3 — the adapter** (AC1): `repo::adopt_declared_attribute` (literals `'adopted'`,
      `'operator'`; bound entity/key/value/obs_id) + `repo::load_observation_by_id` returning
      `Option<Observation>` (composes with `gap::project`, M1); NO already-adopted read (the 409
      rides the index, §6.5); `SANCTIONED_SITES` + 1 named entry
- [x] **T4 — the migration** (AC4): `0005_document_guards.sql`, the unique index, the Epic-7
      comment, DDL gates re-run; `raw_declared_write_for_ddl_test` WIDENED (same name/site)
      for the raw second-adoption probe
- [x] **T4b — the gate read-sanction** (§6.5, AC7): `SANCTIONED_READS` in `xtask`, applied in
      the READ half, one named entry (the test verifier); FR13 rationale in the gate doc; the
      module-doc enumeration updated; M13/M13b prove it bites both ways
- [x] **T5 — the port** (AC1, AC2): `SubjectLookup` → `DocumentPort` (doc retired with the
      rename), `Documented`/`DocumentFailure`, `StoreDocument` (pool INSIDE the impl, in
      `document.rs` — clarify 6.1's module doc: `DocumentState` still holds no pool, `StoreDocument`
      does), `AlwaysUnknown` deleted, one transaction, refusal order (unknown → nothing-to-document
      → write; 409 = index 1062 keyed on the index name), first-occurrence-wins
- [x] **T6 — the CSRF check** (AC3): the pure fn, first in the handler, 403 pinned, the limits
      in the doc
- [x] **T7 — the handler** (AC1, AC2): exhaustive mapping, 201 body, 500 arm, `main::app`
      wiring (merge above the layer, unchanged)
- [x] **T8 — the J3 test** (AC5): §7's four steps, DB-gated, run against live MariaDB
- [x] **T9 — prove-to-red** (AC1–AC7): M1–M13b (sixteen ids, M8 GREEN-by-argument), predictions
      FIRST, each carrier read from its own panic message; ⚠️ **commit the green state BEFORE the
      mutation pass** (Dev Notes); fix the FOURTH broken 6.1 test (§8(d)) and verify IN CI
- [x] **T10 — the register and the documents** (AC8, AC9)

### Review Findings (code review 2026-08-14, three layers: Blind Hunter, Edge Case Hunter, Acceptance Auditor)

🔑 **The core design was MEASURED sound**: the Auditor re-executed M1/M2/M4/M6/M7/M10/M11/M13 — every one matched the record; the Edge Case Hunter REFUTED, by measurement against the live DB, the concurrency (201/409, exactly one entity's rows), the partial-rollback (no orphan), `is_adoption_conflict` (fires on 10.11), the read-sanction (load-bearing), the 500 no-leak, and the whole CSRF taxonomy. All 9 ACs MET except AC9. 13 findings after dedup: 6 patch, 1 decision, 4 defer, 1 dismissed.

- [x] [Review][Defer] (Guy, 2026-08-14) **A same-key multi-value observation is documented "success" (201) but its gap does NOT close** — measured: two `Fact::IpV4` → 201 "2 fields", but `reconcile` reads the two values as CONFLICTING, drops the field, and the just-documented `ipv4` returns as a `NoObservedValue` abstention (the gap the operator was told they closed stays open). Contradicts the write's own "a multi-homed device is normal, not a refusal". ⚠️ **NOT reachable today** (no shipped connector/fixture emits two IpV4 in one observation); §3 documents the abstention. Options: (a) refuse a self-conflicting same-key projection (honest — don't claim success on a value the reconcile will abstain on); (b) defer to the entity-model / connector story with the §3 documentation (the one-value-per-attr_key model is 6.5's). [crates/opencmdb-bin/src/document.rs:181-191]
- [x] [Review][Patch] 🔴 AC9 violated: the live count "566 → 580" was re-committed into `sprint-status.yaml`'s comment (the exact F2 defect 6.1's review removed) AND repeated in both twins (6.1's twins cited the file WITHOUT the number). Remove the number from all three; cite the story file only. [sprint-status.yaml, CLAUDE.md, docs/project-context.md]
- [x] [Review][Patch] `an_absent_origin_passes_the_csrf_check` passes for the WRONG reason: `answer()`→`form_post()` always sets `Origin: http://nas:8080`, so it sends a MATCHING origin, never an absent one. The e2e absent-Origin path is uncovered. Send a request with NO Origin header. [crates/opencmdb-bin/src/document.rs, the CSRF tests]
- [x] [Review][Patch] The 201-body oracle `assert!(body.contains('2'))` is weak — any '2' in the entity UUID satisfies it (green only because the fixture id has no '2'). Assert the exact `"2 field(s)"` substring. [crates/opencmdb-bin/src/document.rs:452]
- [x] [Review][Patch] AC6's prescribed direct oracle is absent: no test asserts documented keys == `gap::project(&obs)` through the REAL fn (the property is carried transitively by the source-scan + J3 abstention). Add the keys-equality test the AC and §3's DRY note name. [crates/opencmdb-bin/src/document.rs or repo.rs test module]
- [x] [Review][Patch] §6.5's "the module-doc enumeration must be updated" is unmet: the authorship gate's module doc (`xtask/src/main.rs`) was not expanded for the read-sanction. It stayed TRUE (no drift), but add the read-half to the enumeration as §6.5 required. [xtask/src/main.rs]
- [x] [Review][Patch] `same_origin` refuses >1 `Origin` header but reads `Host` with `.get()` (first value), applying none of that reasoning to the other half of the compare. Guard `Host` multiplicity symmetrically. [crates/opencmdb-bin/src/document.rs:354-370]
- [x] [Review][Defer] HTTP/2 direct (`:authority`, no `Host`) → 403 on every POST — a stated limit, registered to Epic 19; the product deploys behind a reverse proxy (`architecture.md:168`). Pre-existing design, not this change's defect.
- [x] [Review][Defer] `same_origin` scheme-blind — a stated limit, registered. [document.rs]
- [x] [Review][Defer] `is_adoption_conflict` couples to the DB error message text — REFUTED as a live defect (works on 10.11, index name is data not localized prose); a SQLSTATE+name check would be more robust. [document.rs]
- [x] [Review][Defer] DRY: the first-wins dedup loop is hand-rolled where `gap::project`'s doc promises the convention; a second caller (6.4/Epic 7) re-derives it. Extract a helper when the second caller lands. [document.rs:181-191]
- [x] [Review][Defer] `observed_at` round-trip: the `Z`/UTC assumption holds only for a tz-naive `DATETIME` (measured exact for micro-and-coarser instants); a nano instant truncates at INSERT (pre-existing `%.6f`, write-side, harmless — `observed_at` is not a declared field). [repo.rs::load_observation_by_id]
- **Dismissed** (1): `SANCTIONED_READS`' `None`-means-whole-file semantics is a latent blanket-exempt — but it is the SAME deliberate pattern as `SANCTIONED_SITES` (`docker/seed-example.sql` uses `None`), the current entry is `Some(...)`-scoped, and it is not a defect in the shipped state.

---

## 9. Prove-to-red

| id | mutation | predicted |
|---|---|---|
| **M1** | write `origin='manual'` in the new adapter | RED — the sanctioned test verifier's provenance read (`origin` + `origin_obs_id` asserted on the written rows, §6.5) |
| **M2** | drop the new `SANCTIONED_SITES` write entry | RED — the `authorship` write half, **naming `repo.rs` and `insert into declared_attribute`, NOT the fn name** (validation M4 — read the gate's actual message) |
| **M2b** | move the INSERT into `document.rs` | RED — the gate, unsanctioned write site (the epic's AC2 as a mutation) |
| **M3** | route the write through `insert_declared_attribute` (manual, no obs id) | RED — the provenance verifier read; the DDL CHECK cannot carry it (manual rows legally omit the obs id), which is WHY the test reads the columns |
| **M4** | add `State<MySqlPool>` to the handler | **does not COMPILE** — `error[E0277]` on the `Handler` bound (gap-hunt re-measured on the new `DocumentState`; no side door — no `FromRef`, no downcast) |
| **M5** | answer 404 for an already-documented subject | RED — AC2's 409 pinned pair (constructed by raw plant, §4/M5) |
| **M6** | skip the `NothingToDocument` arm (write zero rows, answer 201) | RED — AC2's empty-projection test (a 201 naming zero fields is the lie the variant exists to prevent) |
| **M7** | swap the refusal ORDER (write attempted before the nothing-to-document guard) | RED — an empty-projection subject must answer 422-domain, not reach the write |
| **M8** | commit between the facts read and the write loop | 🔴 **GREEN by structural argument, measured** (gap-hunt: 27/27 green incl. a `tokio::join!` race, three runs — the unique index converts every interleaving into the same 409, and the loser fails on its FIRST insert, persisting no partial row). Honest carrier = the index (M10) + a stated atomicity limit. **Do NOT invent a fake concurrency test to force a red** (5.9's M4/M5 class) |
| **M9** | drop the CSRF check | RED — the cross-site 403 test (end-to-end, credential attached) |
| **M9b** | compare Origin case-sensitively / accept `Origin: null` / miss the two-Origin refusal | RED — the pure-fn table's rows |
| **M10** | drop the unique index from the migration | RED — AC4's raw-SQL second adoption inserts cleanly where a refusal is asserted. ⚠️ **The driver MUST drop+recreate the test database before this run** (second gap-hunt M-a, measured): on an already-migrated DB, editing `0005` reds every DB test with `VersionMismatch(5)` (sqlx checksum), a red carried by the WRONG thing (5.9b's family) — reset, then the honest carrier is the missing-index insert |
| **M11** | make the documented write use a WRONG attr key (`"ip"` for `"ipv4"`) | RED — **AC6's keys-equality test** (the primary carrier); ⚠️ **NOT the J3 step-4 gap oracle** — a wrong key yields an ABSTENTION, not a `Gap`, so J3 step 4 stays green unless it also pins `abstention_count == 0` (validation H2, measured green). J3 step 4 is a CO-carrier only with the abstention assertion in place |
| **M12** | plant a local `"ipv4"`/`"hostname"`/`"mac"` key table in `document.rs` | RED — AC6's SOURCE-SCAN tripwire. ⚠️ **A FAITHFUL copy of `project` does NOT red the keys-equality test** (validation H4, measured 28/28 green — a faithful copy equals the real fn; only a DRIFTED copy reds, and that is M11). The no-copy property needs the textual scan, not a behavioural test |
| **M13** | remove the read-sanction entry (§6.5) | RED — the sanctioned test verifier's provenance SELECT now reds `cargo xtask ci` (naming `repo.rs` + the column) |
| **M13b** | put an UNSANCTIONED provenance SELECT in `document.rs` | RED — the read half, unsanctioned read site |

⚠️ **Predict first, then measure; a divergence is a FINDING. Read each red's panic message one
by one — a mutation named for one thing and applied to another measures the other thing** (three
occurrences across 5.13/5.14/6.1). **Four of this table's rows were already refuted by the
validation build (M8 GREEN, M11's oracle, M12 not-a-red, M2's message) — do not re-derive their
original predictions.** Sixteen ids; M4 is a compile refusal and M8 is green by argument, so the
headline is not "sixteen reds".

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

### Validation obligations — ✅ ALL DISCHARGED 2026-08-14 (results folded into the sections above)

1. **The two-browser bench** — ✅ DONE, §5's result table. **BOTH** Blink AND Gecko measured
   cleanly on all three probes (same-origin `hx-post`, cross-site `<form>` POST, cross-site
   `fetch`), three runs each, identical; the risk the bench existed to catch (a browser omitting
   `Origin` on same-origin XHR) is answered **NO for both engines**, so §5 is NOT re-arbitrated.
   The Gecko cell — first reported unmeasurable — was captured on a second pass by invoking the
   raw snap Firefox binary (the wrapper's single-instance lock, held by the operator's own
   Firefox, was the blocker, not headless itself). §5's mechanism STANDS.
2. **§1's schema reading** — ✅ VERIFIED by both fact-check layers against `0001_initial.sql`
   and the gate; the epic-AC contradiction is SOUND.
3. **§3 multi-value against the corpus** — ✅ MEASURED by both layers: NO committed fixture
   observation carries two facts of one kind (the `multi-nic` family is N single-MAC
   observations). The first-occurrence-wins test therefore needs a SYNTHETIC seed — stated.
4. **M8 executability** — ✅ MEASURED (gap-hunt): M8 is a valid mutation but GREEN by structure
   (the index serialises every interleaving). Its row is now "GREEN by argument", not a
   prescribed red — the 5.9 M4/M5 class caught before dev.

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

---

## Dev Agent Record

### Agent Model Used

Claude Fable 5 (claude-fable-5).

### Implementation Plan (as executed, 2026-08-14)

T1 → T10 in order, red-green against a live `mariadb:10.11.11` (port 13316). Commit `c6632b1`
froze the green state BEFORE the mutation pass (Dev Notes' trap; and the DB was reset before the
M10 run per validation M-a). All four crates compile with `-D warnings`; seven gates green.

- **T1** core: `DocumentRefusal` gains `AlreadyDocumented` + `NothingToDocument`, Display texts pinned.
- **T2** core: `gap::project` made `pub` with a two-consumer rustdoc; no behaviour change (visibility only).
- **T3** repo: `adopt_declared_attribute` (`'adopted'`/`'operator'` literals, obs id bound),
  `load_observation_by_id` returning the whole `Observation` (observed_at read as TEXT — sqlx has
  no chrono feature here), `read_declared_provenance_for_test` (the ONE sanctioned reader);
  `SANCTIONED_SITES` + 1 named write entry.
- **T4** migration `0005_document_guards.sql`: `CREATE UNIQUE INDEX declared_one_adoption_per_field
  (origin_obs_id, attr_key)`; the Epic-7/PK comment; `raw_declared_write_for_ddl_test` WIDENED
  (same name/site, three existing callers updated).
- **T4b** xtask: `SANCTIONED_READS` (Guy's §6.5 arbitration) applied in the gate's read half; FR13
  rationale in its doc.
- **T5** document.rs: `SubjectLookup`→`DocumentPort` with `document_all` (whole gesture, one
  transaction, 409 via the index keyed on its NAME, first-occurrence-wins per key), `StoreDocument`
  (pool INSIDE the impl — 6.1's M4 carrier survives), `AlwaysUnknown` deleted.
- **T6** the CSRF `same_origin` check, decided FIRST in the handler; the stated limits in its doc.
- **T7** the handler: exhaustive `DocumentRefusal` mapping (no `_` arm), 201 body, 500 arm;
  `main::app` wires `document::router(pool)` above the layer.
- **T8** the J3 end-to-end test (`document_all_closes_the_gap_end_to_end`): document → provenance
  via the sanctioned reader → `gap::reconcile` shows `gaps.is_empty()` AND `abstention_count == 0`.
- **T9/T10** below.

### Debug Log — prove-to-red (T9), predictions FIRST, each carrier read from its own message

| id | mutation | result |
|---|---|---|
| M1 | adapter writes `origin='manual'` | **RED** — J3 provenance assertion `"manual" != "adopted"` |
| M2 | drop the `SANCTIONED_SITES` adopt entry | **RED** — authorship gate: `repo.rs:160: insert into declared_attribute … — NFR5` (names repo.rs + keyword, NOT the fn — validation M4 confirmed) |
| M2b | move the INSERT into `document.rs` | **RED** — gate: `document.rs:153: insert into declared_attribute` unsanctioned |
| M3 | write via `insert_declared_attribute` (manual, no obs id) | **RED** — J3 provenance `"manual" != "adopted"` |
| M4 | add `State<MySqlPool>` to the handler | **does not COMPILE** — `error[E0277]` on the `Handler` bound; no side door (no `FromRef`, no downcast) |
| M5 | `AlreadyDocumented` arm answers 404 | **RED** — `an_already_documented_subject_answers_409`: `404 != 409` |
| M6 | remove the `NothingToDocument` guard in the port | **RED** — `an_empty_projection_subject_answers_nothing_to_document…`: `201 != 422`. 🔴 **First measured GREEN** (no store-level empty-projection test existed — the handler-arm test uses an in-memory port and pins only the mapping); a store-backed Rtt-only test was ADDED, after which M6 reds. *A guard placed where the defect cannot occur reads as coverage and is none — the Epic-5 class, caught in the pass.* |
| M7 | move the empty-projection guard AFTER the write loop | 🟢 **GREEN by structure** — the write loop over an empty `fields` is a no-op, so the guard's position relative to the (empty) loop is invisible; the empty case still answers 422. The guard's real carrier is M6. Divergence from the prediction, recorded. |
| M8 | commit between the facts read and the write loop | 🟢 **GREEN by structure** (pre-decided at validation): the unique index serialises every interleaving; honest carrier = the index (M10) + the stated atomicity limit. Not re-run — constructing it is artificial. |
| M9 | drop the CSRF check | **RED** — `a_cross_site_origin_is_refused_403…`: `422 != 403` (the malformed body was consulted, which is exactly what the 403 wins over) |
| M9b | authority compare case-SENSITIVE | **RED** — `same_origin_decides_each_case` (the case-insensitive assertion). 🔴 The sub-mutation *accept `null`* came back GREEN: the explicit `== "null"` check is **DEAD** (a `null` origin carries no `://` and is refused by that path anyway). The dead check was REMOVED; the `null`-refused tests stay green through the no-scheme path. Divergence → a cleanup. |
| M10 | drop the index from `0005` | **RED** — `a_second_adoption…is_refused_by_the_index`: `the index must refuse the second adoption`. ⚠️ The DB was DROP/CREATEd before this run (validation M-a: on an already-migrated DB the edit reds every test with `VersionMismatch(5)`, a checksum-carried red). |
| M11 | documented write uses a WRONG key (`"ip"` for `"ipv4"`) | **RED** — J3's `abstention_count == 0` assertion: `{NoObservedValue: 1}` (validation H2 confirmed: a wrong key yields an ABSTENTION, not a gap; the dual `gaps` AND `abstentions` assertion is what carries it) |
| M12 | plant a local `"ipv4"`/`"hostname"`/`"mac"` key table in `document.rs` production code | **RED** — `the_projection_is_shared_not_copied` source-scan tripwire names the planted line |
| M13 | remove the read-sanction entry (§6.5) | **RED** — gate: `repo.rs:188: a read of declared_attribute names origin_obs_id — FR13` |
| M13b | plant an UNSANCTIONED provenance SELECT in `document.rs` | **RED** — gate: `document.rs:153: a read … names origin_obs_id — FR13` |

**Sixteen ids: THIRTEEN reds, M4 a compile refusal, M7 and M8 GREEN by structure.** Carriers
MIXED and named per row: M2/M2b/M13/M13b are gate-message-carried; the rest assertion-carried;
none `.expect()`-carried. **Three findings the pass produced, each fixed or recorded**: M6's
missing store-level empty-projection test (added), M9b's dead `null` check (removed), M7's
green-by-structure (recorded, its real carrier is M6). *"every red assertion-carried" is NOT
claimed* — the gate reds are the gate's own output, honestly.

### Completion Notes

- **All 9 ACs MET.** AC1 (201 + adopted provenance through the adapter, one sanctioned write
  site — M1/M2/M2b/M3); AC2 (the total ordered taxonomy, no `_` arm, each refusal reachable
  through the real port, no-write asserted — M4-compile/M5/M6/M9-family); AC3 (CSRF Origin check,
  the stated limits pinned — M9/M9b, the `same_origin` table); AC4 (the index bites, DB-reset
  driver step — M10); AC5 (J3 end-to-end, gaps AND abstentions — M11); AC6 (`project` shared, the
  source tripwire — M11 drift + M12); AC7 (seven gates green, authorship over FOUR write sites +
  ONE read site, 28 fixtures, trap gate still RED 26/15/11, fmt/clippy clean, no new crate,
  templates and `page.rs` zero diff); AC8 (register updated below); AC9 (live count HERE).
- ⚠️ **ONE LIVE COUNT, HERE (AC9): 566 → 580 tests — 357 bin + 161 core + 62 xtask** — verified
  against the live `mariadb:10.11.11`, 0 failed; the twins cite this file.
- **The canonical-form closure is measured**: the J3 test documents through a BRACED spelling of
  the real subject and the stored `origin_obs_id` is canonical (201).
- **`page.rs` byte-identical, templates untouched, `epics.md` not edited, no new crate** (verified).
- **Register (T10)**: appended below to `deferred-work.md`, re-read against AC8's list.

### File List

- `crates/opencmdb-core/src/document.rs` — `AlreadyDocumented` + `NothingToDocument`, tests
- `crates/opencmdb-core/src/gap/mod.rs` — `project` made `pub`, two-consumer rustdoc
- `crates/opencmdb-bin/src/repo.rs` — `adopt_declared_attribute`, `load_observation_by_id`,
  `read_declared_provenance_for_test`, widened `raw_declared_write_for_ddl_test`, AC4 index tests
- `crates/opencmdb-bin/src/document.rs` — `DocumentPort`/`Documented`/`DocumentFailure`,
  `StoreDocument`, the CSRF `same_origin` check, the store-backed handler, tests
- `crates/opencmdb-bin/src/main.rs` — `document::router(pool)` wiring; the fourth broken 6.1 test
  DB-gated; the J3 and empty-projection DB tests
- `crates/opencmdb-bin/migrations/0005_document_guards.sql` — NEW: the unique index
- `xtask/src/main.rs` — `SANCTIONED_SITES` +1 write; `SANCTIONED_READS` NEW; gate read half
- `_bmad-output/implementation-artifacts/deferred-work.md` — register (T10)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — 6.2 status
- `_bmad-output/implementation-artifacts/6-2-route-writes-a-declared-value.md` — this record
- `CLAUDE.md`, `docs/project-context.md` — the 6.2 paragraph, live-count reference

### Change Log

- 2026-08-14 — story 6.2 implemented (T1–T10): the route WRITES an adopted declared value; the
  gate gains a read-sanction (Guy's §6.5); J3's corrected half measured end-to-end. 566 → 580
  tests, seven gates green. Mutation pass: 13 reds + M4 compile + M7/M8 green-by-structure; three
  findings (M6's missing test added, M9b's dead null check removed, M7 recorded). Status →
  `review`.
- 2026-08-14 — three-layer code review: 13 findings after dedup (6 patch, 1 decision→deferred by
  Guy, 4 defer, 1 dismissed). 🔑 The core design was MEASURED sound — every re-executed mutation
  matched the record, and the Edge Case Hunter refuted the concurrency/rollback/CSRF/read-sanction
  concerns by measurement. Patches applied: AC9's leaked count removed from `sprint-status.yaml` +
  both twins (F2 recurrence); the absent-Origin test rewritten to actually send no Origin (it
  passed for the wrong reason); the 201 body oracle tightened to `"2 field(s)"`; AC6's direct
  keys==`gap::project` oracle added to the J3 test (proven red under a wrong key); §6.5's gate
  module-doc enumeration expanded for the read half; `Host` multiplicity guarded symmetrically
  with `Origin`. The two new guards proven red-when-broken. Deferred (Guy): a same-key multi-value
  observation documents "success" while its gap stays open — NOT reachable today, owned by the
  model/connector story. Live count unchanged (580; tests modified, not added). Stays `review` —
  `done` is the MERGE's business.
