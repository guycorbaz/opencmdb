//! Interface and device identity — the composite join, and the vocabulary it answers with.
//!
//! This is the subdomain FR9–FR20 describe: deciding whether two observations describe the same
//! interface, and whether two interfaces belong to the same device. It is the highest-leverage and
//! riskiest problem in the product, and D13 fixes its shape — **all rules are evaluated**, each
//! yields an enumerated verdict, and the verdicts combine by an **algebra, never a sum**.
//!
//! **What lives here is [`cascade`] and [`l1`].** `cascade` holds the engine's vocabulary, its
//! return type and the ALGEBRA that combines them — the verdict, the `(rule, verdict, evidence)`
//! triple, the conclusion, the ruleset version, the abstention cause, and [`cascade::decide`], which
//! maps a verdict set onto a conclusion over D13's table. `l1` holds the deterministic half of the
//! cascade: the join on the scope-qualified key `(l2_domain, mac)`, the two rules the committed
//! corpus names, and the ruleset version constant — and it is what calls `decide`.
//!
//! **There is still no candidate pair generator**: `l1` answers a pair its caller supplies, and the
//! blocker that would propose pairs is the next story's. L1 emits three of the five verdicts;
//! `Supports` and `Opposes` gain a producer with Epic 6's `l2-*` rules.
//!
//! The architecture's source tree names an `IdentityError` on this module [architecture.md:3366].
//! It is absent because there is no fallible operation to carry it: choosing a cause enum cannot
//! fail. It arrives with the first operation that can.
//!
//! **The folder is not the frontier — visibility is** (D54: `pub(in ...)` → `E0603`). Creating a
//! directory buys nothing on its own, and calling it an encapsulation boundary today would be the
//! theatre D54 refuses by name. It starts meaning something when an item here is restricted to
//! this subtree, which nothing yet is: [`cascade::IdentityAbstentionCause`] is plain `pub` because
//! `score::Outcome`, in another subdomain, names it in a field type.

pub mod cascade;
pub mod l1;
