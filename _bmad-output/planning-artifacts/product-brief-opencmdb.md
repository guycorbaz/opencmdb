---
title: "Product Brief: opencmdb"
status: "complete"
created: "2026-07-15"
updated: "2026-07-15"
inputs:
  - docs/brief-initial-bmad.md
  - _bmad-output/planning-artifacts/competitive-analysis.md
---

# Product Brief: opencmdb

## Executive Summary

Every home-lab operator and small business with a "more complex than it looks" network lives with the same quiet problem: the map of what's on the network is never the territory. IP addresses drift, unknown devices appear, and the one document describing it all — a spreadsheet, a stale draw.io diagram, or someone's memory — is wrong the day it's written.

The tools that could fix this are split into two camps that never meet. **Source-of-truth tools** (NetBox, Nautobot, iTop) model what you *declare* but discover nothing on their own and demand a heavy PostgreSQL + Redis + workers stack. **Discovery tools** (Angry IP Scanner, NetAlertX, phpIPAM's agent) see what's *actually there* but don't let you document intent or reason about it. Nobody serving this audience continuously reconciles the two — the only product that genuinely does, NetBox Assurance, is paywalled and sits atop the very complexity these users are fleeing.

**opencmdb** is a self-hosted, single-binary application that unifies lightweight IPAM, a light application CMDB, and network topology around one organizing idea: **continuously compare the *observed* state (auto-discovered) with the *declared* state (documented by you), and turn the gap into action.** An unknown device just appeared. A documented IP hasn't been seen in 30 days. Two devices are fighting over the same address — and unlike your UniFi controller's bare alarm, opencmdb tells you *which* devices. In short, it is **documentation that corrects itself**: your source of truth stays true because it is continuously checked against reality. Deployed on a Synology NAS via a two-service compose (opencmdb + MariaDB), it delivers visible value in under 15 minutes. _(F54, 2026-07-17: the former claim — "one container, under 30 minutes" — was written before the database was a prerequisite. **The 30-minute figure is now UNMEASURED and is therefore not claimed.** It gets measured on the reference NAS, stopwatch running, while writing the shipped doc — one evening — and it returns to this brief as a number or not at all.)_

## The Problem

The person who owns this pain is not a network engineer — it's a competent generalist running real infrastructure without a dedicated IT team: a 5–100-person SMB with several VLANs, self-hosted servers, a NAS and managed switches; or an advanced home-labber with Docker/Proxmox, a Synology NAS, and dozens of devices (a setup often *more* tangled than a comparable small business).

They cope badly, because every available option forces a bad trade:

- **Spreadsheets and diagrams** are wrong the moment they're saved and answer no live question.
- **Professional tools are overbuilt and painful to run.** NetBox and Nautobot need PostgreSQL, Redis, background workers, and a reverse proxy just to store data you still have to enter by hand — and their discovery isn't built in; it's bolted on via agents and plugins that are a project unto themselves.
- **Scanners see but don't remember.** Angry IP Scanner reveals the network beautifully and documents nothing. phpIPAM's agent is shallow and fragile.

The concrete cost is felt in ordinary moments: *"I need a free IP in this subnet — which ones are actually free?"* *"My UniFi controller is alarming about an IP conflict, but which physical device is it?"* *"This service is down — what else depends on the box it lives on?"* Today each of these means a manual hunt across tools that don't talk to each other, or a guess.

## The Solution

opencmdb gives these users one place where discovery and documentation finally meet:

- **Inventory & IPAM.** Devices, their network interfaces (each keyed by MAC), subnets, VLANs, and IP addresses — with a per-subnet occupancy view that answers *"what's used, what's free, what's declared, what's actually observed"* at a glance.
- **Automatic discovery.** A **first-class UniFi connector** polls the local controller and returns clients, exact switch ports, SSIDs, VLANs, and DHCP leases — with zero network privilege. A **generic scanner** (ARP + ping sweep) covers everything non-UniFi.
- **Observed-vs-declared reconciliation — the core.** Discovered devices are matched to declared records by MAC (with hostname/IP-history fallbacks). Anything unmatched lands in a **triage inbox** whose moves are **document** (EN) / « Merger » (FR UI) — one click promotes the discovery into declared documentation and sets it as the baseline — **accept-gap** (« Accepter l'écart »: *I have seen the divergence and have not yet decided what to do* — the gap stays OPEN), create a new record, attach it to an existing device, or **exclude** it (every decision is remembered). That one-click **document** is the near-zero-effort on-ramp that keeps documenting from becoming a chore, and it establishes the baseline against which **future changes are detected and analyzed** — drift over time, not just first sighting. Declared and observed data are never merged: reconciliation *links* them, and the link is where the value lives. _(Vocabulary corrected 2026-07-17 — F59. This paragraph still carried **`accept-as-declared`** and **`ignore`**, both RETIRED on 2026-07-16 and both on the retired-words denylist. **The rename crossed into the PRD and the UX spec and never reached this document.** `accept` named two opposite gestures — you *document* a VALUE; you *accept* a GAP — and `ignore` names a FEELING where every other verb names a state.)_
- **Applications & impact.** Group software instances into logical applications with typed relationships (`depends_on`, `connects_to`, `exposes`) and answer *"if this box goes down, what breaks?"*
- **Alerts on the gap.** New unknown device, IP not seen in N days, same IP on two MACs — surfaced in-app and pushable via a generic webhook. The *"unknown device appeared"* signal doubles as lightweight **rogue-device / shadow-IT detection** — a security tripwire on your own LAN, not just an inventory correction.
- **Plays well with your stack.** A read-only JSON API lets other tools consume the data, and a Prometheus `/metrics` endpoint turns opencmdb's own signals — free IPs per subnet, inbox size, active alerts, devices unseen for N days — into Grafana dashboards and Prometheus alerts in the monitoring stack you already run.

## What Makes This Different

- **Reconciliation, unpaywalled.** The observed-vs-declared loop is opencmdb's reason to exist. As of mid-2026 it's a top-of-market, paid NetBox (Assurance) feature; the closest framing for opencmdb is *"the reconciliation engine you can actually run"* — free, self-hosted, single-binary. That's an open, defensible wedge.
- **One product where the market offers two half-tools.** Source-of-truth tools don't discover; discovery tools don't model intent. opencmdb is the bridge — built by someone who hit this exact wall using iTop, NetBox, phpIPAM, and Angry IP Scanner.
- **A zero-privilege UniFi integration nobody else offers this audience.** Only NetAlertX imports from UniFi, and only for presence alerts. opencmdb's first-class connector feeds IPAM, topology, and DHCP — and reads from the controller with **zero network privilege**: no SNMP community, no SSH, no admin credentials on your gear. That's both a differentiator and a safety property (minimal attack surface, safe to run on an untrusted box).
- **Blast-radius awareness.** Typed application relationships answer *"if this host or IP dies, what breaks?"* — the service-layer view pure IPAM and discovery tools completely lack. It's what earns the "CMDB" in the name.
- **Radically smaller footprint.** **One binary + your MariaDB. No Redis, no workers, no queue, no reverse-proxy tax.** Two pieces against NetBox's five. The heavy stack is the incumbents' single biggest adoption barrier; opencmdb removes it. _(The differentiator is "no Redis/workers/proxy", not "no database" — and it survives D64 intact.)_

## Who This Serves

A **single, unified audience**: technically confident operators of small-but-nontrivial networks, whether an SMB without a full-time network engineer or an advanced home-labber. Their needs are the same — and the home-lab case is often the more demanding of the two. Success for them: a network they can *trust the documentation of*, where the gap between reality and record is visible and one click from resolved. The project author's own all-UniFi, Synology-hosted, multi-building network is the live proving ground for the MVP.

Explicitly **not** for: large enterprises (NetBox already serves them) or households with an ISP box and five devices (no problem to solve).

## Success Criteria

- Install on Synology (Container Manager) in **under 30 minutes**, docs included.
- On a UniFi network: a **complete observed inventory** (clients, ports, SSIDs) with **zero manual entry** after one polling cycle.
- On a non-UniFi network: active devices on the local segment discovered by scan.
- A user can document a 3-tier application and get its **impact view**.
- The observed/declared gap is **visible and actionable** (inbox + alerts) — including resolving "which device owns this conflicting IP."
- **The invariant suite passes on `mariadb:10.11.11` — the exact DSM 7 package — on every PR.** _(D64: dev, CI and production are the same engine at the same version. The dual-backend criterion is retired.)_
- First visible value in **under 15 minutes** from first launch.

## Scope

**In (MVP v1):** device inventory with multi-interface/MAC support; IPAM (subnets, VLANs, DHCP ranges, occupancy view); automatic discovery (first-class UniFi connector + generic ARP/ping scanner, scheduled + on-demand); MAC-keyed reconciliation with triage inbox and remembered decisions; applications/software with typed relationships and impact view; topology as a structured list/table (auto for UniFi, manual elsewhere); minimal alerts (unknown device, stale IP, IP conflict) with in-app view + generic webhook; **read-only JSON API** (best-effort, unversioned); **Prometheus `/metrics` endpoint**; **bilingual EN/FR UI**; local login/session auth with a **multi-user-ready schema**; configurable observation retention (default 90 days) with permanently retained first-seen/last-seen and IP-history rollups; IPv6 documented (subnets/addresses) though not actively scanned.

**Out (v2+):** interactive graphical network map; SNMP/LLDP topology for non-UniFi gear; additional connectors (Omada, Mikrotik, OPNsense/pfSense, Proxmox, Docker); fine-grained roles beyond the single admin (read-only role); rich notifications (email, Telegram, ntfy); CSV/NetBox/phpIPAM import; active IPv6 scanning; service/port scanning to suggest installed software; a **stable, versioned, documented public API** with write access; consuming Prometheus metrics for reverse enrichment; declared-change audit log.

## Technical Approach (firm constraints)

Non-negotiable decisions carried from scoping, to be honored by architecture and planning: **Rust**, **single binary**; **MariaDB 10.11+ as the only supported engine** (native to Synology and included in its automatic backups) — **SQLite and MySQL are not supported, and PostgreSQL is not supported at MVP**; **Docker container** (Synology Container Manager) as the priority deployment target, native binary also possible; **UniFi connector is first-class from day one**. _(Amended 2026-07-17 by D64. The original read "SQLite (small installs) and MySQL/MariaDB (larger installs)" — **"SQLite, small installs" was an unmeasured belief about a user population, load-bearing from this brief onward and never once examined**, which is D59's error on the storage axis. The recorded motive is not "most users will choose MariaDB" (unverifiable) but **"no target user is blocked by MariaDB"** — checkable against our own journeys, and true.)_ All project documentation (README, GitHub Pages, LaTeX manuals) is **English-only**, while the application UI is bilingual EN/FR. Because opencmdb concentrates a full network map plus controller credentials, **security is a design constraint, not an afterthought**: stored credentials are encrypted at rest, and **all HTTP surfaces — web UI, JSON API, and the Prometheus `/metrics` endpoint — sit behind authentication** (a scrape token for `/metrics`).

## Key Risks & Mitigations

- **MAC randomization & reconciliation noise** *(the central UX challenge).* Randomized/virtual/cloned MACs can flood the triage inbox with false "new device" churn — the very noise users are fleeing. Mitigation: the triage inbox with *remembered* decisions plus secondary heuristics (hostname, IP/DHCP history); reconciliation *links*, never blindly merges.
- **UniFi API fragility across UniFi OS versions.** The connector — the headline differentiator — rests on an interface that shifts between controller versions. Mitigation: isolate it behind a Rust trait, target the API-key path, test against current versions, and **degrade gracefully rather than break hard** when a version drifts.
- **Solo, AI-assisted delivery of a broad MVP.** The scope spans IPAM, CMDB, topology, discovery, reconciliation, alerts, API, metrics, i18n, and two backends. Mitigation: strict adherence to the MVP boundary above, and an internal *sequencing* of that MVP (see next step) so the reconciliation hypothesis is proven early rather than last.
- **Security & trust of a source-of-truth tool.** A network map + credentials is a high-value target, and data-loss/migration bugs would be reputationally fatal. Mitigation: encryption at rest, authenticated endpoints, and the same integration suite passing on both backends before release.
- **Will users actually maintain declared state?** Reconciliation's value collapses if people only scan and never declare. Two-pronged mitigation: the **one-click *document*** action makes documenting near-effortless (promote what's discovered instead of retyping it), and a **light community validation** — a few r/selfhosted and r/homelab conversations plus a concept post — tests real appetite before the heaviest build. _(F59: read `accept-as-declared` — retired 2026-07-16.)_

## Vision

In 2–3 years, opencmdb is the default answer to *"what's actually on my network, and does my documentation match reality?"* for the home-lab and small-business world — the tool people reach for instead of a spreadsheet or an over-engineered enterprise platform. The reconciliation engine grows a richer connector ecosystem (Omada, Mikrotik, pfSense/OPNsense, hypervisors), an interactive topology map, and a stable public API, becoming the small-scale operator's trusted source of truth: not another thing to maintain, but the thing that keeps everything else honest. Distribution stays as frictionless as the product: published as a ready-to-run image on **Docker Hub**, so the try-it path is **one compose file: opencmdb and your MariaDB.** _(Rewritten 2026-07-17, D64 condition 2. It read "the entire try-it path is a single `docker run`". That became false the day the database became a prerequisite — and it was **already** false since D27, which puts the KEK in a separate shared folder and makes its absence a startup refusal. **A product whose thesis is "your documentation lies, I show you where" cannot ship a tagline that lies about its own deployment.** It is NFR9's error class — true where it was written, false where it counted — in our own shop window.)_
