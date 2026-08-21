//! Build script with exactly one job: declare the translation file as a build input.
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
fn main() {
    println!("cargo::rerun-if-changed=locales/app.yml");
}
