# opencmdb

**A self-hosted, single-binary network reconciliation engine — lightweight IPAM, a light application CMDB, and network topology — for advanced home labs and small businesses.** It continuously compares the *observed* state of your network (auto-discovered) against the *declared* state you documented; **the gap between them is the product.**

- **Source & full docs:** https://github.com/guycorbaz/opencmdb
- **License:** AGPL-3.0-or-later

---

## ⚠️ Early development — pre-release

opencmdb is being built in the open. This image is published starting at **`0.1.0`** as a *walking skeleton* so it can be tested live — it is **not** production-ready and does not yet reconcile a real network end to end. Watch the [GitHub repo](https://github.com/guycorbaz/opencmdb) for progress. Everything below describes the **intended** way to run it, with placeholders only.

## Image

```
docker pull gcorbaz/opencmdb:0.1.0
```

- Distroless, static, runs as a **non-root** user.
- Amd64 (x86-64 Synology / Linux hosts).

### Tags

| Tag | Meaning |
|-----|---------|
| `0.1.0` | first published pre-release (walking skeleton) |
| `latest` | most recent published tag |

## One database, on purpose

opencmdb supports **MariaDB 10.11+ only** (SQLite and MySQL are out; PostgreSQL is not supported at this stage). On a Synology NAS this is the DSM-managed MariaDB package — so your opencmdb data is covered by the NAS backup you already run. The container connects to your **existing** MariaDB; it does not bundle one.

### Before the first start: provision the database yourself

Nothing in the image or the compose file creates the database, the user or the grants. Do it first, as a MariaDB administrator:

```sql
CREATE DATABASE opencmdb CHARACTER SET utf8mb4 COLLATE utf8mb4_bin;
CREATE USER 'opencmdb'@'%' IDENTIFIED BY 'your-password';
GRANT ALL PRIVILEGES ON opencmdb.* TO 'opencmdb'@'%';
FLUSH PRIVILEGES;
```

The binary collation is required — identity comparison must never depend on the database's locale. Narrow `'%'` once the connection works.

> **Grants match the address the server *sees*, not the one you dial.** If authentication fails, read the error: `Access denied for user 'opencmdb'@'<host>'` names the exact identity you must grant. On a multi-homed machine these differ — traffic sent to one interface can leave by another, and MariaDB matches on the source it observes (or its reverse-resolved name). The tell-tale sign is that the host in the error message *changes* as you change the URL.

## Running with Docker Compose

opencmdb runs as a single service pointing at your MariaDB. A reference `docker-compose.yml` and `.env.example` ship in the [repository](https://github.com/guycorbaz/opencmdb) under `docker/`. Sketch:

```yaml
services:
  opencmdb:
    image: gcorbaz/opencmdb:0.1.0
    container_name: opencmdb
    env_file: .env
    ports:
      - "8080:8080"   # OPENCMDB_BIND (in .env) sets the in-container listener
    cap_add:
      - NET_RAW   # ARP upgrade path (Mac facts, a later release); ping-only works without it
    volumes:
      - ./log:/var/log/opencmdb   # daily-rotating file logs (host ./log must be writable by uid 65532)
    restart: unless-stopped
```

> **Do not use `network_mode: host`.** It is the intuitive choice for a network scanner and it is the wrong one — it *removes* a permission rather than granting one. Docker sets `net.ipv4.ping_group_range=0 2147483647` inside a container's own network namespace, which is exactly what lets opencmdb open its unprivileged ICMP socket and scan as a non-root user. Host mode inherits the *host's* value instead; many hosts (Synology DSM among them) ship an empty range, the socket is refused, and the scan fails with a non-fatal warning — the container looks healthy, `/healthz` returns 200, and it observes nothing. Host mode also costs you port isolation and reverse-proxy discovery. ICMP echo is routed and crosses a NAT fine, so the default above scans a LAN correctly.

Provide configuration through a `.env` file you keep **outside** version control (see `docker/.env.example`):

```dotenv
# Placeholders — set your own; never commit this file.
# The host is your database server as seen FROM INSIDE the container — not 127.0.0.1, which
# would be the container itself.
DATABASE_URL=mysql://opencmdb:CHANGE_ME@192.0.2.5:3306/opencmdb
OPENCMDB_BIND=0.0.0.0:8080
OPENCMDB_LOG=info
OPENCMDB_LOCALE=en
# Optional: ping-scan this CIDR on startup (use your real LAN, e.g. 192.168.x.0/24).
OPENCMDB_SCAN_CIDR=192.0.2.0/24
# Probes in flight at once (default 64) — a politeness bound on your gateway's ARP table,
# not a throughput setting. The scan is I/O-bound and single-threaded either way.
OPENCMDB_SCAN_CONCURRENCY=64
# Bearer token for the Prometheus /metrics endpoint (leave unset to keep it closed).
OPENCMDB_METRICS_TOKEN=CHANGE_ME
# Daily-rotating file logs to this in-container path (mounted from ./log); keep this many days.
OPENCMDB_LOG_DIR=/var/log/opencmdb
OPENCMDB_LOG_RETENTION=14
```

> Use RFC 5737 documentation addresses (`192.0.2.0/24`) and example hostnames in anything you share — never paste your real network into a public place.

> **Special characters in the DB password — two separate traps.**
>
> **1. A `$` must be doubled: `$$`.** Docker Compose interpolates the contents of your `.env`, so a password written `pa$word` is silently truncated to `pa` and you get an opaque "access denied". This happens *before* opencmdb starts, so nothing the application does can recover it. Measured: `abc$def` arrives as `abc`; `abc$$def` arrives as `abc$def`.
>
> **2. URL-reserved characters must be percent-encoded.** Independently of the above, the password sits inside a URL: `@`→`%40`, `:`→`%3A`, `/`→`%2F`, `#`→`%23`, `?`→`%3F`, `%`→`%25`, space→`%20`. opencmdb percent-decodes the user info, so the database receives the original password.
>
> Both at once: the password `s3cr$t@x` is written `s3cr$$t%40x`.

## Troubleshooting the first start

Run the first start in the **foreground** — `docker compose up`, not `up -d`. A fatal startup error is currently printed to stdout only and does not reach the log files, so with `restart: unless-stopped` a crash-looping container leaves you nothing but repeated `opencmdb starting` lines.

| Symptom | Cause | Fix |
|---|---|---|
| `1045 Access denied for user 'opencmdb'@'<host>'` | No grant for the identity MariaDB *sees* | Grant exactly the `<host>` in the message — see above |
| Same, and `<host>` changes when you change the URL | Multi-homed database server: replies leave by another interface | Grant every address it can present, or use `'%'` while testing |
| Same, password looks correct | A `$` was eaten by Compose | Double it: `$$` |
| `Address in use (os error 98)` | Another service already holds the port | Change the host side of `ports:` |
| `startup scan failed … could not open an ICMP socket` | `network_mode: host` inherited an empty `ping_group_range` | Drop host mode — see the note above |
| Page loads, but shows no observed data | The scan failed with a non-fatal warning | Check the logs for `startup scan failed` |
| Log files contain only `opencmdb starting` | The crash reason went to stdout | Run in the foreground, or read `docker logs` |

## Security

- All HTTP surfaces sit behind authentication; TLS is terminated at a reverse proxy in front of opencmdb.
- Stored source credentials are encrypted at rest; the **encryption key is required to live outside the database volume**, so a stolen database file alone does not reveal your secrets.
- opencmdb protects against a stolen database/backup and unauthenticated network access — not a local root attacker with both the database and the key. The full threat model is in the architecture document.

## Links

- **GitHub:** https://github.com/guycorbaz/opencmdb
- **Issues:** https://github.com/guycorbaz/opencmdb/issues
- **License:** AGPL-3.0-or-later

*Built in the open by a solo developer with AI assistance. The name is lowercase, always.*
