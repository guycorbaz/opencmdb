# Story 2.1: Domain observation types

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want the core observation types (`Observation`, `Scope`, `ConnectorId`, a dated `Capabilities` descriptor),
so that every connector emits the same shape and an observation can never express "gone".

## Acceptance Criteria

1. **Given** the `opencmdb-core` crate, **when** the observation types are defined, **then** an `Observation` records what a source saw, dated by the source (`observed_at: Timestamp`), and has NO variant or field meaning "absent"/"gone"/"disappeared" — absence is DERIVED by the engine, never emitted (NFR7 / D35).
2. **Given** a `Scope` on an `Observation`, **then** it carries `Scope { l2_domain: L2DomainId, vantage: VantageId }` (D19) — the MAC's uniqueness space and WHO saw it. (This is D19's OBSERVATION scope, distinct from D34 §3's liveness-blindness scope built later — do not conflate.)
3. **Given** a `Capabilities` descriptor, **then** it is a DATED FACT (`as_of: Timestamp` + which `Fact` kinds the source can emit), able to travel with a batch — not a constant (D34 §1).
4. **And** the types live in `opencmdb-core`, are unit-tested, and `cargo xtask ci` stays green (frontier gate: no `anyhow`/`axum`/`sqlx`/`askama` in core).

## Tasks / Subtasks

- [x] Task 1 — Define the id newtypes and `Timestamp` (AC: #1, #2)
  - [x] `Timestamp` = `chrono::DateTime<chrono::Utc>` (alias). Core's chrono has `clock` OFF (manifest: `default-features = false`), so `Utc::now()` is not even callable here — the engine structurally never touches the clock (D19). Time enters as data.
  - [x] Opaque id newtypes: `ObsId`, `ConnectorId`, `L2DomainId`, `VantageId`. Back them with `uuid::Uuid` (v7, already a dep) via a small newtype each; `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]`.
- [x] Task 2 — Define `MacAddr` as an exact-bytes newtype (AC: #1)
  - [x] `MacAddr([u8; 6])` — exact bytes, NOT a String (identity comparison is byte-exact; a String would invite locale/case ambiguity, the very thing D64's binary collation forbids one layer down). `Display` as lowercase colon-hex; a `FromStr`/parse accepting `aa:bb:cc:dd:ee:ff`. Derive `PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize`.
  - [x] `locally_administered` is a property of the U/L bit — expose it as a method `fn is_locally_administered(&self) -> bool` AND keep it as the `Mac` fact field the connector reports (D19), so the two can be cross-checked later.
- [x] Task 3 — Define the `Fact` enum (AC: #1)
  - [x] `#[non_exhaustive] enum Fact` with the D19 variants, minimal-but-honest fields:
    - `Mac { addr: MacAddr, locally_administered: bool }`
    - `IpV4 { addr: std::net::Ipv4Addr }`
    - `Hostname { name: String, source: HostnameSource }` where `enum HostnameSource { Dhcp, Dns, Mdns, Netbios, Other }`
    - `DhcpLease { ip: std::net::Ipv4Addr, expires_at: Option<Timestamp> }`
    - `Uplink { peer_mac: MacAddr, peer_port: String }`
    - `OuiVendor { vendor: String }`
    - `Rtt { millis: u32 }`
  - [x] `#[non_exhaustive]` so adding a `Fact` kind later (e.g. IPv6) is not a breaking change. **No `Absent`/`Gone`/`Missing` variant** — the enum is incapable of expressing absence (NFR7, the "make the bug not compile" test).
- [x] Task 4 — Define `Scope`, `Capabilities`, and `Observation` (AC: #1, #2, #3)
  - [x] `Scope { l2_domain: L2DomainId, vantage: VantageId }` (D19).
  - [x] `Capabilities { as_of: Timestamp, kinds: BTreeSet<FactKind> }` — a dated set of the `Fact` kinds the source CAN emit (false-merge prevention, D19/D34 §1). Add a `#[non_exhaustive] enum FactKind { Mac, IpV4, Hostname, DhcpLease, Uplink, OuiVendor, Rtt }` (the discriminant of `Fact` without payload) so capabilities are a set of kinds. A `fn kind(&self) -> FactKind` on `Fact` links the two.
  - [x] `Observation { obs_id: ObsId, connector_id: ConnectorId, observed_at: Timestamp, scope: Scope, facts: Vec<Fact>, raw: Option<String> }`. `raw` is opaque provenance (the source's original payload as text) that NO decision ever reads (D19); a `String` avoids pulling `serde_json` into core for a field nothing inspects.
- [x] Task 5 — Module layout + tests (AC: #4)
  - [x] Put the types under a `observation` module in `opencmdb-core` (e.g. `src/observation/mod.rs`), re-exported from `lib.rs`. Follow the crate's rule: the folder is not the frontier — visibility is (D54); keep fields `pub` within the domain as needed.
  - [x] Unit tests: `MacAddr` round-trips through parse/Display and `is_locally_administered` matches the U/L bit; `Fact::kind()` maps each variant to its `FactKind`; `Capabilities` serde round-trips; an `Observation` serde round-trips; a compile-level note (comment) that no `Fact` variant expresses absence.
- [x] Task 6 — Verify (AC: #1–#4)
  - [x] `cargo test -p opencmdb-core` green; `cargo xtask ci` green (frontier stays clean); `cargo clippy --workspace --locked -- -D warnings` and `cargo fmt --all --check` clean; builds on the pinned 1.96 toolchain.

## Dev Notes

### This is the first real domain code — get the schema honest

`opencmdb-core/src/lib.rs` today is a walking-skeleton placeholder that "asserts nothing about identity yet". Story 2.1 lays the observation vocabulary every connector and the engine will speak. Per D19, **the fixture schema IS the Observation schema** — write these types and the `Connector` trait (Story 2.3) falls out. Get them right; everything downstream depends on them.

### The D19 schema (authoritative)

```rust
struct Observation { obs_id, connector_id, observed_at: Timestamp, scope: Scope,
                     facts: Vec<Fact>, raw: Option<Value> }   // raw = provenance, NEVER read by a decision
struct Scope { l2_domain: L2DomainId,   // the MAC's uniqueness space
               vantage: VantageId }     // WHO saw it
enum Fact { Mac{addr, locally_administered}, IpV4{..}, Hostname{name, source},
            DhcpLease{..}, Uplink{peer_mac, peer_port}, OuiVendor{..}, Rtt{..} }
```
This story implements exactly that shape, choosing the concrete `{..}` fields (Task 3) and using `Option<String>` for `raw` instead of a JSON `Value` (nothing reads it).

### NFR7 is a TYPE, not a test (D35)

The single most important constraint: **`Observation` must be INCAPABLE of expressing "gone".** It only says "here is what I saw". Absence is derived by the engine, and only when `liveness == live`. So there is NO `Absent`/`Gone`/`Missing` fact, and no "not seen" flag. "The cheapest NFR7 test that exists: make the bug not compile." A reviewer (or a later contract test) checks that the `Fact` enum has no absence variant.

### Two different "Scope"s — do not conflate

- **D19 `Scope { l2_domain, vantage }`** — carried on every `Observation`: the MAC's uniqueness space + who saw it. **This is what Story 2.1 defines.**
- **D34 §3 liveness-scope `(connector, scope)`** — "the smallest set that can go blind without the others"; keys `source_state`. Built later with liveness (Epic 13 "Ma source devient aveugle"). NOT in scope here.

They share a word, not a type. Story 2.1 defines only the observation `Scope`.

### `capabilities()` is false-merge prevention, not decoration (D19/D34 §1)

The engine must never confuse "no `Uplink` because there is none" with "no `Uplink` because this connector is blind to topology". `Capabilities` records which `Fact` kinds a source CAN emit, **dated** (`as_of`), so it can travel with a batch and a downgrade is a diff `caps(N-1) → caps(N)` (FR5). It is not a constant and not a cache — it is a last-known dated fact (D34 §1). Story 2.1 defines the type; producing it per-poll is Story 2.3's `PollSummary`.

### Dependency & crate posture

- Types live in `opencmdb-core`. No new crate dependencies needed: `chrono` (Timestamp), `serde` (derive), `uuid` (id newtypes) are already present; `std::net::Ipv4Addr` and `std::collections::BTreeSet` are std. **Do NOT** add `serde_json`, `anyhow`, or any of the frontier-forbidden crates. `tokio-util` is NOT needed here (no async/cancellation until Story 2.3).
- `#![forbid(unsafe_code)]` is already set in `lib.rs`; the `MacAddr` byte handling stays safe.
- `cargo xtask ci`'s frontier gate (your Story 1.1) will confirm core stays clean.

### Testing standards summary

- Unit tests inline (`#[cfg(test)] mod tests`) in the `observation` module.
- Pure value-type tests: parse/Display round-trips, `Fact::kind()` mapping, serde round-trips. No async, no I/O, no DB.
- Run `cargo test -p opencmdb-core`.

### Project Structure Notes

- New: `crates/opencmdb-core/src/observation/mod.rs` (or a small set of files under `observation/`); `lib.rs` gains `pub mod observation;` + re-exports.
- No changes to `opencmdb-bin`, `xtask`, or any Cargo manifest (all needed deps already declared).
- Keep the module cohesive; later stories add `ConnectorError` (2.2) and the `Connector` trait (2.3) as sibling modules.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.1: Domain observation types]
- [Source: _bmad-output/planning-artifacts/architecture.md#D19 — The fixture IS the `Connector` trait (lines 1267–1345)] — the Observation/Scope/Fact schema.
- [Source: _bmad-output/planning-artifacts/architecture.md#D34 — Three forced corrections to the Connector trait (lines 1917–2000)] — `capabilities()` dynamic + dated (§1); the liveness-scope (§3) that this story does NOT define.
- [Source: _bmad-output/planning-artifacts/architecture.md#D35 — NFR7 is a TYPE, not a test (lines 2002+)] — the no-"gone" constraint.
- [Source: crates/opencmdb-core/src/lib.rs — current placeholder; the module conventions (D54 visibility-is-the-frontier)]
- [Source: crates/opencmdb-core/Cargo.toml — chrono (clock off), serde, thiserror, uuid v7 already present]
- [Source: CLAUDE.md — the D47 dependency frontier; `cargo xtask ci`]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p opencmdb-core` → 6 passed (MacAddr parse/Display round-trip, malformed rejection, U/L bit, `Fact::kind` mapping, `Capabilities::can_emit`, `Observation` serde round-trip).
- `cargo run -p xtask -- ci` → `✅ frontier  domain graph clean` — the `serde_json` dev-dep does NOT trip the frontier gate (excluded by `-e normal`), confirming the design.
- `cargo clippy --workspace --locked -- -D warnings` clean; `cargo fmt --all --check` clean; full workspace `cargo test` green (6 + 23).
- Two manifest adjustments were required and made: `uuid` gained its `serde` feature (else `Uuid` has no `Serialize`/`Deserialize` and the id-newtype derives fail), and `serde_json` was added as a **dev-dependency** to give the serde round-trip test a concrete format (D47 permits dev-deps; the frontier gate excludes them).

### Completion Notes List

- Implemented the `observation` module in `opencmdb-core` per the D19 schema: `Observation { obs_id, connector_id, observed_at, scope, facts, raw }`, `Scope { l2_domain, vantage }`, the `Fact` enum (Mac/IpV4/Hostname/DhcpLease/Uplink/OuiVendor/Rtt), `FactKind` + `Fact::kind()`, dated `Capabilities`, and the id newtypes + `MacAddr([u8;6])`.
- **AC #1** — `Fact` (and `Observation`) has NO variant/field meaning "gone"; `#[non_exhaustive]` so IPv6 etc. is not a breaking change. NFR7 is enforced structurally.
- **AC #2** — `Scope` is D19's `{ l2_domain, vantage }`; a doc comment marks it distinct from D34 §3's liveness-scope (built later).
- **AC #3** — `Capabilities { as_of, kinds: BTreeSet<FactKind> }` is a dated set with `can_emit()`.
- **AC #4** — types in core, unit-tested, `xtask ci` green; frontier stays clean (serde_json is dev-only).
- Choices per the approved schema: `MacAddr` is exact bytes (byte-exact identity, not a String); `raw` is `Option<String>` (no `serde_json` in production); `Timestamp` is a `chrono::DateTime<Utc>` alias and core's `chrono` has `clock` off so `now()` is uncallable here.
- Scope held: no `Connector` trait, no `ConnectorError`, no async/`tokio-util` yet (Stories 2.2/2.3).

### File List

- `crates/opencmdb-core/src/observation/mod.rs` (new) — the observation vocabulary + tests.
- `crates/opencmdb-core/src/lib.rs` (modified) — `pub mod observation;` + re-exports.
- `crates/opencmdb-core/Cargo.toml` (modified) — `uuid` `serde` feature; `serde_json` dev-dependency.
- `Cargo.lock` (modified) — records the resolved dev-dependency edge / feature.

## Change Log

- 2026-07-20 — Implemented Story 2.1 (domain observation types, D19/D35). Added the `observation` module to `opencmdb-core`: `Observation`, `Scope`, `Fact`/`FactKind`, dated `Capabilities`, id newtypes, `MacAddr([u8;6])` — the vocabulary every connector speaks, structurally incapable of expressing "gone" (NFR7). 6 tests green, frontier gate stays clean (serde_json dev-only), clippy/fmt clean. First real domain code in core. Status → review.
