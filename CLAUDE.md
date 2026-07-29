# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project status

`opencmdb` is a self-hosted, single-binary **Rust** network reconciliation engine (IPAM + light app CMDB + topology) for home-lab/SMB. Core thesis: continuously compare the **observed** state (auto-discovered) against the **declared** state (documented); the gap is the product. **Planning is COMPLETE** (product brief, PRD, UX spec, architecture — all in `_bmad-output/planning-artifacts/`, decision register D1–D66). **As of 2026-07-22 the code RUNS**: `v0.1.1` is tagged and published to Docker Hub as `gcorbaz/opencmdb`, and it scans a CIDR and shows a real observed-vs-declared gap on one page. **Epics 1–4 of 23 are done** (the walking skeleton shipped as v0.1). **Epic 4 closed on 2026-07-25**: the fixture corpus, the metrics harness and the trap corpus — all written BEFORE the identity engine — are committed and locked (25 artefacts, 24 traps across nine families, plus the wire-format spec). Its story 4.19 was SPLIT at closure: 4.19a shipped, **4.19b (the mutation generator) moved to Epic 11** — recorded in `epic-4-correct-course-2026-07-25.md` and GitHub issue #34, not silently. **Epic 5 — the identity engine — is IN PROGRESS since 2026-07-27** (15 stories). Its **three inherited debt stories come first and are all `done`** — 5.1, 5.2 and 5.2b, PRs #41, #44 and #46, all merged 2026-07-28. The engine proper starts at **5.3**, which ships the identity engine's own abstention vocabulary and no engine (PR #48). _(This sentence named only 5.1 until 2026-07-29 — stale for a day, found by story 5.3's code review, not by the two stories that caused the drift.)_ Live status is `_bmad-output/implementation-artifacts/sprint-status.yaml`; grounding is `docs/project-context.md`.

### Build / lint / test commands (the stack is chosen and building)

- **Build:** `cargo build --workspace --locked` (Cargo.lock is committed; always `--locked`).
- **Test:** `cargo test --workspace`.
- **Lint:** `cargo clippy --workspace -- -D warnings` · **Format:** `cargo fmt --all`.
- **Project gates:** `cargo xtask ci` — every gate lives here in Rust, never in YAML (D56/D65): the dependency-frontier check (D47), the DDL binary-collation grep (D64 cond. 1), the retired-vocabulary check (D65), and the fixture corpus lock (sha256 **and** orphan detection, both directions). The `architecture-views.md` staleness hash reports `ℹ STALE` and still exits 0 — that is by design, and **it must not be regenerated inside a story**; regenerate at a milestone. *(Some other xtask subcommands are still stubs.)*
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
4. Per story: `bmad-create-story` → **`bmad-create-story validate` (MANDATORY, see below)** → `bmad-dev-story` (or `bmad-quick-dev`) → `bmad-code-review` → `bmad-retrospective`

**`create-story validate` is NOT optional here** (Guy's decision, Epic 4 retrospective 2026-07-26 — it overrides the story template's "Validation is optional" banner). Every story gets a validation pass by **two fresh-context agents** (fact-check + gap-hunt) *before* `dev-story`. Self-review finds nothing; the measured payoff over the 9 Epic-4 stories that had it was **6 HIGH findings on 4 stories**, two of which would otherwise have shipped a trap that passed for the wrong reason.

**A cause needs a check, not a plausible story.** A symptom may be recorded from observation; a *cause* may not be written down without naming the check that would have failed if the cause were wrong. This is the code rule ("name the test behind every claim") extended to environment and infrastructure diagnoses — Epic 4 recorded a confident Synology-Drive explanation for a flaky local suite that measurement later refuted, and four documents carried it (issue #38).

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

### Engineering conventions (2026-07-23)

Four rules Guy set as durable. The first two are `cargo xtask ci` / lint gates; the last two are review criteria the `bmad-code-review` layers enforce.

- **DRY — mutualise logic, but keep DELIBERATE redundancy.** No accidental duplication of logic; extract a shared helper. But this codebase has redundancy that is *on purpose* and must survive: a test that restates the corpus bytes as a second independent oracle (`fixtures.rs`'s `expected()`), two representations of one concept pinned by an equality test (`score.rs`'s `Column::as_str()` vs `Expectation::column()`), the per-test-module `scratch_dir`. DRY means "one source of truth for behaviour," never "one representation of everything" — do not collapse a redundancy that a test pins and a comment labels as deliberate.
- **No source file over 2000 lines of CODE.** Enforced by the `file-size` gate in `cargo xtask ci`. **Tests do not count** — the ceiling is the lines before the first `#[cfg(test)]` (the house convention is one trailing test module per file, D56b). A file approaching the ceiling is split into modules or a sub-crate, not grown. The gate names the offender and its count.
- **Document every public item, per rustdoc idiom.** Every `pub` struct, enum, **field**, **variant**, and function carries a `///` doc comment. Idiomatic rustdoc, NOT `@param`/`@return`: prose that says what the thing is; a `# Errors` section on a `Result`-returning fn; `# Panics` where relevant; `# Arguments`/`# Returns` only when the signature is not self-evident. **A doc comment must be TRUE** — three reviews caught doc comments asserting behaviour the code did not have; a false doc is a defect, so prefer the weaker true sentence. Enforcement is phased: `#![deny(missing_docs)]` is ON for `opencmdb-bin` and `xtask` (both already clean); `opencmdb-core` has ~70 outstanding field/variant docs (mostly `observation`) and gains the lint once that sweep lands — the CI clippy gate runs `-D warnings`, which would promote a `#![warn]` to a hard error before then, so the lint waits rather than blocks.
- **Test every function, where possible.** Tests live inline in a trailing `#[cfg(test)] mod tests` (D56b). Prove-to-red is the house rule (story 1.3): a guard is observed failing before it passes, and the mutation is recorded. "Where possible" excuses the genuinely untestable (a `Display` impl exercised elsewhere, `main`) — it does not excuse a new guard shipping without a test that reds when it is removed.
