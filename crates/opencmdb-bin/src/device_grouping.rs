//! Which records are ONE device — story 6.14c: a machine the operator called one is shown as one.
//!
//! # What groups, and what never does — Guy's arbitrations of 2026-09-26 (story 6.14c §0.8, §0.10)
//!
//! - **COMPUTED, never stored (A).** The grouping is derived at read time from the operator's current
//!   OPERATOR `match` rows in `l2_pair_decision` — the answers story 6.14b writes, which the engine
//!   never supersedes. No `device` row, no membership table, no migration, no writer. ⚠️ This DIVERGES
//!   from story 6.12's *"a device is a record rather than a computation repeated at each page load"* and
//!   from D15 case A, which assume a stored membership; Guy took it knowing both, and the divergence is
//!   registered to the first consumer needing a stable device id (Epic 15, FR26). The `match` rows stay
//!   the source whatever is stored later.
//! - **A record reaches its interface EXACTLY (B).** By its declared `mac`, compared with
//!   `interface.mac_canon` in Rust (D10 — the two columns carry different collations); without one, by its
//!   ORIGIN observation (`declared_attribute.origin_obs_id`) and that observation's current `match` link.
//!   🔴 **Never by its address**: the validation MEASURED the address path merging a laptop and a phone
//!   that held one address a DHCP lease apart — the product refuses a merge without proof, and the
//!   inventory would have drawn one from a reused address.
//! - **Only an answer joins two interfaces (C).** Records reaching ONE interface are one device — that is
//!   L1 identity. A record reaching SEVERAL interfaces (a MAC seen in two `l2_domain`s, or the registered
//!   interface-mint race) never joins them: it is attached to the FIRST, by interface id.
//!
//! # 🔴 The one provenance read in this module
//!
//! `origin_obs_id` is a provenance column (FR13): the `authorship` gate forbids reading it anywhere a
//! divergence is computed. [`load_record_origin_interfaces`] reads it to DISPLAY a grouping and nothing
//! else, and the gate sanctions it by path AND function name — story 6b.4's precedent for `origin`.

use std::collections::{BTreeMap, BTreeSet};

use sqlx::MySqlPool;

use crate::repo::OPEN_END;

/// What the grouping reads from the store.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct GroupingInput {
    /// `(interface_id, mac_canon, last_seen_at)` for every interface, `last_seen_at` rendered
    /// `%Y-%m-%d %H:%M:%S.%f`.
    pub(crate) interfaces: Vec<(String, String, String)>,
    /// `(entity_id, interface_id)` — the interface(s) each record's ORIGIN observation is currently
    /// placed on.
    pub(crate) origins: Vec<(String, String)>,
    /// `(interface_low, interface_high)` — the current OPERATOR `match` rows ONLY. ⚠️ The reader they come
    /// from returns `no_match` rows too; feeding those to the union-find joined two machines the operator
    /// called distinct, and the validation's mutation of exactly that stayed green.
    pub(crate) matches: Vec<(String, String)>,
}

/// Read what the grouping needs, in three statements.
///
/// # Errors
///
/// Any database error.
pub(crate) async fn load_grouping(pool: &MySqlPool) -> Result<GroupingInput, sqlx::Error> {
    let interfaces: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id, mac_canon, DATE_FORMAT(last_seen_at, '%Y-%m-%d %H:%i:%s.%f') FROM interface",
    )
    .fetch_all(pool)
    .await?;
    let origins = load_record_origin_interfaces(pool).await?;
    let matches = crate::l2_repo::load_current_operator_answers(pool)
        .await?
        .into_iter()
        .filter(|(_, _, outcome, ..)| outcome == "match")
        .map(|(low, high, ..)| (low, high))
        .collect();
    Ok(GroupingInput {
        interfaces,
        origins,
        matches,
    })
}

/// For every ADOPTED record, the interface(s) its origin observation is currently placed on.
///
/// 🔴 **The one provenance read**, sanctioned by the `authorship` gate by path and by THIS name: it
/// feeds the inventory's display of a grouping, and never a divergence (FR13). One row per record and
/// placement — an origin observation carrying two MACs is placed on two interfaces.
///
/// # Errors
///
/// Any database error.
pub(crate) async fn load_record_origin_interfaces(
    pool: &MySqlPool,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT DISTINCT d.entity_id, l.interface_id FROM declared_attribute d \
         JOIN identity_link l ON l.observation_id = d.origin_obs_id \
         WHERE d.origin_obs_id IS NOT NULL AND l.outcome = 'match' AND l.valid_to = ? \
         AND l.interface_id IS NOT NULL",
    )
    .bind(OPEN_END)
    .fetch_all(pool)
    .await
}

/// One record as the grouping sees it: its id and its declared `mac`, if any.
pub(crate) type Record<'a> = (&'a str, Option<&'a str>);

/// What [`group`] returns: the devices (each a list of record ids, in the order given), and for each
/// device the newest `last_seen_at` of its interfaces — `None` when none of its records reached one.
pub(crate) type Devices = Vec<(Vec<String>, Option<String>)>;

/// PURE: the devices the records form.
///
/// Each record reaches at most ONE interface — its declared MAC's, else its origin observation's; when
/// several match, the smallest id (C: a record never bridges interfaces). The interfaces are joined by
/// the operator's `match` rows; records on one component are one device. Devices come in the order of
/// their first record; a record reaching nothing is a device of its own.
pub(crate) fn group(records: &[Record<'_>], input: &GroupingInput) -> Devices {
    let mut by_mac: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for (id, mac, last_seen) in &input.interfaces {
        by_mac.entry(mac.as_str()).or_default().insert(id.as_str());
        seen.insert(id.as_str(), last_seen.as_str());
    }
    let mut by_origin: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (entity, interface) in &input.origins {
        by_origin
            .entry(entity.as_str())
            .or_default()
            .insert(interface.as_str());
    }
    let reached = |(id, mac): &Record<'_>| -> Option<&str> {
        let candidates = match mac {
            Some(mac) => by_mac.get(mac),
            None => by_origin.get(id),
        };
        candidates.and_then(|set| set.iter().next().copied())
    };

    // Union-find over interfaces, joined by the operator's answers ONLY.
    let mut parent: BTreeMap<String, String> = BTreeMap::new();
    fn root(parent: &mut BTreeMap<String, String>, id: &str) -> String {
        let mut current = id.to_string();
        parent
            .entry(current.clone())
            .or_insert_with(|| current.clone());
        while let Some(next) = parent.get(&current).filter(|n| **n != current).cloned() {
            current = next;
        }
        current
    }
    for (low, high) in &input.matches {
        let (a, b) = (root(&mut parent, low), root(&mut parent, high));
        let (keep, fold) = if a <= b { (a, b) } else { (b, a) };
        parent.insert(fold, keep);
    }

    // Devices in the order of their first record; `key` is the component root, or the record itself.
    let mut order: Vec<String> = Vec::new();
    let mut members: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut interfaces_of: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for record in records {
        let key = match reached(record) {
            Some(interface) => {
                let r = root(&mut parent, interface);
                interfaces_of.entry(r.clone()).or_default();
                format!("i:{r}")
            }
            None => format!("r:{}", record.0),
        };
        if !members.contains_key(&key) {
            order.push(key.clone());
        }
        members.entry(key).or_default().push(record.0.to_string());
    }
    // Every interface of a component, for the device's last sighting.
    let ids: Vec<String> = seen.keys().map(|id| id.to_string()).collect();
    for id in ids {
        let r = root(&mut parent, &id);
        if let Some(set) = interfaces_of.get_mut(&r) {
            set.insert(id);
        }
    }
    order
        .into_iter()
        .map(|key| {
            let newest = key.strip_prefix("i:").and_then(|r| {
                interfaces_of
                    .get(r)
                    .and_then(|set| set.iter().filter_map(|i| seen.get(i.as_str())).max())
                    .map(|s| s.to_string())
            });
            (members.remove(&key).unwrap_or_default(), newest)
        })
        .collect()
}

/// Tests for the pure grouping; the route and the store are tested in `page.rs` and below.
#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        interfaces: &[(&str, &str, &str)],
        origins: &[(&str, &str)],
        matches: &[(&str, &str)],
    ) -> GroupingInput {
        GroupingInput {
            interfaces: interfaces
                .iter()
                .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
                .collect(),
            origins: origins
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
            matches: matches
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
        }
    }

    const T1: &str = "2026-09-26 10:00:00.000000";
    const T2: &str = "2026-09-26 11:00:00.000000";

    /// `obelix`: two records, two interfaces, answered *the same machine* — one device, seen at the newer
    /// of its two interfaces. By MAC for one record and by origin for the other, so both paths count.
    #[test]
    fn two_records_joined_by_an_answer_are_one_device() {
        let g = input(
            &[("i1", "aa", T1), ("i2", "bb", T2)],
            &[("r2", "i2")],
            &[("i1", "i2")],
        );
        let devices = group(&[("r1", Some("aa")), ("r2", None)], &g);
        assert_eq!(
            devices,
            vec![(
                vec!["r1".to_string(), "r2".to_string()],
                Some(T2.to_string())
            )]
        );
    }

    /// No answer, or none of these: two devices. Records on ONE interface are one device (L1).
    #[test]
    fn without_an_answer_only_one_interface_groups() {
        let g = input(&[("i1", "aa", T1), ("i2", "bb", T1)], &[], &[]);
        assert_eq!(
            group(&[("r1", Some("aa")), ("r2", Some("bb"))], &g).len(),
            2
        );
        let same_nic = group(&[("r1", Some("aa")), ("r2", Some("aa"))], &g);
        assert_eq!(same_nic.len(), 1, "one NIC documented twice is one device");
    }

    /// A record reaching two interfaces never joins them (C): it attaches to the first, and a record on
    /// the second stays apart.
    #[test]
    fn a_record_on_two_interfaces_does_not_bridge_them() {
        let g = input(&[("i1", "aa", T1), ("i2", "aa", T1)], &[("r2", "i2")], &[]);
        let devices = group(&[("r1", Some("aa")), ("r2", None)], &g);
        assert_eq!(devices.len(), 2, "{devices:?}");
    }

    /// The MAC wins over the origin when both are there, and a record reaching nothing stands alone
    /// with no sighting.
    #[test]
    fn the_mac_wins_and_a_record_reaching_nothing_stands_alone() {
        let g = input(&[("i1", "aa", T1), ("i2", "bb", T1)], &[("r1", "i2")], &[]);
        let devices = group(&[("r1", Some("aa")), ("r2", Some("bb")), ("r3", None)], &g);
        assert_eq!(
            devices.len(),
            3,
            "r1 goes by its MAC to i1, not by its origin to i2"
        );
        assert_eq!(devices[2], (vec!["r3".to_string()], None));
    }

    // ---- through the ROUTE, against a store ----
    //
    // 🔴 The validation measured the whole suite and ten gates GREEN over a prototype whose grouping was
    // wrong (a *distinct* answer fed to the union-find) and whose header was a hard-coded English string:
    // nothing read `GET /devices`. These tests read the served page, with the real documenting gesture
    // and the real answer.

    use crate::document::{DocumentPort, StoreDocument};
    use crate::l2_answer::Answer;
    use crate::l2_answer::tests::fixture::{at_address, at_micros, counted_sweep, sighting};
    use axum::body::Body;
    use axum::http::Request;
    use opencmdb_core::observation::ObsId;
    use tower::ServiceExt;

    /// A migrated store with no identity, answer or record left in it.
    async fn store() -> Option<MySqlPool> {
        let pool = crate::l2_answer::tests::fixture::store().await?;
        sqlx::query("DELETE FROM declared_attribute")
            .execute(&pool)
            .await
            .expect("clean");
        Some(pool)
    }

    /// Document one sighting through the real gesture.
    async fn document(pool: &MySqlPool, id: u128) {
        StoreDocument::new(pool.clone())
            .document_all(ObsId::from_uuid(uuid::Uuid::from_u128(0x6140_0000 + id)))
            .await
            .unwrap_or_else(|_| panic!("documenting sighting {id}"));
    }

    /// Answer the store's one open question through the real adapter.
    async fn answer(pool: &MySqlPool, answer: Answer, micros: i64) {
        let members: BTreeSet<String> = sqlx::query_scalar("SELECT id FROM interface")
            .fetch_all(pool)
            .await
            .expect("read")
            .into_iter()
            .collect();
        let mut conn = pool.acquire().await.expect("connection");
        let mut tx = sqlx::Connection::begin(&mut *conn).await.expect("tx");
        crate::l2_answer::record_operator_answer(&mut tx, &members, answer, at_micros(micros))
            .await
            .expect("the answer");
        tx.commit().await.expect("commit");
    }

    /// `GET /devices`, served — its body, its device-row count and its header.
    async fn devices_page(pool: &MySqlPool) -> (String, usize) {
        let router = crate::page::triage_router(
            pool.clone(),
            None,
            None,
            crate::diagnostic::DiagnosticFacts::new(
                crate::diagnostic::LogDescriptor {
                    directives: "info".to_string(),
                    file: None,
                },
                crate::diagnostic::security_posture(false, false),
                crate::diagnostic::ScanReportSlot::default(),
            ),
            true,
        );
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/devices")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("the route answers");
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let html = String::from_utf8(bytes.to_vec()).expect("utf-8");
        let rows = html.matches("<tr data-entity-id=").count();
        (html, rows)
    }

    /// AC1, AC4: `obelix` documented twice and answered *the same machine* is ONE row naming both
    /// addresses and *2 records*, under a header naming 1 device and 2 records; answered *distinct*, two
    /// rows — the CONTROL without which a union-find fed every answer would pass.
    #[tokio::test]
    async fn obelix_answered_the_same_machine_is_one_row_and_distinct_is_two() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        for (answered, rows_expected) in [(Answer::SameMachine, 1), (Answer::DistinctMachines, 2)] {
            let Some(pool) = store().await else { return };
            let t = 1_790_000_000_000_100;
            counted_sweep(
                &pool,
                vec![
                    at_address(sighting(1, 1, Some("obelix"), t), "192.0.2.10"),
                    at_address(sighting(2, 2, Some("obelix"), t), "192.0.2.11"),
                ],
            )
            .await;
            document(&pool, 1).await;
            document(&pool, 2).await;
            let (_, before) = devices_page(&pool).await;
            assert_eq!(before, 2, "premise: two records, no answer yet");
            answer(&pool, answered, t).await;
            let (html, rows) = devices_page(&pool).await;
            assert_eq!(rows, rows_expected, "{answered:?}");
            let header = rust_i18n::t!(
                "inventory.total",
                devices = crate::page::counted_fields("inventory.n_devices", rows_expected),
                records = crate::page::counted_fields("inventory.n_entities", 2)
            )
            .to_string();
            assert!(
                html.contains(&header),
                "the header names both units: {header}"
            );
            if rows_expected == 1 {
                assert!(
                    html.contains("192.0.2.10, 192.0.2.11")
                        || html.contains("192.0.2.11, 192.0.2.10")
                );
                // 🔑 The CELL, not the page: the header carries *2 records* too, so a bare `contains`
                // was satisfied by the header whatever the row said (found before the mutation pass).
                assert!(html.contains(&format!(
                    "<td>{}</td>",
                    crate::page::counted_fields("inventory.n_entities", 2)
                )));
            }
        }
    }

    /// AC1, C: one network card documented twice — at two addresses, no answer — is one device.
    #[tokio::test]
    async fn one_card_documented_at_two_addresses_is_one_row_without_an_answer() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let t = 1_790_000_000_000_200;
        counted_sweep(
            &pool,
            vec![at_address(sighting(1, 1, Some("gamma"), t), "192.0.2.40")],
        )
        .await;
        counted_sweep(
            &pool,
            vec![at_address(
                sighting(2, 1, Some("gamma"), t + 1),
                "192.0.2.41",
            )],
        )
        .await;
        document(&pool, 1).await;
        document(&pool, 2).await;
        assert_eq!(devices_page(&pool).await.1, 1);
    }

    /// AC2, B: 🔴 the DHCP case the validation MEASURED merging two machines by address. A laptop's record
    /// that carries NO declared MAC reaches its interface by its ORIGIN observation; a phone later holding
    /// the same address is another device. Two rows.
    #[tokio::test]
    async fn a_reused_address_does_not_merge_two_machines() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let t = 1_790_000_000_000_300;
        counted_sweep(
            &pool,
            vec![at_address(sighting(1, 1, Some("laptop"), t), "192.0.2.20")],
        )
        .await;
        document(&pool, 1).await;
        // A record documented before PR #163 carried no MAC: modelled by removing it, which only a test
        // may do — the adapter never deletes a declared value.
        sqlx::query("DELETE FROM declared_attribute WHERE attr_key = 'mac'")
            .execute(&pool)
            .await
            .expect("a pre-#163 record");
        counted_sweep(
            &pool,
            vec![at_address(
                sighting(2, 2, Some("phone"), t + 3_600_000_000),
                "192.0.2.20",
            )],
        )
        .await;
        document(&pool, 2).await;
        let (html, rows) = devices_page(&pool).await;
        assert_eq!(rows, 2, "a reused address is not a proof: {html}");
        assert!(!html.contains("laptop, phone") && !html.contains("phone, laptop"));
    }

    /// ⚠️ **The limit B buys, pinned rather than hidden**: a record documented BEFORE PR #163 came from an
    /// observation with no MAC — which the L1 pass places on no interface — so neither its MAC nor its
    /// origin reaches one, and it stays a device of its own even after *the same machine*. The address
    /// path would have joined it, and was refused because it merged two machines by DHCP (§0.9(B)).
    /// If `obelix`'s records on the NAS predate 2026-09-10, `obelix` stays two rows; registered.
    #[tokio::test]
    async fn a_record_from_a_mac_less_sighting_stands_alone_even_after_an_answer() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let t = 1_790_000_000_000_400;
        // The old, MAC-less sightings: recorded, never placed.
        let mut old = at_address(sighting(1, 1, Some("obelix"), t), "192.0.2.10");
        old.facts
            .retain(|f| !matches!(f, opencmdb_core::observation::Fact::Mac { .. }));
        counted_sweep(&pool, vec![old]).await;
        document(&pool, 1).await;
        // Today's sweep: two cards with MACs, one question, answered.
        counted_sweep(
            &pool,
            vec![
                at_address(sighting(11, 1, Some("obelix"), t + 1), "192.0.2.10"),
                at_address(sighting(12, 2, Some("obelix"), t + 1), "192.0.2.11"),
            ],
        )
        .await;
        document(&pool, 12).await;
        answer(&pool, Answer::SameMachine, t + 1).await;
        assert_eq!(
            devices_page(&pool).await.1,
            2,
            "the MAC-less record reaches no interface, so the answer cannot reach it"
        );
    }

    /// AC6: the build is linear — 300 records against 20 000 observations and 300 interfaces. 🔴 The
    /// validation's naive address path measured **50.7 s** at 18 000 observations, in the synchronous
    /// build `PAGE_STORE_BUDGET` does not wrap. ⚠️ Timed on the PURE build, where that cost lived; the reads
    /// are linear and inside the budget.
    #[test]
    fn the_build_stays_linear_at_reference_scale() {
        use crate::repo::ObservedBatch;
        use opencmdb_core::observation::{ConnectorId, Fact};
        let now = chrono::DateTime::from_timestamp(2_000_000_000, 0).expect("in range");
        let mut declared = Vec::new();
        let mut interfaces = Vec::new();
        for n in 0..300u32 {
            let entity = format!("e{n:04}");
            let mac = format!("00:00:00:00:{:02x}:{:02x}", n / 256, n % 256);
            declared.push((
                entity.clone(),
                "ipv4".to_string(),
                format!("10.0.{}.{}", n / 256, n % 256),
            ));
            declared.push((entity, "mac".to_string(), mac.clone()));
            interfaces.push((
                format!("i{n:04}"),
                mac,
                "2026-09-26 10:00:00.000000".to_string(),
            ));
        }
        let observations: Vec<ObservedBatch> = (0..20_000u32)
            .map(|n| ObservedBatch {
                id: ObsId::from_uuid(uuid::Uuid::from_u128(u128::from(n) + 1)),
                connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()).to_string(),
                observed_at: now,
                facts: vec![Fact::IpV4 {
                    addr: format!("10.0.{}.{}", (n % 300) / 256, (n % 300) % 256)
                        .parse()
                        .expect("address"),
                }],
            })
            .collect();
        let grouping = GroupingInput {
            interfaces,
            origins: Vec::new(),
            matches: (0..150u32)
                .map(|n| (format!("i{:04}", 2 * n), format!("i{:04}", 2 * n + 1)))
                .collect(),
        };
        let started = std::time::Instant::now();
        let view =
            crate::inventory_view::build_inventory(declared, &[], &observations, &grouping, now);
        let elapsed = started.elapsed();
        assert_eq!(view.rows.len(), 150, "150 answered pairs are 150 devices");
        assert!(
            elapsed < std::time::Duration::from_secs(1),
            "the build took {elapsed:?} — it must stay linear"
        );
    }

    /// §0.9(G): a device's LAST SEEN comes from its INTERFACES, never from its records' address — the
    /// validation measured a laptop's row showing the phone that later held its address.
    #[test]
    fn a_devices_last_sighting_is_its_interfaces_not_its_address() {
        use crate::repo::ObservedBatch;
        use opencmdb_core::observation::{ConnectorId, Fact};
        let now = chrono::DateTime::from_timestamp(2_000_000_000, 0).expect("in range");
        let declared = vec![
            (
                "r1".to_string(),
                "ipv4".to_string(),
                "192.0.2.20".to_string(),
            ),
            ("r1".to_string(), "mac".to_string(), "aa".to_string()),
        ];
        // The ADDRESS was seen a minute ago (another machine holds it now) …
        let observations = vec![ObservedBatch {
            id: ObsId::from_uuid(uuid::Uuid::from_u128(1)),
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()).to_string(),
            observed_at: now - chrono::Duration::minutes(1),
            facts: vec![Fact::IpV4 {
                addr: "192.0.2.20".parse().expect("address"),
            }],
        }];
        // … while the record's own card was last seen ten days ago.
        let card = (now - chrono::Duration::days(10))
            .format("%Y-%m-%d %H:%M:%S%.6f")
            .to_string();
        let grouping = input(&[("i1", "aa", card.as_str())], &[], &[]);
        let view =
            crate::inventory_view::build_inventory(declared, &[], &observations, &grouping, now);
        assert_eq!(
            view.rows[0].seen,
            crate::page::relative_time(now, now - chrono::Duration::days(10)),
            "the card's sighting, not the address's"
        );
    }

    /// E1 (Guy, 2026-09-26): a chain of answers groups — A=B and B=C make one device even if the engine
    /// judged A–C distinct. ⚠️ Unreachable with today's rules; pinned so the day a rule makes it
    /// reachable, this is what changes.
    #[test]
    fn a_chain_of_answers_groups_whatever_the_engine_said_about_its_ends() {
        let g = input(
            &[("a", "aa", T1), ("b", "bb", T1), ("c", "cc", T1)],
            &[],
            &[("a", "b"), ("b", "c")],
        );
        let devices = group(&[("r1", Some("aa")), ("r3", Some("cc"))], &g);
        assert_eq!(devices.len(), 1);
    }
}
