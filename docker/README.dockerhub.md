# opencmdb

**A self-hosted, single-binary network reconciliation engine — lightweight IPAM, a light application CMDB, and network topology — for advanced home labs and small businesses.** It continuously compares the *observed* state of your network (auto-discovered) against the *declared* state you documented; **the gap between them is the product.**

- **Source & full docs:** https://github.com/guycorbaz/opencmdb
- **License:** AGPL-3.0-or-later

---

## ⚠️ Early development — pre-release

opencmdb is being built in the open. This image is published starting at **`0.1.0`** as a *walking skeleton* so it can be tested live — it is **not** production-ready and does not yet reconcile a real network end to end. Watch the [GitHub repo](https://github.com/guycorbaz/opencmdb) for progress. Everything below describes the **intended** way to run it, with placeholders only.

## Image

```
docker pull gcorbaz/opencmdb:0.1.1
```

- Distroless, static, runs as a **non-root** user.
- Amd64 (x86-64 Synology / Linux hosts).

### Tags

| Tag | Meaning |
|-----|---------|
| `0.1.1` | deployment fixes: overlapping ping probes, `DATABASE_*` variables, fatal errors logged, `cap_net_raw` on the binary |
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

> **Grants match the address the server *sees*, not the one you dial.** If authentication fails, read the error: `Access denied for user 'opencmdb'@'<host>'` names the exact identity you must grant. On a multi-homed machine these differ — traffic sent to one interface can leave by another, and MariaDB matches on the source it observes (or its reverse-resolved name). The tell-tale sign is that the host in the error message *changes* as you change the address you connect to.

## Running with Docker Compose

opencmdb runs as a single service pointing at your MariaDB. A reference `docker-compose.yml` and `.env.example` ship in the [repository](https://github.com/guycorbaz/opencmdb) under `docker/`. Sketch:

```yaml
services:
  opencmdb:
    image: gcorbaz/opencmdb:0.1.1
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
# DATABASE_HOST is your database server as seen FROM INSIDE the container — not 127.0.0.1,
# which would be the container itself. Write the password exactly as it is; opencmdb builds
# the connection URL and encodes it for you.
DATABASE_HOST=192.0.2.5
DATABASE_PORT=3306
DATABASE_NAME=opencmdb
DATABASE_USERNAME=opencmdb
DATABASE_PASSWORD=CHANGE_ME
OPENCMDB_BIND=0.0.0.0:8080
OPENCMDB_LOG=info
OPENCMDB_LOCALE=en
# Optional: ping-scan this CIDR on startup (use your real LAN, e.g. 192.168.x.0/24).
OPENCMDB_SCAN_CIDR=192.0.2.0/24
# Probes in flight at once (default 64) — a politeness bound on your gateway's ARP table,
# not a throughput setting. The scan is I/O-bound and single-threaded either way.
OPENCMDB_SCAN_CONCURRENCY=64
# How long one probe waits for its reply, in ms (default 1000). This decides what the scan
# MISSES: one probe per host, no retry yet, so a slower device is recorded as absent.
OPENCMDB_SCAN_TIMEOUT_MS=1000
# Bearer token for the Prometheus /metrics endpoint (leave unset to keep it closed).
OPENCMDB_METRICS_TOKEN=CHANGE_ME
# Daily-rotating file logs to this in-container path (mounted from ./log); keep this many days.
OPENCMDB_LOG_DIR=/var/log/opencmdb
OPENCMDB_LOG_RETENTION=14
```

> Use RFC 5737 documentation addresses (`192.0.2.0/24`) and example hostnames in anything you share — never paste your real network into a public place.

> **Single-quote the password if it contains a `$`.** Docker Compose interpolates the contents of your `.env`, so an unquoted `$` begins what it reads as a variable name and the rest of the value is dropped — an opaque "access denied", decided *before* opencmdb starts, so nothing the application can do will recover it. Measured:
>
> | written | arrives as | |
> |---|---|---|
> | `DATABASE_PASSWORD='pa$word'` | `pa$word` | ✅ single quotes are fully literal |
> | `DATABASE_PASSWORD=pa$$word` | `pa$word` | ✅ doubling works, but rewrites your password |
> | `DATABASE_PASSWORD=pa$word` | `pa` | ❌ |
> | `DATABASE_PASSWORD="pa$word"` | `pa` | ❌ double quotes do **not** protect |
>
> The trap is sneakier than it looks: `abc$1def`, `abc$!def` and `abcdef$` all survive unquoted, because interpolation only fires on something resembling a variable name — so whether it bites depends on the character *after* the `$`. The one case single quotes cannot handle is a password containing a single quote, which makes Compose fail to parse the file; double the `$` there instead. Every other character — `@ : / # ? %`, spaces — is written exactly as it is: opencmdb assembles the connection URL and percent-encodes it for you.
>
> **`DATABASE_URL` is deprecated** but still honoured when none of the `DATABASE_*` variables above is set. With it, you must percent-encode the password by hand (`@`→`%40`, `:`→`%3A`, `/`→`%2F`, `#`→`%23`, `?`→`%3F`, `%`→`%25`, space→`%20`); forget one and authentication fails opaquely. That trap is the reason for the discrete variables.

## Troubleshooting the first start

A fatal startup error is logged in full — cause chain included — to both stdout and the daily log files, and the process exits non-zero. If a container is restarting in a loop, `docker logs` or `log/opencmdb.YYYY-MM-DD.log` will say why.

| Symptom | Cause | Fix |
|---|---|---|
| `1045 Access denied for user 'opencmdb'@'<host>'` | No grant for the identity MariaDB *sees* | Grant exactly the `<host>` in the message — see above |
| Same, and `<host>` changes when you change the address | Multi-homed database server: replies leave by another interface | Grant every address it can present, or use `'%'` while testing |
| Same, password looks correct | A `$` was eaten by Compose | Double it: `$$` |
| `Address in use (os error 98)` | Another service already holds the port | Change the host side of `ports:` |
| `startup scan failed … could not open an ICMP socket` | `network_mode: host` inherited an empty `ping_group_range` | Drop host mode; or keep it and grant `NET_RAW`, which the image's `cap_net_raw` binary capability then makes effective |
| Page loads, but shows no observed data | The scan failed with a non-fatal warning | Check the logs for `startup scan failed` |
| The page shows no gap | Observed state exists but nothing is declared yet | Declare an entity carrying an `ipv4` |

## Security

- All HTTP surfaces sit behind authentication; TLS is terminated at a reverse proxy in front of opencmdb.
- Stored source credentials are encrypted at rest; the **encryption key is required to live outside the database volume**, so a stolen database file alone does not reveal your secrets.
- opencmdb protects against a stolen database/backup and unauthenticated network access — not a local root attacker with both the database and the key. The full threat model is in the architecture document.

## Links

- **GitHub:** https://github.com/guycorbaz/opencmdb
- **Issues:** https://github.com/guycorbaz/opencmdb/issues
- **License:** AGPL-3.0-or-later

*Built in the open by a solo developer with AI assistance. The name is lowercase, always.*
