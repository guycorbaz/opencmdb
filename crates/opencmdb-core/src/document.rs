//! The documenting gesture's domain vocabulary (story 6.1, FR13(a)).
//!
//! `document` is the canonical name (`architecture.md:3818`): the operator closes a gap by
//! documenting an observed value into the declared record. This module carries the DOMAIN half
//! of that gesture's refusals — data, not strings (D47): no `axum`, no HTTP status. The status
//! mapping lives in the adapter, which matches on this enum exhaustively (no `_` arm, story
//! 5.3's precedent), so a new variant is a compile error there, never a silent catch-all.
//!
//! Story 6.1 ships the SHAPE only: the one refusal a shape-only route can produce. The variants
//! a real write needs (a conflict, a constraint) arrive with the write itself (story 6.2).

use thiserror::Error;

/// Why the domain refuses a documenting request.
///
/// On [`crate::repo::RepositoryError`]'s precedent: closed, enumerated, and each variant names
/// a condition the caller can act on. The `Display` text is load-bearing — the adapter serves it
/// as the response body, and a test pins it verbatim, because the unknown-subject refusal shares
/// its HTTP status with the router's unregistered-route answer and the BODY is what tells them
/// apart (story 6.1 §6).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DocumentRefusal {
    /// The subject cannot be documented: the id names no observation this lookup recognises.
    ///
    /// The `Display` sentence is written to be true in BOTH eras (story 6.1's code review,
    /// Guy's decision): under 6.1's `AlwaysUnknown` wiring EVERY subject is refused — including
    /// an id that names a genuinely recorded observation — so a sentence claiming *"the id
    /// names no observation"* would be false until story 6.2's store-backed lookup lands.
    /// *"Nothing can be documented"* is true under both.
    #[error("unknown subject: nothing can be documented")]
    UnknownSubject,
    /// The subject was already documented — a later attempt to adopt the same observation's
    /// same field would count one box twice, which is exactly the gesture's name refusing
    /// (story 6.2). Detected by the `declared_one_adoption_per_field` unique index, not a
    /// pre-read (§4/§6.5), so the refusal is race-safe.
    #[error("already documented: this observation's fields are already declared")]
    AlreadyDocumented,
    /// The subject carries nothing this product documents — its facts project to no declared
    /// field (e.g. an Rtt-only or Uplink-only observation). A success answer here would name
    /// zero fields, which is the lie this variant exists to prevent (story 6.2).
    #[error("nothing to document: the observation carries no declarable field")]
    NothingToDocument,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `Display` text is the HTTP discriminator (story 6.1 §6): the adapter's 404 collides
    /// with axum's unregistered-route 404, and the body is what tells them apart. Pinned here in
    /// the domain, next to the words themselves; the adapter pins the same bytes end-to-end.
    #[test]
    fn unknown_subject_display_is_the_pinned_discriminator() {
        assert_eq!(
            DocumentRefusal::UnknownSubject.to_string(),
            "unknown subject: nothing can be documented"
        );
    }

    /// Story 6.2's two new refusals carry distinct, pinned bodies — the adapter maps them to
    /// 409 and 422-domain, and a test end-to-end pins the same bytes.
    #[test]
    fn the_new_refusals_carry_distinct_pinned_bodies() {
        assert_eq!(
            DocumentRefusal::AlreadyDocumented.to_string(),
            "already documented: this observation's fields are already declared"
        );
        assert_eq!(
            DocumentRefusal::NothingToDocument.to_string(),
            "nothing to document: the observation carries no declarable field"
        );
    }
}
