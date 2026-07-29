//! Interface and device identity — the composite join, and the vocabulary it answers with.
//!
//! This is the subdomain FR9–FR20 describe: deciding whether two observations describe the same
//! interface, and whether two interfaces belong to the same device. It is the highest-leverage and
//! riskiest problem in the product, and D13 fixes its shape — **all rules are evaluated**, each
//! yields an enumerated verdict, and the verdicts combine by an **algebra, never a sum**.
//!
//! **What lives here today is [`cascade`], and inside it only the abstention vocabulary.** There is
//! no rule, no verdict set and no join yet; story 5.4 writes the algebra that produces one.
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
