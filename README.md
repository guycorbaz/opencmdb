# opencmdb

**Your network documentation drifts from reality the moment you write it. opencmdb continuously compares the *observed* state of your network against the *declared* state you documented — and turns the gap between them into something you can act on.**

A self-hosted, single-binary reconciliation engine for home labs and small businesses. Lightweight IPAM, a light application CMDB, and network topology, organised around one idea: **the gap is the product.**

> An unknown device just appeared. A documented IP hasn't been seen in 30 days. Two devices are fighting over the same address — and unlike your controller's bare alarm, opencmdb tells you *which* devices. In short: **documentation that corrects itself**, because it is continuously checked against reality.

---

## ⚠️ Project status — early development, honest about it

opencmdb is a product whose entire thesis is that documentation must not lie about reality. This README holds itself to the same rule.

- **Planning is complete.** A full product brief, PRD, UX specification, and a ~5,000-line architecture with a decision register (D1–D66) live in [`_bmad-output/planning-artifacts/`](_bmad-output/planning-artifacts/). If you want to understand *why* every decision is the way it is, that is where the reasoning — including the dead ends and the recorded dissents — is written down.
- **There is a running binary.** `v0.1.1` is tagged and published to Docker Hub as `gcorbaz/opencmdb`. It starts, connects to MariaDB, runs an ARP/ping scan over a CIDR you give it, and serves **one page** showing a real observed-vs-declared gap — reconciled from genuinely ingested observations, abstaining visibly where it cannot conclude. It has been deployed to a real Synology NAS, and everything that broke on the way is in the issue tracker and fixed in `0.1.1`.
- **That is a walking skeleton, not the product.** One page, one connector, no triage inbox, no IPAM, no UniFi connector, no alerts, no admin UI — a scan perimeter is still set by an environment variable. Roughly a fifth of the planned work is done: epics 1–3 of 23 are complete and epic 4 is under way. If you `docker pull` this tonight you will see the idea working end-to-end on a small perimeter, not a tool you can run your network on.
- **Nothing here is production-ready.** No upgrade path is promised between `0.x` tags, and the schema will move.

Everything below describes what opencmdb **is designed to be**, with the current build state called out where it matters.

---

## The core idea

Every network tool shows you *one* of two things:

- **Observed state** — what your controller, your scanner, and your switches actually see right now. Live, but undocumented and unexplained.
- **Declared state** — the spreadsheet, the wiki, the IPAM you maintain by hand. Documented, but stale the moment reality moves.

opencmdb holds both, side by side, and **never merges them**. It *links* an observation to a declaration and continuously asks: do they still agree? Every disagreement — an unknown device, a stale IP, an address conflict, a documented host that vanished — becomes a **gap** in a triage inbox. You resolve gaps one gesture at a time, and the tool learns what you decided.

**It does not moralise.** A wall of gaps on first commissioning is normal, not a reproach. And when opencmdb cannot tell (a device it genuinely cannot identify), it **says so, visibly, and counts it** — rather than guessing. Honest abstention is a first-class state, not a failure.

---

## Why opencmdb

- **Reconciliation, unpaywalled.** The observed-vs-declared loop is the reason opencmdb exists. As of mid-2026 it is a top-of-market, *paid* feature elsewhere. opencmdb's framing: *the reconciliation engine you can actually run* — free, self-hosted, open source.
- **Radically smaller footprint.** **One binary + your MariaDB. No Redis, no workers, no queue, no reverse-proxy tax.** Two moving parts against the incumbents' five. The heavy stack (PostgreSQL + Redis + Celery/RQ workers + a reverse proxy) is the biggest adoption barrier for tools like NetBox; opencmdb removes it.
- **UniFi-first, zero privilege.** The first-class UniFi connector reads clients, switch ports, SSIDs, VLANs and DHCP leases over plain outbound HTTP with **no network privilege** required. Bring-your-own generic ARP/ping scanner for everything else.
- **Built for the operator who runs their own infrastructure**, not for an enterprise with a dedicated IPAM team.

---

## What it will do

Scope is phased. The MVP is the reconciliation core; richer surfaces are deliberately deferred so the core lands correct.

| Capability | Phase |
|---|---|
| Observed-vs-declared reconciliation with a triage inbox (document / accept-gap / attach / create / exclude / snooze) | **MVP** |
| Composite **device identity** (not raw MAC) — survives MAC randomisation, multi-NIC hosts, shared-hardware VMs | **MVP** |
| IPAM: per-subnet occupancy, find a free IP, *which device* owns a conflicting address | **MVP** |
| First-class **UniFi connector** (zero privilege) + generic ARP/ping scanner | **MVP** |
| Honest **source state** — a source that has gone blind is shown as such; a source that simply sees less is a capability to unlock, not a fault | **MVP** |
| Software & applications (with owner and criticality); "Hosted here" on a device record | **MVP** |
| Alerts (unknown device, stale IP, IP conflict *with device identification*) → in-app + webhook, deep-linked | **MVP** |
| Read-only JSON API + authenticated Prometheus `/metrics` | **MVP** |
| Self-diagnostic dashboard; observation history + last-known-state retention | **MVP** |
| Bilingual UI (English / French) | **MVP** |
| Blast-radius / **impact view** (dependency traversal), interactive graphical topology, more connectors (Omada, Mikrotik, pfSense/OPNsense, hypervisors) | **Growth** |
| A stable, versioned public API | **Vision** |

---

## Design principles

These are load-bearing and enforced in code, not just documented:

- **Composite device identity, not a raw MAC.** A MAC identifies an *interface*; a device is a composite. Uniqueness is a *decision the engine makes*, not a database constraint — if it could be expressed in DDL, we misunderstood the problem.
- **Linked, never merged.** Declared and observed data are joined by a link; neither overwrites the other. Documenting a value is a deliberate, field-by-field act.
- **Honest abstention.** When the engine cannot conclude, it abstains — displayed, counted, and grouped by cause. A false merge is catastrophic and has no clean undo; a false split is benign and correctable. The engine is built to never merge on doubt.
- **The engine never lies about a source going blind.** A faulted or offline source can only *remove* knowledge, never invent it; observation-derived alerts are suppressed rather than fabricated.
- **No configuration decides identity in SQL.** Value comparison and normalisation happen in application code, never in the database — so a collation setting can never silently settle a question of identity.

---

## Architecture

A Cargo **workspace** with a deliberate dependency frontier:

```
opencmdb/
├── crates/
│   ├── opencmdb-core/   # the domain: identity, the verdict algebra, the gap predicate.
│   │                    # An error here is domain data, not a string.
│   │                    # It must NOT depend on anyhow, axum, sqlx, or askama.
│   └── opencmdb-bin/    # the composition root: everything that touches the outside
│                        # world — SQL, HTTP, HTML, files, the clock, secrets.
├── xtask/               # the dev-tool runner (cargo xtask ci / css / recapture);
│                        # a workspace member and a dependency of nobody.
└── _bmad-output/        # the complete planning record (brief, PRD, UX, architecture).
```

**Stack:** Rust (edition 2024) · [axum](https://github.com/tokio-rs/axum) · [askama](https://github.com/askama-rs/askama) templates · [HTMX](https://htmx.org/) (committed, never a CDN) · Tailwind (standalone CLI, no Node) · [sqlx](https://github.com/launchbadge/sqlx) · [tokio](https://tokio.rs/). Polling, not SSE, at MVP. Server-rendered; no SPA framework.

**One database, on purpose.** opencmdb supports **MariaDB 10.11+ only** — chosen because Synology ships it natively and includes it in DSM's automatic backups. SQLite and MySQL are not supported; PostgreSQL is not supported at MVP. This is a decision, not a limitation waiting to be lifted: a single engine keeps the whole "budget of unknowns" spent where the value is (reconciliation), not on portability.

---

## Requirements

- **MariaDB 10.11+** reachable from opencmdb (Synology DSM 7 ships a suitable package).
- A host that runs a Docker container, or the ability to run a native binary. **Synology Container Manager is the priority deployment target**; opencmdb is not Synology-exclusive.
- For the generic scanner: nothing special. A container in its own network namespace pings as a non-root user out of the box — Docker sets `net.ipv4.ping_group_range` for it. **Do not use `network_mode: host`**: it inherits the host's value instead, which is empty on Synology DSM, and the scan then fails silently. Raw layer-2 discovery (ARP, MAC facts) is a later release and will want `NET_RAW` plus a macvlan/ipvlan network. The UniFi connector needs neither.

---

## Building from source

The workspace builds today. You need a recent Rust toolchain (1.96+).

```bash
git clone https://github.com/guycorbaz/opencmdb.git
cd opencmdb
cargo build --workspace --locked      # Cargo.lock is committed; always --locked
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

Project-specific gates live in `cargo xtask ci` (in Rust, never in YAML) — the DDL collation gate, the retired-vocabulary check, and the fixture/artifact hash checks. Some `xtask` subcommands are still stubs.

> `cargo run` (with the database configured — `DATABASE_URL`, or the discrete `DATABASE_*` variables) starts an axum server serving a single page at `/`: the observed-vs-declared **gap**, reconciled from the declared records and the ingested observations. Alongside it, a `/healthz` liveness probe and an authenticated Prometheus `/metrics` endpoint (scrape token via `OPENCMDB_METRICS_TOKEN`, `Authorization: Bearer …`). A deny-by-default auth layer refuses every other route — the seam real authentication attaches to. Set `OPENCMDB_SCAN_CIDR` (e.g. `192.0.2.0/24`) to run a one-shot ARP/ping scan on startup so the page shows genuinely observed state; `OPENCMDB_LOCALE=fr` renders the UI in French; `OPENCMDB_LOG_DIR` enables daily-rotating file logs alongside stdout. **This is the walking skeleton, not the product** — see the status section above for what it is not.

## Running it

opencmdb deploys as **two services**: the binary and a MariaDB instance. It is deliberately **not** advertised as a single `docker run` — the encryption key lives in a separate volume by design, and a network-mapping tool that lied about its own deployment would be contradicting its own reason to exist.

A reference [`docker/docker-compose.yml`](docker/docker-compose.yml) and [`docker/.env.example`](docker/.env.example) ship in the repo:

```bash
cp docker/.env.example docker/.env    # set DATABASE_PASSWORD, OPENCMDB_SCAN_CIDR, …
docker compose -f docker/docker-compose.yml up -d
```

Two deployment traps, both found the hard way on a real NAS and both worth reading before you start: **do not use `network_mode: host`** (see Requirements above — the ping scan fails silently), and **a `$` in a password inside `.env` is truncated by Compose interpolation**. Both are documented in the [Administrator manual](docs/manuals/) and in the closed issues.

---

## Security posture (stated as non-guarantees where honest)

opencmdb concentrates a full map of your network plus controller credentials, so security is a design constraint, not an afterthought — and the design is written to be *testable* rather than reassuring:

- **All HTTP surfaces sit behind authentication** — the web UI, the JSON API, and the Prometheus `/metrics` endpoint (via a scrape token).
- **Stored credentials are encrypted at rest** with an envelope scheme; the master key is required to live **outside** the data volume, and opencmdb refuses to start if it finds the key path inside it.
- **What opencmdb does not promise** is written down as a non-promise, not hidden. It does not defend against an attacker who already has the host and the key together; it will say so plainly rather than imply a guarantee it cannot keep. The full threat model is in the architecture document.

---

## The planning record

Unusually for a young repo, opencmdb's design was fully written down *before* the code: see [`_bmad-output/planning-artifacts/`](_bmad-output/planning-artifacts/). The architecture document carries a decision register (D1–D66) where each decision records not just the choice but the argument it survived — and the ones that were rejected. Start at its Decision Index. `architecture-views.md` is a cross-cutting digest (renunciations, measurements, dissents) derived from it.

*(These documents describe the design; for the current state of any requirement, read the `editHistory` frontmatter of `prd.md`.)*

Early-stage **User** and **Administrator** manuals (LaTeX, built with LuaLaTeX) are scaffolded in [`docs/manuals/`](docs/manuals/); sections describing not-yet-built features are marked *Planned*, in keeping with the same honesty this README holds itself to.

---

## Contributing

Early days — the reconciliation core is being built one vertical slice at a time. If the idea resonates, opening an issue to discuss a connector, a design decision, or a use case is the most useful thing right now. Please keep application code out of `_bmad*` and honour the `opencmdb-core` dependency frontier (no `anyhow`/`axum`/`sqlx`/`askama` in the domain crate); `cargo xtask ci` is meant to catch violations.

## License

Copyright © 2026 Guy Corbaz.

opencmdb is free software, licensed under the **GNU Affero General Public License v3.0 or later**
(AGPL-3.0-or-later). The full text is in [`LICENSE`](LICENSE); the same identifier is declared in
every crate's `Cargo.toml`.

**Why the AGPL and not the MIT or the GPL.** opencmdb is a web application that concentrates a map
of your network. The AGPL is the GPL plus one clause that matters here — **§13, Remote Network
Interaction**: if you modify opencmdb and let other people use it *over a network*, you must offer
them the source of your modified version. A plain GPL would let someone take this code, improve it,
run it as a hosted service, and never give anything back; the AGPL closes that gap. It is the same
reasoning that keeps the reconciliation loop unpaywalled: the feature exists to be run by the people
who need it, not resold as a closed product.

**What the AGPL does not do**, stated plainly rather than left to be assumed:

- It does **not** stop anyone using or modifying opencmdb privately. Running a modified copy inside
  your own organisation, forever, without publishing anything, is explicitly allowed — that is true
  of every free-software licence and is not a loophole.
- It does **not** forbid commercial use, or selling it.
- It does **not** protect the *idea*. Nothing stops someone reimplementing the same design in their
  own closed code.

What it does guarantee is that **this code, and anything derived from it, stays open** — including
when it is served over a network.

_Note for anyone opening a pull request:_ contributions are accepted under the same licence. There
is currently **no CLA**, so copyright stays with each contributor and the project could not be
relicensed later without every contributor's agreement.

---

*opencmdb is built in the open by a solo developer with AI assistance. The name is lowercase, always.*
