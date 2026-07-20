# Story 3.9: Packaging — Dockerfile, compose template, `.env.example`

Status: review

## Story

As a maintainer,
I want a Docker image and a reference compose that targets an external MariaDB,
so that opencmdb can be deployed on the NAS without leaking secrets.

## Acceptance Criteria

1. **Given** the workspace, **when** the image is built, **then** a `Dockerfile` produces a distroless, static, non-root image of `opencmdb-bin` (D66), built `--locked`.
2. **Given** the `docker/` directory, **when** it is inspected, **then** it holds a `docker-compose.yml` running ONLY the opencmdb service pointed at an EXISTING external MariaDB (not a bundled DB container), plus a `.env.example` with documented placeholders (RFC 5737 addresses, `CHANGE_ME`) — and the real `.env` is git-ignored.
3. **And** no production secrets, no real hostnames, and no NAS path appear in any committed file (they live only on the NAS).

## Tasks / Subtasks

- [x] Task 1 — Dockerfile (AC: #1)
  - [x] Multi-stage: `rust:1.96-bookworm` builder → static musl binary (`x86_64-unknown-linux-musl`, `cargo build --release --locked -p opencmdb-bin`); runtime `gcr.io/distroless/static-debian12:nonroot` (D66 — CA certs + tzdata + nonroot). The binary is self-contained (migrations, assets, and locales are embedded at compile time), so the runtime layer carries only the binary.
  - [x] `.dockerignore` keeps the build context free of `target/`, `.git/`, `_bmad*`, `docs/`, and any `.env`.
- [x] Task 2 — Compose + env template (AC: #2, #3)
  - [x] `docker/docker-compose.yml`: ONE `opencmdb` service, `network_mode: host` (reach the NAS MariaDB on 127.0.0.1 + real LAN visibility for the scanner), `env_file: .env`, `cap_add: [NET_RAW]` (ARP upgrade path), `restart: unless-stopped`. NO database container.
  - [x] `docker/.env.example`: documented placeholders only — `DATABASE_URL` (127.0.0.1, `CHANGE_ME`), `OPENCMDB_BIND/LOG/LOCALE`, `OPENCMDB_SCAN_CIDR` (RFC 5737 `192.0.2.0/24`), `OPENCMDB_METRICS_TOKEN` (`CHANGE_ME`), commented `OPENCMDB_ENTITY_IPV4`.
  - [x] `.env` / `.env.*` already git-ignored (with `!.env.example`); verified.
  - [x] Docker Hub overview (`docker/README.dockerhub.md`) compose sketch + env aligned to the real files (host networking, real env vars).
- [x] Task 3 — Verify (AC: #1–#3)
  - [x] `docker build` succeeds; the image runs end to end against a local MariaDB (connects, migrates, serves `/` and `/healthz`), runs as `nonroot`, and is ~18 MB.
  - [x] Grep confirms no real secret/hostname/NAS path in any committed file (placeholders only).

## Dev Notes

### Distroless static, non-root — and self-contained (D66)

The runtime base is `gcr.io/distroless/static-debian12:nonroot`: CA certs + tzdata + a nonroot user, no shell, no package manager (Murat's "image-scan-is-theatre" verdict holds — signal→0, noise→∞ on a static Rust binary). A musl target gives the fully static binary distroless/static needs; `musl-tools` supplies `musl-gcc` for ring's C. Everything the app reads is embedded at COMPILE time — `sqlx::migrate!` (SQL), `rust-embed` (assets), `rust_i18n::i18n!` (locales) — so the image is just the binary.

**Build-order gotcha (fixed):** `rust-toolchain.toml` switches rustup's active toolchain, so `rustup target add` must run AFTER copying that file, or it adds musl to the base image's default toolchain and the pinned toolchain builds against a missing `std`. The Dockerfile copies the pin first.

### Host networking — the deliberate choice (Guy, 2026-07-20)

`network_mode: host`: the container reaches the NAS's MariaDB on `127.0.0.1`, AND the ARP/ping scanner gets real L2 LAN visibility — a bridge NAT breaks ping/ARP. Under host mode `OPENCMDB_BIND` decides the listener; `ports:` are ignored. `NET_RAW` is granted for the later ARP (Mac) upgrade; ping-only works without it.

### Secrets discipline (D27/D63, [[no-private-data-in-artifacts]])

The compose points at an EXISTING external MariaDB — no bundled DB. The committed files carry ONLY placeholders (RFC 5737 `192.0.2.0/24`, `CHANGE_ME`). The real `.env`, the production DSN, the metrics token, and the NAS deploy path live ONLY on the NAS and are git-ignored — they never enter the repository.

### Scope — the image + the reference deploy, not the release

This story builds and proves the image and ships the reference compose/env. Pushing a tagged image to Docker Hub via CI is Story 3.10.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.9: Packaging]
- [Source: _bmad-output/planning-artifacts/architecture.md#D66 — Docker base = distroless/static:nonroot]
- [Source: .gitignore — `.env` / `.env.*` ignored, `!.env.example` kept]
- [Source: docker/README.dockerhub.md — the Docker Hub overview]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `docker build -t opencmdb:0.1.0-test .` → success. First attempt failed (`can't find crate for std` — musl target on the wrong toolchain); fixed by copying `rust-toolchain.toml` before `rustup target add`.
- Ran the image `--network host` against `mariadb:10.11.11`: logs show "database connected and migrations applied" + "opencmdb listening"; `GET /healthz` → 200, `GET /` → 200 (honest "No declared record yet"). `docker inspect … User` = `nonroot`; `docker exec … id` fails (no shell — genuinely distroless). Image size ~17.8 MB.
- `cargo xtask ci` all gates green; fmt clean (no Rust changed).

### Completion Notes List

- **AC #1** — multi-stage `Dockerfile`: static musl release build `--locked` → `distroless/static-debian12:nonroot`; verified to build AND run (connect + migrate + serve), nonroot, ~18 MB, self-contained.
- **AC #2** — `docker/docker-compose.yml` (one service, host network, external MariaDB, no DB container) + `docker/.env.example` (placeholders only). `.env` git-ignored.
- **AC #3** — only RFC 5737 addresses + `CHANGE_ME` in committed files; the real DSN, tokens, and NAS path stay on the NAS.
- No Rust code changed — CI's Rust gates are unaffected; the image build is proven locally (CI builds/pushes the image in Story 3.10).

### File List

- `Dockerfile` (new) — multi-stage static-musl → distroless/static:nonroot.
- `.dockerignore` (new) — minimal, secret-free build context.
- `docker/docker-compose.yml` (new) — reference deploy, host network, external MariaDB.
- `docker/.env.example` (new) — documented placeholder configuration.
- `docker/README.dockerhub.md` (modified) — compose sketch + env aligned to the real files.

## Change Log

- 2026-07-20 — Implemented Story 3.9 (packaging). Multi-stage Dockerfile → distroless/static:nonroot static-musl image (~18 MB), built `--locked` and verified to run end to end against MariaDB as nonroot. Reference `docker/docker-compose.yml` (host networking, external MariaDB, `NET_RAW`) + `.env.example` (RFC 5737 / `CHANGE_ME` placeholders); `.env` git-ignored; Docker Hub overview aligned. No real secrets/hostnames/NAS path committed. Status → review.