# Story 3.6: A first real gap, abstaining elsewhere

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want the engine to compute one real gap on a cardinality-1 perimeter and abstain + count everywhere else,
so that the product's core thesis — the gap — is demonstrated end to end.

## Acceptance Criteria

1. **Given** a declared record and a linked observation that differ on a cardinality-1 perimeter, **when** reconciliation runs, **then** it reconciles by identity (FR10) and surfaces exactly that gap.
2. **Given** ambiguous or out-of-perimeter data, **when** the engine runs, **then** it ABSTAINS (never guesses/merges, FR16 min) and the abstention is counted and grouped by cause (reach, not debt — FR16b).
3. **And** the gap computation is a pure function (no clock, no SQL) and is unit-tested on synthetic inputs.

## Tasks / Subtasks

- [x] Task 1 — The `gap` module in `opencmdb-core` (AC: #1, #2, #3)
  - [x] `crates/opencmdb-core/src/gap/mod.rs`: `Gap { field, declared, observed }` (a drift — declared ≠ observed); `AbstentionCause { OutOfPerimeter, NoObservedValue, ConflictingObservations }`; `Reconciliation { gaps: Vec<Gap>, abstentions: BTreeMap<AbstentionCause, usize> }` (counted, grouped by cause). Re-export from `lib.rs`.
  - [x] `fn project(&Observation) -> Vec<(String, String)>` — the vocabulary bridge from `Fact`s to comparable field/value pairs (`IpV4`→`ipv4`, `Hostname`→`hostname`, `Mac`→`mac`; Rtt/etc. not reconciled in the skeleton).
  - [x] `pub fn reconcile(identity: (&str,&str), declared: &[(String,String)], observations: &[Observation]) -> Reconciliation`: an observation is in perimeter iff it carries `identity`; out-of-perimeter → abstain `OutOfPerimeter` (counted). Two in-perimeter observations disagreeing on a field → abstain `ConflictingObservations` (never pick). A declared field with no in-perimeter observed value → abstain `NoObservedValue` (never fabricate absence — NFR7). A field present both sides with different values → a `Gap`. Pure: no clock, no SQL, no history.
- [x] Task 2 — Tests on synthetic inputs (AC: #1, #2, #3)
  - [x] Drift gap: declared `[ipv4=X, hostname=nas]`, one in-perimeter obs `[IpV4(X), Hostname(intruder)]` → exactly one `Gap` on `hostname`, no abstentions.
  - [x] Out-of-perimeter: obs `[IpV4(Y)]` with `Y != X` → `abstentions[OutOfPerimeter] == 1`, no gaps.
  - [x] Conflict: two in-perimeter obs reporting different `hostname` → `abstentions[ConflictingObservations] >= 1`, no gap on hostname (never merged/picked).
  - [x] No observed value: declared `mac` with no observed mac → `abstentions[NoObservedValue] == 1`, no fabricated gap.
  - [x] Clear: declared and observed agree → no gaps, no abstentions.
- [x] Task 3 — Verify (AC: #1–#3)
  - [x] `cargo test -p opencmdb-core` green; `cargo xtask ci` green (frontier: pure core, no new deps); clippy `-D warnings` + fmt clean. No DB, no network — pure-function tests.

## Dev Notes

### The gap is the product — as a pure function (D3/D10)

`gap := declared.value != current_observation.value`, and the computation NEVER reads `origin` or history (D3) — a field adopted yesterday can drift again tomorrow. It descends into no SQL (D10) and reads no clock (D19): `reconcile` is a pure function of (identity, declared, observations). This is what makes it deterministically testable on synthetic inputs, which the AC requires.

### Abstention is reach, not debt (FR16/FR16b)

When the engine cannot conclude it ABSTAINS — it never guesses and never merges. Abstentions are COUNTED and grouped by cause, and the count measures reach ("we saw N things we could not place"), never a reproach. Three causes in the skeleton:
- `OutOfPerimeter` — an observation that is not the perimeter entity (e.g. an undocumented device the ping scan saw). With a ping-only source this is the natural, common abstention.
- `ConflictingObservations` — two in-perimeter observations disagree on a field; the engine refuses to pick (FR16). Never a silent merge.
- `NoObservedValue` — a declared field the source did not report; absence is NOT fabricated (NFR7 — the engine derives absence only when live, which is later).

### Cardinality-1 perimeter

The walking skeleton reconciles ONE entity, identified by an `(field, value)` pair (e.g. `("ipv4", "192.0.2.10")`). Observations carrying it are the entity; the rest abstain. Real multi-entity identity (L1/L2 composite) is Epic 5+. Keep the perimeter a single identity here.

### Honest note on the live demo

The ping connector (3.5) emits only `IpV4` + `Rtt`, so on real data the common outcome is `OutOfPerimeter` abstentions (undocumented IPs) and a `clear` match on the declared entity's IP — a genuine gap of the *drift* kind needs a field both sides carry (e.g. a declared `hostname` differing from an observed one). This story proves the engine on synthetic inputs (a drift gap + counted abstentions); the page (3.7) renders whatever the real data yields.

### Dependency & crate posture

- Pure `opencmdb-core` code (`observation` types only). No new dependencies; frontier gate unaffected.
- No `opencmdb-bin`/`xtask` changes in this story — the engine is domain logic; wiring reconciliation over the persisted records is the page's concern (3.7).

### Testing standards summary

- Synthetic `#[test]`s (no async, no DB, no network) covering: a drift gap, each abstention cause, and a clear match.
- Run `cargo test -p opencmdb-core`.

### Project Structure Notes

- New: `crates/opencmdb-core/src/gap/mod.rs`. Modified: `crates/opencmdb-core/src/lib.rs` (`pub mod gap` + re-exports). Sibling to `observation`, `connector`, `repo`, `clock`.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.6: A first real gap, abstaining elsewhere]
- [Source: _bmad-output/planning-artifacts/architecture.md#D3 — `gap := declared.value != current_observation.value`; the gap never reads origin]
- [Source: _bmad-output/planning-artifacts/architecture.md#D10 — comparison never descends into SQL] · [D19 — the engine never touches the clock]
- [Source: crates/opencmdb-core/src/observation/mod.rs — `Observation`/`Fact` the engine reconciles]
- [Source: _bmad-output/planning-artifacts/prd.md — FR10 reconcile by identity; FR16/FR16b abstention counted, reach-not-debt]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p opencmdb-core gap::` → 5 passed (surfaces_exactly_the_drift_gap, out_of_perimeter_observation_abstains_and_counts, conflicting_observations_abstain_never_pick, declared_field_with_no_observation_abstains_never_fabricates, agreement_is_clear_no_gap_no_abstention).
- Full workspace green: 29 core + 7 bin + 23 xtask. `cargo xtask ci` → frontier gate green (domain graph clean; the gap module is pure core, no new deps). clippy `-D warnings` + fmt clean.

### Completion Notes List

- `gap::reconcile` (core): a PURE function of `(identity, declared, observations)` — no clock (D19), no SQL (D10), no `origin`/history (D3). `Gap { field, declared, observed }` is a drift; `AbstentionCause { OutOfPerimeter, NoObservedValue, ConflictingObservations }`; `Reconciliation { gaps, abstentions: BTreeMap<cause, count> }` counts abstentions grouped by cause (reach, not debt — FR16b).
- **AC #1** — reconciles by identity (a cardinality-1 `(field,value)` perimeter, FR10) and surfaces exactly the drift gap (drift-gap test). **AC #2** — abstains and counts for out-of-perimeter, conflicting, and no-observed-value inputs; never guesses, never merges (FR16); a conflicting field is removed from comparison, never picked (NFR7 — absence is not fabricated). **AC #3** — pure function, 5 synthetic `#[test]`s, no DB/network.
- `project(&Observation)` bridges observed `Fact`s to declared field keys (`IpV4`→`ipv4`, `Hostname`→`hostname`, `Mac`→`mac`); Rtt/DhcpLease/Uplink/OuiVendor are not reconciled as declared fields in the skeleton.
- No new dependencies (core already has serde); frontier gate unaffected. No `bin`/`xtask` changes — persisting reconciliation over stored records lands with the page (3.7).

### File List

- `crates/opencmdb-core/src/gap/mod.rs` (new) — the pure gap engine + 5 synthetic tests.
- `crates/opencmdb-core/src/lib.rs` (modified) — `pub mod gap;` + re-export `AbstentionCause`, `Gap`, `Reconciliation`, `reconcile`.

## Change Log

- 2026-07-20 — Implemented Story 3.6 (the first real gap). `gap::reconcile` computes one drift gap on a cardinality-1 perimeter and abstains + counts everywhere else (out-of-perimeter, conflicting, no-observed-value) — pure (no clock, no SQL, no origin/history). 5 synthetic tests; frontier/clippy/fmt green; full workspace green. Status → review.
- 2026-07-20 — CI (29740442980) FAILED: an unused `MacAddr` import in the test module — CI compiles tests with `-D warnings`, but local `cargo clippy --workspace` (no `--all-targets`) does NOT lint test code, so it slipped through. FIX: removed the import; local gate hardened to `cargo clippy --workspace --all-targets --locked -- -D warnings` (lints test targets too). Re-pushed.
