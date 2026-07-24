# fixtures/scenario/traps/

This directory holds the **truth labelling**: which observations a case judges, which of D18's three
columns is correct — `must-merge`, `must-not-merge`, `must-abstain` — and the author's mandatory
one-sentence reason. A trap references observations by their stable `obs_id`, never by line number,
and a reference to an observation its stream does not contain is refused. See `example.toml` for the
format, `../README.md` for the `scenario/` vs `capture/` split, and `../../README.md` for the corpus
lock. The trap FAMILIES arrive from story 4.9 onward; today only `example.toml` lives here.

This file is also the **reality-debt register** — the honest record of what these traps do NOT cover.

## The honest limit

A trap proves the engine against a case someone thought to write down. It proves nothing about the
case nobody did. That gap is not a detail; it is stated on the record, in D18's own words:

> *"a trap suite proves nothing about what I failed to imagine. At v0.1 the gate is weak and honest
> rather than strong and false."* — D18

Coverage is not completeness. ~50 traps in three columns, each present in positive AND negative form,
make the release gate binary and zero-tolerance (D18) — and they say nothing about the cluster the
author never pictured. Admitting that here is the point: a weakness written down is a weakness a
maintainer can close, not one a user discovers first.

## The register

This register is the **queue** of real-world cases the corpus cannot yet produce. It is where a case
that reality showed us — but no trap expresses — waits until it becomes a trap. **It is the queue from
which trap #51 and beyond are drawn.**

Each entry records the case as a **pattern** (never a real address — see below), **where it came
from**, the D18 column(s) it will assert once written, and — once drawn — the **trap it became**, so
the queue drains rather than only grows:

| Case (pattern) | Source | D18 column(s) | Status → becomes |
|---|---|---|---|
| _e.g. a printer and its print-server share a NIC via SR-IOV, distinct MACs, one chassis_ | _e.g. Tier 2 bulk run, 2026-Qn_ | `must-merge` (+ `must-not-merge` sibling) | `open` → `trap #NN` / family `shared-nic-sriov` |

**The row above is the FORMAT, not a case.** The register opens **empty**: no Tier 2 bulk run has
happened at v0.1, so there is nothing yet to record. Inventing a case here — writing down something
we did not actually meet — would manufacture false coverage, the exact dishonesty this register
exists to prevent. A case is added only when reality produces one.

## Tier 2 is how the unimagined is found

The corpus cannot list what it failed to imagine — by definition. Something outside it has to notice.
That something is **Tier 2**.

> **OBSERVABILITY = Tier 2** (bulk, 300 hosts / 36 subnets), published per release with confidence
> intervals, trended — **blocking nothing**.

Tier 2 gates no release. It measures the engine at scale, in the open, and its job here is to surface
the case the traps missed:

> *"Tier 2 is Tier 1's trap factory. The gate only grows by proof. The day the bulk drops a cluster
> Tier 1 did not foresee, that cluster becomes trap #51."* — D18

The direction is one-way and deliberate: **Tier 2 feeds this register; this register feeds the gate.**
Tier 2 is the only discovery mechanism for the unimagined — *that is why it lives* (D18). A case it
surfaces lands here first, then graduates into a trap in both forms (story 4.9's discipline), and only
then tightens the gate.

## This is not `deferred-work.md`

Two registers, two different debts, kept apart on purpose:

- `../../../_bmad-output/implementation-artifacts/deferred-work.md` records known **engineering** gaps
  deferred from story reviews — code that exists but is incomplete, each owned by a later story.
- **This** register records **spec** gaps — real-world cases the corpus never imagined, discovered by
  Tier 2, each destined to become a future trap.

A finding that "the code does not yet handle X" belongs in `deferred-work.md`. A finding that "reality
contains a case Y we never wrote a trap for" belongs here.

## Never real network data

Synthetic values only: RFC 5737 documentation addresses (`192.0.2.0/24`), locally-administered MACs,
invented hostnames. A recorded case describes the PATTERN — *"two randomized MACs, one physical
interface"* — never a captured MAC, hostname or IP. This repository is public, and a real capture
would carry the topology of someone's network. That is disqualifying, not a preference (D19).
