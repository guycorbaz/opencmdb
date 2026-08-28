//! Build script with one job: declare the files a procedural macro reads as build inputs.
//!
//! 🔴 **Without this, `crates/opencmdb-bin/locales/app.yml` is INVISIBLE to Cargo when building
//! the binary.** `rust_i18n::i18n!("locales", …)` (see `main.rs`) reads the file through a
//! procedural macro, and a proc macro that opens a file registers no Cargo dependency on it. So
//! `cargo build` after a translation-only change finished in **0.08 s with no `Compiling` line**,
//! and the new string was **absent from `target/debug/opencmdb`** — measured at story 6b.10 on the
//! committed tree, with a sentinel value: 0 occurrences in the binary.
//!
//! ⚠️ **`cargo test` was NOT affected, and the reason is worth knowing because it is accidental.**
//! Two guards — `screens.rs`'s `every_key_carries_both_locales` and `page.rs`'s
//! `no_gesture_copy_names_the_story_that_will_build_it` — call
//! `include_str!("../locales/app.yml")`, and rustc records `include_str!` targets in dep-info,
//! which is Cargo's fingerprint input. That gave the *test* target an edge on this file that the
//! *binary* never had (visible in `target/debug/deps/opencmdb-*.d`, absent from
//! `target/debug/opencmdb.d`). Replace those two calls with runtime reads and the same mutation
//! goes green — measured. **This script makes the dependency structural instead of incidental**,
//! so a future refactor of either guard cannot silently take it away.
//!
//! 🔑 **Proving the fix means checking the STRING, never the rebuild time** — and with
//! `grep -a`, never `strings | grep`: GNU `strings` breaks its run on any multibyte character, so
//! it cannot see **163 of the 284 French values** in the file. An instrument that cannot confirm
//! presence is no proof of presence.
//! 🔴 **The same hole existed for `migrations/`, and story 6.5 measured it.** `sqlx::migrate!`
//! (`main.rs`) is a procedural macro too, and it has the same blind spot with one twist that makes
//! it worse: **MODIFYING an existing migration rebuilds** — rustc records the files the macro
//! actually opened — while **ADDING one does not**, because a file the macro never read on the
//! previous build is in no dep-info. Measured on the committed tree: dropping a new
//! `0006_*.sql` into `migrations/` and running `cargo build --workspace --locked` finished in
//! **0.07 s with no `Compiling` line**, `grep -ac "CREATE TABLE entity" target/debug/opencmdb`
//! answered **0**, and the binary then booted printing *"database connected and migrations
//! applied"* over a store that had received five migrations, not six.
//!
//! ⚠️ **That is the ordinary gesture of a schema story** — write the file, build, boot, look — and
//! the log line asserts success while the deliverable is absent. Watching a directory rather than a
//! file is what closes it: Cargo re-runs this script when any entry under `migrations/` changes.
fn main() {
    println!("cargo::rerun-if-changed=locales/app.yml");
    println!("cargo::rerun-if-changed=migrations");
}
