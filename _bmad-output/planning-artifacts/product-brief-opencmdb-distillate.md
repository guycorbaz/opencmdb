---
title: "Product Brief Distillate: opencmdb"
type: llm-distillate
source: "product-brief-opencmdb.md"
created: "2026-07-15"
purpose: "Token-efficient context for downstream PRD creation"
---

# opencmdb — Detail Pack for PRD

Self-hosted, single-binary (Rust) tool unifying lightweight IPAM + light application CMDB + network topology. Core organizing idea: **continuously compare OBSERVED state (auto-discovered) vs DECLARED state (user-documented); the gap is the product.** Deploy target: Synology NAS via Docker. Built solo, AI-assisted (Claude Code), open-source, GitHub-hosted + Docker Hub distribution.

## Firm decisions (non-negotiable — do NOT reopen in PRD)

- **Language: Rust.** Single binary. (Performance, reliability, author preference.)
- **DB: MariaDB 10.11+, THE ONLY SUPPORTED ENGINE** — chosen over PostgreSQL specifically because Synology provides MariaDB natively AND includes it in Synology's automatic backups (PostgreSQL is not natively backed up on Synology). CI is pinned to `mariadb:10.11.11`, the exact DSM 7 package: **dev = CI = prod.**
- **SQLite: NOT supported. MySQL: NOT supported.** _(D64, 2026-07-17.)_ SQLite is a **refusal, not a deferral** — "SQLite later" is banned in writing. MySQL is a different product from MariaDB and **we do not claim what has no CI**. The motive on record is **not** "most users will choose MariaDB" (that is D59's error on the storage axis) but **"no target user is blocked by MariaDB"**.
- **PostgreSQL: NOT supported at MVP.** Re-opened as a *possible* future addition — **and it is not free**: with no second engine, nothing forces the repository trait to stay dialect-neutral, so the trait gets audited BEFORE any such port, not during.
- **Deployment: Docker container** (Synology Container Manager) is the priority target; native binary also possible. Not Synology-exclusive.
- **UniFi connector is first-class from day one** — author and much of the target audience run UniFi; the local API gives clients, switch ports, SSIDs, VLANs, DHCP leases over plain outbound HTTP with **zero network privilege**.
- **Docs are English-only** (README, GitHub Pages, LaTeX manuals) — GitHub = English-dominant audience.
- **Application UI is bilingual EN + FR** at MVP (author is francophone; FR is his own test case). Externalize strings from day 1.
- **Distribution channel: Docker Hub** — **a two-service compose (opencmdb + your MariaDB), never a `docker run` one-liner.** _(D64 condition 2: the one-liner has not existed since D27 either — the KEK lives in a separate shared folder and its absence is a startup refusal.)_
- **`/metrics` and all HTTP surfaces are authenticated** (scrape token for `/metrics`) — chosen over unauthenticated-on-trusted-LAN.

## Resolved open questions (were §2.3 of the initial brief)

- **Q1 CSV import → v2.** Author starts on a fresh install to dogfood all tools from scratch; UniFi + scan already bootstrap inventory, so CSV is a safety net, not a pillar.
- **Q2 Scan/IPAM → MVP pillar.** Active scan is core (IP management is the author's central pain). Priority pains, in order: ① find a free IP in a subnet; ② resolve conflicts WITH device identification (UniFi alarms an IP conflict but doesn't say which physical device — opencmdb answers "which device" via MAC→device→IP); ③ know what occupies a given IP.
- **Q3 IPv6 → document at MVP, actively scan at v2.** Document IPv6 subnets/addresses + observe what UniFi/NDP reports; no active IPv6 sweep (address space too large, no ARP). Not indispensable for MVP.
- **Q4 Multi-user → single admin at MVP; schema must be multi-user-ready from day 1.** Read-only role deferred to v2, but the data model + auth must not block later multi-user (User table, ownership/creator notion on entities).
- **Q5 i18n → bilingual EN/FR UI at MVP; docs English-only.** (See firm decisions.)
- **Q6 Historisation → two-tier model.** Raw `Observation` records (per scan hit) with configurable retention, **default 90 days**, purged beyond. Derived rollups (per-MAC first_seen/last_seen, IP↔MAC history, presence timeline) retained **indefinitely** (cheap; powers "IP not seen in N days" + conflict detection). One config knob = raw-journal retention. **Declared-change audit log → v2.**

## Key product mechanic (author-proposed, high value)

- **One-click "document"** (EN) / « Merger » (FR UI) in the triage inbox: promote a discovered device into declared documentation and set it as the **baseline**. Triage inbox moves: **document / accept-gap / create new / attach to existing / exclude** (all remembered). _(Vocabulary corrected 2026-07-17 — F59: this line carried `accept-as-declared` and `ignore`, both RETIRED 2026-07-16 and both on the denylist. **You document a VALUE; you accept a GAP** — `accept` named two opposite gestures. `accept-gap` = "I have seen the divergence and have not yet decided" — the gap stays OPEN, and it can later lead to document.)_
- This is the **near-zero-effort on-ramp to declaration** — directly mitigates the biggest product risk (users scanning but never declaring). Establishes the baseline against which **future drift is detected and analyzed over time**, not just first-sighting.

## Requirements hints (for PRD elaboration)

- **Discovery scheduling**: configurable interval + on-demand manual scan.
- **Generic scanner**: ARP scan on local segment + ping sweep for routed nets; hostname enrichment via mDNS/reverse DNS. MVP impl may shell out to `nmap -sn`; native Rust (`pnet`) is a later option. NOTE: ARP scan in a container needs `network_mode: host` or `NET_RAW` capability — must be documented; UniFi path needs no privilege (recommended when available).
- **Reconciliation matching**: primary key = MAC; secondary heuristics = hostname, IP/DHCP history. Handle MAC randomization (phones), multi-interface devices, VMs/containers sharing hardware. Systematic timestamping (first_seen, last_seen, per-MAC IP history).
- **Alerts (minimal set)**: new unknown device (with VLAN/segment); documented IP unseen for N days; potential conflict (same IP on two MACs). Delivery: in-app + generic webhook. The "unknown device" signal doubles as rogue-device/shadow-IT security tripwire.
- **Applications/software model**: Software = a program instance on a device (name, version, listening port(s), host device). Application = logical group of Software (owner, criticality, notes). Typed relations between Software: `connects_to`, `depends_on`, `hosts` (device→software), `exposes` (reverse proxy→service). Impact view: "if this device dies, which apps are affected?"
- **Topology at MVP**: structured list/table only (auto-populated for UniFi: client↔switch-port, client↔AP/SSID; manual elsewhere: device-A-port ↔ device-B-port). Graphical interactive map explicitly v2.
- **API**: read-only JSON API on core entities at MVP (best-effort, unversioned). Stable versioned + OpenAPI-documented + write access → v2.
- **Prometheus**: expose `/metrics` (opencmdb-specific gauges: device count, IPs used/free per subnet, triage-inbox size, active alerts, devices unseen N days, conflicts, last-scan timestamp/duration). Consuming Prometheus data for reverse enrichment → v2.
- **Auth**: local login/password + session at MVP. No SSO/LDAP. Credentials encrypted at rest.
- **Config**: file + env vars (12-factor for container).
- **Background tasks**: internal scheduler (tokio) for scans + UniFi polling; no external cron/redis.

## Data model vocabulary (from initial brief §4 — refine in architecture)

Entities: `Site` (optional MVP), `Device`, `Interface` (holds MAC; physical/virtual), `Subnet` (CIDR, VLAN, DHCP range), `Vlan`, `IpAddress` (static-declared / DHCP-observed / reservation), `Ssid`, `Link` (declared | observed(unifi)), `Software`, `Application`, `Relation` (typed edge), `Observation` (timestamped, source: unifi|arp_scan), `ReconciliationDecision` (triage memory), `Alert` (state: new/seen/resolved/ignored). **Structuring principle (firm): systematic separation declared (hand-entered) vs observed (source-derived, timestamped, never hand-edited). Reconciliation links, never merges.**

## Key user journeys (from initial brief §5 — detail in PRD)

1. **First run**: setup wizard → connect UniFi controller (URL + API key) OR declare a subnet to scan → first scan → pre-filled triage inbox → user names their ~10 most important devices. Target: visible value in <15 min.
2. **Weekly routine**: check inbox (new devices), handle alerts.
3. **Document an application**: create "Paperless", attach software (Paperless-ngx, PostgreSQL, Redis), declare relations, see impact view.
4. **Diagnostic**: "what is this IP?" → search → full record (device, history, switch port, hosted apps). Includes resolving "which device owns this conflicting IP" (the UniFi-alarm pain).
5. **Migration prep**: impact view of a device before powering it off.

## Technical directions to VALIDATE in architecture (not yet decided)

- **Data access**: ~~SeaORM for dialect abstraction~~ — **DECIDED, and the opposite way: raw `sqlx =0.9.0`, disciplined portable SQL, no re-abstraction. SeaORM/Diesel and `sqlx::Any` are BANNED; `sqlx::query!` is banned.** With D64 there is no dialect to abstract at all. _(Superseded by the architecture — this line is kept for provenance, not as an option.)_
- **Web server**: Axum (de-facto standard) leaning candidate.
- **Frontend**: two options for Architect — (a) HTMX + templates (Askama/Maud): full-Rust, single binary, simplest deploy, well-suited to an admin tool; (b) SPA (Svelte/Vue) served static: needed if the v2 graphical map requires rich JS anyway — anticipate to avoid a rewrite. Both work with an API-first backend.

## Competitive intelligence (preserve — see competitive-analysis.md for full table + sources)

- **The reconciliation wedge is nearly empty.** Only **NetBox Assurance** genuinely markets continuous "intended vs observed" drift — paywalled (Enterprise/Cloud), atop NetBox's heavy stack. No lightweight/self-hosted tool offers first-class declared-vs-observed for IPAM+CMDB. opencmdb's clearest wedge.
- **Market split into two non-overlapping camps**: source-of-truth (NetBox, Nautobot, iTop) don't auto-discover OOTB; discovery tools (NetAlertX, Netdisco, Angry IP Scanner, phpIPAM) don't model declared intent. Nobody unifies them for the mid-market.
- **Footprint is the incumbents' biggest adoption barrier**: NetBox/Nautobot need PostgreSQL + Redis + Celery/RQ workers + reverse proxy. Single-binary opencmdb counters this directly.
- **UniFi under-served**: only NetAlertX has native UniFi import, and only for presence alerts — not IPAM/CMDB/topology.
- **phpIPAM discovery is fragile** (binary/extension misconfig; marks reachable hosts offline) and shallow (IP up/down only) — easy bar to clear.
- Author's lived pain: used iTop, NetBox, phpIPAM (all too complex and/or missing IP discovery — NetBox discovery notably hard to set up); uses Angry IP Scanner (great scan, documents nothing). "Scanner ≠ documenter" is the founding insight.

## Rejected / explicitly deferred ideas (do NOT re-propose for MVP)

- Distinct product modes for SMB vs individual — **rejected**: needs are the same (individual often more complex than a small SMB). One product, one mode.
- Read-only role at MVP — deferred v2 (but schema stays multi-user-ready).
- FR-first / EN-later — rejected: bilingual EN/FR from MVP, docs EN-only.
- CSV import at MVP — deferred v2.
- Active IPv6 scanning — deferred v2.
- Stable versioned/documented public API + write access — deferred v2.
- Consuming Prometheus metrics (reverse enrichment) — deferred v2.
- Declared-change audit log — deferred v2.
- Config-as-code / GitOps declared state (YAML + drift) — raised as an opportunity, NOT adopted; possible future.
- Graphical interactive topology map — v2.
- SNMP/LLDP topology for non-UniFi gear — v2.
- Additional connectors (Omada, Mikrotik, OPNsense/pfSense, Proxmox, Docker) — v2 (design connector as a Rust trait / documented plug-in contract early so community can extend).
- Rich notifications (email, Telegram, ntfy) — v2 (generic webhook at MVP).
- Port/service scanning (nmap -sV) to suggest installed software — v2.

## Open items to carry into PRD/architecture

- **Reference environment (author's, = MVP test bench)**: a UniFi gateway + 100% UniFi (incl. NanoStations as point-to-point bridge), recent UniFi OS; Synology NAS with MariaDB + Container Manager; multi-building home/small network (house + garage bridged over 5 GHz), VLANs, self-hosted services (local AI stack, Paperless envisaged).
- **UniFi API fragility** across UniFi OS versions is a top risk — isolate behind a Rust trait, target API-key path, test current versions, degrade gracefully rather than break hard. Specify the exact auth method + minimum controller-version support matrix in architecture.
- **MAC randomization** is the central reconciliation UX challenge — triage inbox + remembered decisions + secondary heuristics are the mitigation.
- **Security threat model** to detail: credential-at-rest encryption, endpoint auth (incl. /metrics token), and that a full network map is itself sensitive read data.
- **Light market validation recommended** before the heaviest build: a few r/selfhosted / r/homelab conversations + concept post, to confirm target users will maintain declared state (not just the author, N=1).
- **MVP internal sequencing** recommended: prove reconciliation first (UniFi discovery → **document** → gap view), then layer generic scanner / applications / i18n. _(Superseded on three counts: **D1** — MariaDB is day-1, not a later layer · **D64** — it is the only engine · **D57-scope** — the impact view is Growth. F59: read `accept-as-declared`, retired 2026-07-16.)_

## Success criteria (measurable)

- Synology (Container Manager) install < 30 min, docs included.
- UniFi network: complete observed inventory (clients, ports, SSIDs), zero manual entry, one polling cycle.
- Non-UniFi: active local-segment devices discovered by scan.
- User documents a 3-tier application and gets its impact view.
- Observed/declared gap visible + actionable (inbox + alerts), incl. "which device owns this conflicting IP".
- The invariant suite passes on `mariadb:10.11.11` (the DSM 7 package) on every PR. _(D64: the dual-backend criterion is retired.)_
- First visible value < 15 min from first launch.
