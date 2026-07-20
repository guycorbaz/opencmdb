# Story 3.5: A minimal ARP/ping connector, ingested

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want a real minimal ARP/ping source that implements the `Connector` trait and whose observations are ingested,
so that observed state comes from a genuine source, not a stub.

## Acceptance Criteria

1. **Given** a declared subnet (FR3), **when** the connector polls, **then** it pings the hosts (FR4, unprivileged ICMP; `CAP_NET_RAW` in the container as the fallback path) and emits an `Observation` (IpV4 + Rtt facts, dated by the poll's `now`) through the `ObservationSink` for each host that answers — and it honours the `Connector` contract's universal invariants (cancellation respected, no total loss, no panic).
2. **Given** a poll's observations, **when** ingestion runs, **then** they are persisted as `observation_record` rows (immutable, linked-never-merged, dated by the source, facts serialized) via the adapter.
3. **And** the connector lives in `opencmdb-bin` (it touches the network); no private network data is committed (tests use loopback / documentation ranges).

## Tasks / Subtasks

- [x] Task 1 — Dependencies (AC: #1, #2)
  - [x] `bin`: `surge-ping = "0.9"` (async ICMP, unprivileged DGRAM socket — no `NET_RAW` needed where `net.ipv4.ping_group_range` allows) and `serde_json` (serialize `Vec<Fact>` for ingestion). Both `bin`-only; frontier unaffected.
- [x] Task 2 — The `ArpPingConnector` (AC: #1)
  - [x] `ArpPingConnector { id, scope, targets: Vec<Ipv4Addr>, timeout }` implementing `Connector`. `poll(now, sink, cancel)`: for each target, check `cancel.is_cancelled()` (the cancellation point), send an ICMP echo (unprivileged), and on a reply emit an `Observation { observed_at: now, scope, facts: [IpV4{addr}, Rtt{millis}] }`. Return a `PollSummary { capabilities: { as_of: now, kinds: {IpV4, Rtt} }, scopes_covered: [scope] }`.
  - [x] Parse a declared subnet (CIDR) into `targets` — for the walking skeleton a small helper (e.g. expand a `/30`–`/24`); reject huge ranges.
  - [x] Ping-only: emits IpV4 + Rtt, not Mac (ARP needs raw sockets; that is the NET_RAW upgrade, later).
- [x] Task 3 — Ingestion (AC: #2)
  - [x] `insert_observation<'e, E: Executor>(executor, &Observation)` in the repo adapter: `INSERT INTO observation_record (id, connector_id, observed_at, l2_domain, vantage, facts, raw)` binding Strings (D48) — `observed_at` formatted as a MariaDB datetime string; `facts` = `serde_json::to_string(&obs.facts)`. Written once, generic over `Executor`, like the read bodies.
- [x] Task 4 — Tests (AC: #1, #2, #3)
  - [x] Ingestion round-trip (`DATABASE_URL`-gated, serialized via `DB_TEST_LOCK`): `transact` an insert of a synthetic observation, then count `observation_record` = 1. Deterministic, no network.
  - [x] Contract invariants: a cancellation test — `poll` with a pre-cancelled token emits nothing and returns cleanly (universal invariant; exact emission is not asserted for a live scanner).
  - [x] Network test (gated on `OPENCMDB_NET_TESTS=1`, so it runs locally but is skipped in CI where ICMP is not guaranteed): scan `127.0.0.1/32` → assert one observation carrying an `IpV4(127.0.0.1)` fact is emitted.
- [x] Task 5 — Verify (AC: #1–#3)
  - [x] `cargo test -p opencmdb-bin` green (ingestion vs local MariaDB; network test gated). `cargo xtask ci` green (frontier: surge-ping/serde_json are bin-only). clippy `-D warnings` + fmt clean.

## Dev Notes

### Ping mechanism (approved 2026-07-20): unprivileged ICMP, NET_RAW as fallback

`surge-ping` over an **unprivileged ICMP datagram socket** (Linux `ping` sockets) — no `CAP_NET_RAW` where `net.ipv4.ping_group_range` permits (it does in dev and typically on a NAS). The container (Story 3.9) grants `CAP_NET_RAW` as the robust fallback; with it, a later story can add ARP (Mac facts). Distroless has no `ping` binary, so shelling out is not an option — a Rust ICMP client is required.

### Testing a non-deterministic scanner

A live scanner emits whatever answers, so the `run_connector_contract` exact-emission check (built for scripted/fixture connectors) does not apply. This connector is verified on the UNIVERSAL invariants — it respects cancellation, emits only valid observations, and never panics. The one live network assertion (scan loopback → an `IpV4(127.0.0.1)` observation) is gated on `OPENCMDB_NET_TESTS=1` so it runs in dev but is skipped in CI (GitHub runners do not reliably permit ICMP). Ingestion, which is deterministic, is tested against MariaDB.

### Ingestion — immutable, linked-never-merged (FR11)

Observations are inserted into `observation_record` and never updated. Facts serialize to JSON in the `facts` column (the engine deserializes and compares in Rust; SQL never compares — D10). `observed_at` is bound as a MariaDB datetime string (D48: bind Strings — the walking-skeleton sqlx subset; no sqlx `chrono` feature). Reuse Story 3.3's adapter and `classify`.

### DB test discipline (learned in 3.3)

DB-touching tests race on `migrate!` in CI (parallel) — hold `crate::DB_TEST_LOCK` for their duration, and run BOTH DB tests unfiltered locally (`cargo test -p opencmdb-bin`), not just the new one.

### Dependency & crate posture

- `bin`: `surge-ping` + `serde_json` (both bin-only — the connector and ingestion live in the composition root). `chrono` (from 3.4) formats `observed_at`. Frontier gate unaffected (core stays clean).
- The connector uses Story 2.1's `Observation`/`Fact`/`Scope`/`Capabilities`, Story 2.3's `Connector`/`ObservationSink`/`PollSummary`, and Story 3.3's adapter.

### Scope — the connector + ingestion, not the scheduler

This story delivers a working connector and ingestion, tested. The periodic scheduler (FR6 coalescing) and the reconciliation that turns ingested observations into a gap (3.6) are separate. Wiring a real scan into the running app can be minimal (a configured startup scan) or land with the page (3.7) — keep 3.5 to the connector + ingestion + tests.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.5: A minimal ARP/ping connector, ingested]
- [Source: _bmad-output/planning-artifacts/architecture.md#D19 — Fact/Observation/Scope; ping-only vs NET_RAW]
- [Source: crates/opencmdb-core/src/connector/mod.rs · observation/mod.rs — the contract and vocabulary this implements/emits]
- [Source: crates/opencmdb-bin/src/repo.rs — the adapter, `classify`, and the query-body pattern to extend with `insert_observation`]
- [Source: CLAUDE.md — Discovery: generic ARP/ping scanner, NET_RAW → ping-only fallback]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `OPENCMDB_NET_TESTS=1 cargo test -p opencmdb-bin arp_ping::` → `scans_loopback_when_enabled` ok — surge-ping's default **unprivileged** ICMP (SOCK_DGRAM) pinged 127.0.0.1 and emitted an `IpV4(127.0.0.1)` observation; `cancelled_poll_emits_nothing` ok.
- Ingestion + DB tests vs local `mariadb:10.11.11`, 2× on a fresh DB: 6 bin tests pass. CI scenario (DATABASE_URL set, OPENCMDB_NET_TESTS unset): the network test skips, DB tests run — 6 pass.
- Full workspace green (6 bin + 24 core + 23 xtask); frontier/clippy/fmt clean.

### Completion Notes List

- `arp_ping::ArpPingConnector` (bin): implements `Connector`; `poll` opens surge-ping's default SOCK_DGRAM (unprivileged) ICMP client, pings each target (cancellation point between hosts), and emits an `Observation` (`IpV4` + `Rtt` facts, dated by `now`, obs_id a UUIDv7) per reply; returns a `PollSummary` with `{IpV4, Rtt}` capabilities.
- **AC #1** — real ICMP, unprivileged; respects cancellation (test), emits valid observations (loopback test, gated). **AC #2** — `insert_observation`/`count_observations` in the adapter: `observation_record` insert binding Strings (D48), `facts` = `serde_json`, `observed_at` a datetime string; ingest round-trip test. **AC #3** — connector in bin; tests use loopback / synthetic data only.
- Ping mechanism: surge-ping defaults to DGRAM (unprivileged) — `.sock_type_hint` not needed; no `socket2` dep. `CAP_NET_RAW` remains the container fallback (3.9).
- New bin deps: `surge-ping`, `serde_json`, `tokio-util` (CancellationToken in the poll signature), `uuid` (v7 obs_ids). All bin-only; frontier unaffected.
- DB tests hold `DB_TEST_LOCK` (the 3.3 lesson); network test gated on `OPENCMDB_NET_TESTS=1` (skipped in CI). The connector is `#![allow(dead_code)]` (wired into a scan loop in a later step; contract + ingestion prove it).

### File List

- `crates/opencmdb-bin/src/arp_ping.rs` (new) — the ARP/ping connector + cancellation & gated-loopback tests.
- `crates/opencmdb-bin/src/repo.rs` (modified) — `insert_observation`, `count_observations`, ingest round-trip test.
- `crates/opencmdb-bin/src/main.rs` (modified) — `mod arp_ping;`.
- `crates/opencmdb-bin/Cargo.toml` (modified) — `surge-ping`, `serde_json`, `tokio-util`, `uuid`.
- `Cargo.lock` (modified).

## Change Log

- 2026-07-20 — Implemented Story 3.5 (minimal ARP/ping connector + ingestion). `ArpPingConnector` pings a declared set over unprivileged ICMP (surge-ping) and emits observations; ingestion persists them to `observation_record` (facts as JSON, bind Strings). Verified: real unprivileged ping of loopback, cancellation invariant, ingest round-trip vs MariaDB; network test gated out of CI. Frontier/clippy/fmt green. Status → review.
