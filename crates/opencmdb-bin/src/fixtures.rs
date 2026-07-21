//! Reading the committed fixture corpus (Story 4.1).
//!
//! The corpus lives at the workspace ROOT, in `fixtures/`, outside every crate (D56): a file
//! under `tests/` reads as the property of the test, and the first reflex of someone refactoring
//! the engine is to edit it until the red goes away. At the root, changing one is a commit that
//! says *"I am changing the spec"*.
//!
//! **The fixture schema IS the [`Observation`] schema** (D19) — one serialized `Observation` per
//! line, in the domain types' own serde representation. There is no DTO, no wrapper and no
//! second format to keep in step: *"the fixture is a serialised stream of Observations … write
//! the fixture and the trait falls out."*
//!
//! Nothing here reads a clock or mints an id. `obs_id` is stable so truth can point at it, and
//! `observed_at` comes from the file so the engine never touches the clock — determinism is what
//! makes the corpus an oracle rather than a snapshot.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use opencmdb_core::observation::Observation;

/// The one and only expression of where the corpus lives (D56 path discipline).
///
/// There is no `CARGO_WORKSPACE_DIR`, so the root is reached relatively from this crate's
/// manifest directory. **If this string appears anywhere else in the tree, it is already
/// broken** — take the path from [`fixtures_dir`] instead of writing it again.
const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/");

/// The corpus root, as an owned path.
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(FIXTURES_DIR)
}

/// Resolve a corpus-relative path (e.g. `scenario/replay/minimal.jsonl`).
///
/// The argument is CONTAINED to the corpus: an absolute path would make `join` discard the root
/// entirely, and a `..` component would climb out of it. The MANIFEST parser already refuses
/// both, so accepting them here would make the reader strictly more permissive than the lock
/// that is supposed to guard it — and any future connector taking a fixture name from
/// configuration would read arbitrary files.
pub fn fixture_path(relative: &str) -> Result<PathBuf, FixtureError> {
    let candidate = Path::new(relative);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(FixtureError::OutsideCorpus {
            requested: relative.to_string(),
        });
    }
    Ok(fixtures_dir().join(candidate))
}

/// Why a fixture could not be read. A malformed line names its own 1-indexed number: a corpus
/// that silently skips a line it cannot parse is not an oracle.
#[derive(Debug)]
pub enum FixtureError {
    /// The file could not be opened or read.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// A line did not deserialize into an [`Observation`].
    Line {
        path: PathBuf,
        lineno: usize,
        source: serde_json::Error,
    },
    /// The requested path would leave the corpus.
    OutsideCorpus { requested: String },
}

impl std::fmt::Display for FixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixtureError::Io { path, source } => {
                write!(f, "reading fixture {}: {source}", path.display())
            }
            FixtureError::Line {
                path,
                lineno,
                source,
            } => write!(f, "{}:{lineno}: {source}", path.display()),
            FixtureError::OutsideCorpus { requested } => write!(
                f,
                "fixture path `{requested}` leaves the corpus (absolute paths and `..` are refused)"
            ),
        }
    }
}

impl std::error::Error for FixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FixtureError::Io { source, .. } => Some(source),
            FixtureError::Line { source, .. } => Some(source),
            FixtureError::OutsideCorpus { .. } => None,
        }
    }
}

/// Read a JSONL fixture into its observations, in file order.
///
/// Blank lines are skipped; every other line must deserialize, or the read fails naming the
/// line. Order is preserved because replay order is part of what a trap asserts.
pub fn read_jsonl(path: &Path) -> Result<Vec<Observation>, FixtureError> {
    let text = std::fs::read_to_string(path).map_err(|source| FixtureError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut observations = Vec::new();
    for (index, line) in text.lines().enumerate() {
        // Only a truly empty line is skipped. A whitespace-only line carries content, and this
        // module's rule is that content it cannot parse is named, never silently dropped.
        if line.is_empty() {
            continue;
        }
        let observation = serde_json::from_str(line).map_err(|source| FixtureError::Line {
            path: path.to_path_buf(),
            // 1-indexed, counted over the raw lines: a blank line still occupies its number, so
            // the message points at what an editor shows.
            lineno: index + 1,
            source,
        })?;
        observations.push(observation);
    }
    Ok(observations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencmdb_core::observation::{
        ConnectorId, Fact, HostnameSource, L2DomainId, MacAddr, ObsId, Scope, Timestamp, VantageId,
    };
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use uuid::Uuid;

    const MINIMAL: &str = "scenario/replay/minimal.jsonl";

    fn ts(s: &str) -> Timestamp {
        chrono::DateTime::parse_from_rfc3339(s)
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    fn u(s: &str) -> Uuid {
        Uuid::parse_str(s).unwrap()
    }

    /// The corpus, written out in Rust. The test is the second, independent statement of what
    /// the committed bytes mean — if serde's representation ever shifts under us, these two
    /// disagree and the corpus stops silently meaning something else.
    fn expected() -> Vec<Observation> {
        let scope = Scope {
            l2_domain: L2DomainId::from_uuid(u("11111111-1111-4111-8111-111111111111")),
            vantage: VantageId::from_uuid(u("22222222-2222-4222-8222-222222222222")),
        };
        let connector_id = ConnectorId::from_uuid(u("33333333-3333-4333-8333-333333333333"));
        vec![
            Observation {
                obs_id: ObsId::from_uuid(u("aaaaaaaa-0000-4000-8000-000000000001")),
                connector_id,
                observed_at: ts("2026-01-01T00:00:00Z"),
                scope,
                facts: vec![
                    Fact::Mac {
                        addr: MacAddr::from_str("02:00:5e:00:53:01").unwrap(),
                        locally_administered: true,
                    },
                    Fact::IpV4 {
                        addr: Ipv4Addr::new(192, 0, 2, 10),
                    },
                    Fact::Hostname {
                        name: "doc-host-a".into(),
                        source: HostnameSource::Dhcp,
                    },
                ],
                raw: None,
            },
            Observation {
                obs_id: ObsId::from_uuid(u("aaaaaaaa-0000-4000-8000-000000000002")),
                connector_id,
                observed_at: ts("2026-01-01T00:00:05Z"),
                scope,
                facts: vec![
                    Fact::IpV4 {
                        addr: Ipv4Addr::new(192, 0, 2, 11),
                    },
                    Fact::Rtt { millis: 7 },
                ],
                raw: None,
            },
            Observation {
                obs_id: ObsId::from_uuid(u("aaaaaaaa-0000-4000-8000-000000000003")),
                connector_id,
                observed_at: ts("2026-01-01T00:00:10Z"),
                scope,
                facts: vec![
                    Fact::Mac {
                        addr: MacAddr::from_str("02:00:5e:00:53:02").unwrap(),
                        locally_administered: true,
                    },
                    Fact::OuiVendor {
                        vendor: String::new(),
                    },
                ],
                raw: Some("{\"provenance\":\"never read by a decision\"}".into()),
            },
        ]
    }

    #[test]
    fn the_committed_fixture_reads_back_exactly() {
        let observations =
            read_jsonl(&fixture_path(MINIMAL).unwrap()).expect("the fixture must read");
        assert_eq!(observations, expected());
    }

    /// Re-serializing must reproduce the committed bytes. This is what freezes the FORMAT and
    /// not merely the values: a serde rename would still round-trip in memory while silently
    /// changing what every future trap file means.
    #[test]
    fn re_serializing_reproduces_the_committed_bytes() {
        let path = fixture_path(MINIMAL).unwrap();
        let on_disk = std::fs::read_to_string(&path).expect("the fixture must exist");
        let mut rendered = String::new();
        for observation in expected() {
            rendered.push_str(&serde_json::to_string(&observation).expect("serialize"));
            rendered.push('\n');
        }
        assert_eq!(rendered, on_disk);
    }

    /// Every value is synthetic: RFC 5737 addresses and locally-administered MACs. A real
    /// capture in a public repository is disqualifying (D19).
    ///
    /// This reads the COMMITTED FILE, not `expected()`. The whole argument of this module is
    /// that the file is the spec — a privacy guard that inspects a Rust literal would stay green
    /// while someone pasted a real MAC into the corpus.
    #[test]
    fn the_corpus_carries_no_real_network_data() {
        for observation in read_jsonl(&fixture_path(MINIMAL).unwrap()).expect("the fixture reads") {
            for fact in &observation.facts {
                // Exhaustive on purpose — no `_` arm. `Fact` is `#[non_exhaustive]`, so a new
                // variant carrying an address must break THIS test and force a decision, rather
                // than slipping past a catch-all that asserts nothing.
                match fact {
                    Fact::IpV4 { addr } => assert_documentation_ip(*addr),
                    Fact::DhcpLease { ip, .. } => assert_documentation_ip(*ip),
                    Fact::Mac { addr, .. } => assert_synthetic_mac(*addr),
                    Fact::Uplink { peer_mac, .. } => assert_synthetic_mac(*peer_mac),
                    Fact::Hostname { name, .. } => assert!(
                        name.starts_with("doc-"),
                        "hostnames must be invented, not captured: {name}"
                    ),
                    Fact::OuiVendor { .. } | Fact::Rtt { .. } => {}
                    other => panic!(
                        "a new Fact variant reached the corpus with no privacy rule: {other:?}"
                    ),
                }
            }
        }
    }

    /// RFC 5737 reserves three ranges for documentation. Accepting only one and blaming the
    /// standard in the message would send a future author looking for a defect that is not there.
    fn assert_documentation_ip(addr: Ipv4Addr) {
        let o = addr.octets();
        let documentation = matches!(
            [o[0], o[1], o[2]],
            [192, 0, 2] | [198, 51, 100] | [203, 0, 113]
        );
        assert!(
            documentation,
            "{addr} is not in an RFC 5737 documentation range \
             (192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24)"
        );
    }

    fn assert_synthetic_mac(addr: MacAddr) {
        assert!(
            addr.is_locally_administered(),
            "{addr} is not locally administered — a real vendor address must never be committed"
        );
    }

    /// A line that cannot be parsed is named, never skipped: a corpus that drops what it does
    /// not understand cannot be an oracle.
    #[test]
    fn a_malformed_line_names_its_line_number() {
        let dir = scratch_dir("malformed");
        let path = dir.join("broken.jsonl");
        let good = serde_json::to_string(&expected()[0]).unwrap();
        std::fs::write(&path, format!("{good}\n\n{{ not json\n")).unwrap();

        let err = read_jsonl(&path).expect_err("a malformed line must fail the read");
        match &err {
            FixtureError::Line { lineno, .. } => {
                assert_eq!(*lineno, 3, "blank lines do not shift the count")
            }
            other => panic!("expected a line error, got {other:?}"),
        }
        // The message must carry the file and the underlying reason, not just a number.
        let rendered = err.to_string();
        assert!(rendered.contains("broken.jsonl"), "{rendered}");
        assert!(std::error::Error::source(&err).is_some());
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A whitespace-only line carries content, so it must be named rather than skipped.
    #[test]
    fn a_whitespace_only_line_is_not_silently_skipped() {
        let dir = scratch_dir("whitespace");
        let path = dir.join("spaces.jsonl");
        let good = serde_json::to_string(&expected()[0]).unwrap();
        std::fs::write(&path, format!("{good}\n   \n")).unwrap();

        match read_jsonl(&path).expect_err("a whitespace-only line must fail the read") {
            FixtureError::Line { lineno, .. } => assert_eq!(lineno, 2),
            other => panic!("expected a line error, got {other:?}"),
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A missing file is an `Io` error that names the path — the variant the reader's own
    /// acceptance criterion promised and nothing exercised.
    #[test]
    fn a_missing_file_is_an_io_error_naming_the_path() {
        let path = scratch_dir("missing").join("absent.jsonl");
        let err = read_jsonl(&path).expect_err("a missing file must fail the read");
        assert!(matches!(err, FixtureError::Io { .. }), "{err:?}");
        assert!(err.to_string().contains("absent.jsonl"), "{err}");
        assert!(std::error::Error::source(&err).is_some());
    }

    /// The reader must not be more permissive than the lock that guards it: the MANIFEST parser
    /// refuses absolute paths and `..`, so the reader refuses them too.
    #[test]
    fn a_path_leaving_the_corpus_is_refused() {
        for escaping in [
            "/etc/passwd",
            "../../etc/passwd",
            "scenario/../../outside.jsonl",
        ] {
            let err = fixture_path(escaping)
                .expect_err("a path leaving the corpus must be refused: {escaping}");
            assert!(matches!(err, FixtureError::OutsideCorpus { .. }), "{err:?}");
        }
        assert!(fixture_path("scenario/replay/minimal.jsonl").is_ok());
    }

    /// A private scratch directory per test. A shared constant path races between concurrent
    /// `cargo test` runs and panics as a parser failure when it is owned by another user.
    fn scratch_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("opencmdb-fixtures-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    /// D56: the corpus path is "a single constant, in one module, never copied. If it appears
    /// more than once in the tree, it is already broken." That is checkable, so it is checked.
    ///
    /// Three things this deliberately does NOT do, each of which was a real defect:
    /// it does not follow directory symlinks (a link to an ancestor made the walk loop forever);
    /// it does not swallow read errors (an unreadable subtree used to shrink the search space
    /// into a false green); and it does not walk the whole workspace (a `git worktree` or a
    /// vendored copy under the root produced a red that blamed path discipline for nothing).
    #[test]
    fn the_fixtures_path_is_expressed_once() {
        const NEEDLE: &str = "/../../fixtures";
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        // The source roots this rule governs — not an unbounded walk of everything present.
        let mut roots = vec![workspace.join("xtask/src")];
        for crate_dir in std::fs::read_dir(workspace.join("crates"))
            .expect("crates/ must be readable")
            .flatten()
        {
            roots.push(crate_dir.path().join("src"));
        }

        let mut occurrences = 0usize;
        let mut files = Vec::new();
        while let Some(dir) = roots.pop() {
            if !dir.exists() {
                continue;
            }
            for entry in
                std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
            {
                let entry = entry.expect("a directory entry must be readable");
                // `file_type()` does NOT follow symlinks: a link is neither descended into nor
                // read, so a cycle cannot exist and a link cannot smuggle in a second copy.
                let file_type = entry.file_type().expect("a file type must be readable");
                let path = entry.path();
                if file_type.is_dir() {
                    roots.push(path);
                } else if file_type.is_file() && path.extension().is_some_and(|e| e == "rs") {
                    let text = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
                    // Count OCCURRENCES, not files: the constant written twice in one file is
                    // exactly as broken as the same constant written in two files.
                    let n = text.matches(NEEDLE).count();
                    if n > 0 {
                        occurrences += n;
                        files.push((path, n));
                    }
                }
            }
        }

        // Two expected occurrences, both in THIS file: the constant itself, and NEEDLE above.
        assert_eq!(
            occurrences, 2,
            "the corpus path must be expressed once (plus this test's own needle); found {files:?}"
        );
    }
}
