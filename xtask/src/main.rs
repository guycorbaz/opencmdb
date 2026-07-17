//! xtask — `cargo xtask <cmd>`. All CI gates live here, in Rust, not YAML (D56).
//!
//! Planned subcommands: `ci` (every gate — D65's `--ddl-collation` and `--vocabulary`,
//! plus the fixture MANIFEST sha256 and the `architecture-views.md` staleness check),
//! `css` (the pinned Tailwind CLI — never build.rs, D55), `recapture` (D56).

fn main() {
    let cmd = std::env::args().nth(1);
    match cmd.as_deref() {
        Some("ci") => eprintln!("xtask ci — gates not yet implemented (D65)"),
        Some(other) => eprintln!("xtask: unknown command {other:?}"),
        None => eprintln!("usage: cargo xtask <ci|css|recapture>"),
    }
}
