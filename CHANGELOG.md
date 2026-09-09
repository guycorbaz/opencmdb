# Changelog

All notable changes to opencmdb. Versions follow the `v*.*.*` tags; the image for each is
published to Docker Hub as `gcorbaz/opencmdb`.

⚠️ **Nothing here is production-ready.** No upgrade path is promised between `0.x` tags, and the
schema will move.

---

## Unreleased — the queue says a name, a rename is a gap, and documenting no longer hides a machine

🔑 **`/devices` shows what YOU documented.** Until now it showed eight invented machines and none of
yours, so the one live control in this product — *Add*, on a triage row — wrote a record that landed
nowhere you could see. The gesture worked and its result was invisible, which is a worse dead end
than a screen that says it is empty. Your records come first now, with the name, the address, how
many fields they carry, whether they were adopted from a sighting or entered by hand, when they were
written, and **when the network last showed the address** — a documented machine nobody has seen for
a month is this product's own subject.

The example list keeps its place below them, under its marker: it goes on showing the shape Epic 6's
grouping will fill. ⚠️ With one or two records of your own the screen still shows mostly
demonstration, and putting yours above it mitigates that rather than removing it.

⚠️ **`/devices` now needs the database.** It read nothing before, so it always rendered; it answers
an honest 500 within five seconds when the store cannot be reached, like `/triage` and `/dashboard`.

🔑 **And `/sources` stopped hanging.** It has read the store since `v0.2.0` without a time budget, so
an unreachable database left it holding the browser for **thirty seconds** before the calm sentence
arrived. Nobody had noticed; the guard written for the inventory found it on its first run, because
it walks the route table instead of a list.

### The reverse-DNS half

⚠️ **A name the resolver refuses is not a name.** An answer that is itself an IP address, or that
carries a directional or invisible control character, is discarded rather than repeated: the first
told the operator what they already knew and the second reorders the line it is displayed on. A
hostname is the first data in this product that neither the operator nor the product controls — it
comes from whoever answers the query.

**The scan asks who answers.** Every host that replies to a ping is now looked up in reverse DNS,
and the name it answers to is recorded as an observed fact and shown in the triage queue. On the
reference deployment that turns a list of 69 bare addresses into `sw03`, `wifi01-grange`,
`imprimante-bureau` — and it is the cheapest fact this product can learn about a host it can only
ping.

**`OPENCMDB_DNS_SERVER` — set it to your DHCP server.** Unset, the product asks the system
resolver, which is the unsurprising default and, on a real home network, nearly useless: measured
over those 69 addresses, the configured system resolver answered for **2** and the router that
handed out the leases answered for **37**. A small network resolves its own hosts at the box that
leases the addresses, and that box is usually not the resolver the hosts are pointed at. A value
that is not a bare IP address refuses to start, by name.

🔑 **A documented record now has two fields, so it can drift** — and this is the release where the
product's founding claim is exercised end to end for the first time. Until now the only declarable
field was `ipv4`, which is also the key by which an entity is recognised, so a documented machine
could never disagree with the network. Document a host today and rename it tomorrow, and the queue
shows one **Gap** — `wifi01-grange.home.arpa → wifi01-grange-2.home.arpa` — which the gesture can
close.

### 🔴 Two corrections the hostname made visible, and neither was new

**A source that changes its mind is a history, not a disagreement.** `reconcile` compared a
declared value against *every observation ever recorded*, and refused to conclude when two of them
disagreed. That was written when a field could only come from two SOURCES; `hostname` is the first
field on which one source can disagree with itself, an hour apart. Measured on a real network
before the fix: a renamed host produced a *Conflict* row **and** an *Absence* row, neither closable
by any gesture, with the *Absence* pane reading *"no source reported a value"* about a host seen one
minute earlier that was reporting two. The comparison now reads the most recent sighting of each
source. A conflict is again what FR16 means: two sources that cannot both be right.

**A restart no longer creates a second source.** `connector_id` was minted fresh at every boot, so
the reference store carries **eight** ids for one connector, seven of them dead. It is now derived
from what the source is and what it watches — **from the NETWORK, not from the string you typed**:
`192.0.2.0/24`, `192.0.2.1/24` and `192.0.2.0/024` name the same 254 hosts and are one source, so
correcting a typo in `OPENCMDB_SCAN_CIDR` no longer leaves two sources arguing over one network. ⚠️ This also stabilises the source label on screen, which was
the top 32 bits of a v7 UUID and therefore a clock reading that rolled every 65 seconds.

⚠️ **Upgrading does not repair existing rows**: sightings recorded before this release keep the
per-boot source ids they were written with, and **nothing ages them out** — the store is
append-only and never pruned, so each of those ids keeps one live vote for ever. What makes them
harmless is not time but content: they carry no hostname, so they can only agree with the present
on the perimeter key. *That is a property of today's data, not a mechanism.* The product does not
edit observations (NFR5); the mechanism is a source registry, Epic 11's.

---

## 0.3.1 — the button says what it does

One label, and the reason it changed is worth more than the change.

**The gesture that closes a gap is now "Add" / « Ajouter ».** It read "Document" / « Merger », a
pair the PRD fixed deliberately in July. Guy installed `v0.3.0` on his own network, met the button,
and said the word did not tell him what it does. Two facts settled it:

- « Merger » is an anglicism; the French is « Fusionner ».
- **The action cannot fuse anything.** There is no `ON DUPLICATE KEY UPDATE` in the adapter, the
  primary key is `(entity_id, attr_key)`, and a second write for one field errors 1062. The engine
  can only ADD a declared value, never replace one — NFR5, which is what protects an operator's
  testimony from a machine.

So « fusionner » names a forbidden operation in French for exactly the reason `merge` does in
English, and retiring one while keeping the other was never coherent.

⚠️ **The identifier is unchanged.** `document-all`, `document-field` and FR13 still say `document`;
renaming a route buys nothing and breaks a deployment. This is the one concept whose UI label
differs from its identifier, and all three binding tables now say so in the row rather than leaving
the two soldered together.

**And the dead controls say what they would do.** The group sentence read *"The gestures marked Not
yet are still to come"* — answering the badge with the badge. It now names them: resolve an
ambiguity, accept a gap as permanent, snooze a question, attach a sighting to a device.

### 🔑 What this release records, beyond the words

Decision **D11** — Guy's own, 16 July, naming the gesture "Merger"/"Merge" over the round table's
`document`/`adopt` — carries a dated supersession rather than a rewrite. It also carries the dissent
it overruled, and one line of it came true:

> *"Vocabulary is architecture — if the action is called merge in the UI, someone will end up
> implementing it as a merge."*

It did **not** come true in the code: the mitigation held, and a destructive merge is structurally
impossible. It came true in the **reader**. That is the *pedagogy risk* D11 names and sets aside —
and it took the author using his own product to see it, five weeks later, with no review layer, gate
or retrospective having raised it.

### ⚠️ Registered, not fixed

A human asking the product to change a value **they themselves declared** is legitimate and has no
gesture — NFR5 binds the machine, not the author. And documenting a value already declared on
another entity is unguarded: the unique index is keyed on `(origin_obs_id, attr_key)`, so two
entities may claim one address.

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
