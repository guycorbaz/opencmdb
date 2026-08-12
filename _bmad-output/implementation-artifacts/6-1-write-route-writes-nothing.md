# Story 6.1: The write route exists, and it writes nothing

Status: ready-for-dev

<!-- 🔴🔴 **DO NOT DEVELOP THIS STORY AS WRITTEN. IT WAS VALIDATED ON 2026-08-12 AND ITS CENTRAL
     PREMISE IS REFUTED.** `ready-for-dev` here means only *"a story file exists"* — the vocabulary's
     own definition — and NOT that it is fit to build. **§0 is the rewrite brief; read it first.**

     The validation is what this project's mandatory two-layer pass exists for, and it paid for
     itself: 3 HIGH from the fact-check layer, 5 HIGH from the gap-hunt layer that BUILT the story,
     and of fourteen mutations **two came back GREEN with two more green in the shape an implementer
     would naturally write.** Both layers found the same central defect independently. -->

## 0. 🔴 REWRITE BRIEF — what the validation of 2026-08-12 established

**Three findings kill the design as written. Two more change its scope. One is intendance.**

### 0.1 The central mechanism does not work in the shape §6 prescribes (BOTH layers, independently)

§2 claims *"`auth_deny` is a deny-by-default middleware over every route… so a new POST answers 401
by construction"*. **False, and order-dependent.** axum 0.8's own doc: *"the middleware is only
applied to existing routes… Additional routes added after `layer` is called will not have the
middleware added."* Measured on axum 0.8.9, in the exact conditional shape §6 asks for:

```
BEFORE-layer status = 401 Unauthorized
AFTER-layer  status = 200 OK
conditional-after-layer status = 200 OK    ← §6's shape
```

The gap-hunt reproduced it end-to-end: a POST with **no `Authorization` header at all** answered
**400** — the handler ran, `auth_deny` never did, and `metrics::HTTP_REQUESTS` never counted it.
🔴 **So the exposure the whole story exists to force into the open would happen silently: AC3's test
stays green and M5 reds nothing.** Epic 5's named defect class, on this story's own central mechanism.

### 0.2 §2 contradicts §4, and the precedent §4 adopts is what refutes it

`/metrics` works and is **not** in `is_public` — it has its own branch (`auth.rs:25-30`). So
*"both are required for the route to work"* is false. And the placement decides everything:
**token check in `auth_deny`** → AC3 works, `is_public` untouched, M5 still reds; **token check in
the handler** → `auth_deny`'s default arm 401s first and **every token test passes vacuously**.
The story names neither placement.

### 0.3 🔴 THE TOKEN DOES NOT REACH THE BROWSER — arbitration 2 does not cover the gesture it exists for

A browser form or `hx-post` sends **no `Authorization` header** (measured: 401). For story 6.4's
button to work, the shared token would have to be handed to the page's JavaScript — and `/` is on
the **public** allowlist. **That serves the token to any stranger on the network**, which is the
exact principal arbitration 2 exists to exclude.

**The proposal on the table, not yet decided (Guy, end of 2026-08-12): HTTP Basic.** It is the only
mechanism where the browser sends `Authorization` by itself, on every request to the origin,
including HTMX's. One code path (a second scheme on the same header), no JavaScript, no cookie, no
session store; `curl -u` covers the machine caller; a cross-site form cannot forge the header, so
CSRF is closed **for as long as the credential never becomes a cookie**.
⚠️ **It requires the page to leave `is_public`** — the UI stops being publicly readable, which
`auth.rs`'s own doc already anticipates (*"the public UI moves behind sessions and the seam keeps its
shape"*). ⚠️ **And its load-bearing mechanism is UNMEASURED**: whether an `hx-post` carries the Basic
credential once the browser holds it. Browsers differ between a navigation challenge and an XHR.
**Measure it with a real browser before rewriting — do not assert it.** Two assertions about
mechanism were already wrong today.

### 0.4 "Writes nothing" was carried by nothing — and the fix is built and measured

**M4b: a handler that writes `declared_attribute` through the sanctioned adapter and ROLLS BACK
leaves all 532 tests and all seven gates green.** ⚠️ The `authorship` gate is green too — it keys on
the SQL's *site*, and `insert_declared_attribute` is sanctioned for any caller.
🔑 The gap-hunt **built the correction and measured it**: a `SubjectLookup` trait, `Arc<dyn …>`, and
`axum::extract::FromRef` projecting it out of an `AppState` so existing `State<MySqlPool>` handlers
are untouched — ~35 lines. Under it **M4b does not compile** (`E0599`). *The carrier moves from
nothing to the compiler*, at no cost in tests, gates or dependencies.

### 0.5 Two ACs are unreachable as written

- **AC2** — *"the subject is unknown"* is a STORE question, while the Dev Notes tell the implementer
  the story *"should need very little database"*. Measured: M3 reds 1 test with a database and
  **0 without** (533 green). The read-only seam of §0.4 makes both branches reachable in memory —
  measured reddening with no database at all. ⚠️ And the AC must demand the **discriminating** pair:
  that a *known* subject does not answer `UnknownSubject`, or the refusal is unconditional.
- **AC1** — the natural status for `UnknownSubject` is **404**, which is exactly what axum returns
  for an unregistered route. Measured: with the route wrongly registered and the switch unset, the
  call answered **404, body `"the subject is unknown"`**. **Change one character in the test's
  request body and M1 goes green while the route IS registered.** AC1 must name the discriminator —
  the body, or a status that cannot collide — and pin the request it sends.
  ⚠️ Related: with the switch unset the caller sees **401**, not 404, because `.layer()` wraps the
  fallback too (already pinned at `main.rs:511-515`).

### 0.6 Smaller, all measured

- **The `vocabulary` gate is inert here.** `epics.md` is not in its `DOCS` list and no pair names
  `promote`; AC4's carrier must be a test this story writes. ⚠️ And it cannot be gated by a word ban:
  `promote` occurs three times as ordinary English in the committed tree.
- ⚠️ **AC4 collides with AC6**: writing *"not `accept-as-declared` (retired)"* in a doc comment
  **reds the `vocabulary` gate** — `CODE_RETIRED` forbids the string outright in `crates/` and strips
  no comments. Measured.
- **§7's colour collision is conditional**, not certain: it reds only if 6.4's rule begins
  `.identity`; on `.refresh`'s precedent (a top-level class) it stays green. And `--accent` is
  already used at `app.css:62,77,78,83` — the *"never decorative"* comment is not held by its file.
- **The switch's variable, the route's path, the HTTP method, the request shape and what "a subject"
  IS are named nowhere**, yet M5 mutates *"the route's path"* and AC4 governs naming.
- **What copying `scrape_authorized` copies**: `bearer` lowercase is refused though RFC 7235 makes
  the scheme case-insensitive; two `Authorization` headers right-then-wrong reach the handler; and
  **a token containing a non-ASCII character refuses everyone, permanently, with no diagnostic** —
  which matters for a French operator setting `sécret`. Also `==` on `String` is not constant-time.
- **AC5 has no carrier at all**; **M7b and M8 are green in the natural test shape** (the empty-token
  guard fires only for a request presenting `Bearer ` exactly; the prefix guard reds on a superstring
  and not on the obvious substring).
- **Two divergences from `epics.md` go unregistered** while a third is registered: `epics.md:1750`
  lists *"the absent switch"* as an enumerated refusal, and `epics.md:1746` records arbitration 2
  **with no token**. Epic 5 ate this exact defect three times.
- **`server_error` is cited for something it does not do** — it collapses every classified variant to
  500. There is no per-variant status mapping anywhere in the tree.
- ⚠️ **L-1, and it touches open issue #38**: the tests mutate two process-global env vars with no
  stated serialization, in a suite that runs threads in one process.

### 0.7 Intendance

⚠️ **This file was on NO branch when it was validated** — untracked in the working directory.
*A validation pass on an uncommitted file is a pass on something that can vanish.* Committed
2026-08-12 with this brief.

---

## Story

As the operator,
I want the product to expose the SHAPE of a documenting action before it can perform one,
So that the route's refusals are settled while nothing is at stake.

**And as the next developer, I want the exposure decision to be impossible to take by accident** —
because the seam that protects this route today refuses it by default, and the obvious way to make
the route work is also the act that makes it public.

---

## What this story does NOT do

- it does **not** write a `declared_attribute` row. **Nothing at all is written** — that is story
  6.2, and the split follows story 5.3's precedent: the vocabulary ships before the engine, so the
  refusals are testable before any write path exists;
- it does **not** touch `SANCTIONED_SITES` (story 5.12's authorship gate). No write, no site;
- it does **not** put a button on the page. That is story 6.4, and a diff touching
  `crates/opencmdb-bin/templates/` is a FINDING;
- it does **not** implement SESSIONS — no login, no users, no cookies. That is Epic 19's, and
  refusing it here is a choice rather than an oversight. ⚠️ **What ships is caller authentication by
  a shared token**, and §4 states exactly what that buys and what it does not;
- it does **not** add a dependency, and it does **not** edit `epics.md`.

---

## 1. Arbitrations

| # | when | question | decision |
|---|---|---|---|
| 1 | Epic 6 decomposition | the documenting gesture opens the epic | 🔴 **Inherited** — issue #85, and the reason is measured: J3 wants a gap *detected AND corrected*, and the correction has no surface. |
| **2** | contexting, 2026-08-12 | what stands between a stranger on the network and the declared records? | 🔴 **A SHARED TOKEN — `OPENCMDB_WRITE_TOKEN` + `Authorization: Bearer`, on `/metrics`'s existing pattern** (§4). Refused: joining the public allowlist (defensible on a private network, and the exposure would have been registered), and real sessions (Epic 19's, and refusing it is a choice rather than an oversight). ⚠️ **The question was asked badly twice** — it conflated *"is the feature enabled"* with *"who may call it"*. The switch answers the first and is NOT a security decision; the token answers the second. **The story ships both.** |
| 3 | contexting | the epic says *"promote"*; the architecture says the gesture is `document` | 🔴 **`document` wins** (§3). |

---

## 2. 🔴 The finding that changes this story: the route is ALREADY denied

`auth.rs`'s `auth_deny` is a **deny-by-default middleware over every route**, and its default arm
refuses everything that is not explicitly allowlisted:

```rust
fn is_public(path: &str) -> bool {
    path == "/" || path == "/gap" || path == "/healthz" || path.starts_with("/assets/")
}
```

**So a new `POST` route answers `401` by construction, switch or no switch.**

🔑 **And that is the trap this story exists to disarm.** A developer implements the switch, calls the
route, gets `401`, and "fixes" it by adding one path to `is_public` — **and that single line is the
exposure decision**, taken while debugging, with no document recording it.

**Therefore:**

- the switch decides whether the route **EXISTS** (is registered on the `Router`);
- `is_public` decides whether it is **REACHABLE** without a credential;
- **both are required for the route to work, and only the second is an exposure.**

⚠️ **The story's acceptance criteria pin BOTH**, and AC3 exists so that the allowlist entry cannot be
added silently. `auth.rs`'s own doc calls its allowlist *"an EXPLICIT, temporary allowlist"* and says
*"when real user auth lands, the public UI moves behind sessions and the seam keeps its shape"* —
real sessions are **Epic 19**, seven months out in the project's Gantt.

---

## 3. 🔴 The gesture is called `document`, not `promote`

`architecture.md:3818` fixes the vocabulary:

| gesture | code name | FR UI | effect |
|---|---|---|---|
| Close the gap — write observed values into the declared record | **`document`** (`document-field` / `document-all`) | **« Merger »** | the gap **closes**; the observed record is untouched, the link holds |

⚠️ **`epics.md`'s Epic 6 line says *"a one-click promote"*** — that wording predates the
decomposition and is not the canonical name. **Use `document` / `document-all` in code, routes,
locale keys and tests.** Introducing a third word for one gesture is exactly what D65's
`vocabulary` gate exists to prevent, and its retired list already carries two spellings of an
earlier name for this same act (`accept_as_declared`, `accept-as-declared`).

**This story implements `document-all`'s SHAPE only** (FR13(a), *the day-one case*).
`document-field` is **Epic 7's**, which the FR coverage map already assigns.

---

## 4. 🔴 Two orthogonal questions, two mechanisms — and the story ships both

**The contexting asked one question badly, twice, and the correction is the section.** *"Does the
write route carry authentication?"* conflated two things that have nothing to do with each other:

| question | mechanism | is it a security decision? |
|---|---|---|
| **is the feature enabled?** | the **switch** — an env var; without it the route is not registered | **NO.** It distinguishes *configured* from *unconfigured* and says nothing about WHO calls |
| **who may call it?** | the **token** — `OPENCMDB_WRITE_TOKEN` + `Authorization: Bearer` | **YES.** This is the one that stands between a stranger on the network and the declared records |

### The token — and the pattern is already in the file

`auth.rs`'s `scrape_authorized` protects `/metrics` in twelve lines: read
`OPENCMDB_METRICS_TOKEN`, refuse if unset or empty, compare
`Authorization: Bearer <token>`. **The write route follows it exactly**, with its own variable.
*(Guy's arbitration, 2026-08-12.)*

**Two properties come free with that pattern and must both be kept:**

- **unset token → no write, ever.** `scrape_authorized` returns `false` when the variable is missing
  OR empty — the secure default, and the reason it is safe is that it fails closed;
- **the comparison is on the whole header**, not a prefix.

⚠️ **State the strength honestly, and no higher.** A shared token authenticates the CALLER, not a
PERSON: there are no users, no sessions, no revocation, and everyone who holds it is the same
principal. It is real, and it is crude. **Epic 19 is the closure** — registered, not implied. 🔑 On
story 5.12's precedent, where a gate was narrowed to *"a tripwire against the good-faith violation,
never a barrier against a determined one"*: **write the narrow true sentence and do not let the doc
comment grow into a promise.**

### What was refused, recorded because both were defensible

- **joining `auth.rs`'s public allowlist** — anyone reaching the port could write. Defensible on a
  private network, and the exposure would have been registered rather than hidden. ⚠️ **The blast
  radius is real but bounded**: NFR5 keeps the observed side untouchable, so a bogus declared value
  surfaces as a divergence — visible and reparable, a soiling rather than a catastrophe. **That
  bound is what made the option arguable; it is not what made it chosen.**
- **real sessions now** — Epic 19's scope, seven months out in the project's Gantt, and it would
  have grown Epic 6 by a whole slice.

### The switch, kept — because it answers the other question

A deployment nobody configured does not carry the route at all. ⚠️ **It is NOT authentication, and no
document, comment or commit message may call it so.** With the token in place its value is smaller
than the first framing suggested — it is defence in depth and an off-by-default posture, not a
guard. Say that, rather than letting two mechanisms share one justification.

## 5. The refusal taxonomy, and where each half lives

**D47 is the constraint**: `opencmdb-core` must not depend on `axum` — *an error there is domain
data, not a string*. So the taxonomy splits, and the split is the design:

| refusal | nature | where it lives |
|---|---|---|
| the switch is unset | deployment | **bin** — the route is not registered, so there is nothing to refuse; the 404 is axum's |
| the request shape is wrong | HTTP | **bin** — axum's extractor rejection, mapped deliberately rather than defaulted |
| the subject is unknown | **domain** | **core** — an enumerated variant, on `RepositoryError`'s and `ConnectorError`'s precedent |

⚠️ **Do not put an HTTP status inside `opencmdb-core`.** The mapping from a domain refusal to a
status code is `opencmdb-bin`'s, next to the handler. `page.rs`'s `server_error` is the existing
idiom for that direction.

**And every refusal must be enumerated, not a `_` catch-all.** Story 5.3's whole subject was an
abstention vocabulary shipped before the engine that would use it; the reason it holds is that a new
variant produces `error[E0004]` at every match rather than falling into a silent arm.

---

## 6. Where the code goes

| file | what |
|---|---|
| `crates/opencmdb-bin/src/document.rs` | **NEW.** The route, its refusals, its tests |
| `crates/opencmdb-bin/src/main.rs` | `mod document;`, the conditional `.route(...)`, and ⚠️ **the `is_public` decision (AC3)** |
| `crates/opencmdb-bin/src/auth.rs` | the allowlist entry, **if AC3 decides it belongs there** |
| `crates/opencmdb-core/src/…` | the domain refusal variant only |

⚠️ **Not in `page.rs`.** Its code half is **516 lines** and it is the *reading* surface; a writing
handler in it would make one file both. The `file-size` gate counts the lines before the first
`#[cfg(test)]` and its ceiling is 2000 — `page.rs` is nowhere near it, so **this is a separation of
concerns, not a size constraint**, and the story says which.

---

## 7. 🔴 A conflict this story CREATES for story 6.4, recorded now

`app.css:12` reads: `--accent: #d99a4e; /* amber — reserved for the document action, never
decorative */`. The UX spec agrees — *"Document is the only high-contrast element, a single chromatic
accent per card"* — and binds the key **E** to it.

⚠️ **And story 5.14b shipped a test asserting that the identity section's rules NEVER reach for
`--accent`.** When story 6.4 puts the Document button in that section, **the button is legitimately
amber and that test will red.**

🔑 **It is not a defect on either side**: 5.14b's check was right that a *reach counter* must not
borrow the document action's colour, and 6.4 will be right that the *document button* is that action.
**Story 6.4 must revisit the guard rather than delete it** — narrow it to the counter and the cause
lines, leaving the action its colour. Recorded here because 6.1 is where the collision becomes
visible, and a conflict discovered at implementation time gets resolved by whoever is annoyed.

---

## 8. What must be pinned

- **the route does not exist without the switch** — and the test must distinguish *not registered*
  from *registered and refusing*. ⚠️ Both answer with an error; only one of them is the requirement;
- **every enumerated refusal is reachable**, each by a test that constructs its condition. ⚠️ **Epic
  5's dominant defect class, named at its retrospective: *a guard placed where the defect cannot
  occur reads as coverage and is none.*** For each refusal, name the code path where it is produced,
  and check the test drives THAT path;
- **nothing is written.** The strongest form is not "no row appeared" — it is that **the handler
  holds no writing capability at all**. A test asserting an empty table passes for a handler that
  writes and rolls back, and passes for one that writes to the wrong table. Assert the absence at the
  narrowest place the design allows, and **say in the doc which of the two it is**;
- **the allowlist decision is visible in a test**, so AC3 cannot be satisfied by a line added while
  debugging.

---

## Acceptance Criteria

**AC1 — the route exists only when the switch is set.**
**Given** a default deployment with the switch unset
**When** the documenting route is called
**Then** it is **not registered** — the router does not carry it.
**And** the test distinguishes *not registered* from *registered and refusing*: both produce an
error response, and only the first is what this AC asserts.
_Reddened by: M1._

**AC2 — every refusal is enumerated, reachable, and writes nothing.**
**Given** the switch set and the route registered
**When** it is called with a malformed request, or with a subject that does not exist
**Then** each answers its own enumerated refusal — no `_` arm, no string error crossing the D47
frontier — and **no row is written anywhere**.
_Reddened by: M2, M3, M4._

**AC3 — 🔴 the route is reachable ONLY with the token, and the exposure decision cannot be taken while debugging.**
**Given** the token set and the correct `Authorization: Bearer` header
**When** the route is called
**Then** it is reached (and refuses on its own enumerated grounds, AC2).
**Given** the token unset, or empty, or a wrong header
**When** the route is called
**Then** `401` — **it fails CLOSED**, exactly as `scrape_authorized` does for `/metrics`.

**Given** `auth_deny`'s deny-by-default seam, whose default arm refuses every path not in
`is_public`
**When** the route is registered
**Then** whether it is reachable is decided **in the open**, with a test that reds if the allowlist
changes silently.
⚠️ **This is the story's real subject.** Implementing the switch, seeing `401`, and adding one path
to `is_public` "to make it work" is the failure mode; the test is what makes that line a decision
rather than a fix.
_Reddened by: M5._

**AC4 — the vocabulary is `document`, everywhere.**
Route, module, types, tests and any string use `document` / `document-all` — not *promote*, not
*accept-as-declared* (retired, D65), not *merge*. ⚠️ `epics.md`'s Epic 6 line says *"promote"*; the
architecture's table is what governs, and this story does not edit `epics.md`.
_Reddened by: M6._

**AC5 — the two mechanisms are never conflated, in any wording.**
The **switch** is never called authentication — not in code, not in a doc comment, not in the commit
message; its doc says it distinguishes *configured* from *unconfigured* and nothing about who calls.
The **token** is never called a session: it authenticates a CALLER, not a person — no users, no
revocation, everyone holding it is the same principal. **Epic 19's closure is registered, not
implied.**

**AC6 — gates and corpus untouched.** `cargo xtask ci`: seven gates green, 28 fixtures; trap gate
still RED at 26/15/11 (this story implements no `l2-*` rule). `cargo fmt --all --check` clean —
⚠️ story 5.14b's code review found it red and CI fails before any test runs.

**AC7 — the register.** §7's colour conflict, §4's Epic 19 closure, and the `document-field` boundary
with Epic 7 appended to `deferred-work.md`. ⚠️ **Re-read afterwards and check each row landed
against the REQUIREMENT, not against your own output** — story 5.14b's AC10 failed on exactly that,
and its own re-read counted the seven bullets written against the seven bullets written.

**AC8 — documents in the same commit, ONE live count in ONE place.**

---

## Tasks / Subtasks

- [ ] **T1 — the domain refusal** (AC2, AC5): the enumerated variant in `opencmdb-core`, no `axum`
- [ ] **T2 — the route and the switch** (AC1, AC4): `document.rs`, conditional registration
- [ ] **T3 — the token and the exposure decision** (AC3): `scrape_authorized`'s pattern, fail-closed on unset AND empty, and the `is_public` question decided in the open
- [ ] **T4 — the refusals** (AC2): each reachable, each with the code path named
- [ ] **T5 — prove-to-red** (AC1–AC4): M1–M6, predictions first, each carrier read from its own
      panic message one by one
- [ ] **T6 — the register** (AC7), then re-read against §7's list
- [ ] **T7 — gates and documents** (AC6, AC8)

---

## 9. Prove-to-red

| id | mutation | predicted |
|---|---|---|
| **M1** | register the route unconditionally | RED — AC1 |
| **M2** | accept a malformed request instead of refusing it | RED — AC2 |
| **M3** | accept an unknown subject | RED — AC2 |
| **M4** | make the handler write a `declared_attribute` row | RED. ⚠️ **Predict WHERE it reds.** If only an "empty table" assertion catches it, the guard is weaker than §8 asks — a handler that writes and rolls back passes that |
| **M5** | add the route's path to `is_public` | RED — AC3, and this is the mutation the whole story is for |
| **M7** | accept the request with the token **unset** | RED — AC3's fail-closed half. ⚠️ `scrape_authorized`'s precedent: unset OR empty must both refuse, so **M7b** empties it |
| **M8** | compare only the token's PREFIX instead of the whole header | RED — the comparison is on the whole header |
| **M6** | rename the route to `/promote` | RED — AC4 |

⚠️ **Predict first, then measure, and record every divergence as a finding.** Epic 5 shipped
mutation tables whose headline contradicted their own rows in five consecutive stories, and a
mutation named for one thing and applied to another measures the other thing.

---

## Dev Notes

### Traps, each measured on this project

- ⚠️ **`cargo test --workspace A B` passes TWO filters where cargo accepts ONE** and silently runs
  nothing — filed as a confirmation once before it was caught;
- ⚠️ **Never read a measurement through a truncation** (`head -8` on driver output cost a story a
  false claim in five documents);
- ⚠️ **Commit before mutating; revert the MUTATION, never the FILE** — a `git checkout -- <file>`
  once ate a guard written minutes earlier;
- ⚠️ **`cargo fmt --all --check` runs in CI before the tests** (`ci.yml:56`);
- ⚠️ **`app(pool)` is factored out of `main` precisely so the HTTP surface is testable without
  binding a socket** (`main.rs:150`). That is the seam for AC1 and AC3;
- ⚠️ **`DATABASE_URL` is unset locally**, so DB-backed tests pass by `return`ing and the suite
  reports the same counts either way. The witness they ran is the timing (~0.05 s without a
  database, ~4 s with one). This story should need very little database — if it needs a lot, the
  design has drifted toward 6.2.

### The tree this story extends, to be RE-MEASURED

`master` at `458b9d2` plus PR #87 (Epic 6's decomposition): **523 tests** (302 bin + 159 core + 62
xtask), seven gates green, 28 fixtures, trap gate RED at 26/15/11. ⚠️ **Re-measure rather than
quote** — this figure has drifted three times in this project.

### References

- [Source: `epics.md#Story 6.1`] — and its four decomposition constraints
- [Source: `crates/opencmdb-bin/src/auth.rs:18-40`] — the deny-by-default seam and `is_public`
- [Source: `crates/opencmdb-bin/src/main.rs:150-161`] — `app(pool)`, the router, the layer
- [Source: `crates/opencmdb-bin/src/page.rs:516`] — the reading surface, and why the writer is not here
- [Source: `architecture.md:3818`] — the gesture is `document` (`document-all` / `document-field`)
- [Source: `crates/opencmdb-bin/assets/app.css:12`] — the amber reserved for the document action
- [Source: `xtask/src/main.rs` `CODE_RETIRED`] — `accept_as_declared` is retired vocabulary
- [Source: `prd.md#FR13`, `#NFR5`] — the day-one case, and whose subject the invariant has
- [Source: `_bmad-output/implementation-artifacts/epic-5-retro-2026-08-12.md`] — the named defect class

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

---

## Change Log

| date | what |
|---|---|
| 2026-08-12 | **Arbitration 2 SETTLED — a shared token, `OPENCMDB_WRITE_TOKEN` + `Authorization: Bearer`, on `/metrics`'s existing twelve-line pattern.** 🔴 **And the correction is the deliverable, not the choice**: the question had been asked badly twice, conflating *"is the feature enabled"* (the switch) with *"who may call it"* (the token). **They are orthogonal and the story ships both**, each with its own justification, because letting two mechanisms share one is how a switch ends up being called authentication. Refused, both recorded as defensible: the public allowlist (bounded blast radius — NFR5 keeps the observed side untouchable — which made it arguable, not chosen) and real sessions (Epic 19's). ⚠️ The token authenticates a CALLER, not a person: no users, no revocation, one principal. Stated at that strength and no higher. |
| 2026-08-12 | Created. 🔴 Contexting found that **`auth_deny` already refuses the route by construction**, so the switch alone cannot make it work — and the obvious fix, one line in `is_public`, IS the exposure decision. AC3 exists so it cannot be taken while debugging. Also found: the gesture's canonical name is **`document`**, not the epic line's *"promote"*; and story 5.14b's *"never reach for `--accent`"* guard **will collide with story 6.4's amber Document button**, which is a conflict to resolve rather than a defect on either side. ⚠️ Arbitration 2 (authentication) remains **open**; the story is written on the proposed answer and says so where it bites. |
