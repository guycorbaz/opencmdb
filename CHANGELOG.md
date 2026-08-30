# Changelog

All notable changes to opencmdb. Versions follow the `v*.*.*` tags; the image for each is
published to Docker Hub as `gcorbaz/opencmdb`.

⚠️ **Nothing here is production-ready.** No upgrade path is promised between `0.x` tags, and the
schema will move.

---

## 0.3.0 — the product does something

`v0.2.0` served ten screens and could not act on any of them. `v0.3.0` is the first release an
operator can install, point at a network, and use.

### 🔑 What changed, in one sentence each

**The documenting gesture is reachable.** It shipped on 2026-08-26 and **was written in no
document an operator reads** — not the README, not `.env.example`, not the compose file, not the
Docker Hub page. The one place that named it, the administrator manual, said *"setting it changes
nothing an operator can see"*, which had been false for four days. An outside review installed the
product, followed the manual, and could not make it do a single thing. `.env.example` now sets
`OPENCMDB_DOCUMENT_ENABLED=1` and explains what it turns on.

**The scan repeats.** Until now it ran ONCE, at startup: the only way to see a change on your
network was to restart the container, and the README's word *"continuously"* was false.
`OPENCMDB_SCAN_INTERVAL_SECS` now sets the period, **five minutes by default**. Set it to `0` for
the old behaviour. A value that is not a whole number of seconds refuses to start, by name.

**The identity table stops growing on its own.** The engine no longer writes a link for a sighting
it can never place. A sighting with no hardware address has no key, so it could not be placed by
this pass or by any future one — the row said something about the *connector*, and repeated it at
every sweep. Measured on a real `/24`: **fifty observations, fifty current links**, and once the
scan became periodic that would have been millions of rows a year. ⚠️ **This mattered more the
moment the scan became periodic**, which is why the two ship together.

### ⚠️ What this release does NOT change

**The connector still reads no hardware address.** It sees an IPv4 address and a round-trip time,
so the identity engine can group nothing, and the reach section on the triage screen stays empty.
`/sources` says which fact kinds this instance cannot observe. Documenting a machine still adds it
to the declared side without giving you an inventory screen fed by it.

### Upgrading from `0.2.0`

Nothing breaks. Add `OPENCMDB_DOCUMENT_ENABLED=1` to your `.env` — without it the product behaves
as `0.2.0` did, which is to say it shows you things and does nothing. Optionally set
`OPENCMDB_SCAN_INTERVAL_SECS`; unset means five minutes.

---

## 0.2.0 — the interface

`v0.1.1` served one page. `v0.2.0` serves ten screens, in the reference design, with a triage
inbox on the real observed-vs-declared gap.

### 🔴 Breaking — read this before you upgrade

**1. The product is no longer publicly readable, and this is the first thing you will meet.**
Every screen answers `401` unless you supply operator credentials. `/healthz` and the static
assets stay public; nothing else does.

```
OPENCMDB_BASIC_USER=your-name
OPENCMDB_BASIC_PASSWORD=your-password
```

Both or neither — half a pair refuses to start, by name. The user half may not contain a colon
(RFC 7617), and neither half may carry a non-ASCII or control character, since no browser dialog
can type one. ⚠️ **With neither set, every screen answers `401` and nobody can sign in.** That is
the deliberate posture of a fresh instance, not a fault — but on an upgrade from `v0.1.1`, which
was readable by anyone who could reach the port, it looks exactly like a broken deployment. It is
listed first here for that reason.

**2. `/` is now a redirect.** It answers `303` to `/triage`. A bookmark on `/` still lands
somewhere useful; a script that expected HTML at `/` will follow the redirect or need updating.

**3. The interface is light, not dark.** Every existing deployment changes colour. The palette is
the reference design's; a dark set exists in the stylesheet and is selected by nothing.

**4. `OPENCMDB_LOCALE` refuses a value it does not recognise, naming the variable**, rather than
falling back to English in silence. `en` and `fr`, with a region suffix such as `fr-CH` accepted.
⚠️ **`OPENCMDB_LOCALE=FR` now stops the boot** where it used to be ignored.

### Added

- **Ten screens** behind a navigation: triage, dashboard, devices, the device record,
  applications, IPAM, sources, alerts, self-diagnostic, commissioning.
- **The triage inbox on the real gap** (`/triage`): a queue of gap rows, and two photos side by
  side — the declared and the observed — each carrying its own provenance and its own freshness.
  An age sort is available and off by default.
- **A keyboard layer**: `↑`/`↓` move the selection in the queue, the focus follows immediately
  and the URL catches up when you stop. The focus ring is the product's own.
- **A self-diagnostic** (`/diagnostic`): which build you run, whether your schema is current,
  whether your instance is closed, where your logs went, and whether the scan you configured has
  ever run.
- **A sources screen** that names the five fact kinds the shipped scanner **cannot** see — which
  is why the identity engine abstains on almost everything it observes.
- **The interface in French and English**, every string a key.
- **Two accessibility gates in CI**: axe-core over every screen the navigation offers plus two
  query-string states, and a keyboard gate of twenty checks. Both fail the build.

### What this release does NOT do

⚠️ Stated so that nothing here reads as a promise.

- **No gesture acts.** The triage screen shows five controls — document, accept-gap, attach,
  exclude, snooze — and every one is labelled *not yet*. Epic 7 builds them. There is no form,
  no button and no write an operator can reach on any screen.
- **Six of the ten screens carry example content**, fabricated and labelled as such on the page:
  devices, the device record, applications, IPAM, alerts, commissioning. The dashboard is mixed —
  its reach figures are real, the cards beside them are not.
- **One connector, and it sees very little.** The ARP/ping scanner observes an IPv4 address and a
  round-trip time — **no hardware address at all** — so the identity engine abstains on
  everything it scans. `/sources` says so on screen. There is no UniFi source.
- **No credential storage.** Nothing is stored, nothing is encrypted; `/diagnostic` reports
  *none stored*. ⚠️ The README, the Docker Hub page and the administrator manual asserted
  encryption at rest as current fact until this release. They no longer do.
- **The scan perimeter is still an environment variable**, and the scan is one-shot at start-up.

### Fixed

- A shared test scratch directory that could race between two helpers — a reproduced candidate
  cause for the long-open local test non-determinism (issue #38). ⚠️ **The issue stays open**:
  one reproduced occurrence establishes *a* cause, not *the* cause.
- Documents that described a product that no longer existed: the status narrative, the build
  chain, the theme, the single page, and six manual chapters presenting unbuilt features as
  current.

---

## 0.1.1 — deployment fixes

Overlapping ping probes, the `DATABASE_*` variables, fatal errors logged, `cap_net_raw` on the
binary.

## 0.1.0 — first published pre-release

The walking skeleton: one page showing a real observed-vs-declared gap.
