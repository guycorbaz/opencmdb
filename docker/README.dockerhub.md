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

## Running with Docker Compose

opencmdb runs as a single service pointing at your MariaDB. A reference `docker-compose.yml` and `.env.example` ship in the [repository](https://github.com/guycorbaz/opencmdb) under `docker/`. Sketch:

```yaml
services:
  opencmdb:
    image: gcorbaz/opencmdb:0.1.0
    restart: unless-stopped
    env_file: .env
    ports:
      - "8080:8080"
    volumes:
      # The encryption key lives OUTSIDE the database volume, by design.
      - ./secrets:/secrets:ro
```

Provide configuration through a `.env` file you keep **outside** version control:

```dotenv
# Placeholders — set your own; never commit this file.
DATABASE_URL=mysql://opencmdb:CHANGE_ME@192.0.2.10:3306/opencmdb
OPENCMDB_ENCRYPTION_KEY_FILE=/secrets/encryption.key
OPENCMDB_LOG=info
```

> Use RFC 5737 documentation addresses (`192.0.2.0/24`) and example hostnames in anything you share — never paste your real network into a public place.

## Security

- All HTTP surfaces sit behind authentication; TLS is terminated at a reverse proxy in front of opencmdb.
- Stored source credentials are encrypted at rest; the **encryption key is required to live outside the database volume**, so a stolen database file alone does not reveal your secrets.
- opencmdb protects against a stolen database/backup and unauthenticated network access — not a local root attacker with both the database and the key. The full threat model is in the architecture document.

## Links

- **GitHub:** https://github.com/guycorbaz/opencmdb
- **Issues:** https://github.com/guycorbaz/opencmdb/issues
- **License:** AGPL-3.0-or-later

*Built in the open by a solo developer with AI assistance. The name is lowercase, always.*
