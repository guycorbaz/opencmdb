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

use opencmdb_core::observation::{ConnectorId, FactKind, L2DomainId, Observation, VantageId};
use opencmdb_core::trap::{TrapError, TrapFile};

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
    // `CurDir` is refused alongside `ParentDir`: `./x` and `x` name one file but are two cache
    // keys and two MANIFEST spellings, and only one of them is the spelling the lock records.
    if candidate.is_absolute()
        || candidate.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(FixtureError::OutsideCorpus {
            requested: relative.to_string(),
        });
    }
    Ok(fixtures_dir().join(candidate))
}

/// Why a fixture could not be read, or why a stream may not CLAIM what it claims. A malformed
/// line names its own 1-indexed number: a corpus that silently skips a line it cannot parse is
/// not an oracle.
///
/// Two shapes live here. The reading variants carry a `path: PathBuf` — they always come from a
/// file. The four replay-admissibility variants added by story 4.4 carry an `origin: String`
/// instead, because a stream handed to `FixtureConnector::from_observations` may never have been
/// on disk; a fabricated `PathBuf::from("<in-memory>")` would be a lie in the type, told only to
/// preserve a habit.
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
    /// A trap file did not parse.
    Toml {
        path: PathBuf,
        source: toml::de::Error,
    },
    /// A trap file parsed but is not admissible.
    Trap { path: PathBuf, source: TrapError },
    /// A replay stream contains the same `obs_id` twice.
    DuplicateObservationId {
        path: PathBuf,
        obs_id: String,
        first_line: usize,
        second_line: usize,
    },
    /// A trap judges an observation that its replay stream does not contain.
    DanglingObservation {
        path: PathBuf,
        trap: String,
        obs_id: String,
        replay: String,
    },

    // ── Replay admissibility (story 4.4). These carry `origin`, not `path`. ──
    /// A stream carries an observation attributed to a different connector than the one
    /// replaying it. Emitting it would fabricate provenance.
    ForeignConnectorId {
        origin: String,
        expected: ConnectorId,
        found: ConnectorId,
        obs_id: String,
    },
    /// A stream observes a scope the poll does not claim to have covered. The reverse —
    /// covered and empty — stays legitimate: it is what makes an absence meaningful.
    UncoveredScope {
        origin: String,
        l2_domain: L2DomainId,
        vantage: VantageId,
        obs_id: String,
    },
    /// A stream emits a fact of a kind its `Capabilities` say the source cannot emit. The
    /// reverse — capable and unseen — stays legitimate: it is the whole point of the
    /// descriptor (D34 §1).
    UndeclaredFactKind {
        origin: String,
        kind: FactKind,
        obs_id: String,
    },
    /// An in-memory stream repeats an `obs_id`. [`read_jsonl`] already refuses this for a
    /// file, naming both lines; a `Vec` has no lines, so this variant names the id alone.
    RepeatedObservationId { origin: String, obs_id: String },
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
            FixtureError::Toml { path, source } => {
                write!(f, "trap file {}: {source}", path.display())
            }
            FixtureError::Trap { path, source } => write!(f, "{}: {source}", path.display()),
            FixtureError::DuplicateObservationId {
                path,
                obs_id,
                first_line,
                second_line,
            } => write!(
                f,
                "{}: observation {obs_id} appears on lines {first_line} and {second_line} — \
                 within one stream an obs_id must name exactly one observation, or a trap \
                 referencing it does not say which",
                path.display()
            ),
            FixtureError::DanglingObservation {
                path,
                trap,
                obs_id,
                replay,
            } => write!(
                f,
                "{}: trap `{trap}` judges observation {obs_id}, which `{replay}` does not contain",
                path.display()
            ),
            FixtureError::ForeignConnectorId {
                origin,
                expected,
                found,
                obs_id,
            } => write!(
                f,
                "{origin}: observation {obs_id} is attributed to connector {found}, but this \
                 replay is connector {expected} — one stream is one connector, and emitting \
                 another's observations would fabricate provenance"
            ),
            FixtureError::UncoveredScope {
                origin,
                l2_domain,
                vantage,
                obs_id,
            } => write!(
                f,
                "{origin}: observation {obs_id} is in scope (l2_domain {l2_domain}, vantage \
                 {vantage}), which this poll does not claim to have covered — a poll may cover \
                 more than it saw, never less"
            ),
            FixtureError::UndeclaredFactKind {
                origin,
                kind,
                obs_id,
            } => write!(
                f,
                "{origin}: observation {obs_id} emits a {kind:?} fact, which these capabilities \
                 say the source cannot emit — a source may be capable and see nothing, never \
                 the reverse"
            ),
            FixtureError::RepeatedObservationId { origin, obs_id } => write!(
                f,
                "{origin}: observation {obs_id} appears more than once — within one stream an \
                 obs_id must name exactly one observation, or a trap referencing it does not \
                 say which"
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
            FixtureError::Toml { source, .. } => Some(source),
            FixtureError::Trap { source, .. } => Some(source),
            FixtureError::DuplicateObservationId { .. } => None,
            FixtureError::DanglingObservation { .. } => None,
            FixtureError::ForeignConnectorId { .. } => None,
            FixtureError::UncoveredScope { .. } => None,
            FixtureError::UndeclaredFactKind { .. } => None,
            FixtureError::RepeatedObservationId { .. } => None,
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
    let mut seen: std::collections::BTreeMap<uuid::Uuid, usize> = std::collections::BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        // Only a truly empty line is skipped. A whitespace-only line carries content, and this
        // module's rule is that content it cannot parse is named, never silently dropped.
        if line.is_empty() {
            continue;
        }
        let observation: Observation =
            serde_json::from_str(line).map_err(|source| FixtureError::Line {
                path: path.to_path_buf(),
                // 1-indexed, counted over the raw lines: a blank line still occupies its number, so
                // the message points at what an editor shows.
                lineno: index + 1,
                source,
            })?;
        // `obs_id` is the anchor the whole labelling format rests on — a trap points at one
        // "never by line number" (story 4.2). Two lines sharing an id void that guarantee, and
        // a trap referencing it would silently judge whichever one the reader happened to keep.
        let id = observation.obs_id.as_uuid();
        if let Some(first) = seen.insert(id, index + 1) {
            return Err(FixtureError::DuplicateObservationId {
                path: path.to_path_buf(),
                obs_id: id.to_string(),
                first_line: first,
                second_line: index + 1,
            });
        }
        observations.push(observation);
    }
    Ok(observations)
}

/// Read a trap file, validate it, and check that every observation it judges actually exists in
/// the replay stream it names.
///
/// The cross-check is the point: a trap that points at an `obs_id` absent from its stream is a
/// trap that can never fire, and it would sit in the corpus looking like coverage. The gate
/// counts traps, so a trap that cannot fail is worse than no trap at all.
pub fn read_traps(path: &Path) -> Result<TrapFile, FixtureError> {
    let text = std::fs::read_to_string(path).map_err(|source| FixtureError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let traps: TrapFile = toml::from_str(&text).map_err(|source| FixtureError::Toml {
        path: path.to_path_buf(),
        source,
    })?;
    traps.validate().map_err(|source| FixtureError::Trap {
        path: path.to_path_buf(),
        source,
    })?;

    // One read per distinct replay stream, not one per trap.
    let mut streams: std::collections::BTreeMap<String, std::collections::BTreeSet<uuid::Uuid>> =
        std::collections::BTreeMap::new();
    for trap in &traps.trap {
        if !streams.contains_key(&trap.replay) {
            let stream = read_jsonl(&fixture_path(&trap.replay)?)?;
            // `Uuid`, not `String`: comparing formatted text would couple correctness to
            // `Display` (hyphenation, case) on both sides, and allocate per observation.
            let ids: std::collections::BTreeSet<uuid::Uuid> =
                stream.iter().map(|o| o.obs_id.as_uuid()).collect();
            streams.insert(trap.replay.clone(), ids);
        }
        let known = &streams[&trap.replay];
        for obs_id in &trap.observations {
            if !known.contains(&obs_id.as_uuid()) {
                return Err(FixtureError::DanglingObservation {
                    path: path.to_path_buf(),
                    trap: trap.id.0.clone(),
                    obs_id: obs_id.as_uuid().to_string(),
                    replay: trap.replay.clone(),
                });
            }
        }
    }
    Ok(traps)
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

    // ── Trap files (story 4.2) ───────────────────────────────────────────────

    /// `obs_id` is the anchor the labelling format rests on: a trap points at one, "never by
    /// line number". Two lines sharing an id void that guarantee — a trap would silently judge
    /// whichever the reader happened to keep.
    #[test]
    fn a_stream_repeating_an_obs_id_is_refused() {
        let dir = scratch_dir("dup-obs");
        let path = dir.join("dup.jsonl");
        let line = serde_json::to_string(&expected()[0]).unwrap();
        std::fs::write(&path, format!("{line}\n{line}\n")).unwrap();

        let err = read_jsonl(&path).expect_err("a repeated obs_id must be refused");
        match &err {
            FixtureError::DuplicateObservationId {
                first_line,
                second_line,
                ..
            } => {
                assert_eq!((*first_line, *second_line), (1, 2));
            }
            other => panic!("expected a duplicate-id error, got {other:?}"),
        }
        assert!(err.to_string().contains("appears on lines"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Every trap file in the corpus must parse, validate and cross-check — DISCOVERED by
    /// WALKING, not by listing one directory. Trap FAMILIES (story 4.9 onward) are exactly what
    /// will introduce a subdirectory, and a non-recursive scan would hash them and never read
    /// them — reintroducing the hole this test exists to close.
    #[test]
    fn every_trap_file_in_the_corpus_is_valid() {
        let traps_dir = fixture_path("scenario/traps").unwrap();
        let mut checked = 0usize;
        let mut stack = vec![traps_dir.clone()];
        while let Some(dir) = stack.pop() {
            for entry in
                std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
            {
                let entry = entry.expect("a directory entry must be readable");
                let path = entry.path();
                // `file_type()` does not follow symlinks, so a link can neither smuggle a file
                // in nor be walked out of the corpus — but it must not pass unnoticed either.
                let file_type = entry.file_type().expect("file type");
                if file_type.is_symlink() {
                    panic!(
                        "{}: the corpus must contain its own bytes, not a symlink",
                        path.display()
                    );
                }
                if file_type.is_dir() {
                    stack.push(path);
                    continue;
                }
                // Case-insensitive: a `broken.TOML` skipped silently would be hashed by the
                // gate and read by nobody.
                let is_toml = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("toml"));
                assert!(
                    is_toml,
                    "{}: only .toml trap files belong under scenario/traps/",
                    path.display()
                );
                read_traps(&path).unwrap_or_else(|e| {
                    panic!("corpus trap file {} is invalid: {e}", path.display())
                });
                checked += 1;
            }
        }
        // A discovery test that finds nothing must not pass silently — the vacuity the fixtures
        // gate carried from Epic 1 until story 4.1 put a file on disk.
        assert!(checked > 0, "no trap file found in {}", traps_dir.display());
    }

    const EXAMPLE_TRAPS: &str = "scenario/traps/example.toml";

    /// The committed example must parse, validate, and point only at observations that exist.
    #[test]
    fn the_committed_trap_file_reads_and_cross_checks() {
        let traps = read_traps(&fixture_path(EXAMPLE_TRAPS).unwrap()).expect("the example reads");
        // Coverage, not order: reordering the `[[trap]]` blocks is a no-op and must stay one.
        let columns: std::collections::BTreeSet<&str> =
            traps.trap.iter().map(|t| t.expect.column()).collect();
        assert_eq!(
            columns,
            ["must-abstain", "must-merge", "must-not-merge"]
                .into_iter()
                .collect(),
            "the example must exercise all three of D18's columns"
        );
        // Every decision names a rule, and it is not blank — the premise of `(verdict, rule)`.
        for trap in &traps.trap {
            if let Some(rule) = trap.expect.rule() {
                assert!(
                    !rule.0.trim().is_empty(),
                    "trap {:?} names no rule",
                    trap.id
                );
            }
            assert!(!trap.reason.trim().is_empty());
        }
        // The traps span two streams: a trap names the stream it judges, and nothing assumes one.
        let streams: std::collections::BTreeSet<&str> =
            traps.trap.iter().map(|t| t.replay.as_str()).collect();
        assert_eq!(
            streams.len(),
            2,
            "the example must exercise more than one stream"
        );
    }

    fn write_traps(tag: &str, body: &str) -> PathBuf {
        let path = scratch_dir(tag).join("traps.toml");
        std::fs::write(&path, body).unwrap();
        path
    }

    /// The oracle rests on the author's reason. Its absence must stop the read, not warn.
    #[test]
    fn a_trap_without_a_reason_is_refused() {
        let path = write_traps(
            "no-reason",
            r#"
[[trap]]
id = "nameless"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001"]
reason = "   "
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );
        let err = read_traps(&path).expect_err("a reasonless trap must be refused");
        assert!(matches!(err, FixtureError::Trap { .. }), "{err:?}");
        let rendered = err.to_string();
        assert!(
            rendered.contains("nameless"),
            "must name the trap: {rendered}"
        );
        // Assert the CAUSE, not a constant of the message: an earlier version checked for the
        // literal "must-abstain", which every reason error contains and which therefore could not
        // tell a correct message from a wrong one.
        assert!(
            matches!(
                err,
                FixtureError::Trap {
                    source: opencmdb_core::trap::TrapError::ReasonMissing { .. },
                    ..
                }
            ),
            "an empty reason is ReasonMissing, not another reason error: {err:?}"
        );
    }

    /// An ABSENT `reason` key must be refused by a message that names the TRAP — not by serde,
    /// which can only name the field. That is why `reason` carries `#[serde(default)]`.
    #[test]
    fn a_trap_whose_reason_key_is_absent_names_the_trap() {
        let path = write_traps(
            "absent-reason",
            r#"
[[trap]]
id = "keyless"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001"]
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );
        let err = read_traps(&path).expect_err("an absent reason must be refused");
        assert!(matches!(err, FixtureError::Trap { .. }), "{err:?}");
        assert!(
            err.to_string().contains("keyless"),
            "must name the trap, not just the field: {err}"
        );
    }

    /// A `./` prefix names the same file under a spelling the MANIFEST never records.
    #[test]
    fn a_dot_slash_replay_is_refused() {
        assert!(matches!(
            fixture_path("./scenario/replay/minimal.jsonl")
                .expect_err("a `./` prefix must be refused"),
            FixtureError::OutsideCorpus { .. }
        ));
    }

    /// A misspelled field must fail loudly rather than be ignored — the rule story 4.1
    /// established for observations, applied to the labelling.
    #[test]
    fn an_unknown_field_in_a_trap_file_is_refused() {
        let path = write_traps(
            "unknown-field",
            r#"
[[trap]]
id = "typo"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001"]
reason = "a reason long enough to state something about this trap"
resaon = "the misspelling that motivates deny_unknown_fields"
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );
        let err = read_traps(&path).expect_err("an unknown field must be refused");
        assert!(matches!(err, FixtureError::Toml { .. }), "{err:?}");
        let rendered = err.to_string();
        assert!(
            rendered.contains("resaon"),
            "must name the field: {rendered}"
        );
        assert!(
            rendered.contains("trap file"),
            "must say what failed: {rendered}"
        );
    }

    /// A decision carrying an abstention cause is not merely invalid, it is unrepresentable —
    /// so the parse fails rather than a validator catching it later.
    #[test]
    fn a_decision_carrying_an_abstention_cause_is_refused() {
        let path = write_traps(
            "both",
            r#"
[[trap]]
id = "confused"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001"]
reason = "a reason long enough to state something about this trap"
expect = { must-merge = { rule = "r", cause = "NoObservedValue" } }
"#,
        );
        let err = read_traps(&path).expect_err("a column must carry only its own payload");
        assert!(matches!(err, FixtureError::Toml { .. }), "{err:?}");
        assert!(
            err.to_string().contains("cause"),
            "must name the offending key: {err}"
        );
    }

    /// A trap pointing at an observation its stream does not contain can never fire, and would
    /// sit in the corpus looking like coverage. The gate counts traps.
    #[test]
    fn a_trap_judging_an_absent_observation_is_refused() {
        let path = write_traps(
            "dangling",
            r#"
[[trap]]
id = "points-at-nothing"
replay = "scenario/replay/minimal.jsonl"
observations = ["ffffffff-0000-4000-8000-00000000dead"]
reason = "this observation is deliberately absent from the stream it names"
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );
        let err = read_traps(&path).expect_err("a dangling reference must be refused");
        match &err {
            FixtureError::DanglingObservation { trap, replay, .. } => {
                assert_eq!(trap, "points-at-nothing");
                assert_eq!(replay, "scenario/replay/minimal.jsonl");
            }
            other => panic!("expected a dangling-observation error, got {other:?}"),
        }
        assert!(err.to_string().contains("does not contain"), "{err}");
    }

    /// A trap file may not reach outside the corpus through its `replay` field either.
    #[test]
    fn a_trap_replaying_outside_the_corpus_is_refused() {
        let path = write_traps(
            "escape",
            r#"
[[trap]]
id = "escapes"
replay = "../../etc/passwd"
observations = ["aaaaaaaa-0000-4000-8000-000000000001"]
reason = "a reason long enough to state something about this trap"
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );
        assert!(matches!(
            read_traps(&path).expect_err("a replay path leaving the corpus must be refused"),
            FixtureError::OutsideCorpus { .. }
        ));
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
