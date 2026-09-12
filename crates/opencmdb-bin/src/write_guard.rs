//! The guard every write route shares, and the reason it is ONE function.
//!
//! # Why this module exists
//!
//! 🔴 **Story 14.2b's validation measured the alternative.** `same_origin` lived private in
//! `document.rs`; the story said the new IPAM routes should *"reuse the machinery"*, and the
//! gap-hunt layer's verdict was that a dev agent would settle the silence **by copying
//! `same_origin`** — which is the one function a CSRF check must never be duplicated for. Two
//! copies means two places to weaken, and the second is the one nobody re-reads.
//!
//! ⚠️ **What is NOT here, by decision**: the refusal BODIES. `document.malformed` and
//! `ipam.malformed` are different sentences about different gestures, and a shared body would be
//! the *"one message for two surfaces"* shape this project has paid for on screens (story 6.4's
//! success sentence, written once and wrong on the second page). What is shared is the DECISION;
//! what stays local is the WORDING.

use axum::http::{HeaderMap, header};

/// The body a cross-origin refusal carries.
///
/// ⚠️ A Rust literal and not an i18n key, deliberately, on story 6.1's reasoning: this is refused
/// before any gesture is identified, so it belongs beside the check rather than in the copy of a
/// surface it may not have come from.
pub(crate) const CSRF_REFUSED_BODY: &str = "cross-origin request refused";

/// The CSRF Origin check (story 6.2 §5), pure over the request headers. It is a TRIPWIRE against
/// a browser holding the cached Basic credential being made to forge a cross-site write, at the
/// stated strength and no higher:
///
/// - **`Origin` absent → PASS** — a machine caller (`curl -u`) sends none; the threat is a
///   BROWSER, which sends `Origin` on every cross-site POST (measured, Blink);
/// - **`Origin: null` → REFUSE** — sandboxed iframes / some redirect chains; refused because it
///   carries no `://` authority to match (no dedicated branch — measured redundant);
/// - **`Origin` present → compare its authority against `Host`**, ASCII case-insensitively;
///   match → pass, mismatch → refuse. ⚠️ Stated limits: this needs the reverse proxy to FORWARD
///   `Host` (`proxy_set_header Host $host;` — nginx's default rewrite would refuse every POST);
///   `Host` ABSENT (HTTP/2 `:authority`) → refuse; the compare is authority-only, SCHEME-BLIND;
///   default-port elision is compared literally. All registered, none silently absorbed;
/// - **more than one `Origin` header → REFUSE** (6.1's `Authorization` precedent: first-value
///   semantics let right-then-wrong through).
pub(crate) fn same_origin(headers: &HeaderMap) -> bool {
    let mut origins = headers.get_all(header::ORIGIN).into_iter();
    let Some(origin) = origins.next() else {
        return true; // absent → machine caller, pass
    };
    if origins.next().is_some() {
        return false; // more than one Origin
    }
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    // Strip the scheme; compare host[:port] against Host. `Origin: null` (opaque origins —
    // sandboxed iframes, some redirect chains) carries no `://`, so it falls into the refusal
    // below without a dedicated branch (a dedicated `== "null"` check was measured redundant at
    // the mutation pass — it is refused by the missing scheme regardless).
    let origin_authority = origin.split_once("://").map(|(_, rest)| rest);
    let Some(origin_authority) = origin_authority else {
        return false;
    };
    let Some(host) = headers.get(header::HOST).and_then(|h| h.to_str().ok()) else {
        return false; // Host absent (HTTP/2 :authority) → refuse
    };
    origin_authority.eq_ignore_ascii_case(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `same_origin` pure fn table (§5), including the stated limits pinned as behaviour.
    #[test]
    fn same_origin_decides_each_case() {
        let with = |pairs: &[(&str, &str)]| {
            let mut h = HeaderMap::new();
            for (k, v) in pairs {
                h.append(
                    if *k == "origin" {
                        header::ORIGIN
                    } else {
                        header::HOST
                    },
                    v.parse().unwrap(),
                );
            }
            h
        };
        assert!(
            same_origin(&with(&[("host", "nas:8080")])),
            "absent origin passes"
        );
        assert!(
            same_origin(&with(&[
                ("origin", "http://nas:8080"),
                ("host", "nas:8080")
            ])),
            "match passes"
        );
        assert!(
            same_origin(&with(&[
                ("origin", "HTTP://NAS:8080"),
                ("host", "nas:8080")
            ])),
            "case-insensitive authority match passes"
        );
        assert!(
            !same_origin(&with(&[
                ("origin", "http://attacker"),
                ("host", "nas:8080")
            ])),
            "mismatch refused"
        );
        assert!(
            !same_origin(&with(&[("origin", "null"), ("host", "nas:8080")])),
            "null refused"
        );
        assert!(
            !same_origin(&with(&[("origin", "http://nas:8080")])),
            "host absent refused"
        );
        // SCHEME-BLIND, stated limit: https origin passes against a bare-authority Host.
        assert!(
            same_origin(&with(&[
                ("origin", "https://nas:8080"),
                ("host", "nas:8080")
            ])),
            "scheme-blind (stated limit): same authority passes across schemes"
        );
    }

    /// 🔴 **ONE implementation, and this guard is what keeps it one.**
    ///
    /// The defect it exists against is not a weakening of the check — it is a COPY of it, made in
    /// good faith by whoever writes the next write route and finds this function private to
    /// somebody else's module. Two copies means two places to weaken, and the second is the one
    /// nobody re-reads. Measured at story 14.2b's validation: that is exactly what a dev agent was
    /// predicted to do.
    ///
    /// ⚠️ **A TRIPWIRE, not a barrier** (story 5.12's framing): it greps for the comparison's own
    /// shape, so a second implementation written differently is invisible to it. It catches the
    /// COPY, which is the gesture that actually happens.
    #[test]
    fn the_origin_comparison_exists_exactly_once() {
        let mut found = Vec::new();
        for entry in std::fs::read_dir("src").expect("the crate's own sources") {
            let path = entry.expect("a directory entry").path();
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a source file");
            let code = text.split("\n#[cfg(test)]").next().unwrap_or(&text);
            if code.contains("eq_ignore_ascii_case(host)") {
                found.push(path.display().to_string());
            }
        }
        assert_eq!(
            found,
            vec!["src/write_guard.rs".to_string()],
            "the Origin/Host comparison must exist in exactly ONE place. A second copy is how a \
             CSRF check comes to be weakened in a file nobody re-reads — and copying it is what a \
             dev agent was measured to do when it was private to another module"
        );
    }
}
