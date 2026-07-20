# Story 3.10: Release 0.1.0 to Docker Hub

Status: review

## Story

As a maintainer,
I want a `0.1.0` image published to Docker Hub via CI on a version tag,
so that live testing can begin from a real published artifact.

## Acceptance Criteria

1. **Given** a pushed git tag `v0.1.0`, **when** the release workflow runs, **then** it builds the image and pushes `gcorbaz/opencmdb:0.1.0` (and `:latest`) to Docker Hub using the `DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN` repository secrets.
2. **Given** the release workflow, **when** it completes, **then** it syncs `docker/README.dockerhub.md` to the Docker Hub repository description.
3. **And** `docker pull gcorbaz/opencmdb:0.1.0` works and the container starts against a MariaDB; the release is reachable for live testing. Closes Epic 3 (v0.1).

## Tasks / Subtasks

- [x] Task 1 — The release workflow (AC: #1, #2)
  - [x] `.github/workflows/release.yml`, triggered on `push: tags: ['v*.*.*']`: checkout, Buildx, Docker Hub login (secrets), `docker/metadata-action` to derive `:{{version}}` + `:latest` from the tag, `docker/build-push-action` (context `.`, `linux/amd64`, push, gha cache).
  - [x] `peter-evans/dockerhub-description` syncs `docker/README.dockerhub.md` + a short description (best-effort — `continue-on-error`, since the Docker Hub description API may require the account password rather than a push PAT).
- [ ] Task 2 — Cut the release (AC: #3) — **awaiting Guy's explicit go-ahead to publish**
  - [ ] Tag `v0.1.0` on green master and push it → the workflow publishes `gcorbaz/opencmdb:0.1.0` + `:latest`.
  - [ ] Verify `docker pull gcorbaz/opencmdb:0.1.0` and a container start against MariaDB.
- [x] Task 3 — Verify the workflow (AC: #1, #2)
  - [x] YAML parses; action versions pinned; the image already builds + runs (Story 3.9).
- [x] Task 4 — Daily file logs for on-NAS debugging (Guy's request, 2026-07-20)
  - [x] `init_tracing` now writes to stdout AND, when `OPENCMDB_LOG_DIR` is set, a DAILY-rotating file (`opencmdb.YYYY-MM-DD.log`, non-ANSI, retention `OPENCMDB_LOG_RETENTION` days, default 14) via `tracing-appender` (bin-only). Unwritable dir → stderr warning, stdout only, never a crash.
  - [x] Compose mounts `./log:/var/log/opencmdb` (host `docker/log`), with the nonroot-uid ownership note; `.env.example` documents `OPENCMDB_LOG_DIR`/`OPENCMDB_LOG_RETENTION`; `docker/log/` is git-ignored (logs carry IPs/hostnames). Verified end to end in the real image (file owned by uid 65532).

## Dev Notes

### The trigger is a version tag — publishing is a deliberate act

The release runs ONLY on a `v*.*.*` tag push, never on a branch — so cutting a release is an explicit `git tag` + `git push`, not a side effect of merging. `docker/metadata-action` turns the tag `v0.1.0` into `0.1.0` (`type=semver,pattern={{version}}`) plus `latest`. The image build reuses Story 3.9's `Dockerfile` (static-musl → distroless/nonroot), so the published artifact is exactly what was verified locally.

### Publishing is outward-facing — gated on Guy's go-ahead

Pushing to Docker Hub publishes a public image (cacheable, indexable, hard to fully unpublish). The workflow is committed and ready, but the tag that triggers it is NOT created until Guy explicitly approves cutting v0.1.0.

### Docker Hub description sync caveat

`peter-evans/dockerhub-description` updates the repo overview via the Docker Hub API, which historically needs the ACCOUNT password (a push-scoped PAT may be refused). The step is `continue-on-error` so a description-sync failure never fails the release itself; if it fails, set `DOCKERHUB_TOKEN` to an account password (or add a dedicated secret) — the image push still succeeds with the PAT.

### Scope — the pipeline, then the cut

This story delivers the release pipeline and (on approval) the v0.1.0 cut that closes Epic 3. After it, v0.1.0 is a real published artifact and live testing on the NAS begins.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.10: Release 0.1.0 to Docker Hub]
- [Source: .github/workflows/ci.yml — the thin-runner conventions + checkout@v5]
- [Source: Dockerfile · docker/README.dockerhub.md — the image and the overview this publishes]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

### Completion Notes List

- **AC #1** — `release.yml` on `v*.*.*` tags: Buildx + login + `metadata-action` (`{{version}}` + `latest`) + `build-push-action` (linux/amd64, gha cache) pushing `gcorbaz/opencmdb:<version>` and `:latest` with the repo secrets.
- **AC #2** — `dockerhub-description` syncs `docker/README.dockerhub.md` (best-effort; caveat documented).
- **AC #3** — deferred to the actual cut: creating/pushing `v0.1.0` publishes a PUBLIC image, so it waits for Guy's explicit go-ahead. The image itself is already build- and run-verified (Story 3.9).

### File List

- `.github/workflows/release.yml` (new) — tag-triggered Docker Hub release + description sync.
- `crates/opencmdb-bin/src/main.rs` (modified) — `init_tracing` gains a daily-rotating file layer (`build_file_writer`); the guard is held in `main`.
- `crates/opencmdb-bin/Cargo.toml` (modified) — `tracing-appender` (bin-only).
- `docker/docker-compose.yml` · `docker/.env.example` · `docker/README.dockerhub.md` (modified) — `./log` mount + `OPENCMDB_LOG_DIR`/`OPENCMDB_LOG_RETENTION`.
- `.gitignore` (modified) — ignore `docker/log/`.
- `README.md` (modified) — note file logging.

## Change Log

- 2026-07-20 — Implemented Story 3.10 (release pipeline). `release.yml` builds and pushes `gcorbaz/opencmdb:{version,latest}` on a `v*.*.*` tag and syncs the Docker Hub description. The actual v0.1.0 cut (the tag) is held for Guy's explicit go-ahead — publishing a public image is a deliberate act. Status → review.
- 2026-07-20 — Added daily-rotating file logs (Guy's request): `tracing-appender` writes `opencmdb.YYYY-MM-DD.log` to `OPENCMDB_LOG_DIR` (retention default 14) alongside stdout, degrading gracefully if the dir is unwritable. Compose mounts `docker/log`; verified end to end in the real image (log file owned by nonroot uid 65532). Frontier/clippy/fmt green; 65 tests.