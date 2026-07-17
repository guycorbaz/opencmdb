# Competitive Landscape: opencmdb vs. Incumbents

> Supporting research for the opencmdb Product Brief (Phase 1 — Analysis).
> Scope: self-hosted IPAM + lightweight CMDB + topology for home-lab / SMB.
> Axes: (a) auto-discovery out-of-the-box, (b) deploy difficulty for non-IT users,
> (c) reconciles observed vs declared, (d) footprint.

| Tool | (a) Auto-discovery OOTB? | (b) Deploy difficulty | (c) Observed vs declared reconciliation? | (d) Footprint |
|---|---|---|---|---|
| **NetBox** (core) | No. Core is a manual "source of truth"; discovery needs add-ons (orb-agent/Diode, IP Fabric plugin, netbox-agent scripts). | High — Django + PostgreSQL 15+ + Redis (2 DBs) + Gunicorn/nginx + workers. | No in core; only if you bolt on Discovery + Assurance. | Heavy multi-service stack. |
| **NetBox Discovery + Diode + Assurance** | Yes, via containerized Go orb-agent (ICMP/TCP/UDP/SSH/SNMP). | High — adds agent + Diode ingestion on top of NetBox. Assurance is **Enterprise/Cloud (paid)**. | **Yes** — the one incumbent explicitly marketing continuous "intended vs observed" drift detection. But gated behind paid tiers and NetBox complexity. | Very heavy (NetBox stack + agents + Diode). |
| **Nautobot** | No in core; needs Jobs + apps (Device Onboarding uses Netmiko/NAPALM; Golden Config for config drift). | High — like NetBox plus mandatory Celery workers + Redis for Jobs. | Only config-level via Golden Config (intended Jinja vs backup); no general IP/device declared-vs-observed. | Heavy: Django + PostgreSQL + Redis + Celery workers. |
| **phpIPAM** | Partial — built-in scan agent (ping/fping/pear/telnet) discovers new IPs and up/down status, writes to MySQL. | Medium — PHP + MySQL/MariaDB; agent needs correct ping/fping binaries + PHP extensions (common failure point). | Weakly — flags new/offline IPs vs records, but no rich declared-model reconciliation or device CMDB. | Medium: LAMP-style (PHP + MySQL). |
| **iTop** | No — autodiscovery must be done externally (OCS Inventory, custom collectors, third-party agents). | Medium-High — PHP + MySQL; CMDB modeling is manual/tedious; CSV mass-import to bootstrap. | No — stores declared CIs only; no observed-state loop. | Medium: PHP + MySQL. |
| **Netdisco** | Yes — SNMP/CLI polling of switches/routers; MAC-to-port, ARP, L2 topology. | Medium — Perl + PostgreSQL; enterprise-LAN oriented. | No — inventory/mapping only, no declared model. | Medium: Perl + PostgreSQL. |
| **NetAlertX** (ex-PiAlert) | Yes — arp-scan, DHCP leases, Pi-hole, SNMP, **and native UniFi controller import**. | **Low** — single Docker container, local-only. | Partial — alerts on *new/changed/offline* devices vs known list; presence-focused, not a declared IPAM/CMDB model. | Light: one container. |
| **Angry IP Scanner** | Yes — GUI on-demand IP/port scan. | Very low (desktop app). | No — ephemeral scan, no persistence/model. | Trivial (desktop). |

## Gaps opencmdb can exploit

1. **Reconciliation is only real at the top of the market.** Only **NetBox Assurance** genuinely markets continuous "intended vs observed" drift — and it's paywalled (Enterprise/Cloud) and sits atop NetBox's heavy stack. No lightweight/self-hosted tool offers first-class declared-vs-observed reconciliation for IPAM+CMDB. **This is opencmdb's clearest wedge.**

2. **Discovery ↔ documentation is split across two tool classes.** Source-of-truth tools (NetBox, Nautobot, iTop) don't discover OOTB; discovery tools (NetAlertX, Netdisco, Angry IP) don't model declared intent. opencmdb unifying both in one product is unserved in the mid-market.

3. **Footprint pain.** NetBox/Nautobot demand PostgreSQL + Redis + Celery/RQ workers + reverse proxy — overkill and intimidating for home-lab/SMB. A single-binary / light-container opencmdb directly counters the top-3's biggest adoption barrier.

4. **UniFi is under-served.** Only **NetAlertX** has a native UniFi controller import — and it's presence-alerting, not IPAM/CMDB/topology. No source-of-truth tool has a first-class UniFi connector for DHCP/clients/topology. A polished UniFi integration is a concrete, defensible differentiator for exactly opencmdb's audience.

5. **phpIPAM's discovery is fragile** (binary/extension misconfig, marks reachable hosts offline due to network-path issues) and shallow (IP up/down, no device model) — an easy bar to clear on reliability and depth.

## Sources
- NetBox Discovery / orb-agent / Diode: https://netboxlabs.com/docs/discovery/ , https://netboxlabs.com/docs/orb-agent/ , https://netboxlabs.com/blog/announcing-diode-agent-a-lightweight-new-network-device-discovery-tool-to-streamline-netbox-data-entry-with-diode/ , https://netboxlabs.com/blog/expanding-netbox-discovery-snmp-support/
- NetBox Assurance (observed vs intended): https://netboxlabs.com/products/netbox-assurance/ , https://netboxlabs.com/docs/assurance/ , https://netboxlabs.com/blog/netbox-assurance-netbox-enterprise-automatically-detect-fix-operational-drift/
- NetBox stack/footprint: https://netboxlabs.com/docs/netbox/installation/ , https://deepwiki.com/netbox-community/netbox/1.2-installation-and-deployment
- IP Fabric plugin: https://docs.ipfabric.io/7.5/integrations/netbox/
- Nautobot Celery/Redis/Jobs: https://docs.nautobot.com/projects/core/en/stable/user-guide/administration/configuration/redis/ , https://docs.nautobot.com/projects/device-onboarding/en/latest/user/faq/
- Nautobot Golden Config: https://docs.nautobot.com/projects/golden-config/en/latest/
- phpIPAM scanning: https://deepwiki.com/phpipam/phpipam/6.1-network-scanning-and-discovery , https://github.com/phpipam/phpipam-agent
- iTop discovery: https://github.com/Combodo/iTop , https://faddom.com/top-9-open-source-cmdb-solutions-and-their-pros-cons/
- NetAlertX (UniFi import): https://github.com/netalertx/NetAlertX , https://netalertx.com/
- Netdisco: https://netcontroler.com/program/netdisco/
- Angry IP Scanner: https://sourceforge.net/software/product/Angry-IP-Scanner/alternatives
