# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project status

`opencmdb` is a self-hosted, single-binary **Rust** network reconciliation engine (IPAM + light app CMDB + topology) for home-lab/SMB. Core thesis: continuously compare the **observed** state (auto-discovered) against the **declared** state (documented); the gap is the product. **Planning is COMPLETE** (product brief, PRD, UX spec, architecture — all in `_bmad-output/planning-artifacts/`, decision register D1–D66). **As of 2026-07-17 the code exists**: a three-crate Cargo workspace that builds. The identity engine / walking skeleton (story 1) is the next implementation work.

### Build / lint / test commands (the stack is chosen and building)

- **Build:** `cargo build --workspace --locked` (Cargo.lock is committed; always `--locked`).
- **Test:** `cargo test --workspace`.
- **Lint:** `cargo clippy --workspace -- -D warnings` · **Format:** `cargo fmt --all`.
- **Project gates:** `cargo xtask ci` — every gate lives here in Rust, never in YAML (D56/D65): the DDL binary-collation grep (D64 cond. 1), the retired-vocabulary check (D65), the fixture MANIFEST sha256, the `architecture-views.md` staleness hash. *(xtask commands are being implemented; some are stubs.)*
- **Toolchain:** Rust 1.96+, edition 2024. **Stack:** axum 0.8 · askama 0.16 · sqlx `=0.9.0` (MariaDB-only, `mysql`+`tls-rustls-ring`) · tokio · `config` · `rust-i18n` (YAML) · `prometheus` (raw) · Tailwind standalone CLI via `cargo xtask css`. **Never invent a version — pin from the real `Cargo.lock`.**

### The dependency frontier is load-bearing (D47), and it is a gate

`crates/opencmdb-core` is the domain: it **must not** depend on `anyhow`, `axum`, `sqlx`, or `askama` — an error there is domain data, not a string. `crates/opencmdb-bin` is everything that touches the outside world (SQL, HTTP, HTML, files, the clock, secrets). `xtask/` is a workspace member and a dependency of nobody. Do not cross these lines; `cargo xtask ci` is meant to catch it.

### One database only

**MariaDB 10.11+ is the ONLY supported engine (D64).** SQLite and MySQL are OUT; PostgreSQL is out at MVP. Do not reintroduce a second backend or a dialect abstraction. Comparison/normalization never descends into SQL (D10) — it is a correctness rule, not portability.

## How work is done here: the BMad Method

This project uses BMad, a spec-driven, agent-orchestrated development framework. Instead of jumping straight to code, work flows through named agent personas and skills that produce planning artifacts first, then implementation. Agents are invoked as skills (e.g. `/bmad-agent-pm`) or by name.

Key agent personas (from `_bmad/config.toml`):
- **Mary** — Business Analyst (`bmad-agent-analyst`): research, requirements discovery
- **John** — Product Manager (`bmad-agent-pm`): PRD creation
- **Winston** — System Architect (`bmad-agent-architect`): solution/architecture design
- **Sally** — UX Designer (`bmad-agent-ux-designer`)
- **Amelia** — Senior Software Engineer (`bmad-agent-dev`): test-first (red/green/refactor) story implementation
- **Murat** — Test Architect (`bmad-tea`): risk-based test strategy, automation

Typical greenfield lifecycle (each step is a skill — see the skills list, prefix `bmad-`):
1. `bmad-product-brief` / `bmad-domain-research` → `bmad-create-prd` → `bmad-validate-prd`
2. `bmad-create-ux-design` → `bmad-create-architecture`
3. `bmad-create-epics-and-stories` → `bmad-sprint-planning`
4. Per story: `bmad-create-story` → `bmad-dev-story` (or `bmad-quick-dev`) → `bmad-code-review` → `bmad-retrospective`

Use `bmad-help` when unsure which skill applies next.

## Repository layout

- `_bmad/` — BMad framework: agents, workflows, and skills for modules `core`, `bmm` (core dev lifecycle), `bmb` (builder), `cis` (creative intelligence), `tea` (test architect). **Installer-managed — treat as read-only.** Edits here are overwritten on the next install.
- `_bmad/config.toml`, `_bmad/config.user.toml` — installer-generated config. **Do not edit directly.** To change values durably use `_bmad/custom/config.toml` (team, committed) or `_bmad/custom/config.user.toml` (personal, gitignored).
- `_bmad-output/` — where BMad writes generated artifacts: `planning-artifacts/`, `implementation-artifacts/`, `test-artifacts/`.
- `docs/` — project knowledge base (`modules.bmm.project_knowledge`). Includes `docs/manuals/` — the LaTeX **User Manual** and **Administrator Manual** (English, LuaLaTeX; shared style in `docs/manuals/common/`, one folder per manual; build with `make` — see `docs/manuals/README.md`).
- `.claude/skills/` — installed BMad skill definitions (SKILL.md, instructions.md, templates, checklists).

## Conventions

- **Communication language is French** (`config.user.toml`, user "Guy"). Converse with the user in French unless they switch.
- **Document/artifact output language is English** (`config.toml` → `document_output_language`). Generated PRDs, specs, and docs should be written in English.
- Application code, once it exists, should be placed at the project root (not under `_bmad*`). When a stack is chosen, add its build/lint/test commands to this file.
- **Issue tracking: all bugs, change requests, and other issues are recorded as GitHub Issues** on `guycorbaz/opencmdb` — never tracked only in local notes, commit messages, or planning docs. Reference the issue number in the related commit/PR (e.g. `Fixes #12`). This is the single source of truth for work items outside the BMad story flow.
- **Docs-current-before-push:** before any `git push`, make sure every affected document is updated and matches what is being pushed — the manuals (`docs/manuals/`), `README.md`, the GitHub Pages landing site (`gh-pages` branch), `docs/project-context.md`, and this file. A push whose docs contradict the code/state is not ready. If a change touches behaviour, stack, or layout that a document describes, update that document in the same push.
