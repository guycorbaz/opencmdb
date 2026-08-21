# Story 6b.9: Self-diagnostic and commissioning

Status: review

Epic: 6b — *L'interface de la maquette*. **Ninth numbered slot, tenth story file.** It takes the
`Empty` screen count from two to **ZERO**, and it is **the second screen of this epic whose centre
must be real — and the first that reads the database ABOUT ITSELF.**

## Story

As the operator,
I want the tool's self-report to contain facts,
so that a screen about the product's state is not the least reliable screen in it.

## Acceptance Criteria

Transcribed from `epics.md:2262-2278`, **unmodified** — divergences are raised in §0 (a story may not
edit an AC; only a retrospective or a correct-course may).

1. **Given** the facts the product really holds (version, database engine and version, migrations
   applied, last scan and its duration, the reach counts), **when** the diagnostic renders, **then**
   each of those rows is **measured at runtime**, not written into a template.
2. **Given** the rows the product cannot support, **when** they render, **then** they carry the
   marker. 🔴 **The mock's security group asserts properties this product does not have** — *"all
   HTTP surfaces authenticated"* is FALSE (`auth.rs` is a deny-by-default seam with a public
   allowlist its own doc calls temporary, and story 6.1's brief measured a POST to an unknown route
   answering **400** without the middleware ever running), and the credential rows describe Epics 10
   and 19. **A diagnostic screen that states a security property the product lacks is worse than no
   screen: it is a false claim about security, made by the product about itself.**
3. **And** the commissioning screen and its baselining are an example surface (Epic 9), marked as
   such.

**Added by contexting and CORRECTED by validation** (numbered from 4 so the three above keep the
epic's numbering):

4. **Given** AC1's five facts, **when** the screen ships, **then** each is either **measured** or
   **named as unmeasured**. 🔴 The product held **four of the five** at contexting, the fifth — *last
   scan and its duration* — being missing on **BOTH halves**, not one. ✅ **Arbitration 1 was taken
   on 2026-08-21 (option (c′), Guy)**, so AC1 ships **MET on all five**, the fifth scoped to
   *"depuis ce démarrage"* and empty by construction after a restart (§0b).
5. **Given** AC2, **when** the security group renders, **then** it states no security property the
   product does not hold — and ✅ **per arbitration 2 (option (b), Guy, 2026-08-21)** the carrier is a
   **SHAPE, not a word list**: the group has **no
   free-text arm**, every value is a `bool` or a derived list, so a false claim cannot be typed into
   it (§0d). A forbidden-sentence guard is a **second** line, never the first.
6. **Given** the security group, **when** it names the public surface, **then** the row is **derived
   by probing `is_public`**, never a hand-copied literal, and a guard asserts the two agree (§0d).
7. **Given** an unreachable database, **when** `/diagnostic` is opened, **then** the screen still
   renders the half that needs no pool — version, embedded schema, security, journal — because *the
   one state in which an operator opens a self-diagnostic screen is the one where the store is
   down* (§0f, measured: `/sources`, `/triage` and `/dashboard` all answer a bare `500 internal
   error` today).
8. **Given** `Nature::Empty`, **when** this story closes, **then** **there is no `Empty` screen
   left**, and the deletion follows the **checklist** in §0i — the compiler names three sites and is
   blind to five more (§0i).
9. **Given** the two mock buttons, **when** they render, **then** they reuse story 6b.4b's `Gesture`
   type and `_action_bar.html`, and **carry their own render-level a11y guard**: 6b.4b's is scoped to
   `/triage` and covers this screen with **nothing** (§0h, measured).
10. **Given** the whole delivery, **when** `cargo xtask ci`, `clippy -D warnings` and `cargo fmt
    --check` run, **then** eight gates are green and the suite is run **both ways** with both
    wall-clocks recorded; the live count lives in THIS file; and both screens are **looked at in a
    browser**, in French, on a **rebuilt** binary.

---

⚠️ **VALIDATED 2026-08-21 BY TWO FRESH-CONTEXT LAYERS, AND THIS FILE IS THE CORRECTED VERSION.** One
checked every claim against its sources on the committed tree; one **built the screen in its own git
worktree** — `diagnostic.rs`, a template, 18 key pairs, the route on `triage_router` — and ran the
real binary against a live `mariadb:10.11` on port 13322. Between them they refuted **twenty-six
claims of the first draft**, listed in the recountable table at §0j so the number can be checked
rather than believed. **Each refutation is kept in place rather than overwritten**: the corrected
findings are sharper than the ones they replace, and a reader in six months must be able to
re-derive the decisions rather than only read them.

🔴 **The single most consequential refutation: the layer that BUILT the story found that the first
draft's own recommendation on arbitration 1 would have shipped a FALSE FACT** — see §0b. Six stories
running, the compiling layer beats the reading one.

## §0 — What contexting found, and what validation refuted

🔑 **This story is NOT another example screen, and it is not `/sources` either.** `/sources` had one
real section derivable with no database at all. This one asks the product to **read its own
database, its own build and its own configuration** and report them without inventing a single row.
The screen's whole value is that the operator can believe it; a single decorative row destroys the
other fifteen.

### §0a. THE ROW-BY-ROW MEASUREMENT — the mock's sixteen rows against this tree

⚠️ **The MOCK has sixteen; the screen that ships has SEVENTEEN** — the *Journal* group carries five,
because rotation and file retention are two facts and only one of them applies when nothing is
written to a file. This section's title is the mock's count and is correct as such; the Completion
Notes said sixteen about the SCREEN until the code review counted.

The mock is `~/travail/Projets actuels/opencmdb/documents/opencmdb - maquette.html`; its
`diagGroups` array sits at offset ≈490 584 in the de-escaped source. **Four groups of four**, and
the fact-check layer confirmed all sixteen key/value pairs **byte for byte**.

| Mock group | Mock row | Mock value | Verdict on THIS tree | Anchor |
|---|---|---|---|---|
| Moteur | version | `v0.1.1` | ✅ **REAL** | `env!("CARGO_PKG_VERSION")`, already rendered by the shell (`page.rs:92`) and pinned (`screens.rs:547`) |
| Moteur | base de données | `MariaDB 10.11.11` | ✅ **REAL**, one new query. ⚠️ It returns `10.11.16-MariaDB-ubu2204` — see §0a-bis | `SELECT VERSION()`, absent from the tree today |
| Moteur | migrations | `4 appliquées` | ⚠️ **REAL BUT TAUTOLOGICAL** — the tree embeds **5**, and applied **cannot** differ on a reachable instance (§0d-bis) | `sqlx::migrate!("./migrations")` + `_sqlx_migrations` (sqlx **0.9.0**, `Cargo.lock:2130`; table name at `sqlx-core-0.9.0/src/migrate/migrator.rs:40`) |
| Moteur | dernier scan | `il y a 4 min · 1,8 s` | 🔴 **ABSENT ON BOTH HALVES** — the first draft said *"the instant is real"* and **that was refuted on a live binary** (§0b) | `ScanOutcome` (`scan_pass.rs:55-63`) times nothing; `last_observed_at` is not the scan's instant |
| Observation | constats du dernier passage | `359` | ⚠️ **NOT AS WRITTEN** — no pass identifier the HTTP app can see | `connector_id` minted inside the detached scan thread (`main.rs:479`); `count_observations` counts **all time** |
| Observation | rattachés | `312` | ✅ **REAL** — in **sightings** | `repo::count_engine_reach` (`repo.rs:1357`), `COUNT(DISTINCT observation_id)` |
| Observation | non rattachés | `47` | ✅ **REAL** — in **sightings**, ventilated by cause | same query, `GROUP BY outcome, abstention_cause` |
| Observation | rétention | `90 jours · dernier état conservé` | 🔴 **FALSE** — nothing purges an observation or a link, ever | every `DELETE FROM` in `crates/` sits past its file's first `#[cfg(test)]`; `purge_engine_links` has no production caller |
| Sécurité | clé de chiffrement | `hors du volume de données` | 🔴 **ABSENT** — no key, no crypto call site | ⚠️ `argon2` and `chacha20poly1305` are direct dependencies with **zero** usages in `crates/` — a diagnostic reasoning from the manifest would say the opposite |
| Sécurité | identifiants stockés | `2 · chiffrés au repos` | 🔴 **ABSENT** — no credential store, no table | Epic 10 |
| Sécurité | surfaces HTTP | `toutes authentifiées` | 🔴 **FALSE**, but **not for AC2's stated reason** — §0c | `auth.rs:72-73` |
| Sécurité | /metrics | `jeton requis` | ✅ **REAL**, both ways | `auth.rs:44-52`, exercised at `main.rs:2808-2837` |
| Journal | répertoire | `/var/log/opencmdb` | ⚠️ **REAL ONLY IF READ FROM THE RIGHT PLACE** — §0e | `OPENCMDB_LOG_DIR` read at `main.rs:639` |
| Journal | rotation | `quotidienne` | ✅ **REAL** — literally `Rotation::DAILY` | `main.rs:649` |
| Journal | niveau | `info` | ⚠️ **REAL ONLY IF READ FROM THE RIGHT PLACE** — §0e | `OPENCMDB_LOG`, defaulting to `info` (`main.rs:602`) |
| Journal | dernière erreur | `blind ? '401 sur Synology DSM · 12 h 06' : 'aucune depuis 6 j'` | 🔴 **ABSENT** — nothing stores an error | Epic 13 |

🔑 **THE SURPRISE IS WHERE THE TRUTH IS.** The group a reader would expect to be decorative —
*Journal* — is the one the product supports almost entirely. The group a reader would expect to be
authoritative — *Sécurité* — is three-quarters false. **Do not assume; the table above is measured.**

⚠️ **AND ONE WORD CARRIES TWO MEANINGS ACROSS TWO GROUPS.** The mock's *rétention* (Observation
group) means **data** retention and is false here. `OPENCMDB_LOG_RETENTION` (default **14** days,
`main.rs:643`) means **log-file** retention and is true. Rendering both under one word reproduces
exactly what `prd.md:988` forbids, and story 6b.7 qualified « Conflit d'adresse » rather than mint a
term for precisely this reason. **Qualify both, or render one.**

### §0a-bis. THE DATABASE VERSION STRING IS DECIDED HERE, NOT IN THE BROWSER

Measured on a live `mariadb:10.11`: `SELECT VERSION()` returns **`10.11.16-MariaDB-ubu2204`**, not
the mock's tidy `MariaDB 10.11.11`. **Verbatim is the only measured option**; any prettifying is a
transformation of a fact on the screen whose subject is facts. ⚠️ And a test pinning the literal is
brittle across images — assert the SHAPE (non-empty, contains `MariaDB`), not the value.

### §0b. 🔴 ARBITRATION 1 (Guy) — AC1's fifth fact is missing on BOTH halves, and the first draft was wrong about which half

**What the first draft said:** *"the instant is real, the duration is held by NOTHING"*, and it
recommended option (b) — show the instant, say the duration is unmeasured.

🔴 **The gap-hunt layer refuted the first half on a running binary.** `OPENCMDB_SCAN_CIDR=203.0.113.0/30`:

```
before: count=2  max=2023-11-14 22:13:20.123456
INFO opencmdb: startup scan complete ingested=0 failed=0 resolved=false   (at 2026-08-21T10:53:39Z)
after : count=2  max=2023-11-14 22:13:20.123456
```

**A scan ran and succeeded, and `MAX(observed_at)` did not move.** A row labelled *dernier scan* fed
by `repo::last_observed_at` would have read *"il y a 2 ans 9 mois"* thirty seconds after that scan.
And when a scan *does* find something, `observed_at` is `clock.now()` handed into
`spawn_startup_scan` at boot (`main.rs:412`) — one instant for the whole sweep — so the row shows the
**boot instant**, not the scan instant.

🔑 *Option (b) would have shipped a FALSE FACT under an honest-sounding label, one group away from
the group AC2 exists to police.* The draft's own recommendation was the defect.

🔑 **And the product already computes the scan's own instant.** From the same log:
`poll summary … as_of=2026-08-21 10:53:39.624076890 UTC`. `summary.capabilities.as_of`
(`scan_pass.rs:113`) **is** the missing value — bound and traced by story 6b.8, persisted nowhere,
and not mentioned anywhere in the first draft.

**The options, re-stated on the measurements:**

- **(a) Persist a scan record.** 🔴 Story 6b.8 refused the structurally identical thing on
  2026-08-20, and its reason holds: the row-per-scan-versus-current-row question is **D32's and Epic
  13's**. Refusing it there and taking it here would make the earlier refusal arbitrary. **Still
  refused.**
- **(b) Show `last_observed_at` as the last scan.** 🔴 **REFUTED — it is a false fact.**
- **(c′) Hold `Option<ScanReport { as_of, duration, ingested, resolved }>` in an
  `Arc<RwLock<…>>`**, rendered as *"depuis ce démarrage"*, empty after a restart and empty when no
  CIDR is configured. `as_of` already exists; the duration is two `Instant`s around one call. ⚠️
  **And the first draft over-stated this option's cost**: it said the state would land *"inside the
  one region nothing can assert on"*, but `poll_ingest_resolve` **is** driven end-to-end by the
  committed `FixtureConnector`, so the write lives inside the testable seam and only the `Arc`
  plumbing is uncarried.
- **(d) Show the most recent OBSERVATION, labelled as such** — *"observation la plus récente"*, never
  *"dernier scan"* — and say plainly that no pass is measured.

✅ **GUY'S ARBITRATION, 2026-08-21: option (c′)**, and the two refused options are kept with what
they cost. **(d)** — show the most recent observation, labelled as such — was the honest minimum and
would have shipped AC1 **NOT MET on that fact for two reasons rather than one**; it is refused
because the product *does* hold the value, and choosing not to compute it would be a decision to stay
ignorant. **(a)** stays refused on story 6b.8's precedent, unchanged. **(b)** is not an option at
all: it was refuted by measurement, and that is the difference between an option and a mistake.

⚠️ **What (c′) obliges, written now so it is not discovered later**: the report is scoped to the
running process and **must say so on the screen** — a figure that silently resets at every restart
while reading as an all-time fact is the growing-counter family in mirror image (the UX spec's first
hard ban; story 5.14b's arbitration 13). Empty is a legitimate state and gets its own sentence — *no
scan since this boot* — which is also what an unconfigured `OPENCMDB_SCAN_CIDR` produces. 🔑 And the
guard that matters is the one story 6b.8's finding demands: **the report shows what the pass DID,
never what the configuration ASKS FOR** — a mutation that returns a configured-but-never-run state
must red.

### §0c. 🔴 AC2's CONCLUSION SURVIVES; ITS STATED REASON DOES NOT — and my first replacement cited the wrong test

AC2 says *"all HTTP surfaces authenticated"* is false because the allowlist's doc *"calls itself
temporary"* and because story 6.1's brief measured *"a POST to an unknown route answering **400**
without the middleware ever running"*.

**Both halves of that reason are dead on this tree:**

- The allowlist doc no longer calls itself temporary. Story 6.1 shrank it to `/healthz` +
  `/assets/*` and rewrote the doc to *"adding a path back here IS the exposure decision"*
  (`auth.rs:61-74`).
- **Measured on a live binary** (gap-hunt): `POST /nonexistent` with no credential → **401**;
  `GET /nonexistent` → 401; `POST /document-all` → 401; `GET /metrics` without a token → 401.

⚠️ **AND THE FIRST DRAFT'S REPLACEMENT CARRIER WAS WRONG.** It cited
`without_the_switch_an_authenticated_post_reaches_the_empty_fallback` (`main.rs:744`) and quoted a
sentence from its **doc comment**. That test sends an **authenticated** POST and asserts a 404; it
contains no unauthenticated request and no 401 assertion — and the doc line it quotes says the
unauthenticated shape *"proves nothing"*. 🔑 *I criticised an AC for resting a conclusion on a bad
reason, and rested mine on a doc comment.*

**The real carriers, measured:** `main.rs:1552` (`an_unset_pair_refuses_without_the_challenge`
iterates `/anything`, an unregistered path, unauthenticated → **401**) and `main.rs:2813`
(`auth_denies_by_default_and_gates_metrics`, `GET /admin` → **401**). Both are only reachable if
`auth_deny` runs above routing.

**What is still true, and it is a better fact than the mock's:** two path classes are public **by
decision** — `/healthz` and `/assets/*` — and when the Basic pair is unconfigured, every non-public
path answers 401 **without a challenge** (`auth.rs:54-58`). *The product is closed by default*, which
no mock row says and which an operator cannot learn any other way.

### §0d. 🔴 ARBITRATION 2 (Guy) — the security group ships as a SHAPE, not as a word list

AC2's letter says the unsupportable rows *"carry the marker"*. Applied literally, that ships
« Clé de chiffrement · hors du volume de données » under an *Exemple* badge — a false security claim,
correctly labelled, still on the screen, still quotable out of context. **AC2's own 🔴 paragraph is
an argument against its own letter.**

**Options:** (a) render the mock's four rows marked example — refused by AC2's spirit; (b) replace
them with rows that are TRUE and measured; (c) omit the group and say why.

✅ **GUY'S ARBITRATION, 2026-08-21: option (b)**, with the THREE amendments below — each measured by
the gap-hunt layer, none of them in the first draft. **(a)** is refused by AC2's own 🔴 paragraph: a
correctly-labelled false security claim is still a false security claim on the screen, and still
quotable out of context. **(c)** is refused because it costs the operator the one place they can
learn that their instance is closed by default and which two prefixes are not — *omitting a true fact
to avoid a false one is a trade this screen does not have to make.*

1. 🔴 **No free-text arm.** The draft called a forbidden-sentence guard *"AC2's only falsifiable
   carrier"*. Measured: three of the mock's claims planted as a **French literal** in the template
   left **679/679 tests and eight gates GREEN** — `key_names_in_text` sees keys, the locale guard
   sees keys, and *a literal is not a key* (story 6b.3's own sentence). A forbidden-sentence list is
   an **enumeration**, in two languages, against paraphrase, and *an enumeration cannot claim the
   completeness of a property* (story 5.12, third application in this epic). **Give the group no
   free-text arm** — every value a `bool` or a derived list, the row set an enum — so a false claim
   **cannot be typed into it**. The word guard becomes a second line, not the first.
2. 🔴 **Derive the public-surface row by probing `is_public`.** It is a *predicate*, not a list
   (`path == "/healthz" || path.starts_with("/assets/")`), so a "named" row is necessarily a
   duplicated literal. Measured drift: widening it with `|| path == "/ipam"` reds **one** test
   (`every_screen_is_refused_without_a_credential`) and leaves
   `is_public_is_exactly_healthz_and_assets` **GREEN** — that guard is six negative examples and
   `/ipam` is not among them. *A screen that states the security perimeter from a hand-copied
   literal is the false security claim AC2 forbids, created by the fix for AC2.*
3. 🔴 **Hand the handler a `bool`, never the credential.** Measured with both secrets configured and
   the flags as bools: `grep -c` for either secret in the rendered page = **0**. ⚠️ **The draft's
   prescribed leak guard (M11) then is not a mutation but a compile error** — the value is not in
   scope. *The story prescribed a guard for a leak it can make unrepresentable.*

⚠️ (b) means **AC2 ships MET IN SPIRIT AND DIVERGENT IN LETTER**; register the divergence.

⚠️ **And the `/metrics` token must NOT get a second reader.** `scrape_authorized` reads
`OPENCMDB_METRICS_TOKEN` with `std::env::var` at request time (`auth.rs:151`, story 6.1's
arbitration 5) and refuses only on `is_empty()`, while `AppConfig::from_env` filters with
`carries_a_visible_glyph` — so a token of `" "` **protects `/metrics` while the screen reports it
unconfigured**. 🔑 `auth_deny` **already takes `State<AppConfig>`**: put the token there and pass
`config.metrics_token.as_deref()` into `scrape_authorized`, which removes the second reader in one
line rather than creating one.

### §0d-bis. ⚠️ THE MIGRATION ROW CAN ONLY EVER SAY ONE THING, AND THAT IS MEASURED

Both divergence directions, driven against a live instance:

```
version 99, success=0  → boot FAILS: "migration 99 is partially applied; fix and remove row …"
version 6,  success=1  → boot FAILS: "migration 6 was previously applied but is missing …"
```

`run()` migrates **before** binding the listener (`main.rs:382-386`), so **applied == embedded on
every instance that can answer HTTP**. A ratio row is decoration whatever its provenance.

🔴 **This also collapses two mutation rows into one.** The draft predicted M1 (hardcode `5`) RED and
M2 (embedded-for-applied) GREEN — but its own argument for M2 applies verbatim to M1, so **both are
green** and AC1 has no carrier on the migration rows at all. ⚠️ And M2's stated barrier is not one:
`_sqlx_migrations` is a plain InnoDB table with `PRIMARY KEY (version)`, and this suite already
issues raw `DELETE FROM` inside rolled-back transactions (`repo.rs:1913`, `main.rs:1889`). A test can
delete version 5 inside `transact`, read *"4 applied of 5 embedded"*, and roll back.

**Recommended:** the row carries the **schema version** (`0005 — document guards`, measured: 5
embedded / 5 applied, descriptions `initial` … `document guards`) rather than a ratio, and the
applied count is guarded by the rolled-back delete so *"measured at runtime"* has a carrier.

### §0e. 🔴 THE JOURNAL GROUP MUST SHOW WHAT WAS **INSTALLED**, NOT WHAT THE ENVIRONMENT HOLDS

The draft's T3 said: read the log facts through `AppConfig`, never `std::env::var` in a handler.
**Built exactly that way, the group ships two false rows.** Booted with a bad directory, a bad level
and a bad retention:

```
BOOT LOG:  file logging disabled — cannot use "/proc/definitely-not-writable"
SCREEN:    Log directory  /proc/definitely-not-writable
           Log level      notalevel
           Log files kept 14
```

File logging is **off** and the screen names a directory; `EnvFilter::new` is `parse_lossy`, so
`notalevel` was **discarded** and the screen presents it as the level in force.

🔴 **This is story 6b.8's own HIGH finding, one story later**: `OPENCMDB_SCAN_CIDR=nonsense` rendered
as an in-force perimeter. The draft cites that finding **twice** and applies it nowhere. Cause:
`AppConfig` reads the *environment*; `init_tracing`/`build_file_writer` (`main.rs:596-660`) decide
what is *in force*, and they diverge — `build_file_writer` returns `None` on any build error and
gates on `dir.is_empty()` where `AppConfig` uses `carries_a_visible_glyph`.

🔑 **Fix shape: `init_tracing` already returns the writer guard — have it return the DESCRIPTOR it
installed, and hand that to the router.** *The environment is the request; the descriptor is the
answer, and a diagnostic screen must show the answer.* ⚠️ The same class applies to `OPENCMDB_LOG`
alone: a **blank** value is kept by `init_tracing` and passed to `EnvFilter::new("")`, while an
`AppConfig` field filtered the house way would display `info`.

### §0f. WHERE THE CODE GOES — the ceilings, the router, and the state the screen must survive

- **`page.rs` is at 1727 code lines against 2000** (the gate's own rule: the 0-based position of the
  first `#[cfg(test)]`, `xtask/src/main.rs:102-110`). ⚠️ The first draft wrote 1728, 788 and 1178;
  the measured figures are **1727**, **787** and **1177** — each was the 1-based line number of the
  attribute rather than the count. *Numbers written in flight, in the story warning about numbers
  written in flight — fifth story running.*
- ⚠️ **And the file nearest the ceiling is not `page.rs`**: the gate reports *largest: **1908***, which
  is `xtask/src/main.rs`, 92 lines from refusal. §0f's *"two ceilings"* omitted it.
- ⚠️ *"Does not fit"* **overstated**: there are **273** lines of headroom and `/sources`' builder +
  handler pair is ~70. The new-module conclusion stands on `CLAUDE.md`'s *"split, not grown"* and on
  story 6b.6's precedent, **not** on arithmetic. → `crates/opencmdb-bin/src/diagnostic.rs`.
- 🔴 **`/diagnostic` must survive an unreachable database.** Measured on the shape T2 says to copy:
  `/sources`, `/triage` and `/dashboard` each answer a bare **`500 internal error`** — no shell, no
  navigation — when the pool is dead. *The one state in which an operator opens a self-diagnostic
  screen is the one where it renders nothing.* Half this screen (version, embedded schema, security,
  journal) needs no pool at all. Story 5.14b's arbitration 11 — *the reader never fails* — settled
  this principle for an unfamiliar token and nothing carried it to a dead store. **AC7.**
- 🔴 **REGISTER THE ROUTE BEFORE CHANGING THE NATURE.** Re-measured by the gap-hunt on this very
  screen: right order **0 red / 0 red**; nature-first **0 red locally, 1 in CI**; route-first-only
  **19 red**, all `Overlapping method route. Handler for GET /diagnostic already exists`. *The wrong
  order is silent.*
- ⚠️ **SUSPECTED, not measured**: whether the handler can live on `page::triage_router` without
  widening `TriageState { pool, perimeter }` — the screen also needs the log descriptor and two
  security flags. Splitting the **pool-free config half** onto its own sub-router closes AC7 and the
  M3 blind spot together, and is the shape the gap-hunt recommends.

### §0g. 🔴 ARBITRATION 3 (Guy) — rename `/onboarding`, and for the right reason

`Screen::Onboarding`, `href = "/onboarding"`, key `nav.onboarding` (`screens.rs:211, 230, 246`). The
UX spec's **F11 correction** reads: *"bootstrap is a MODE, not an onboarding. Filing it under 'first
run' was a design error"* (`ux-design-specification.md:184-187`, repeated at `:365-375`), and the
spec's own screen list names it **Commissioning** (`:561`).

**Measured blast radius** (the gap-hunt performed the rename): **6 sites in `screens.rs`** — variant,
`ALL`, `href`, `label_key`, `group`, `nature` — plus **1 key rename in `app.yml`**. Nothing in
`xtask/`, `docs/`, `README.md` or the planning artifacts. Result: **676/676 green both ways, eight
gates green, no doc edit.**

⚠️ **But the draft's stated reason was FALSE and is corrected rather than kept.** It argued this was
the last release-free moment to change an operator-visible name. The operator-visible **label already
reads "Commissioning" / « Mise en service »** in both locales (`app.yml:164-166`) — only the URL and
the identifier carry the retired framing. And **neither `onboarding` nor `commissioning` is in the
binding glossary**, so the rename does not swap an absent word for a present one. 🔑 The true reason
is narrower and sufficient: *it aligns the identifier and the address with a correction the spec
took explicitly, at the one moment it costs nothing* — `git tag` shows only `v0.1.0` and `v0.1.1`,
and `v0.2.0` is story 6b.12.

✅ **GUY'S ARBITRATION, 2026-08-21: option (a)** — rename the variant, the route and the key.
**(b)** (variant only) is refused on the measurement above: the half it would leave standing is the
URL, which is precisely the half that stops being free to change at story 6b.12. **(c)** (leave and
register) is the only option that becomes permanently more expensive by waiting.

### §0h. WHAT THIS STORY DOES **NOT** CLOSE, AND THE GUARDS THAT DO NOT COVER IT

- 🔴 **FR36 is Epic 17's and stays there** (`prd.md:949`, `epics.md:487`): *partial at MVP* = source
  health + the *"what changed since last visit"* view (FR18). This story ships a **facts table**, not
  the what-changed lead. On the NFR5 (5.12) and FR7 (6b.8) precedents, *"6b.9 covers FR36"* must
  never be read as true.
- **The baselining is Epic 9's**, an example surface (AC3). ⚠️ `baseline` has **no glossary row** in
  either binding table — confirmed absent from both — and extending a binding table is **Guy's**, not
  a story's (6b.7's precedent, refused there as *premature, not wrong*). **Register it with Epic 9.**
- 🔴 **Story 6b.4b's a11y guards are scoped to `/triage` and cover this screen with NOTHING.**
  `a_planned_control_is_reachable_and_never_natively_disabled` (`page.rs:3773`) reads
  `rendered_triage_body()`. Measured: two controls planted on `/diagnostic` with **no `tabindex`**
  and a bare uppercase native `DISABLED` left **679/679 green and eight gates green**. That is
  6b.4b's shipped defect reachable again. ⚠️ And `action_bar()` hardcodes the mock's **five triage
  gestures with triage owners** while the guard's premise is `roles >= 5`; the diagnostic's two
  controls need a **second builder** and a re-shaped premise. *T6's "extend them" is not a
  one-liner.* **AC9.**
- 🔴 **The per-section marker guard is dashboard-scoped.** `every_example_section_carries_its_own_marker`
  (`page.rs:4066`) reads `rendered_dashboard(...)` and splits on the dashboard's own anchors.
  Measured on a `Mixed` `/diagnostic`: no marker anywhere → 0 red locally / **1** with a database;
  **two markers in section 1 and none in sections 2–4 → 0 red in BOTH conditions**. That is story
  6b.5's exact defect — *"it could not tell each section has exactly one from they happen to add
  up"* — alive on the new screen. The draft said the existing guard *"must cover it"*; it
  structurally cannot.
- ⚠️ **The marker is NOT always emitted by the dispatch.** `_dashboard.html:48` and `:70` each carry
  `{% include "_example_marker.html" %}`, and **four** addresses live outside `demonstration_screen`
  today — `/triage`, `/dashboard`, `/sources` and `/devices/{id}`. `/dashboard` is the precedent
  `/diagnostic` will follow if it ships `Mixed`; the draft named the record route instead.
- ⚠️ **The mechanisms to reuse are PRIVATE and not shaped for two controls.** `Gesture`
  (`page.rs:658`), `GestureView` (`:671`) and `action_bar` (`:688`) carry no visibility modifier —
  module-private to `page.rs` — while §0f puts the screen in `diagnostic.rs`. Reuse needs three
  `pub(crate)` widenings. *The type and the template are reusable; the builder is not.*
- ⚠️ `EXAMPLE_SECTION_ANCHOR = "example-section\""` (`page.rs:1737`) requires `example-section` to be
  the **last** class in the attribute; a section written the other way round unmatches silently.
- ✅ **Corrections to the draft's inherited-trap list, measured**: `keyish()` **accepts uppercase
  since story 6b.7** (`example_screens.rs:795`, `is_ascii_alphanumeric()`) — the 6b.6 hole is closed
  and must not be carried as a live limitation. `every_key_carries_both_locales` genuinely cannot see
  an **absent** key ✅. The stylesheet guard genuinely skips a `class` containing a brace
  (`page.rs:3666`) ✅ and walks subdirectories ✅. The route-table `probed` floor genuinely **is**
  derived from `Screen::ALL` (`main.rs:1066-1075`) ✅.
- 🔴 **A live floor has stopped measuring what it names, and this story is the worst possible one to
  meet it in.** `every_key_carries_both_locales` (`screens.rs:653-691`) ends with `checked >= 47`
  under a message reading *"48 entries minus `_version`"*; `app.yml` holds **221 top-level keys, 220
  non-`_version`**. The floor is **4.7× below what is there**, in the guard this story leans on, in
  the story that will add the epic's largest key batch. Story 6b.7's own sentence — *a floor is only
  a guard while it equals what is there* — live. **Fix it here or register it; do not walk past it.**

### §0i. 🔴 `Nature::Empty` DIES HERE — and the compiler is blind to five of its eight artefacts

`Nature::Empty`'s own doc says *"when story 6b.9 closes there should be no `Empty` left"*
(`screens.rs:120-135`), and `deferred-work.md:3894-3896` names this story as owner.

⚠️ **The draft's *"the compiler names every remaining site"* is FALSE.** Measured by deleting the
variant:

**Named by the compiler — three**, one of which is a REWRITE and not a deletion:
- `screens.rs:313-314` — the arm that **produces** it; both screens need a new nature.
- `screens.rs:480` — the body dispatch.
- `main.rs:943` and `main.rs:956` — two test arms.

**Named by nothing** — grep them:
- `templates/_not_built_yet.html` — referenced by no gate.
- the `pending.badge` / `pending.sentence` keys — an orphan key is outside
  `every_key_carries_both_locales`'s population.
- `Strings::pending_badge` / `pending_sentence`.
- `.not-yet` / `.not-yet-badge` at `app.css:392,404` — the stylesheet guard runs template→sheet only.
- **five rustdoc intra-doc links** (`screens.rs:89, 129, 463`, `page.rs:1226, 1249`) — they break
  under `cargo doc`, which no gate runs; `CLAUDE.md` already records four such links standing.

⚠️ `not_built_yet_body` (`page.rs:1255`) gets a **warning**, and `-D warnings` lives outside
`cargo xtask ci`. ⚠️ And the struct is **`NotBuiltYet`** at `page.rs:1194`, not `NotBuiltYetBody` —
the draft named a type that does not exist. **Ship a deletion CHECKLIST, not a promise.**

### §0j. THE REFUTATION TABLE — recountable rather than believed

| # | What the first draft said | What refuted it |
|---|---|---|
| 1 | *"the instant is real"* for the last-scan row | a scan ran; `MAX(observed_at)` did not move |
| 2 | arbitration 1 → recommend option (b) | (b) ships a false fact (§0b) |
| 3 | option (c) puts state where nothing can assert | `poll_ingest_resolve` is driven by `FixtureConnector` |
| 4 | `main.rs:744` proves the fallback is gated | that test is an authenticated POST asserting 404 |
| 5 | log facts through `AppConfig` are honest | two false rows on a live boot (§0e) |
| 6 | forbidden-sentence guard = *"AC2's only falsifiable carrier"* | French literal planted → 679/679 green |
| 7 | §0d(b) *"which surfaces are public, named"* | a literal drifts; `is_public` widened stayed green |
| 8 | M11 (secret leak) is a mutation to run | with bools it is a compile error |
| 9 | M1 predicted RED | applied == embedded == 5; green by M2's own argument |
| 10 | M2's barrier (*cannot separate without corrupting*) | plain InnoDB table; rolled-back DELETE works |
| 11 | M2's reason (*on a healthy database*) | on **every reachable** one — both divergences refuse to boot |
| 12 | M3 has one number | 0 local / 1 with a database |
| 13 | M4 *"move to `screens::router`"* | the natural reading compiles; name the edit (`E0308` confirmed) |
| 14 | M6 / M10 measure the screen | they mutate shared repo fns with existing carriers |
| 15 | M8 measures something | it restates its own premise (`E0599` by definition) |
| 16 | M12's carrier | a `vec![…]` count IS a literal; needs an enum |
| 17 | M9 is executable | no guard covers this screen's controls yet |
| 18 | *"the per-section guard must cover it"* | dashboard-scoped; 2-in-1-and-0-in-3 → 0 red both ways |
| 19 | *"the marker is emitted by the dispatch, not by templates"* | `_dashboard.html:48,70` include it |
| 20 | *"reuse them, do not rebuild them"* | three items are module-private; `action_bar` hardcodes five |
| 21 | `keyish()` requires lowercase | fixed in 6b.7 |
| 22 | `NotBuiltYetBody` | the struct is `NotBuiltYet` |
| 23 | *"the compiler names every remaining site"* | three named, five invisible |
| 24 | 1728 / 788 / 1178 code lines | **1727 / 787 / 1177**; largest is `xtask` at 1908 |
| 25 | *"does not fit"* | 273 lines of headroom |
| 26 | §0g's reason (last operator-visible moment) | the label already reads *Commissioning* |

✅ **Refuted suspicions kept so nobody re-chases them**: raw reads of `_sqlx_migrations` trip no gate
(eight green with two of them); sqlx exposes no public applied-migration list from a pool (a raw
query is the route); `example_data.rs` will **not** cross the split threshold (1177 vs ~1600); the
rename touches no doc; and AC2's own premise (a 400 above the middleware) is dead on a live binary.

## Tasks / Subtasks

- [x] **T0 — The three arbitrations are TAKEN (Guy, 2026-08-21) (AC: 4, 5, 6)**
  - [x] §0b → **(c′)**, the in-memory `ScanReport` scoped to *"depuis ce démarrage"*.
  - [x] §0d → **(b)**, the security group as a SHAPE, with its three amendments.
  - [x] §0g → **(a)**, rename variant + route + key to `Commissioning` / `/commissioning`.
  - [x] Each recorded in §0 **with the option refused and what refusing it costs**.
- [x] **T1 — Stand the bench up first (AC: 10)**
  - [x] `mariadb:10.11` on a port that is **not 3306** (`kesh-mariadb-dev` holds it — confirmed).
        Apply migrations, export `DATABASE_URL`. Baseline is **676 tests**, **~0.05 s** without a
        database and **~5.5 s** with — the clock is the tell.
- [x] **T2 — The module, the route, and the pool-free half (AC: 1, 7, 10)**
  - [x] `crates/opencmdb-bin/src/diagnostic.rs`. **Register `/diagnostic` FIRST, then change the
        nature** (§0f).
  - [x] **Split the pool-free half onto its own state** so version / embedded schema / security /
        journal render with the store down (AC7), and so M3's guard is reachable without a database.
- [x] **T3 — The four groups (AC: 1, 4, 5, 6)**
  - [x] *Moteur*: version from the **one** source the shell already uses; `SELECT VERSION()`
        **verbatim**, asserted by shape; the **schema version** rather than a ratio (§0d-bis), with
        the applied count guarded by a rolled-back `DELETE FROM _sqlx_migrations`; the last scan per
        T0's arbitration.
  - [x] *Observation*: observations recorded (say **all time**), sightings placed / not placed by
        cause, both in **sightings** (5.14b's arbitration 13). Data retention: **nothing is purged**,
        qualified against the log retention.
  - [x] *Sécurité*: **no free-text arm**; the public row **derived by probing `is_public`**; bools,
        never credentials; `/metrics` through `auth_deny`'s existing `State<AppConfig>`.
  - [x] *Journal*: the descriptor `init_tracing` **installed**, not the environment (§0e). *Dernière
        erreur*: absent — Epic 13, said not shown.
- [x] **T4 — `/commissioning` as an example surface (AC: 3)**
  - [x] New `ExampleContent` variant + body + dataset (mock's four steps and the baselining block),
        **RFC 5737 / RFC 7042 only**. ⚠️ `example_contents.len() == 5` will red with a message reading
        *update this number* — a developer follows those (6b.6's review). The new witness must be
        **distinctive**, or the property that catches a screen serving another's body is defeated.
- [x] **T5 — Kill `Nature::Empty` by CHECKLIST (AC: 8)** — the three compiler sites plus the five
      invisible artefacts in §0i, each ticked individually.
- [x] **T6 — The two planned gestures and their OWN guard (AC: 9)**
  - [x] Widen `Gesture`/`GestureView` to `pub(crate)`; **second builder** for two controls.
  - [x] A **render-level** a11y guard for this screen, premise count **2**, measured red on a
        stripped `tabindex` and on a bare uppercase `DISABLED`.
- [x] **T7 — The guards the draft did not prescribe (AC: 5, 6, 7, 9)** — §0h's list, plus a
      per-section marker guard for this screen if it ships `Mixed`, plus the
      `every_key_carries_both_locales` floor (fix or register).
- [x] **T8 — Mutation pass; run every row (AC: 10)** — the corrected table below. Driver exits
      non-zero when a mutation fails to apply; **touch restored files** (askama compiles templates
      into the binary); never mix a scratchpad restore with `git checkout --`.
- [x] **T9 — Both runs, the browser, the documents (AC: 10)** — both wall-clocks; eight gates,
      clippy, `fmt --check` on the committed tree; **Chrome, French, rebuilt binary**; twins and
      register rows in the same push.

## Dev Notes

### Prescribed mutations — corrected by validation, with the condition each number belongs to

| id | Mutation | Prediction (measured where marked ✅) |
|---|---|---|
| M1 | Hardcode the applied-migration count | **GREEN** until the rolled-back-DELETE guard exists; RED after. Same mutation as M2 up to the constant. |
| M2 | Embedded count where applied belongs | **GREEN**, and for a stronger reason than the draft gave: the two cannot diverge on any *reachable* instance ✅ |
| M3 | `std::env::var("OPENCMDB_LOG")` in the handler | **0 red locally, 1 with a database** ✅ — two numbers, each with its condition |
| M4 | Give the diagnostic handler `State<TriageState>` inside a fn returning `Router` | ✅ `E0308`, *expected `MethodRouter<()>`, found `MethodRouter<TriageState>`*. ⚠️ Name the EDIT: relaxing `screens::router`'s `matches!` instead compiles cleanly and panics at runtime. |
| M5 | Route deleted, nature kept | ✅ **0 local / 1 in CI**; the mirror reds **19**, all `Overlapping method route` |
| M6 | Links instead of sightings | RED — but write it **in the screen's builder**, not in `count_engine_reach`, whose existing carriers would take the red |
| M7 | Plant a false security claim as a literal | ✅ **GREEN today** — this is a tripwire, not a barrier (§0d). Its strength is the SHAPE, and this row measures the second line only. |
| M8 | Reintroduce `Nature::Empty` | ✅ `E0599`. ⚠️ Restates its own premise — kept as a deletion receipt, not as evidence |
| M9 | Strip `tabindex="0"` | **NOT EXECUTABLE TODAY** — no guard covers this screen. It is a guard to WRITE (AC9), then a mutation to run |
| M10 | Freeze the last-scan value | Reconsider entirely — §0b may remove the row this guards |
| M11 | Render a configured secret | ✅ **Not a mutation** — a compile error once the flags are bools |
| M12 | Delete one group | Needs an **enum of groups**; a `vec![…]` count is a literal and the prediction's own condition fails |

### What the previous stories leave you

🔴 **The BLIND review layer found both HIGH findings for three stories running (6b.6, 6b.7, 6b.8),
and each time they were the author's own sentences.** Keep it blind. ⚠️ **And hand the review layers
the register file** — 6b.7's auditor reported *"no register rows"* on a diff that was `crates/` only.

⚠️ **`epics.md:2108` says the epic DoD is *"seven gates"*; the tree has eight.** The planning document
is stale and AC10 is right. Register it.

### The house rules that bite here

- **D47's frontier**: everything is `opencmdb-bin`; `opencmdb-core` gets **no BEHAVIOUR change** —
  never *byte-identical* (5.13b: a promise of non-modification shelters false sentences).
- **D64**: MariaDB only; `SELECT VERSION()` adds no dialect abstraction.
- **Every `pub` item carries a TRUE doc.** A false doc is a defect — this epic has found one per
  story for four stories.
- **Prove-to-red**: a guard is observed failing before it passes, and the mutation is recorded.

### What the operator will be able to DO — asked on purpose

**Look, and read two dead buttons that say why they are dead.** No form, no write — **ten** well-lit
dead ends, a count Epic 6b's retrospective owes a look.

🔑 **But this screen tells the operator things TRUE they could not learn any other way**: which build
they run, which schema it carries, that their instance is closed by default and which two prefixes
are not, where the logs actually went — *and, with (c′), whether the scan they configured has ever
actually run.* Not a gesture; an instrument.

### References

- `epics.md:2262-2278` (the three ACs), `:2086-2108` (goal, Guy's four premises at `:2092`, the six
  constraints, the DoD — ⚠️ which says *seven* gates).
- `prd.md:949` (FR36, partial, Epic 17), `:1444` (NFR29), `:985-1002` (the binding vocabulary — **ten**
  rows here against **eleven** at `ux-design-specification.md:1341-1351`; `attach` is missing from the
  PRD's table. `baseline` is in neither).
- `ux-design-specification.md:184-187, 365-375` (F11), `:561` (*Commissioning*), `:1332` (glossary).
- `deferred-work.md:3894-3896, 3905-3908` — the two register rows this story owns.
- `screens.rs:120-135, 211, 230, 246, 313-314, 480, 547, 653-691`; `page.rs:92, 658-688, 1194, 1255,
  1287-1301, 1546-1614, 1737, 3666, 3773, 4066`; `auth.rs:38-58, 61-74, 151`; `main.rs:135, 382-386,
  412, 439-444, 462, 479, 596-660, 1066-1075, 1552, 2813`; `repo.rs:472, 1357-1378`;
  `scan_pass.rs:55-63, 113`; `xtask/src/main.rs:102-110`.
- `~/travail/Projets actuels/opencmdb/documents/opencmdb - maquette.html` — `diagGroups`, `steps`,
  `base`.

## Dev Agent Record


### Agent Model Used

Claude Opus 5 (1M context), 2026-08-21.

### Debug Log References

- Bench: `mariadb:10.11` on port **13323** (port 3306 holds `kesh-mariadb-dev`, an unrelated
  project's container). `SELECT VERSION()` → `10.11.16-MariaDB-ubu2204`.
- Mutation driver: `scratchpad/mutate.py` — exits non-zero when a mutation fails to apply, restores
  through `git checkout --` against a **committed** baseline only, and `touch`es every restored file
  (askama compiles templates into the binary).
- Binary look: two real boots on a REBUILT binary, French locale — one with no perimeter, one with
  `OPENCMDB_SCAN_CIDR=127.0.0.1/32`, `OPENCMDB_LOG=notalevel`, `OPENCMDB_LOG_DIR` set. ⚠️ **These
  were `curl` plus text extraction, NOT a browser**, and T9 was ticked citing *"Chrome, French"* —
  a ticked task that delivered something different from what it says. The acceptance layer refused
  to settle AC10 for exactly that reason and was right to.
- **The browser look was supplied by the CODE REVIEW, not by the implementation**: its edge-case
  layer drove both screens through a real headless **Chrome 151 over raw CDP**, dispatching genuine
  `Tab` keydown/keyup, and confirmed every control is reached in the tab order as
  `SPAN[role=button][tabindex=0]` with a singly-occurring `aria-describedby` target, no `|safe` in
  either template and no duplicate `id` on either page. Recorded under the layer that did it.

### Completion Notes List

⚠️ **THE LIVE COUNT: 676 → 697 tests** (470 bin + 161 core + 66 xtask, after the code review). **0.15 s** without a
database and **5.13 s** against the live bench — the clock is the tell that the database-backed
tests genuinely executed. Eight gates green, `clippy -D warnings` clean, `cargo fmt --check` clean on
the committed tree. 28 fixtures, trap gate still RED at 26/15/11, `opencmdb-core` **no behaviour
change** (untouched), no migration, no new dependency, `epics.md` and the UX spec not edited.

**What shipped.** `/diagnostic` is the epic's second wholly real screen — **seventeen** rows across four
groups, every value measured at runtime — and `/commissioning` is the last example surface. **`Nature::Empty`
is gone**, so the epic's ten screens now hold either real content or labelled example content, and
none holds nothing.

#### 🔴 THE THREE ARBITRATIONS, AND WHAT BUILDING THEM CHANGED

**(c′) — the scan report — is VINDICATED BY A LIVE BOOT.** With `OPENCMDB_SCAN_CIDR=127.0.0.1/32`
the row reads *"dernier passage : à l'instant · 6 ms · 1 enregistrées (depuis ce démarrage)"* and
the observation count moved 2 → 3. The refuted option (b) would have shown a `MAX(observed_at)`
that a successful scan leaves untouched. The measurement lives in `poll_ingest_resolve`, which a
`FixtureConnector` drives end to end; the `Arc` clone in `spawn_startup_scan` is the uncarried half
and is written as such.

**(b) — the security group as a SHAPE — is what made AC2 falsifiable.** `security_rows` takes a
`SecurityPosture` and nothing else, so no sentence can be typed in; the public-paths row is derived
by probing `auth::is_public` over `Screen::ALL` plus the fixed routes; the credentials are `bool`s,
which turns the prescribed leak guard into a compile property. ⚠️ **`OPENCMDB_METRICS_TOKEN` moved
into `AppConfig` and `scrape_authorized` now takes it as a parameter** — that REMOVED a reader
rather than adding one, and it retired the **last three `std::env::set_var` calls in this crate's
tests**, so story 6.1's *"not one test mutates an environment variable"* is now true of `/metrics`
too.

**(a) — the rename — cost exactly what the validation measured**: `Screen::Commissioning`,
`/commissioning`, `nav.commissioning`; the old address 404s; no document touched.

#### 🔴 FOUR OF THIS STORY'S OWN GUARDS CAME BACK GREEN, AND NONE WAS REACHABLE BY READING

Each is the epic's dominant class — *a guard placed where the defect cannot occur reads as coverage
and is none* — and each is CORRECT about what it tests:

- **M6** swapped the placed/not-placed filters and changed nothing: the oracle read
  `contains("11 sightings") && contains("26 sightings")`, satisfied whichever row carries which.
  Fixed by asserting **row by row** — and then M6 was *still* green, because those assertions run on
  a hand-built `StoreFacts` while the swap lives in the READER. 🔑 **Third occurrence in one story**;
  closed by a database-backed test that seeds one placed and two unplaced sightings and reads them
  back through `read_store`.
- **M7** planted *"Toutes les surfaces HTTP sont authentifiées."* as a French literal in the template
  and left the suite green: the guard read `build_diagnostic`'s output, and *a template's sentence
  cannot appear in its builder's data*. Story 6b.4b's headline, verbatim. Now on the rendered HTML.
- **M8b** slipped a bare uppercase `DISABLED` past three literal needles (`" disabled>"`,
  `" disabled "`, `" disabled="`) because it was followed by a **newline**. Replaced by a token scan.
- **M13** raised the store-read budget to sixty seconds and the timing assertion moved with it,
  because it compared against `STORE_READ_BUDGET` itself. *An oracle that restates the expectation
  cannot fail.* Now a literal bound.

#### 🔴 AC7 WAS DEFEATED BY A TIMEOUT NOBODY HAD CONSIDERED, AND THE TEST THAT FOUND IT WAS MEASURING SOMETHING ELSE

The handler rendered without the store exactly as AC7 asks — **after thirty seconds**, sqlx's
default acquire timeout, which surfaced only because the end-to-end test itself took thirty seconds.
*A page that eventually says the database is unreachable is not a page an operator can use when it
is.* `STORE_READ_BUDGET` is two seconds, and the guard asserts a wall-clock bound.

#### 🔴 A CLAIM I INHERITED FROM THE VALIDATION WAS REFUTED BY BOOTING THE BINARY

Four documents said `EnvFilter`'s lossy parse *"DISCARDS"* an invalid directive. Measured:
`OPENCMDB_LOG=notalevel` becomes **`notalevel=trace`** — a TARGET, not a discard — after which the
product logs nothing of its own. 🔑 **The conclusion survives in a stronger form**: `opencmdb=nope`,
`!!!` and the empty string collapse to **`error`**, so *whichever way the parse goes, the variable
and the filter in force differ*. Pinned by
`the_filter_in_force_is_not_the_variable_that_was_typed`, and the four sentences were corrected.

#### ⚠️ THE MUTATION DRIVER LIED AGAIN, IN A NEW WAY, AND ONLY A CONTRADICTED PREDICTION CAUGHT IT

Three guard repairs never reached the disk: the edit script writes at the END, and one failed anchor
discarded every earlier edit silently. The re-run showed M6/M7/M8b still green **against a
prediction that they would red** — had the prediction been *green*, the lost repairs would have been
filed as confirmations. 🔑 *A batch edit that commits at the end is a batch edit that can lose
everything before its first failure.* Fifth epic running for this family.

#### ⚠️ FOUND BY LOOKING AT THE PAGE, INVISIBLE TO EVERY TEST

*"constats rattachés : **2 constats**"* — the unit was in the label AND in the value. The assertions
read `contains("2 sightings")` and were satisfied. The value is now a bare number and a new
assertion pins the unit **on the label**, where story 5.14b's arbitration 13 requires it.

#### ⚠️ REGISTERED RATHER THAN FIXED

- **`every_key_carries_both_locales` carries a floor of `checked >= 47`** under a message reading
  *"48 entries"* while `app.yml` holds **284 keys**. This story added **63** and did not touch the floor
  — it is not this story's guard, and moving it silently would hide how far it had drifted. → Epic 6b's
  retrospective, with story 6b.7's sentence: *a floor is only a guard while it equals what is there.*
- **`epics.md:2108` still says the epic DoD is *seven* gates**; the tree has eight.
- **The mock's *Vérifier maintenant* and *Exporter le journal* are not live**, owners FR6's scheduler
  and Epic 13. **FR36 stays Epic 17's** — this screen ships facts, not the *what changed since last
  visit* lead.
- ⚠️ **Issue #38 recurred once** during development (`fixtures::tests::a_decision_carrying_an_abstention_cause_is_refused`,
  a missing scratch path), green on the immediate re-run and on every run since. **No cause named** —
  the house rule holds.

### Mutation pass — eighteen mutations, seventeen reds and one compile refusal, replayed in full

⚠️ Every row below was RE-EXECUTED after the guards were repaired; the table is the last run's
output, not the first's. Two numbers where two conditions exist.

| id | mutation | result |
|---|---|---|
| M1 | hardcode the applied-migration count | RED 1 — `the_applied_count_is_read_and_not_assumed` (needs a database) |
| M2 | embedded count where applied belongs | RED 1 — same carrier. ⚠️ Both are the same mutation up to a constant, and both were GREEN until the guard read back **through `read_store`** inside the rolled-back transaction |
| M3 | re-read `OPENCMDB_LOG` in the builder | RED 2 — and **without a database**, which the story's draft predicted it could not do |
| M4 | diagnostic handler onto the pool-free router | RED 30 (`unreachable!` — see M5b) |
| M5 | route deleted, nature kept | RED 2 — `the_route_answers_with_the_store_unreachable` + the marker partition |
| M5b | route kept, nature reverted | RED 30 |
| M6 | swap the placed / not-placed filter | RED 1 — `the_reader_puts_each_reach_count_on_its_own_row`. **Green twice before that guard existed** |
| M7 | false security claim as a template literal | RED 1 — **green until the guard moved to the rendered HTML** |
| M8 | strip `tabindex` | RED 1 |
| M8b | bare uppercase `DISABLED` + newline | RED 1 — **green until the enumeration became a token scan** |
| M9 | widen `is_public` with a known address | RED 1 — `every_screen_is_refused_without_a_credential` |
| M10 | report a configured perimeter as a completed pass | RED 1 |
| M11 | make a store failure a 500 again | RED 1 |
| M12 | delete one group from `DiagGroup::ALL` | **COMPILE refusal** — `E0308` |
| M13 | raise the store-read budget to 60 s | RED 1 — **green until the oracle stopped citing the constant it guards** |
| M14 | reintroduce the *not built yet* line | RED 1 — the stylesheet guard, one of the five artefacts the compiler could not name |


### Code review — three layers, 2026-08-21, on a DIFFERENT model (Sonnet), each isolated

**11 patches, 10 register rows written, 0 arbitrations. 696 → 697 tests.** Eight gates green,
clippy and fmt clean, suite run both ways.

🔴 **THE BLIND LAYER FOUND BOTH HIGH FINDINGS FOR THE FOURTH STORY RUNNING — the diff alone, no
repository, no build — and both were my own sentences.**

**H1 — AC7's guard could not reach the branch it exists for, and the test's own comment explained
why in the language of a justification.** The end-to-end pool carries `acquire_timeout(150 ms)`,
far below the 2 s budget, so `read_store_from` always resolved with an error and
`tokio::time::timeout` never elapsed: *the arm that enforces the budget was dead code under test*.
⚠️ And M13 was red for a reason I had misread — its carrier was the second assertion
(`STORE_READ_BUDGET <= 3 s`), never the timing one. 🔑 **This is the epic's dominant class a FOURTH
time in one story, and the first instance no mutation caught** — eighteen mutations ran past it,
because a mutation can only red a guard that executes the mutated code. Closed by making the budget
a PARAMETER (`store_within`) and driving a pool at a non-routable address, which hangs instead of
refusing; proved to red at **5.002 s** when the budget is neutralised.

**H2 — the screen has SEVENTEEN rows and I wrote sixteen everywhere.** Journal carries five where
the mock carries four. Two layers counted it independently — one from the diff, one from the live
render (17 `div.diag-row`) — and **nothing pinned the total**: the group guard asserts four groups
and no empty group, and is satisfied whatever the rows do. Now `ROWS = 17`, asserted at the builder
AND at the render level.

🔴 **THE ACCEPTANCE LAYER FOUND THAT THE REGISTER WAS NEVER TOUCHED.** `deferred-work.md` had not
changed since story 6b.8 — while §0d says *"register the divergence"*, §0h says *"Register it with
Epic 9"*, and the Completion Notes carry a section headed **REGISTERED RATHER THAN FIXED**. 🔑 *A
section that says "registered" is not a registration, and nothing in the story could tell the
difference.* Fourth occurrence of the class story 6b.7 named against itself. **Ten rows are now
written**, and the defence is recorded with them: derive what a story OWES from its own §0 and its
arbitrations, then diff the register before the commit.

🔴 **THE EDGE LAYER FOUND A HAZARD THAT IS THE PROJECT'S, NOT THIS STORY'S: `app.yml` is invisible
to Cargo's incremental build.** Changing only the translation file and rebuilding finishes in
**0.07 s with no recompile**, and the new string is **absent from the binary**. Two consequences,
both live: CI caches `target/`, so a translation-only PR could be validated against the old strings;
and **any mutation that edits `app.yml` alone measures nothing**. Registered, owner Epic 6b's
retrospective — and story 6b.10 is the first whose subject IS that file.

⚠️ **Two MEDIUMs, both false sentences of mine.** The code cites *"arbitration (c′)"* five times
while the module doc numbers the same decision *(1)* and nothing ties them — *a citation a reader
cannot resolve from the material in front of them*. And the `metrics_token` doc described a
COUNTERFACTUAL filter (`carries_a_visible_glyph`) in the indicative, while the actual filter is
`!is_empty()`. Both corrected in place. The *"eight artefacts"* of `Nature::Empty` are now itemised
rather than counted.

⚠️ **I DESTROYED WORK WITH `git checkout --` DURING THE REPAIR — the fourth occurrence in this
project, committed in the story whose review had just named the class.** The budget repair was
uncommitted when a prove-to-red reverted the file. Redone and committed first. *A file revert equals
a mutation revert only on a COMMITTED baseline*, and the cheap defence is to commit before every
prove-to-red rather than to remember the rule.

✅ **What the layers CONFIRMED by measuring rather than reading**: AC7 reproduced end-to-end on a
REAL database paused mid-session — **200 in 2.002 s**, degraded rows correct, shell intact; seven
mutations re-executed and conformant, carrier and count included; the test delta verified on **both
terms** (676 on `master`, 697 here); all four file sizes; the eight `Nature::Empty` artefacts
genuinely absent; zero `std::env::set_var` left in this crate's tests; `/commissioning` 200 and
`/onboarding` 404; and, in a real browser, every control reachable by `Tab` on both screens.

✅ **Refuted, with the check, so nobody re-chases them**: the *"nothing placed"* sentence reads
correctly when a scan simply found nothing (measured on an empty `/28`); a poisoned report lock
degrades to *no pass since this start-up* by construction; `OPENCMDB_METRICS_TOKEN=" "` fails
CLOSED, not open; a zero-width-space perimeter is filtered; an unwritable, blank or
file-not-a-directory `OPENCMDB_LOG_DIR` all degrade to *standard output only* and never name a
directory; 50 concurrent requests and 30 fired mid-scan all answer 200; and hostile input on both
routes reflects nothing and 500s nothing.

⚠️ **Two measured limits are now register rows rather than sentences**: a widening of `is_public`
with a **brand-new, unprobed prefix** leaves 0/696 red and the screen still claims the old
perimeter; and a **paraphrased** false security claim slips the word guard and renders live. Both
are the stated shape of those guards — a tripwire, never a barrier — now measured instead of argued.

### File List

**New**
- `crates/opencmdb-bin/src/diagnostic.rs`
- `crates/opencmdb-bin/templates/_diagnostic.html`
- `crates/opencmdb-bin/templates/_commissioning_example.html`

**Modified**
- `crates/opencmdb-bin/src/main.rs`, `page.rs`, `screens.rs`, `auth.rs`, `scan_pass.rs`,
  `example_screens.rs`, `example_data.rs`
- `crates/opencmdb-bin/locales/app.yml` (**284** keys), `crates/opencmdb-bin/assets/app.css`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

**Deleted**
- `crates/opencmdb-bin/templates/_not_built_yet.html`
- `Nature::Empty`, `page::not_built_yet_body`, `page::NotBuiltYet`, `Strings::pending_badge`,
  `Strings::pending_sentence`, the `pending.*` keys, `.not-yet` / `.not-yet-badge`

**Sizes** (the gate's own rule — lines before the first `#[cfg(test)]`): `page.rs` **1727** and
unchanged, `diagnostic.rs` **698**, `example_data.rs` **1255**, `example_screens.rs` **867**. Largest
in the tree is `xtask/src/main.rs` at **1908**/2000.

### Change Log

| Date | Change |
|---|---|
| 2026-08-21 | Story created by `create-story`. |
| 2026-08-21 | **CODE-REVIEWED (three layers, Sonnet, each isolated) and REPAIRED.** The blind layer found BOTH HIGHs for the fourth story running, both of them my own sentences; the acceptance layer found the register untouched; the edge layer found an `app.yml`/Cargo hazard that is the project's, not this story's. **11 patches, 10 register rows, 0 arbitrations.** 696 → **697 tests**. |
| 2026-08-21 | **IMPLEMENTED.** 676 → **696 tests**, eight gates green, both runs recorded. Status → `review`; `done` is the MERGE's business. Four of the story's own guards were measured GREEN and repaired; AC7's timeout, the `EnvFilter` claim and the duplicated unit were each found by running or looking rather than by reading. |
| 2026-08-21 | **THE THREE ARBITRATIONS TAKEN by Guy** — §0b **(c′)**, §0d **(b)**, §0g **(a)** — each recorded with the option refused and what refusing it costs. T0 closed: **no arbitration blocks `dev-story`.** |
| 2026-08-21 | **VALIDATED by two fresh-context layers and REWRITTEN on their measurements** — twenty-six claims of the first draft refuted (§0j). Arbitration 1's recommendation **changed from (b) to (c′)** on a measurement that showed (b) shipping a false fact. Three new ACs (7, 8 as a checklist, 9). Status stays `ready-for-dev`. |
