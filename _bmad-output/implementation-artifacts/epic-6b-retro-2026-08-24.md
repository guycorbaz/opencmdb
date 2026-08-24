# Epic 6b retrospective — the interface

**Held 2026-08-24, after `v0.2.0` was cut and published.** Thirteen stories, all `done`. The
retrospective is what closes an epic's *how*; the **project review** that follows closes its *what*,
and neither is sufficient alone — at Epic 5's close the retrospective looked finished and the review
then found milestone J3 unreachable by construction.

---

## 1. What the epic delivered

`v0.1.1` served **one page**. `v0.2.0` serves **ten screens** in the reference design, with a triage
inbox on the real observed-vs-declared gap, a keyboard layer, the copy in two languages, and **two
browser gates in CI** the repository did not have at all.

Thirteen stories: 6b.1–6b.12 plus the **6b.4b insertion** (the action bar, split out at 6b.4's
validation). Every one was contexted, validated by two fresh-context layers, developed, and reviewed
by three isolated layers before merge.

⚠️ **And not one gesture acts.** The count of *well-lit dead ends* ran from **four to ten** across
the epic. `v0.2.0` is installable and an operator can look, filter, sort, open a record and go back
— and write nothing. That is the honest sentence, and §5 says what was decided about it.

---

## 2. Epic 5's action items — 2 done, 1 partial, 3 not addressed

| # | Action | State | Measured |
|---|---|---|---|
| 1 | The promote gesture moves to the FRONT of Epic 6 | ✅ **done** | issue #85 open as the record; stories 6.1 and 6.2 shipped the write route |
| 2 | Carry the named defect class into every prove-to-red | ⚠️ **diagnostic, not preventive** | it recurred through the epic — five guards in 6b.1, a totals comparison in 6b.5, the marker guard in 6b.11, the *check* of 6b.12 |
| 3 | Add *"what can the operator DO?"* to the review mandate | ✅ **done and sustained** | five story files carry the question; *well-lit dead ends* appears eight times as a running count |
| 4 | Fix the mutation driver once, in `xtask` | ❌ **not addressed** | **seven** recorded driver defects in this epic |
| 5 | Regenerate `architecture-views.md` | ❌ **not addressed** | `views-hash` still `STALE` today |
| 6 | Decide the fate of the residual worktrees | ❌ **not addressed** | five remain, three with uncommitted edits |

### 🔑 The structural finding, and it is about the mechanism rather than the people

**The three unaddressed actions are exactly the three whose owner was *"Guy"* or *"unassigned"*.**
The two that were done had *"whoever writes a story"* as owner — **the work carried them**. An action
that needs a separate decision is carried by nothing, and nothing reminds anyone of it. That is a
defect of the action-item format, and §5's first decision is the response: the mutation driver
becomes a **story**, not an action.

### ⚠️ And action 2 earns the epic's most useful sentence

***Naming a defect class makes it findable, not avoidable.*** Every review cited the class by name,
and the defect was committed anyway — in 6b.1's five guards, in 6b.5's anti-sum, in the guard story
6b.11 wrote *to close it*, and in 6b.12's own AC4 delivery. The naming was not wasted: it is why the
reviews found each one quickly. But it prevented none of them, and a retrospective that recorded
*"we will carry the class forward"* as a win would be reporting the wrong thing.

---

## 3. The epic's own dominant defect class

**A guard that reads the SOURCE where the defect lives in the RENDER.** Epic 5's class was *a guard
placed where the defect cannot occur*; Epic 6b's is its interface-shaped descendant, and it appeared
in at least five stories:

- **6b.4b** — four times in one story: an attribute assembled in Rust and emitted with `|safe`; a
  bare uppercase `DISABLED`; a YAML block scalar; and `tabindex`, which no source needle could see.
- **6b.6** — a route filter tested through the pure builder while the route ignored its argument.
- **6b.10** — the prose-attribute guard read the template while the defect lived in the producer.
- **6b.11** — the whole of AC5, and the amended criterion is the epic's answer to it.
- **6b.12** — the "prescribed check" that was a reading.

🔑 **The resolution is story 6b.11's AC5, amended on 2026-08-23**: *where the defect lives in the
DOM, a guard that reads the source **does not suffice**; the two cumulate, the source guard naming
the CAUSE and the browser gate the REALITY.* ⚠️ Its stated residual is **proportionality** — it is a
rule about where the defect lives, never about how much apparatus to build.

### The second class: a measurement landing on the wrong artefact

Seven driver defects, plus the stale binary (6b.4b), the stale screenshot (6b.4b), the stale process
holding the port (6b.10), and — in this very epic's last story — screenshots lifted on a check that
did not cover the version the shell renders. 🔑 **Every one was caught the same way: a result
contradicted a prediction written in advance.** Where no prediction existed, the defect would have
been filed as a confirmation.

### The third: the blind review layer wins

**Six consecutive stories** where the layer given only the diff — no repository, no build, no run —
found HIGH findings the two sighted layers missed. It is no longer an impression: it is why the
layer is kept blind. Its mechanism is that it must read the diff **against itself**, cross-checking
hunks that a reader holding the whole tree never thinks to compare.

---

## 4. What only a browser, or an operator's hands, could find

⚠️ **Four stories deferred the visual check on a sentence nobody had run `command -v` against** —
until 6b.4b found Chrome 151 installed all along. *A limit believed is a limit unmeasured.*

And the findings that no text guard could reach: the dashboard's **salience** (invented figures at
22 px against real counts in body text); the applications table showing a divergence without naming
it; `/devices` serving all eight devices under a filter; four identical marker banners down one
page; five controls unreachable by keyboard; and 6b.12's `.env.example` — **found by following the
instructions rather than reading them**, which is the sharpest instrument this epic used.

---

## 5. Decisions taken (Guy, 2026-08-24)

1. 🔑 **The mutation driver becomes a STORY**, not an action item. ⚠️ **This decision read *"at
   the head of Epic 6"* and decision 4 said *"Epic 6 resumes at 6.4"* — a contradiction inside one
   retrospective, surfaced on 2026-08-24 rather than settled in silence.** Guy's sequencing the
   same day: **6.4 first** (two epics without a gesture is the heaviest cost this project carries),
   then the driver as `6-4b`, then the mobile story as `6-4c` — before the engine work, because 6.4
   is the last story that touches the interface. ⚠️ 6.4's own mutation pass therefore runs on the
   unrepaired driver, accepted on the measurement that the known defect is caught by writing the
   prediction first. It must
   refuse two filters, refuse a truncated read, fail when a mutation does not apply, and anchor
   compiler-error detection. *Carrying it as a story is the whole point* — §2's structural finding
   is that actions needing a decision are carried by nothing.
2. 🔑 **The specification wins over the mock, and it is a story.** NFR24's touch targets and the
   **zero `@media` rules** are one arbitration and are taken together: the product becomes usable on
   a telephone — breakpoints, targets ≥ 44 px, density revisited. ⚠️ **This revises Guy's own
   premise (3) of 2026-08-13** (*the mock's palette and typography are adopted*), which only a
   retrospective may do. *Refused:* amending NFR24 to match the mock, which would retire an
   accessibility requirement instead of meeting it.
3. 🔑 **`epics.md` is corrected in three places**, each with its measurement and each traceable —
   see §6. *Refused:* registering the divergences instead, which is precisely what produced a
   criterion false four ways that story 6b.12's contexting had to rediscover.
4. **The ten well-lit dead ends are WRITTEN as a finding and the plan does not change.** Epic 6
   resumes at 6.4, which makes a gesture live; Epic 5's reordering holds. *Refused:* a short slice
   advancing a gesture — new scope where 6.4 already carries it, the same refusal Epic 5 made — and
   re-running the drift count, which the project review that follows does properly.

---

## 6. `epics.md` edited — a retrospective may; a story may not

| Line | Was | Now | Why it mattered |
|---|---|---|---|
| 2324 | *"the epic's **eleven** preceding stories"* | **twelve** | written before the 6b.4b insertion; 6b.12's contexting had to rediscover it |
| 2330 | *"the **eight** example screens"* | **six** | 6b.8 and 6b.9 made two screens real |
| 2332 | *"the manuals, **whose screenshots** show a product that no longer looks like that"* | *they carry **no screenshots at all*** | 🔴 a story hunting images would have found none, concluded the manuals were fine, and shipped `user-manual.tex:151`'s *"A dark theme is the default"* |
| 2308 | *"the arrows and `j`/`k` … `⏎` performs the gesture"* | arrows only, with the arbitration and **Epic 7** named | 6b.11 skipped them by decision and the divergence lived in no register — the epic's criterion would have read as met |
| 1608 | *"each observation carrying a MAC lands on exactly ONE `interface`"* | **one interface PER L1 KEY** | 🔴 falsified by the code it describes since 2026-08-04; story 5.9b could not edit this file and the correction waited **twenty days** in a register |

🔑 **The last row is the argument for decision 3.** A correction that a story is forbidden to make
has no other home than a retrospective, and there was no retrospective between Epic 5's close and
this one for it to land in.

---

## 7. Action items

Owners are stated the way §2 says they must be: **an action that needs a decision becomes a story,
or it is not an action.**

1. **The mutation-driver story**, at the head of Epic 6. **Owner: whoever runs `create-story` next.**
   Success: a driver that exits non-zero when the mutation fails to apply, refuses two filters, and
   cannot report a truncated read.
2. **The mobile story** — breakpoints and touch targets, taking NFR24 and the `@media` gap together.
   **Owner: Epic 6's breakdown**, sequenced by Guy.
3. **Keep the review layer blind, and say why in the mandate.** Six stories of evidence.
   **Owner: whoever runs the next review.** Effective immediately.
4. **Write a prediction before every mutation, and treat a contradicted result as the finding.**
   Every driver defect in this epic was caught that way and only that way.
   **Owner: whoever writes a story.** Effective immediately.
5. **Regenerate `architecture-views.md`** — issue #50, inherited unaddressed from Epic 5, and this
   close is a milestone. ⚠️ **Owner: Guy** — and §2 predicts this will not be done unless something
   carries it. It is named here a second time with that prediction attached.
6. **Decide the five residual worktrees**, three carrying uncommitted edits. ⚠️ **Owner: Guy**, same
   prediction. Nothing is deleted without his word.

---

## 8. Readiness — Epic 6b is complete and NOT closed

- **Stories**: 13 of 13 `done`.
- **Release**: `v0.2.0` tagged on `9079b45`, whose CI was green **on that commit**; the image pulls
  and boots; the GitHub Release carries the changelog; `gh-pages` pushed after the tag and live.
- **Quality**: 729 tests both ways, nine `cargo xtask ci` gates, two browser gates in CI, clippy over
  `--all-targets`, both LaTeX manuals building with zero missing characters.
- ⚠️ **Carried forward, open**: issue #38 with a reproduced candidate cause and no cause named;
  issue #50; the six register rows this epic could not take; and **the operator still cannot write**.

🔴 **The PROJECT REVIEW is what closes the epic**, conducted on `~/travail/Projets actuels/opencmdb/`
under that tree's own `CLAUDE.md`, and it comes **after** this retrospective because it corrects what
this one surfaces. Do not flip Epic 6b to `done` on the story count.
