//! Tells Cargo that the translation file is an input to this crate.
fn main() {
    println!("cargo::rerun-if-changed=locales/app.yml");
}
