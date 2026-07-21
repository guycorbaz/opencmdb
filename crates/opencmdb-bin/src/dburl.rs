//! Building the MariaDB connection URL, and explaining it when it is refused.
//!
//! A password is not URL-safe, and asking an operator to percent-encode one by hand inside a
//! connection string is a trap that fires as an opaque `1045 Access denied` (issue #6). The
//! discrete `DATABASE_*` variables are therefore the documented path: opencmdb assembles the
//! URL and does the encoding itself. `DATABASE_URL` stays supported as a deprecated fallback so
//! CI, the gated tests and existing deployments keep working unchanged.
//!
//! What this CANNOT fix, and must not pretend to: a `$` in the password is eaten by Docker
//! Compose interpolating the contents of `env_file`, long before this process starts. That one
//! is only ever addressed by documentation and by the hint in [`explain_connect_error`].

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

/// The discrete variables, in the order an operator should think about them.
const DISCRETE: [&str; 5] = [
    "DATABASE_HOST",
    "DATABASE_PORT",
    "DATABASE_NAME",
    "DATABASE_USERNAME",
    "DATABASE_PASSWORD",
];

/// Where a connection URL came from — worth knowing, because one of the two paths is deprecated
/// and the other is the one we tell people to use.
#[derive(Debug, PartialEq, Eq)]
pub enum Source {
    /// Assembled from `DATABASE_HOST` / `PORT` / `NAME` / `USERNAME` / `PASSWORD`.
    Discrete,
    /// Taken verbatim from the deprecated `DATABASE_URL`.
    Url,
}

/// Read the environment and produce a connection URL.
///
/// The discrete variables win when any of them is present, so a deployment that has migrated is
/// never silently overridden by a stale `DATABASE_URL` left in the same file.
pub fn from_env<F>(var: F) -> Result<(String, Source), String>
where
    F: Fn(&str) -> Option<String>,
{
    let any_discrete = DISCRETE.iter().any(|key| var(key).is_some());

    if any_discrete {
        let required = |key: &str| -> Result<String, String> {
            var(key).filter(|v| !v.is_empty()).ok_or_else(|| {
                format!("{key} must be set (the DATABASE_* variables are used as a group)")
            })
        };
        let host = required("DATABASE_HOST")?;
        let name = required("DATABASE_NAME")?;
        let username = required("DATABASE_USERNAME")?;
        // An empty password is a legitimate, if unwise, configuration — so it is read directly
        // rather than through `required`, which rejects the empty string.
        let password = var("DATABASE_PASSWORD").unwrap_or_default();
        let port = var("DATABASE_PORT")
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "3306".to_string());
        port.parse::<u16>()
            .map_err(|_| format!("DATABASE_PORT must be a number in 1..=65535, got `{port}`"))?;

        return Ok((
            build(&host, &port, &name, &username, &password),
            Source::Discrete,
        ));
    }

    match var("DATABASE_URL").filter(|v| !v.is_empty()) {
        Some(url) => Ok((url, Source::Url)),
        None => Err(format!(
            "no database configuration: set {} (or the deprecated DATABASE_URL)",
            DISCRETE.join(", ")
        )),
    }
}

/// Assemble a `mysql://` URL, percent-encoding the user info so no character in a password can
/// change the shape of the URL.
///
/// Encoding is deliberately aggressive — everything outside `[A-Za-z0-9]` — rather than exactly
/// the RFC 3986 `userinfo` set. Over-encoding is harmless because the driver percent-decodes the
/// user info before authenticating, whereas under-encoding is a silent authentication failure.
fn build(host: &str, port: &str, name: &str, username: &str, password: &str) -> String {
    let user = utf8_percent_encode(username, NON_ALPHANUMERIC);
    let pass = utf8_percent_encode(password, NON_ALPHANUMERIC);
    // A bare IPv6 literal has to be bracketed or the colons read as the port separator.
    let host = if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    format!("mysql://{user}:{pass}@{host}:{port}/{name}")
}

/// Turn a connection failure into something an operator can act on.
///
/// `1045 Access denied` is the single most expensive error in this product's deployment story:
/// it names a real cause only if you know to read the host in it, and it is equally the symptom
/// of a password Compose truncated. Left bare, it sends people to check the password — which is
/// usually the one thing that is right.
pub fn explain_connect_error(error: &sqlx::Error) -> Option<String> {
    let sqlx::Error::Database(db) = error else {
        return None;
    };
    if db.code().as_deref() != Some("1045") {
        return None;
    }
    Some(
        "MariaDB refused the credentials. Check these three, in this order:\n\
         1. THE GRANT MUST MATCH THE HOST IN THE MESSAGE ABOVE. MariaDB matches on the address \
         it SEES, which is not always the one you dialled — on a multi-homed server, traffic \
         sent to one interface can leave by another. Grant exactly that host.\n\
         2. A `$` in the password is TRUNCATED by Docker Compose, which interpolates the \
         contents of env_file before opencmdb ever runs. Write it doubled: `$$`.\n\
         3. Only then, the password itself. If you are still using DATABASE_URL, prefer the \
         DATABASE_HOST / DATABASE_PORT / DATABASE_NAME / DATABASE_USERNAME / DATABASE_PASSWORD \
         variables, which need no manual percent-encoding."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |key: &str| map.get(key).cloned()
    }

    #[test]
    fn discrete_variables_build_a_url() {
        let (url, source) = from_env(env(&[
            ("DATABASE_HOST", "192.0.2.5"),
            ("DATABASE_NAME", "opencmdb"),
            ("DATABASE_USERNAME", "opencmdb"),
            ("DATABASE_PASSWORD", "plain"),
        ]))
        .unwrap();
        assert_eq!(source, Source::Discrete);
        // Port defaults to 3306.
        assert_eq!(url, "mysql://opencmdb:plain@192.0.2.5:3306/opencmdb");
    }

    /// The whole point of issue #6: none of these may be hand-encoded by the operator, and none
    /// of them may change the shape of the URL.
    #[test]
    fn every_url_reserved_character_survives_the_password() {
        let (url, _) = from_env(env(&[
            ("DATABASE_HOST", "db.example"),
            ("DATABASE_PORT", "3307"),
            ("DATABASE_NAME", "opencmdb"),
            ("DATABASE_USERNAME", "open cmdb"),
            ("DATABASE_PASSWORD", "s3cr$t@x:/#?%"),
        ]))
        .unwrap();
        assert_eq!(
            url,
            "mysql://open%20cmdb:s3cr%24t%40x%3A%2F%23%3F%25@db.example:3307/opencmdb"
        );
    }

    #[test]
    fn an_ipv6_host_is_bracketed() {
        let (url, _) = from_env(env(&[
            ("DATABASE_HOST", "2001:db8::1"),
            ("DATABASE_NAME", "opencmdb"),
            ("DATABASE_USERNAME", "u"),
            ("DATABASE_PASSWORD", "p"),
        ]))
        .unwrap();
        assert_eq!(url, "mysql://u:p@[2001:db8::1]:3306/opencmdb");
    }

    #[test]
    fn an_empty_password_is_allowed() {
        let (url, _) = from_env(env(&[
            ("DATABASE_HOST", "192.0.2.5"),
            ("DATABASE_NAME", "opencmdb"),
            ("DATABASE_USERNAME", "u"),
            ("DATABASE_PASSWORD", ""),
        ]))
        .unwrap();
        assert_eq!(url, "mysql://u:@192.0.2.5:3306/opencmdb");
    }

    /// A half-migrated deployment must fail loudly, naming the missing key — not fall back to a
    /// stale DATABASE_URL and connect somewhere unexpected.
    #[test]
    fn a_partial_discrete_set_is_an_error_even_with_a_url_present() {
        let err = from_env(env(&[
            ("DATABASE_HOST", "192.0.2.5"),
            ("DATABASE_URL", "mysql://u:p@192.0.2.9:3306/other"),
        ]))
        .unwrap_err();
        assert!(err.contains("DATABASE_NAME"), "{err}");
    }

    #[test]
    fn database_url_still_works_and_is_reported_as_such() {
        let (url, source) = from_env(env(&[(
            "DATABASE_URL",
            "mysql://u:p@192.0.2.5:3306/opencmdb",
        )]))
        .unwrap();
        assert_eq!(source, Source::Url);
        assert_eq!(url, "mysql://u:p@192.0.2.5:3306/opencmdb");
    }

    #[test]
    fn a_bad_port_is_rejected() {
        let err = from_env(env(&[
            ("DATABASE_HOST", "192.0.2.5"),
            ("DATABASE_PORT", "not-a-port"),
            ("DATABASE_NAME", "opencmdb"),
            ("DATABASE_USERNAME", "u"),
            ("DATABASE_PASSWORD", "p"),
        ]))
        .unwrap_err();
        assert!(err.contains("DATABASE_PORT"), "{err}");
    }

    #[test]
    fn no_configuration_at_all_names_what_to_set() {
        let err = from_env(env(&[])).unwrap_err();
        assert!(err.contains("DATABASE_HOST"), "{err}");
        assert!(err.contains("DATABASE_URL"), "{err}");
    }
}
