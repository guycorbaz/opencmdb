# syntax=docker/dockerfile:1
#
# opencmdb — a distroless, static, non-root single-binary image (D66).
#
# The binary is fully self-contained: the migrations (sqlx `migrate!`), the front-end assets
# (rust-embed) and the translations (rust-i18n `i18n!`) are all embedded at COMPILE time, so the
# runtime layer carries only the binary — no folders, no shell, no package manager.

# ── Builder: a fully static musl binary, built --locked on the pinned toolchain ──────────────
FROM rust:1.96-bookworm AS builder

# musl produces a static binary distroless/static can run; musl-tools provides musl-gcc, which
# ring (the rustls backend) needs to compile its C.
RUN apt-get update \
 && apt-get install -y --no-install-recommends musl-tools \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /build
# Copy the toolchain pin FIRST so `rustup target add` targets the exact toolchain cargo will use
# (rust-toolchain.toml switches rustup's active toolchain; adding the target before that would
# add it to the wrong one — the base image's default).
COPY rust-toolchain.toml .
RUN rustup target add x86_64-unknown-linux-musl

COPY . .

# --locked: the committed Cargo.lock is authoritative (CLAUDE.md).
RUN cargo build --release --locked --target x86_64-unknown-linux-musl -p opencmdb-bin

# ── Runtime: distroless static, non-root (D66 — CA certs + tzdata + a nonroot user) ──────────
FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/opencmdb /usr/local/bin/opencmdb
# Documentation only under host networking, but correct for a bridge deployment.
EXPOSE 8080
USER nonroot
ENTRYPOINT ["/usr/local/bin/opencmdb"]
