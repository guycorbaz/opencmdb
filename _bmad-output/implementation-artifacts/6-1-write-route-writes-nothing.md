# Story 6.1: The write route exists, and it writes nothing

Status: ready-for-dev

<!-- ✍️ REWRITTEN 2026-08-14 on the rewrite brief the validation of 2026-08-12 produced, after Guy
     decided HTTP Basic on 2026-08-14 — a decision taken ON a measurement: the browser bench of
     2026-08-14 (§4) established that an `hx-post` carries the Basic credential.
     The superseded draft and its §0 brief are in git at `db6f63a` (its mutation table has EIGHT
     rows, ten mutation ids counting M4b and M7b).

     ✅ VALIDATED 2026-08-14, two fresh layers (fact-check: 6 findings; gap-hunt: 15 findings,
     4 HIGH), ALL 21 APPLIED — none dismissed. 🔴 The headline killed this rewrite's own central
     claim: the "no-write held by the COMPILER" was FALSE under the design §7 itself prescribed —
     the `FromRef<AppState> for MySqlPool` impl added so `page.rs` keeps compiling made the pool
     extractable from EVERY handler, the documenting one included; the gap-hunt BUILT the design
     and M4 in the good-faith shape COMPILED CLEAN. §7 now prescribes the sub-router shape that
     restores the compiler carrier for real, and drops `AppState` entirely. Second HIGH: the one
     existing test through `/` breaks ONLY in CI (locally it self-skips without `DATABASE_URL`) —
     named in §8 now, not discovered. Third: the prescribed CSRF doc sentence was FALSE (ambient
     authority — the cached credential rides a cross-site form POST; the §4 bench only ever probed
     same-origin initiators). Fourth: AC1's "router's shape" named no mechanism axum offers. -->

## Story

As the operator,
I want the product to expose the SHAPE of a documenting action before it can perform one,
So that the route's refusals are settled while nothing is at stake.

**And as the next developer, I want the exposure decision to be impossible to take by accident** —
the seam that protects this route today refuses it by default, and the obvious way to make the
route work is also the act that changes who may read the product.

---

## What this story does NOT do

- it does **not** write a `declared_attribute` row. **Nothing at all is written** — that is story
  6.2, and the split follows story 5.3's precedent: the vocabulary ships before the engine, so the
  refusals are testable before any write path exists;
- it does **not** touch `SANCTIONED_SITES` (story 5.12's authorship gate). No write, no site;
- it does **not** put a button on the page. That is story 6.4, after Epic 6b's triage screen; a
  diff touching `crates/opencmdb-bin/templates/` is a FINDING;
- it does **not** implement SESSIONS — no users, no cookies, no session store, no login form.
  Epic 19's, and refusing them here is a choice rather than an oversight. ⚠️ **What ships is HTTP
  Basic with ONE shared credential**, and §3 states exactly what that buys and what it does not;
- it does **not** implement CSRF protection — **and the register must carry that as story 6.2's
  obligation, because this route has no effect to forge yet and 6.2's does** (AC7(j); validation
  finding F3);
- it does **not** grow the lockfile — Basic parsing needs base64 decoding, and **`base64 v0.22.1`
  is already in the tree** (transitive, by TWO chains: sqlx-core, and serde-saphyr → rust-i18n —
  measured 2026-08-14, `cargo tree -i base64`). Promoting it to a direct dependency of
  `opencmdb-bin` is one `Cargo.toml` line **pinned to the lockfile's version** (never invent a
  version), compiles nothing new and is NOT "adding a dependency" in the sense this list forbids.
  ⚠️ Say so in the commit body, or the diff line reads as a new crate to whoever audits it;
- it does **not** edit `epics.md`.

---

## 1. Arbitrations

| # | when | question | decision |
|---|---|---|---|
| 1 | Epic 6 decomposition | the documenting gesture opens the epic | 🔴 **Inherited** — issue #85, and the reason is measured: J3 wants a gap *detected AND corrected*, and the correction has no surface. |
| 2 | contexting, 2026-08-12 | what stands between a stranger on the network and the declared records? | First answered *"a shared Bearer token on `/metrics`'s pattern"* — **struck by the validation the same day**: a browser form and an `hx-post` send no `Authorization` header on their own (measured, 401), so the token would have had to be served to the page's JavaScript on a PUBLIC route — handing the credential to the principal it exists to exclude (§0.3 of the superseded draft, in git). |
| **2′** | **2026-08-14** | same question, after the browser measurement | 🔴 **HTTP BASIC, one credential, and the UI leaves `is_public`** (Guy). The browser attaches the credential itself — navigation, subresources, **and HTMX's XHR** (measured, §4) — so ONE mechanism covers the operator's browser and the machine caller (`curl -u`). ⚠️ **The price is paid in the open: the product stops being publicly readable.** `auth.rs`'s own doc anticipated the shape (*"the public UI moves behind sessions and the seam keeps its shape"*); this is that move, with Basic standing where sessions will stand. Epic 19 remains the closure — registered, not implied. |
| 3 | contexting | the epic says *"promote"*; the architecture says `document` | 🔴 **`document` wins** (§5). |
| 4 | 2026-08-14 | does the enable switch survive the Basic decision? | 🔴 **YES, and its meaning is narrowed** (§3): it decides whether the write route EXISTS, nothing else. It is defence in depth and an off-by-default posture. ⚠️ **It is NOT authentication and no document, comment or commit message may call it so** — with Basic in place the temptation inverts: the switch now looks LESS like auth than the token did, and the discipline is the same in both directions. |
| 5 | validation, 2026-08-14 | where does configuration enter the app? | 🔴 **As a PARAMETER of `app()`, never as an env read inside it** (author's decision on gap-hunt F6/F11): `AppConfig { document_enabled, basic: Option<BasicCredentials> }`, parsed and VALIDATED by a pure `AppConfig::from_env() -> Result<…>` that only `run()` calls. Tests construct `AppConfig` directly — **the new tests mutate no env var at all**, which deletes the race class the superseded draft's L-1 finding tied to open issue #38, puts M8's validation on a tested seam, and makes AC1's two router shapes trivially constructible. ⚠️ `OPENCMDB_METRICS_TOKEN` stays an env read in `scrape_authorized` — `/metrics` is deliberately untouched (§3). |
| 6 | validation, 2026-08-14 | does an UNCONFIGURED deployment get the Basic challenge? | 🔴 **NO — pair unset means 401 WITHOUT `WWW-Authenticate`** (author's decision on gap-hunt F13): a challenge nothing can satisfy is an infinite browser dialog on every existing deployment's upgrade path. The challenge header is emitted **only when the pair is configured**, and **only from the default arm** — never on `/metrics`' branch, where M5b pins that Basic is not accepted and the response bytes must not advertise it (F14). |

---

## 2. 🔴 The finding that shaped this story: the deny seam does NOT cover a route added after `.layer()`

The superseded draft's central claim — *"a new POST answers 401 by construction"* — is **false and
order-dependent**, measured on axum 0.8.9 (§0.1, in git; the axum doc sentence is verbatim: *"the
middleware is only applied to existing routes… Additional routes added after `layer` is called
will not have the middleware added"*): the conditional registration the old draft prescribed
produced a route that **bypassed `auth_deny` entirely** — a POST with no credential answered
**400** (the handler ran), and `metrics::HTTP_REQUESTS` never counted it.

**The rule this story therefore installs, load-bearing for every route any later story adds:**

> 🔴 **Every route is registered BEFORE `.layer(auth_deny)` — the conditional included.** The
> switch decides what the `Router` carries; it must never decide on which SIDE of the layer a
> route lands. `app()`'s doc comment carries this rule, and mutation M9 proves the suite reds
> when a route slips below the layer. ⚠️ M9's red is carried by ONE specific test — the
> 401-challenge test **on the write route with the switch SET** (AC3's third block): with the
> switch unset the mutation is invisible (both shapes 401), and the page-path challenge tests
> never touch the mutated registration (gap-hunt F5, measured).

---

## 3. The names, the two mechanisms, and what each answers

**The names, so nothing is left for a debugging session to invent** (superseded §0.6's finding —
the old draft named none of these):

| thing | name |
|---|---|
| the route | **`POST /document-all`** — the vocabulary's own token (§5); GET answers 405, pinned by a cheap assert |
| the request shape | **`Form<DocumentAllRequest>`** — `application/x-www-form-urlencoded`, ⚠️ because **that is what the vendored htmx 2.0.4 posts** (measured: form-values encoding, zero `fetch(`, no `json-enc` extension vendored). A JSON route here would force story 6.4 to vendor an extension or redo this shape (gap-hunt F7) |
| the subject field | **`subject`: an observation id (UUIDv7 text)** — FR13(a) documents a SIGHTING's whole record; the observation is what the reach section's cause line will name |
| the switch | **`OPENCMDB_DOCUMENT_ENABLED`** (read by `AppConfig::from_env` only) |
| the credential pair | **`OPENCMDB_BASIC_USER` / `OPENCMDB_BASIC_PASSWORD`** (same; no collision in the tree, and the names follow the existing `OPENCMDB_*` pattern — measured against all eleven existing vars) |
| the realm | **`Basic realm="opencmdb"`** |

| question | mechanism | is it a security decision? |
|---|---|---|
| **is the feature enabled?** | the **switch** — without it the route is not in the `Router` | **NO.** It distinguishes *configured* from *unconfigured* and says nothing about who calls |
| **who may call — and now, who may READ?** | **HTTP Basic** — the pair, enforced in `auth_deny`'s default arm and over the pages that leave `is_public` | **YES.** This is the one that stands between a stranger on the network and the product |

### Basic — what it buys, measured; and what it does not, stated

**Buys** (each half measured on the bench of §4 or the superseded draft's):

- the browser attaches the credential **by itself** after one challenge — navigation, `<script>`,
  `<img>`, **and the XHR htmx 2.0.4 issues** (the vendored file contains two `XMLHttpRequest` and
  zero `fetch(`, checked not assumed). Story 6.4's button needs no JavaScript, no cookie, no token
  served to the page;
- `curl -u` covers the machine caller with the same code path.

**Does not buy, and the doc must say so on story 5.12's narrow-promise precedent:** one credential
authenticates a CALLER, not a person — no users, no revocation short of changing the variable,
everyone holding it is the same principal, and Basic sends it base64-encoded on EVERY request, so
its confidentiality is TLS's business, which this product does not terminate (a reverse proxy
does — `architecture.md:168`). 🔴 **And it does NOT close CSRF** (validation finding F3, replacing
a false sentence this story nearly shipped): a cross-site form cannot *set* a header, but once the
browser holds the credential it attaches it by AMBIENT AUTHORITY — to a cross-site form POST
included. The §4 bench probed only same-origin initiators and must not be quoted wider. Harmless
in THIS story (the route has no effect to forge); **story 6.2 owns CSRF protection and AC7(j)
registers it. It is real and it is crude. Epic 19 is the closure.**

### The visibility change, priced in the open

`is_public` today allowlists `/`, `/gap`, `/healthz`, `/assets/`. Under this story:

| path | before | after | why |
|---|---|---|---|
| `/healthz` | public | **public** | the liveness probe cannot authenticate |
| `/assets/*` | public | **public, and measured harmless**: assets are CSS/JS/the vendored htmx — style, not data. ⚠️ Re-check at review that no asset carries data | the login-free surface |
| `/`, `/gap` | public | **Basic-challenged** | 🔴 **the UI stops being publicly readable — arbitration 2′'s price, and the FIRST RELEASE CONTAINING THIS STORY names it in its release notes** (today that is Epic 6b's story 6b.12; the obligation follows the release, not the story number — AC7(f)) |
| `/metrics` | Bearer token | **Bearer token, UNCHANGED — and never challenged with Basic** (arbitration 6) | a Prometheus does not answer a Basic challenge; one caller class, one mechanism |
| `POST /document-all` | (new) | **exists only with the switch; Basic-challenged like every non-public path** | §3's table |

### The fail-closed defaults, and the boot refusal

- **pair unset or empty → every non-public path refuses (401), WITHOUT the challenge header**
  (arbitration 6). `scrape_authorized`'s precedent: unset OR empty, both closed. **An
  unconfigured deployment gets narrower, never wider** — the public allowlist still serves, the
  rest was already denied.
- **A credential containing a byte outside ASCII is refused by `AppConfig::from_env` with a named
  error — at boot, never at request time silently.** The superseded draft measured this trap
  (*"a token containing a non-ASCII character refuses everyone, permanently, with no diagnostic —
  which matters for a French operator setting `sécret`"*); Basic inherits it through the base64 of
  `user:password`, and RFC 7617 records that the original scheme *failed to specify* the charset.
  The guard is the pure fn; its test is M8. ⚠️ **The call-site is `run()`, which no test drives —
  state the uncarried call in the doc on story 5.14's precedent** (*"recording an unavoidable
  GREEN is honest; recording it without measuring how much it covers is not"*). The uncovered
  region is one call and one `?`.
- **The comparison covers the WHOLE decoded pair — both halves, and the extent is MEASURED**:
  two tests (right user + wrong password; wrong user + right password) carry it, and M10 (compare
  the user half only) reds them (gap-hunt F8 — the natural single both-halves-wrong test leaves a
  user-only comparison green). ⚠️ `==` on `String` is not constant-time; a stated limit
  (single-operator LAN product, TLS at the proxy), registered AC7(d), not silently "fixed" with a
  new dependency.
- **The scheme match is case-insensitive on `Basic`** (RFC 7235 §2.1, kept by RFC 9110 §11.1) —
  tested with a mixed-case scheme, reddened by M12. The superseded draft caught
  `scrape_authorized` refusing lowercase `bearer`; do not copy that defect into the new arm, and
  do not "fix" `scrape_authorized` here (registered AC7(c)).
- **Decode robustness, each answering 401, each with a test**: garbage base64 after the scheme; a
  decoded pair with no colon; a password CONTAINING a colon (split on the FIRST colon only —
  RFC 7617 §2: the user-id must not contain one, the password may); **two `Authorization` headers**
  (refused outright — the superseded §0.6 measured right-then-wrong reaching the handler through
  `HeaderMap::get`'s first-value semantics).

---

## 4. ✅ The measurement arbitration 2′ stands on (2026-08-14, in the story record, not re-run)

Bench: a local server challenging with `WWW-Authenticate: Basic`, a page loading **the
repository's own vendored htmx 2.0.4**, `hx-post` on `load`, every verdict recorded SERVER-SIDE.
Firefox 153.0.1 headless, fresh profile per run, three runs, identical results:

| probe | request | credential carried |
|---|---|---|
| P1 | the navigation | first NONE (401 challenge), then **YES** |
| P0 / P5 | `<script>` / `<img>` subresources | **YES** |
| P4 | raw XHR POST to a path **never challenged** | **YES** |
| **P2** | **`hx-post`** | **YES** |

🔑 **P4 is the structural lesson**: the browser generalises the credential over the origin — a
second protection space on the same origin is not a boundary the browser keeps. One realm,
therefore; do not design a second.

⚠️ **Limits, so the measurement is not read wider than it is:** Gecko only (no Chromium on the
bench machine — owed, AC7(a)); **every probe was a SAME-ORIGIN initiator, so the bench says
NOTHING about CSRF** (F3 — the false conclusion it nearly licensed is corrected in §3); credential
EXPIRY mid-session unmeasured — ⚠️ though the vendored htmx's default `responseHandling` was READ
during validation: a 4xx is **not swapped** into `#gap-card`, so the residual concern is only the
browser's native dialog, correctly 6.4's (AC7(b)); and the bench's `fetch()` anomaly (no request,
silently, three runs) is recorded WITHOUT a cause — it bears on nothing here because the product's
transport is XHR, but nothing measured through `fetch` on that bench is trustworthy.

---

## 5. 🔴 The gesture is called `document`, not `promote`

`architecture.md:3818` fixes the vocabulary: **`document`** (`document-field` / `document-all`),
FR UI « Merger ». ⚠️ `epics.md`'s Epic 6 line says *"a one-click promote"* — pre-decomposition
wording, not canonical. **Use `document` / `document-all` in code, routes, locale keys and
tests.** This story implements `document-all`'s SHAPE only (FR13(a)); `document-field` is Epic
7's. ⚠️ **The vocabulary gate will NOT catch a violation here** (measured: `epics.md` is not in
its `DOCS` list, and `promote` occurs three times as ordinary English in the tree — plus two
inflected forms, `promoted` and `Promoting`, in fixtures.rs and trap_gate.rs) — AC4's carrier is
a test this story writes, a grep over `document.rs` and the route table, NOT a tree-wide word
ban. ⚠️ And do not write the retired term in a doc comment — the string itself reds the
`vocabulary` gate (`CODE_RETIRED` reads whole files, lowercased, stripping no comments;
measured). Name the retirement by pointing at D65, not by quoting the corpse.

---

## 6. The refusal taxonomy, and where each half lives

**D47 is the constraint**: `opencmdb-core` must not depend on `axum` — an error there is domain
data, not a string.

| refusal | nature | where it lives | status | discriminator |
|---|---|---|---|---|
| the switch is unset | deployment | **bin** — the route is not in the `Router` | see AC1 — asserted by the AUTHENTICATED discriminator, never by the unauthenticated status | the fallback's **empty body** under a valid credential (measured: 404, `b""`) against the registered route's every response, all non-empty |
| no/wrong credential | HTTP auth | **bin** — `auth_deny`'s default arm | **401**, + `WWW-Authenticate: Basic realm="opencmdb"` **iff the pair is configured** (arbitration 6) | the challenge header |
| the request shape is wrong | HTTP | **bin** — `Form` extractor rejection, mapped deliberately | **422** | body names the field |
| the subject is unknown | **domain** | **core** — an enumerated variant on `RepositoryError`'s precedent | **404** ⚠️ colliding with axum's unregistered-route 404 — **the BODY is the discriminator and AC2 pins it VERBATIM**; the fallback's body is empty, so the two 404s cannot be confused by a test that reads the body | the pinned body |

**Every refusal enumerated, no `_` arm** — story 5.3's precedent: a new variant must produce
`error[E0004]`, not fall into a silent catch-all.

**The unknown-subject branch runs WITHOUT a database**: `SubjectLookup`, a read-only trait behind
`Arc<dyn …>`, is the document sub-router's WHOLE state (§7). ⚠️ **The discriminating pair is
mandatory**: a KNOWN subject must NOT answer `UnknownSubject`, or the refusal is unconditional
and the test vacuous. **And the production wiring is stated rather than left to invention**
(gap-hunt F12): **in this story, production wires an `AlwaysUnknown` stub** — a shape-only route
truthfully answers *unknown* for every subject since nothing can be documented yet — **its doc
says so, a test pins it, and story 6.2 replaces it with the store-backed impl.** The known-subject
branch is reached through the in-memory test impl.

---

## 7. Where the code goes — and the shape that makes the no-write claim TRUE

🔴 **The previous draft of this rewrite claimed the no-write invariant was compiler-carried and
the validation FALSIFIED it** (gap-hunt F1, by building it): with a shared `AppState` and
`FromRef<AppState> for MySqlPool` — required so `page.rs`'s extractors keep compiling — the pool
is extractable from EVERY handler, and M4 in the good-faith shape (add one `State<MySqlPool>`
parameter) **compiled clean**. The two halves of that design defeated each other.

**The shape that holds instead — no `AppState`, no `FromRef`, page.rs untouched:**

```text
main router  (Router<MySqlPool>: /, /gap, /assets, /metrics, /healthz)  .with_state(pool)
document sub-router (Router<DocumentState>: POST /document-all)         .with_state(doc_state)
        └── DocumentState = { lookup: Arc<dyn SubjectLookup> }  — NO pool field
merge the two Router<()>  →  .layer(from_fn_with_state(config, auth_deny))  — §2's rule intact
```

- `page.rs` keeps `State<MySqlPool>` extractors **byte-for-byte unchanged** — if this story's
  diff touches their signatures, the seam is wrong;
- the document handler can extract only what `DocumentState` holds. **M4 (add a
  `State<MySqlPool>` parameter) must FAIL TO COMPILE** — predicted `E0277` (no
  `FromRef<DocumentState>` impl for `MySqlPool`); ⚠️ **record the ACTUAL compiler error in the
  story record** — the previous prediction (`E0599`) was measured on a design that then changed,
  and a carrier named without re-measurement is how this story nearly shipped a false claim;
- `auth_deny` takes the config by `from_fn_with_state` (arbitration 5) and keeps reading
  `OPENCMDB_METRICS_TOKEN` from env in `scrape_authorized` — `/metrics` untouched.

| file | what |
|---|---|
| `crates/opencmdb-bin/src/document.rs` | **NEW.** The sub-router, `DocumentState`, `SubjectLookup` + `AlwaysUnknown`, the refusals, the tests |
| `crates/opencmdb-bin/src/auth.rs` | `basic_authorized`, the conditional challenge, `is_public` shrunk to `/healthz` + `/assets/`, the narrowed doc |
| `crates/opencmdb-bin/src/main.rs` | `AppConfig` + `from_env` (pure) + the boot `?` in `run()`, `app(pool, config)`, the merge **above `.layer()`** (§2), and the fix to the ONE test the visibility change breaks (§8) |
| `crates/opencmdb-core/src/…` | the domain refusal variant only — no `axum`, no status |

⚠️ **Not in `page.rs`** (516 code lines, the *reading* surface) — separation of concerns, not
size.

---

## 8. What must be pinned — and the one existing test this story breaks

- **the route does not exist without the switch** — asserted by the authenticated discriminator
  (§6's table), never by the unauthenticated status;
- **every enumerated refusal is reachable, each by a test that constructs its condition and names
  the code path that produces it.** ⚠️ Epic 5's dominant defect class: *a guard placed where the
  defect cannot occur reads as coverage and is none*;
- **nothing is written — the carrier is layered, and each layer is named honestly** (F1's
  correction): **(1)** the pool is unreachable from the document handler BY TYPE (§7) — M4 is a
  compile error, re-measured not quoted; **(2)** a write to `declared_attribute` from
  `document.rs` through any smuggled connection reds story 5.12's **authorship gate**
  (unsanctioned site — the validation's own probe file redded `cargo xtask ci` exactly so);
  **(3)** ⚠️ a handler that opens its OWN connection from env and writes any OTHER table is
  carried by NOTHING — on 5.12's precedent this design is a TRIPWIRE against the good-faith
  mistake, never a barrier against a determined one, and the doc says which;
- **the visibility change is visible in a test** — `is_public`'s new shape pinned path by path;
- **the layer-order rule** (§2) — M9, carried by AC3's write-route challenge test alone (F5);
- **the boot refusal** (M8) — on the pure fn; the `run()` call-site stated as uncarried;
- 🔴 **`index_renders_the_real_gap` (`main.rs:398`) is the ONE existing test the visibility
  change breaks — and it breaks ONLY IN CI** (gap-hunt F2, measured: exactly three
  `app(…).oneshot` sites exist; `/healthz`'s stays public, the auth test probes unaffected paths,
  and this one drives `/` expecting 200 but is `DATABASE_URL`-gated, so it self-skips locally and
  reds only where the database exists). **The story fixes it by constructing `AppConfig` with a
  pair and sending the header — no env mutation, so no lock interaction with `DB_TEST_LOCK`.**
  A green local suite is NOT evidence here; the CI run is.

---

## Acceptance Criteria

**AC1 — the route exists only when the switch is set.**
**Given** `AppConfig { document_enabled: false, basic: Some(pair) }`
**When** `POST /document-all` is called **with the valid credential**
**Then** the layer passes and the FALLBACK answers: **404 with an empty body** — measured
distinguishable from every response the registered route can produce (all non-empty).
**Given** the same config with `document_enabled: true`
**When** the same authenticated call is made
**Then** the route answers (one of AC2's refusals — non-empty body).
⚠️ The discriminator is the AUTHENTICATED response pair; the ban is on concluding anything from
the UNAUTHENTICATED status, which is 401 in both shapes.
_Reddened by: M1._

**AC2 — every refusal is enumerated, reachable, and writes nothing.**
**Given** the switch set, the pair set, and a correctly authenticated caller
**When** the route is called with a malformed form, an unknown subject, and a KNOWN subject (the
in-memory `SubjectLookup` impl)
**Then** the malformed form answers 422 naming the field; the unknown subject answers 404 with
**the exact pinned body**; the known subject does **NOT** answer `UnknownSubject`; and the
no-write invariant holds by §8's three named layers — M4 as a compile error whose ACTUAL
diagnostic is recorded.
**And** the production wiring is `AlwaysUnknown`, pinned by its own test (§6).
_Reddened by: M2, M3, M3b, M4 (compile), and the GET→405 pin._

**AC3 — 🔴 Basic stands where the allowlist stood, fails closed, and the visibility change is a pinned decision.**
**Given** the pair set and a request carrying it
**When** a formerly-public page is called
**Then** it is **reached — anything but 401, and never the challenge** (⚠️ with a lazy pool and no
database the page handlers answer 500 through `server_error`; asserting 200 here would need a
database and belongs to the ONE CI-gated test §8 names).
**Given** the pair set and the header absent, wrong, malformed (§3's decode list), mixed-case
scheme `bAsIc` + correct pair (must be ACCEPTED — M12), or the pair half-right in either direction
**When** any non-public path is called
**Then** **401 with `WWW-Authenticate: Basic realm="opencmdb"`** — except the half-right pairs
and decode-garbage which also 401 (M10 reds the two half-right tests).
**Given** the pair UNSET
**When** any non-public path is called
**Then** **401 WITHOUT the challenge header** (arbitration 6 — no infinite dialog on unupgraded
deployments), and the test asserts the header's ABSENCE.
**Given** the switch set AND the pair set
**When** `POST /document-all` is called with NO credential
**Then** **401 with the challenge** — ⚠️ this exact test is M9's only carrier (§2) and it must
exist by name.
**Given** `is_public`
**When** its shape is tested
**Then** exactly `/healthz` and `/assets/` remain, pinned path by path.
**And** `/metrics` still answers to its Bearer token, does NOT accept Basic (M5b), and its 401
never carries the Basic challenge (arbitration 6 / F14).
_Reddened by: M5, M5b, M7, M7b, M9, M10, M11, M12._

**AC4 — the vocabulary is `document`, everywhere, carried by a test this story writes.**
Route path, module, types, tests and strings use `document` / `document-all` — the carrier is a
test over `document.rs` and the route table (§5: the `vocabulary` gate is inert here, measured),
and no doc comment quotes a retired term.
_Reddened by: M6._

**AC5 — the two mechanisms are never conflated, in any wording, in either direction.**
The **switch**'s doc says *configured vs. unconfigured* and nothing about callers. **Basic**'s
doc states its strength honestly and no higher (§3: caller not person, base64 not encryption,
TLS is the proxy's, **CSRF is NOT closed and 6.2 owns it**, Epic 19 is the closure). ⚠️ The
temptation runs both ways: do not let the switch be described as a safety net for a weak
credential, nor Basic as making the switch redundant. **Two questions, two mechanisms, two docs.**

**AC6 — gates and corpus untouched.** `cargo xtask ci`: seven gates green, 28 fixtures; trap gate
still RED at 26/15/11 (no `l2-*` rule here). `cargo fmt --all --check` clean — CI runs it at
`ci.yml:56`, before the tests.

**AC7 — the register, each row WITH ITS OWNER.** Appended to `deferred-work.md`, re-read
afterwards against THIS list and not against the output (story 5.14b's AC10 failed exactly
there): **(a)** the Chromium half of the Basic measurement — owner: **story 6.2's validation
bench**; **(b)** credential expiry / the native dialog mid-swap — owner: **story 6.4** (the
htmx `responseHandling` measurement in §4 shrinks it to the dialog alone); **(c)**
`scrape_authorized`'s lowercase-`bearer` refusal — owner: **Epic 19**; **(d)** constant-time
comparison — stated limit, owner: **Epic 19**; **(e)** §9's colour conflict — owner: **story
6.4**, with §9's conditionality; **(f)** the release-notes obligation (the UI stops being
publicly readable) — owner: **the FIRST release containing this story**, today Epic 6b's 6b.12;
**(g)** `document-field` — **Epic 7**; **(h)** Basic's closure — **Epic 19**; **(i)** the D37
filename drift (`htmx.min.js` unversioned, `architecture.md:3406`) — owner: **Epic 6b story
6b.1**; **(j)** 🔴 **CSRF protection for the write route — owner: story 6.2**, the story where
the route first has an effect to forge (F3).

**AC8 — documents in the same commit, ONE live count in ONE place.** The story file carries the
final test count; every other document cites it by reference.

---

## Tasks / Subtasks

- [ ] **T1 — the domain refusal** (AC2): the enumerated variant in `opencmdb-core`, no `axum`, no status
- [ ] **T2 — `AppConfig`** (arbitration 5, AC1, M8): the pure `from_env`, the boot `?` in `run()` (uncarried call STATED), `app(pool, config)`
- [ ] **T3 — Basic in `auth_deny`** (AC3): `basic_authorized` via `from_fn_with_state`, the conditional challenge (arbitration 6), `is_public` shrunk, `/metrics` untouched, the decode-robustness list (§3)
- [ ] **T4 — the sub-router** (AC1, AC2, AC4): `document.rs`, `DocumentState` (no pool), `SubjectLookup` + `AlwaysUnknown` pinned, the merge above the layer (§2)
- [ ] **T5 — the broken test** (§8): `index_renders_the_real_gap` updated via `AppConfig` — and verified IN CI, not locally
- [ ] **T6 — prove-to-red** (AC1–AC4): M1–M12, predictions FIRST, each carrier read from its own panic message one by one; M4's compiler diagnostic recorded verbatim
- [ ] **T7 — the register** (AC7), re-read against AC7's list
- [ ] **T8 — gates and documents** (AC6, AC8)

---

## 9. 🔴 A conflict this story leaves for story 6.4, recorded now

`app.css:12`: `--accent: #d99a4e; /* amber — reserved for the document action, never decorative */`
— ⚠️ and the superseded draft measured that comment already false in its own file (`--accent` used
at `app.css:62, 77, 78, 83`). Story 5.14b shipped a test asserting the identity section's rules
never reach `--accent` (`page.rs:1164`), and story 6.4's Document button in that section will be
legitimately amber. **The test reds IF 6.4 styles the button under an `.identity`-prefixed
selector; a top-level class evades the guard entirely** (measured — the scan reads only selectors
starting `.identity`), **which is the second reason 6.4 must re-examine the guard rather than
merely satisfy it**: narrowing it to the counter and cause lines, not deleting it. ⚠️ Epic 6b's
story 6b.1 re-tokenises `app.css` before 6.4 runs — whoever lands second must re-check this guard
still exists and still names what it guards.

---

## 10. Prove-to-red

| id | mutation | predicted |
|---|---|---|
| **M1** | register the route unconditionally | RED — AC1's authenticated discriminator |
| **M2** | accept a malformed form | RED — AC2 |
| **M3** | answer `UnknownSubject` for a KNOWN subject | RED — AC2's discriminating half |
| **M3b** | accept an unknown subject | RED — AC2's pinned body |
| **M4** | add a `State<MySqlPool>` parameter to the document handler | **does not COMPILE** — predicted `E0277` (no `FromRef<DocumentState> for MySqlPool`); ⚠️ record the ACTUAL diagnostic — the previous prediction was measured on a design that then changed |
| **M5** | add `/` back to `is_public` | RED — AC3's pinned list; **the mutation the story is for** |
| **M5b** | make `/metrics` accept Basic | RED — AC3's one-mechanism-per-caller half |
| **M6** | rename the route to `/promote` | RED — AC4's test |
| **M7** | accept with the pair unset | RED — fail-closed |
| **M7b** | accept with the pair set but EMPTY | RED — `scrape_authorized`'s unset-OR-empty precedent |
| **M8** | let a non-ASCII pair pass `from_env` | RED — the pure-fn test |
| **M9** | move the merge BELOW `.layer()` | RED — carried by AC3's write-route challenge test ALONE (§2 says why the page-path tests cannot see it) |
| **M10** | compare only the USER half of the pair | RED — the two half-right tests (§3; the both-halves-wrong shape stays green, which is why the half-right pair is prescribed) |
| **M11** | drop `WWW-Authenticate` from the configured-pair 401 | RED — AC3's challenge assertion, proven live |
| **M12** | match the scheme case-SENSITIVELY | RED — the `bAsIc` acceptance test |

Plus four pinned tests without mutation rows: GET on the route → 405; garbage base64 → 401; no
colon in the decoded pair → 401; a password containing a colon → ACCEPTED (split on first colon);
two `Authorization` headers → 401.

⚠️ **Predict first, then measure, and record every divergence as a finding.** Five consecutive
Epic-5 stories shipped a mutation table whose headline contradicted its own rows; and a mutation
named for one thing and applied to another measures the other thing — read each red's panic
message one by one.

---

## Dev Notes

### Traps, each measured on this project

- ⚠️ **`cargo test --workspace A B` passes TWO filters where cargo accepts ONE** and silently runs nothing;
- ⚠️ **Never read a measurement through a truncation**;
- ⚠️ **Commit before mutating; revert the MUTATION, never the FILE**;
- ⚠️ **`cargo fmt --all --check` runs in CI before the tests** (`ci.yml:56`);
- ⚠️ **`app()` is factored out of `main` so the HTTP surface is testable without a socket** — the seam for every AC here, now carrying `AppConfig` as a parameter (arbitration 5), so **no new test mutates an env var**. The one existing env-mutating test (`OPENCMDB_METRICS_TOKEN`, `main.rs:506-537`, unserialized today) is left as it is — do not enrol it, do not add neighbours;
- ⚠️ **`DATABASE_URL` is unset locally** — DB-backed tests pass by `return`ing. This story needs NO database (both subject branches run in memory; §8's broken test is the one CI-gated exception). If it needs more, the design has drifted toward 6.2.

### The tree this story extends, RE-MEASURED at validation (2026-08-14)

⚠️ **PR #87 is OPEN, not merged** — local and `origin/master` HEAD is `458b9d2` (#86) — and the
correct-course of 2026-08-13 (Epic 6b, which AC7(f) and §9 cite) is **uncommitted working-tree
text at validation time**. 🔴 **The superseded draft's own §0.7 rule applies: a validation pass on
an uncommitted file is a pass on something that can vanish. Commit the correct-course and this
story file before `dev-story` runs.** Measured state: **523 tests** (302 bin + 159 core + 62
xtask, 0 failed, DB tests self-skipped), seven gates green + `views-hash ℹ STALE exit 0`, 28
fixtures, trap gate RED at 26/15/11, `cargo fmt --all --check` clean.

### References

- [Source: `epics.md#Story 6.1`] — ⚠️ its second AC says *"the route does not exist at all: a deployment nobody configured is not writable"* — **compatible**: the switch still governs existence (AC1); what changed is that the READ surface is now also gated, which `epics.md` does not discuss and this story does not edit into it (AC7(f))
- [Source: git `db6f63a`] — the superseded draft and its §0 rewrite brief, with its eight-row mutation table (ten ids counting M4b and M7b) and the two browser measurements
- [Source: `crates/opencmdb-bin/src/auth.rs`] — the deny seam (`is_public` 36-38, `scrape_authorized` 46-58), the doc sentence quoted in arbitration 2′ (lines 6-7)
- [Source: `crates/opencmdb-bin/src/main.rs:150-161`] — `app(pool)`, the router, the layer; the fallback-401 pin at `main.rs:510-515`
- [Source: `architecture.md:3818`] — the `document` vocabulary table; [`architecture.md:168`] — TLS at the reverse proxy
