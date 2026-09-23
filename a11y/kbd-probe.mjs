// The keyboard gate — the arrow layer, the focus contract, and what must NOT be bound.
//
// 🔴 **UNTIL STORY 6b.11's REPAIR PASS THIS FILE WAS RUN BY NOTHING, and all three review
// layers reached that independently.** It was in no `package.json` script and in no CI step,
// so `crates/opencmdb-bin/assets/app.js` — the story's central deliverable — had no automated
// carrier of any kind: emptying the whole file left **490 tests, nine gates and the axe gate
// green**. ⚠️ It was also the very shape `axe-gate.mjs` builds its route count to prevent —
// *a harness that derives nothing and reports success* — reproduced one file over: it gated
// its real checks on the queue being long enough and printed `TOUT VERT` with exit 0 when it
// had measured almost nothing.
//
// 🔑 **So it now has the same three-way contract as the axe gate, and a FLOOR of its own:**
//
//   0 — every check passed
//   1 — a check failed                        (a real regression: fix the product)
//   2 — the gate could not run                (fix the harness, the environment or the seed)
//
// A queue too short to exercise the layer is **2**, never 0: *"the store was empty"* and
// *"the keyboard layer is correct"* must not be the same answer. And the number of checks
// actually EXECUTED is compared with `MIN_CHECKS`, so a future edit that skips half of them
// cannot report success.
//
// ⚠️ Written in English, like every other artefact in this repository. It was French —
// comments and operator-facing output both — with the credentials and the base URL hardcoded
// and a typo in one label, which is what an artefact nobody runs drifts into.
import { readFileSync } from "node:fs";

const BASE = process.env.AXE_BASE_URL ?? "http://127.0.0.1:8080";
const USER = process.env.OPENCMDB_BASIC_USER ?? "";
const PASS = process.env.OPENCMDB_BASIC_PASSWORD ?? "";
const CHROME = process.env.AXE_CHROME ?? "/usr/bin/google-chrome";
const QUEUE = ".queue .queue-row > a";
// The settle in `app.js` is 250 ms; everything here waits past it with room for a document.
const SETTLE_WAIT_MS = 900;
// 🔑 The floor, and it EQUALS what is there rather than sitting under it: **sixty-one** checks run
// on a queue of two, which is the shortest queue this gate accepts. ⚠️ This sentence said *twenty*
// until story 14.4's slice-D review — story 6b.11's figure, left behind by every floor move since —
// so the ONE place a reader verifies *the floor equals what is there* asserted a number 33 short of
// the constant beneath it. ⚠️ And the repair's first version said *fifty-two*: the same review added
// a check in the same breath, so the corrected sentence was stale before it was saved. **A floor is
// a MINIMUM, so that drift reds nothing** — it just quietly stops equalling what is there, which is
// the whole property. 🔴 **AND IT DRIFTED A THIRD TIME, in the story that moved the constant**: story
// 14.5 took it 59 → 61 (the subnet correction's pre-fill and the outside list's release) and left
// every sentence here saying fifty-nine — caught by the code review's edge layer on a live run, not
// by this comment, which narrates the same defect twice above. The number below is read off
// `kbd gate: 61 check(s) run` rather than counted by hand. A floor under what exists tolerates losing a check while still reading as a pass
// — this project has caught that twice, once in a privacy floor and once in a word count. If a
// check is added this number moves deliberately; if one is skipped, the gate says so instead of
// printing a green.
// 🔑 61 → 62 on 2026-09-23: the confirmation's own words, added where the gate already presses the
// gesture. The number below is READ OFF the run, never counted by hand — see the sentence above.
const MIN_CHECKS = 62;
const MIN_ROWS = 2;
// 🔑 The seed's own two-hardware-address sighting, in ONE place. It was written twice — typed into
// the field at one site and spelled out inside the expected triage href at another — so a seed that
// moved the address would have left the two disagreeing, with the gate asserting a link to a
// question about an address it had not typed.
const SEEN_ADDRESS = "192.0.2.20";
/// How long a navigation or a response may take before the gate calls it *could not run*.
const NAV_TIMEOUT_MS = 20_000;
// The product's one live control — a CLASS the stylesheet guard already pins, never a label
// and never a selector vocabulary. Story 6.4; see the block that presses it.
const GESTURE = "button.btn-document";

/** The gate could not run. Never conflated with "a check failed". */
class CannotRun extends Error {}
function cannotRun(message) {
  throw new CannotRun(message);
}

let executed = 0;
let failed = 0;
function check(ok, label, detail) {
  executed += 1;
  console.log(`${ok ? "✅" : "🔴"} ${label}${detail ? `  ${detail}` : ""}`);
  if (!ok) failed += 1;
}
const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

let browser;

async function main() {
  const { default: puppeteer } = await import("puppeteer-core");
  // 🔴 **The SOURCE on disk, read so it can be COMPARED with what the server serves.**
  // ⚠️ **And this comparison is UNREDDENABLE in the configuration CI runs, which is stated
  // rather than left to be discovered**: `rust-embed` reads assets from disk in a debug build,
  // measured, so served and source are the same bytes by construction and no mutation of the
  // source can separate them. It is a tripwire for a RELEASE build, where the assets are
  // embedded and the two genuinely can diverge — the shape this project's stale-binary
  // incidents took. Kept because it costs one fetch; recorded as green-by-construction because
  // a check nobody can red must say so. This
  // read the file and threw the contents away, under a comment claiming *"the served file and
  // the source must be the same thing"* and invoking this project's own stale-binary incident
  // by name — a guarantee the code could not deliver, since it never touched the binary or an
  // HTTP response. The comparison happens below, once a page exists.
  const sourceJs = readFileSync(
    new URL("../crates/opencmdb-bin/assets/app.js", import.meta.url),
    "utf8",
  );

  try {
    browser = await puppeteer.launch({
      executablePath: CHROME,
      args: ["--no-sandbox", "--disable-gpu"],
    });
  } catch (error) {
    cannotRun(`Chrome would not start at ${CHROME} — ${error.message}`);
  }

  async function open(route) {
    const page = await browser.newPage();
    if (USER !== "" || PASS !== "") {
      await page.authenticate({ username: USER, password: PASS });
    }
    let response;
    try {
      response = await page.goto(BASE + route, { waitUntil: "networkidle0" });
    } catch (error) {
      cannotRun(`${route} did not answer — ${error.message}`);
    }
    if ((response?.status() ?? 0) !== 200) {
      cannotRun(`${route} answered ${response?.status() ?? 0}`);
    }
    return page;
  }

  const focusedIndex = (page) =>
    page.evaluate(
      (sel) => [...document.querySelectorAll(sel)].indexOf(document.activeElement),
      QUEUE,
    );

  let page = await open("/triage");
  // The stale-binary check, and it is not decoration: `cargo test` builds the test target, not
  // `target/debug/opencmdb`, so a probe can measure a server carrying yesterday's asset. Assets
  // are read from disk in a debug build, which is exactly why this must be MEASURED rather than
  // assumed — a release build embeds them and the two can then diverge.
  const servedJs = await page.evaluate(async () => {
    const response = await fetch("/assets/app.js");
    return response.ok ? await response.text() : null;
  });
  check(
    servedJs === sourceJs,
    "the server serves the keyboard layer this checkout contains",
    servedJs === null
      ? "/assets/app.js did not answer"
      : `served ${servedJs.length} bytes, source ${sourceJs.length}`,
  );
  const rows = await page.$$eval(QUEUE, (all) => all.length);
  console.log(`queue: ${rows} row(s)\n`);
  if (rows < MIN_ROWS) {
    cannotRun(
      `/triage carries ${rows} queue row(s) and this gate needs at least ${MIN_ROWS}. ` +
        `An empty store is not a passing keyboard layer — seed it.`,
    );
  }

  // ── The arrow moves focus, the highlight and the accessible state TOGETHER ──
  await page.evaluate(() => document.body.focus());
  await page.keyboard.press("ArrowDown");
  const first = await focusedIndex(page);
  check(first >= 0, "arrow down: focus lands inside the queue", `index=${first}`);
  const together = await page.evaluate((sel) => {
    const links = [...document.querySelectorAll(sel)];
    const active = links.indexOf(document.activeElement);
    const highlighted = [...document.querySelectorAll(".queue .queue-row")].findIndex(
      (row) => row.classList.contains("selected"),
    );
    const announced = links.findIndex(
      (link) => link.getAttribute("aria-current") === "true",
    );
    return { active, highlighted, announced };
  }, QUEUE);
  check(
    together.highlighted === together.active,
    "the highlight follows the focus immediately",
    `focus=${together.active} highlight=${together.highlighted}`,
  );
  // 🔴 The one the code review found: the class moved and `aria-current` did not, so for the
  // whole settle window the eye and the screen reader named different rows.
  check(
    together.announced === together.active,
    "aria-current follows the focus immediately, so the screen reader and the eye agree",
    `focus=${together.active} aria-current=${together.announced}`,
  );

  // ── The URL catches up, the focus survives it, and history does not grow ──
  const historyBefore = await page.evaluate(() => window.history.length);
  const urlBefore = page.url();
  await wait(SETTLE_WAIT_MS);
  check(page.url() !== urlBefore, "the URL catches up after the settle", page.url().replace(BASE, ""));
  const afterSettle = await page.evaluate(
    (sel) => ({
      active: document.activeElement === null ? "null" : document.activeElement.tagName,
      row: [...document.querySelectorAll(sel)].indexOf(document.activeElement),
      history: window.history.length,
    }),
    QUEUE,
  );
  // 🔴 Focus dropped to <body> at every settle: the arrows kept working via the
  // `aria-current` fallback, but one Tab threw the operator back to the navigation.
  check(
    afterSettle.row >= 0,
    "the operator keeps their focus position across the settle navigation",
    `activeElement=${afterSettle.active} row=${afterSettle.row}`,
  );
  // 🔴 `assign` pushed one history entry per settled press, so Back walked the operator's own
  // selections and could not leave /triage — on the screen whose template justifies its ARIA
  // choice by keeping the back button.
  check(
    afterSettle.history === historyBefore,
    "a settled arrow REPLACES rather than stacks, so Back still leaves the screen",
    `history ${historyBefore} → ${afterSettle.history}`,
  );

  // ── The operator's own gesture outranks the pending navigation ──
  for (const [label, cancel] of [
    ["a pointer press", async (p) => p.evaluate(() => document.body.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true })))],
    ["a non-arrow key", async (p) => p.keyboard.press("Escape")],
  ]) {
    const p = await open("/triage");
    await p.evaluate(() => document.body.focus());
    await p.keyboard.press("ArrowDown");
    const url0 = p.url();
    await cancel(p);
    await wait(SETTLE_WAIT_MS);
    check(
      p.url() === url0,
      `${label} inside the settle window cancels the queued navigation`,
      p.url().replace(BASE, ""),
    );
    await p.close();
  }
  // The control, without which the two above are satisfied by a layer that never navigates.
  {
    const p = await open("/triage");
    await p.evaluate(() => document.body.focus());
    await p.keyboard.press("ArrowDown");
    const url0 = p.url();
    await wait(SETTLE_WAIT_MS);
    check(p.url() !== url0, "CONTROL: with nothing to cancel it, the arrow does navigate", p.url().replace(BASE, ""));
    await p.close();
  }

  // ── A stale marker does not steal focus on a load nobody drove with the keyboard ──
  // 🔴 The marker carried a bare "1" and outlived the navigation that wrote it, so a settle
  // whose document never committed left it sitting there and the NEXT plain load of /triage
  // pulled focus into the queue — the autofocus the design refuses. Planted verbatim here.
  // ⚠️ **TWO plants, and the first draft of this check had only the second — measured GREEN
  // under the very reversion it exists to catch.** Planting the marker's CURRENT
  // representation for a different address is refused by the OLD bare-sentinel code too, for
  // its own reason, so that plant proves nothing about the reversion. The property is
  // representation-free and both plants state it: *no pre-existing marker value whatsoever may
  // restore focus on a load the keyboard did not drive.* `"1"` is the value the old code
  // accepted unconditionally, which is what makes it the plant that reds.
  for (const [label, planted] of [
    ["the old bare sentinel", "1"],
    ["another address", "/triage?sel=not-this-address"],
  ]) {
    const p = await open("/triage");
    await p.evaluate(
      (value) => window.sessionStorage.setItem("opencmdb.kbd.restore", value),
      planted,
    );
    await p.goto(`${BASE}/triage`, { waitUntil: "networkidle0" });
    const stolen = await p.evaluate((sel) => ({
      inQueue: [...document.querySelectorAll(sel)].includes(document.activeElement),
      markerLeft: window.sessionStorage.getItem("opencmdb.kbd.restore"),
    }), QUEUE);
    check(
      stolen.inQueue === false && stolen.markerLeft === null,
      `a marker left over from ${label} is consumed WITHOUT stealing the focus`,
      `focus in queue=${stolen.inQueue} marker left=${stolen.markerLeft}`,
    );
    await p.close();
  }

  // ── A net movement of zero costs no document ──
  // ⚠️ It has to start from a page where a row is ALREADY current — the row's own address,
  // which is what the operator is looking at after any selection. From a bare `/triage`
  // nothing is current, so ↓ then ↑ lands on row 0 having started from no row at all, and
  // that IS a movement. The first draft of this check started there and reddened a correct
  // product: written from the finding's summary instead of from the input it names.
  {
    const seed = await open("/triage");
    const firstHref = await seed.$$eval(QUEUE, (all) => all[0].getAttribute("href"));
    await seed.close();
    const p = await open(firstHref);
    await p.evaluate(() => {
      window.__opencmdbMark = true;
      document.body.focus();
    });
    await p.keyboard.press("ArrowDown");
    await p.keyboard.press("ArrowUp");
    await wait(SETTLE_WAIT_MS);
    const survived = await p.evaluate(() => window.__opencmdbMark === true);
    check(
      survived,
      "down then up back to the row already current replaces no document",
      `mark kept=${survived}`,
    );
    await p.close();
  }

  // ── The focus ring is the product's own, on every focusable kind ──
  {
    const p = await open("/triage");
    const ring = await p.evaluate((sel) => {
      const link = document.querySelector(sel);
      link.focus();
      const style = getComputedStyle(link);
      return `${style.outlineWidth} ${style.outlineStyle} ${style.outlineColor}`;
    }, QUEUE);
    check(
      !ring.startsWith("1px") && ring.includes("solid"),
      "the focus ring on a queue row is a rule of the product's, not the browser's default",
      ring,
    );
    await p.close();
  }

  // ── Inert wherever there is no queue — every such screen, not a sample of three ──
  {
    const p = await open("/triage");
    const others = await p.$$eval("nav.nav a.nav-entry", (entries) =>
      entries.map((entry) => entry.getAttribute("href")).filter((href) => href !== "/triage"),
    );
    await p.close();
    if (others.length === 0) cannotRun("the navigation offers no screen other than /triage");
    let inert = 0;
    for (const route of others) {
      const q = await open(route);
      const seen = await q.evaluate((sel) => {
        const event = new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true });
        document.body.dispatchEvent(event);
        return { prevented: event.defaultPrevented, rows: document.querySelectorAll(sel).length };
      }, QUEUE);
      if (seen.prevented === false && seen.rows === 0) inert += 1;
      await q.close();
    }
    check(
      inert === others.length,
      `the layer leaves the arrow to the browser on all ${others.length} screens with no queue`,
      `inert=${inert}/${others.length}`,
    );
  }

  // ── Focus in the navigation is outside the queue ──
  {
    const p = await open("/triage");
    const url0 = p.url();
    await p.evaluate(() => document.querySelector("nav.nav a.nav-entry").focus());
    await p.keyboard.press("ArrowDown");
    await wait(SETTLE_WAIT_MS);
    check(p.url() === url0, "an arrow with focus in the NAVIGATION moves nothing", p.url().replace(BASE, ""));
    await p.close();
  }

  // ── No letter, no ⏎, no ⌫ — measured at BOTH ends, each with its positive control ──
  // ⚠️ At an end only one arrow is bound (the other leaves the press to the browser, so the
  // page still scrolls), which is why this is two checks and not one at a "middle" index —
  // a middle index needs a third row this gate does not require.
  {
    const p = await open("/triage");
    const inert = ["a", "j", "k", "x", "Enter", "Backspace", "Home", "PageDown", " "];
    for (const [where, index, bound] of [["the FIRST row", 0, "ArrowDown"], ["the LAST row", rows - 1, "ArrowUp"]]) {
      const seen = await p.evaluate(
        ({ sel, index, keys }) => {
          const links = [...document.querySelectorAll(sel)];
          links[index].focus();
          const out = {};
          for (const key of keys) {
            const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
            links[index].dispatchEvent(event);
            out[key] = event.defaultPrevented;
          }
          return out;
        },
        { sel: QUEUE, index, keys: [...inert, bound] },
      );
      check(
        inert.every((key) => seen[key] === false),
        `no letter, ⏎ or ⌫ is bound at ${where}`,
        JSON.stringify(seen),
      );
      check(seen[bound] === true, `CONTROL: ${bound} IS bound at ${where}`, `${bound}=${seen[bound]}`);
    }
    await p.close();
  }

  await page.close();

  // ── The reach section is POPULATED where the gates run, and every line answers ──
  //
  // 🔴 **MEASURED, and it is why this check exists: axe walks this section and asserts nothing
  // about it.** Story 6.4 seeded identity abstentions so the section would render its cause
  // lines — each carrying the sentence that says why it offers no documenting gesture — and
  // with that seed block removed the page served ZERO of those sentences while the axe gate
  // exited 0 over the same URL. *A gate that walks a page is not a gate that reads it.*
  //
  // ⚠️ Its limit: it says the lines are THERE and carry a sentence, never which sentence. The
  // wording is pinned in Rust, where the render assertion can name a phrase of the
  // translation; here the property is that the section is not silently empty.
  {
    const p = await open("/triage");
    const reach = await p.$$eval(".identity .abstentions.causes li", (rows) =>
      rows.map((row) => ({
        cause: (row.querySelector(".cause")?.textContent ?? "").trim(),
        why: (row.querySelector(".why")?.textContent ?? "").trim(),
      })),
    );
    await p.close();
    // 🔴 **AN EMPTY SECTION IS *THE GATE COULD NOT RUN*, NEVER *THE PRODUCT REGRESSED*.** This
    // read `check(reach.length > 0 && …)` until story 6.4's code review measured what that costs:
    // with the seed's `identity_link` block removed — a HARNESS shortfall — the gate printed *"the
    // keyboard layer has regressed"* over a correct product. The 0/1/2 contract is this project's
    // own invention and its whole point is that those two never wear each other's clothes.
    if (reach.length === 0) {
      cannotRun(
        `/triage renders no identity cause line, so the sentences this checks are on no ` +
          `page. Seed the store (a11y/seed.sql writes the abstentions) before the gate runs.`,
      );
    }
    check(
      reach.every((row) => row.cause !== "" && row.why !== ""),
      "every identity cause line on the served page carries the sentence that says why it offers no gesture",
      `${reach.length} line(s): ${JSON.stringify(reach.map((r) => r.why.slice(0, 24)))}`,
    );
  }

  // ── The gesture that ACTS: pressed by keyboard, and the focus follows the swap ──
  //
  // 🔴 **STORY 6b.11 REGISTERED THE FOCUS-AFTER-SWAP CONTRACT TO STORY 6.4 BECAUSE NO SWAP
  // EXISTED TO ATTACH IT TO.** It exists now, and it is the one behaviour in this product that
  // no Rust test can reach: `hx-on::after-swap` is a string in the served HTML until a browser
  // runs it, so a render assertion measures that the ATTRIBUTE is there and nothing about what
  // happens when the operator presses ⏎. A keyboard operator who presses a control and is left
  // where they were has no way to reach what just appeared — announcing is not reaching, which
  // is why `aria-live` and the focus move are both required and neither substitutes.
  //
  // ⚠️ **THIS BLOCK WRITES TO THE STORE** — and it is NOT the only one, which this line claimed until
  // story 14.4b's code review: the plan forms' block writes a subnet, and the release block presses a
  // release. It adopts
  // the undeclared sighting `a11y/seed.sql` plants. So it must stay LAST, or be preceded by a
  // re-seed — a second run against the same store answers 409, htmx swaps nothing on a non-2xx,
  // and the focus checks below would then red over a product that is working. That is a HARNESS
  // failure and it is reported as one: the response status is read off the wire so the two
  // cannot be confused.
  //
  // 🔑 **The row is FOUND, never named** — each pane is opened and asked whether it carries the
  // control, the same idiom `axe-gate.mjs` uses. Matching the `nouveau:` selector would couple
  // this gate to a Rust identifier and matching *New* / *Nouveau* would be story 6b.3's
  // `role_key` defect: a real, resolving, wrong value that every shape check passes.
  {
    const hrefs = await (async () => {
      const p = await open("/triage");
      const all = await p.$$eval(QUEUE, (rows) => rows.map((r) => r.getAttribute("href")));
      await p.close();
      return all;
    })();
    let gesturePage = null;
    for (const href of hrefs) {
      const p = await open(href);
      if ((await p.$$eval(GESTURE, (found) => found.length)) > 0) {
        gesturePage = p;
        break;
      }
      await p.close();
    }
    if (gesturePage === null) {
      cannotRun(
        `no queue row carries \`${GESTURE}\`, so the product's only live gesture is on no page ` +
          `this gate can press. Either the store holds no undeclared sighting, or the server ` +
          `was started without OPENCMDB_DOCUMENT_ENABLED.`,
      );
    }

    // 🔴 **THE AMBER, MEASURED WHERE IT PAINTS RATHER THAN WHERE IT IS DECLARED.** Story 6.4's
    // own guard counts `var(--accent-document)` reads in the SHEET and a browser says whether
    // any of them reaches a pixel: measured before this check existed, `.btn-gesture.live`
    // (specificity 0-2-0) beat `.btn-document` (0-1-0) on all four declarations, so the
    // product's primary control computed to `rgb(233,233,234)` on `rgb(29,31,32)` at weight
    // 400 — plain grey — while the sheet, the count and every Rust assertion stayed green.
    //
    // 🔑 **The token is READ, never spelled here.** Hard-coding `#8d5e2d` would make this an
    // enumeration that goes stale the day the palette moves; comparing the control against the
    // value `:root` actually carries is a property of the reservation itself.
    const amber = await gesturePage.evaluate((sel) => {
      const hex = getComputedStyle(document.documentElement)
        .getPropertyValue("--accent-document")
        .trim();
      const rgb = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(hex);
      const token = rgb
        ? `rgb(${parseInt(rgb[1], 16)}, ${parseInt(rgb[2], 16)}, ${parseInt(rgb[3], 16)})`
        : null;
      const live = getComputedStyle(document.querySelector(sel));
      const planned = document.querySelector(".btn-gesture.planned");
      return {
        token,
        background: live.backgroundColor,
        border: live.borderColor,
        color: live.color,
        plannedBackground:
          planned === null ? null : getComputedStyle(planned).backgroundColor,
      };
    }, GESTURE);
    check(
      amber.token !== null &&
        amber.background === amber.token &&
        amber.border === amber.token &&
        amber.color !== amber.token,
      "the documenting gesture PAINTS with the amber the sheet reserves for it — FILLED, with its label on top",
      `--accent-document=${amber.token} background=${amber.background} border=${amber.border} label=${amber.color}`,
    );
    check(
      amber.plannedBackground !== null && amber.plannedBackground !== amber.token,
      "CONTROL: a PLANNED control does not carry it — the reservation is a difference, not a wish",
      `planned background=${amber.plannedBackground}`,
    );

    const focused = await gesturePage.evaluate((sel) => {
      const control = document.querySelector(sel);
      control.focus();
      return {
        reached: document.activeElement === control,
        tabIndex: control.tabIndex,
        tag: control.tagName,
      };
    }, GESTURE);
    check(
      focused.reached && focused.tabIndex === 0 && focused.tag === "BUTTON",
      "the live gesture is focusable by nature, not by an attribute someone remembered",
      `<${focused.tag.toLowerCase()}> tabIndex=${focused.tabIndex} reached=${focused.reached}`,
    );

    // 🔴 **A SECOND TAB, opened on the same row BEFORE the first press** — this is what makes the
    // refusal half measurable, and it is the ordinary case rather than a contrivance: an operator
    // with the screen open in two tabs, or one who came back to a page the store has moved past.
    const stalePage = await open(gesturePage.url().replace(BASE, ""));

    // The status comes off the wire, so *the harness re-ran* and *the product broke* stay apart.
    // 🔴 **409 IS THE HARNESS; EVERYTHING ELSE IS THE PRODUCT** — and this block routed BOTH to
    // *the gate could not run* until story 6.4's code review measured it: a real 500 inside
    // `document_all` exited 2 under a message asserting 409 as the cause, i.e. a cause with no
    // check behind it for four of the five reachable statuses. A developer following that message
    // re-seeds and re-runs forever.
    //
    // 🔑 And the response is AWAITED rather than slept on. A fixed 900 ms then reading a variable
    // leaves `posted` null on a slow-but-correct POST — the same page carries a 2 s store budget —
    // and the checks below would then red as *the product broke*, which is the confusion this
    // paragraph exists to prevent.
    const pressed = gesturePage.waitForResponse(
      (response) => response.request().method() === "POST",
      { timeout: NAV_TIMEOUT_MS },
    );
    await gesturePage.keyboard.press("Enter");
    let posted = null;
    try {
      posted = (await pressed).status();
    } catch (error) {
      cannotRun(`the documenting gesture never answered — ${error.message}`);
    }
    if (posted === 409) {
      cannotRun(
        `the documenting gesture answered 409: this store was already documented. Re-run ` +
          `a11y/seed.sql before this gate rather than reading the absence of an answer as a ` +
          `product defect.`,
      );
    }
    check(
      posted === 201,
      "the documenting gesture reaches the route and the route accepts the write",
      `status=${posted}`,
    );
    await gesturePage
      .waitForNavigation({ waitUntil: "networkidle0", timeout: NAV_TIMEOUT_MS })
      .catch(() => {});
    const landed = await gesturePage.evaluate(() => ({
      url: window.location.pathname + window.location.search,
      confirmation: (
        document.querySelector(".documented")?.textContent ?? ""
      ).trim(),
      rows: document.querySelectorAll(".queue .queue-row").length,
      live: document.querySelectorAll("button.btn-document").length,
    }));
    // 🔴 **A SUCCESS REDIRECTS; it does not patch one paragraph.** Until story 6.4's code review
    // the answer was swapped into `#gesture-result` and everything else stayed put: the queue row
    // was still there, the amber button was still live, and the declared pane still read *nothing
    // declared at this address* over a message saying it had just been declared. Two review layers
    // measured it in this browser. The screen is now re-rendered from the store, and the
    // confirmation rides in the URL.
    check(
      landed.url.includes("documented=") && landed.confirmation !== "",
      "a successful gesture RE-RENDERS the screen and the confirmation rides in the URL",
      `url=${landed.url} confirmation=${JSON.stringify(landed.confirmation)}`,
    );
    // 🔴 **THE ONE STRING THE OPERATOR READS AFTER ACTING, and until 2026-09-23 this check asked
    // only whether it was non-empty.** Guy pressed « Ajouter » on his own network and the product
    // answered « Documenté » — `prd.md:888` binds the interface to *"Add" / « Ajouter »* while
    // documentation, API and code keep `document`, and the BUTTON had conformed since `v0.3.1`.
    // *The word came back through the sentence the product ANSWERS.*
    // 🔑 This is the only automated check in the repository standing where that defect occurred:
    // it presses the gesture for real and reads the rendered confirmation out of the DOM. A
    // source guard names the cause and is cheaper; this one measures what was SERVED, which is
    // story 6b.11's amended AC5 — *a source-reading guard does not SUFFICE where the defect lives
    // in the DOM* — and the two cumulate rather than replace each other.
    // ⚠️ Its limit is stated: it knows the ENGLISH stems, and this gate runs in the default
    // locale. A French deployment's « Documenté » is caught by the two Rust carriers and not here.
    check(
      !/\bdocument(s|ed|ing)?\b/i.test(landed.confirmation),
      "and the confirmation does not name the gesture by its IDENTIFIER — the interface says Add",
      `confirmation=${JSON.stringify(landed.confirmation)}`,
    );
    check(
      landed.live === 0,
      "and the question has left the queue — the row it was asked about is gone, with its control",
      `${landed.rows} row(s) left, ${landed.live} live control(s)`,
    );

    // ── The refusal, in the tab that did not act ──
    //
    // 🔴 **htmx swaps NOTHING on a non-2xx by default, so before this story's code review a
    // second press produced total silence** — over the first press's success sentence, still on
    // screen, still saying the opposite. `hx-on::before-swap` now lets any 4xx/5xx through, and
    // `aria-live` announces it. Measured here rather than reasoned about.
    check(
      await stalePage.evaluate(
        () => (document.getElementById("gesture-result")?.textContent ?? "").trim() === "",
      ),
      "the premise: the answer region is EMPTY before the refusal, or the checks below pass on what was already there",
    );
    await stalePage.evaluate((sel) => document.querySelector(sel).focus(), GESTURE);
    const refusal = stalePage.waitForResponse(
      (response) => response.request().method() === "POST",
      { timeout: NAV_TIMEOUT_MS },
    );
    await stalePage.keyboard.press("Enter");
    let refused = null;
    try {
      refused = (await refusal).status();
    } catch (error) {
      cannotRun(`the stale tab's press never answered — ${error.message}`);
    }
    await wait(SETTLE_WAIT_MS);
    const answer = await stalePage.evaluate(() => ({
      text: (document.getElementById("gesture-result")?.textContent ?? "").trim(),
      focusedId: document.activeElement?.id ?? "",
    }));
    if (refused !== 409) {
      cannotRun(
        `the stale tab's press answered ${refused} where 409 was due — the two tabs did not ` +
          `name the same subject, so the refusal path was not exercised at all.`,
      );
    }
    check(
      answer.text !== "",
      "a REFUSED gesture says so on the page rather than failing in silence",
      `status=${refused} answer=${JSON.stringify(answer.text.slice(0, 60))}`,
    );
    check(
      answer.focusedId === "gesture-result",
      "and FOCUS FOLLOWS THE SWAP — story 6b.11's contract, which no render assertion can see",
      `activeElement id=${JSON.stringify(answer.focusedId)}`,
    );
    await stalePage.close();
    await gesturePage.close();
  }

  // ── `/ipam`'s three forms: the keyboard reaches them, and the focus contract holds ──────────
  //
  // 🔴 **THIS FILE HAD NO `/ipam` COVERAGE AT ALL until story 14.2b** — all sixteen of its `open(`
  // sites named `/triage` — while story 6b.11's focus contract and story 6.4's focus-after-swap
  // contract both apply to any live gesture, and 14.2b puts three of them on this screen.
  //
  // 🔑 **It PRESSES, it does not read.** A `<details>` collapsed by default hides its fields from
  // the tab order entirely, so the only honest question is whether an operator using a keyboard
  // can get from the page to a written row — open the disclosure, reach the fields, submit — and
  // whether focus then lands where the answer is. No render assertion can see any of that.
  {
    const page = await open("/ipam");

    // 🔑 **SCOPED to the definition forms since story 14.4**, which added five CORRECTION
    // disclosures in the rail: unscoped this counted 8 and reddened over a correct page. ⚠️ Scoped
    // rather than loosened to `>= 3` — a floor that tolerates losing one of the three gestures is
    // the shape this project has caught twice; the rail's own controls are counted separately.
    const summaries = await page.$$eval(
      ".ipam-forms details.ipam-form > summary",
      (nodes) => nodes.length,
    );
    check(
      summaries === 3,
      "`/ipam` offers the three write gestures as disclosures",
      `summary count=${summaries}`,
    );

    // 🔴 **COUNTING NODES IS NOT REACHING THEM, and story 14.2b's review said so**: the check above
    // claimed "keyboard-reachable" over a count, and every step below focuses by script. This one
    // PRESSES Tab from the top of the page, as an operator would, until the subnet form's
    // disclosure has focus — bounded, so a control Tab never reaches fails rather than loops.
    // ⚠️ BLUR, not `body.focus()`: a `<body>` without `tabindex` takes no focus and blurs nothing, so
    // the walk would start from whatever held focus (the second review).
    await page.evaluate(() => {
      document.activeElement?.blur();
      window.scrollTo(0, 0);
    });
    let reached = false;
    for (let press = 0; press < 400 && !reached; press += 1) {
      await page.keyboard.press("Tab");
      reached = await page.evaluate(
        () => document.activeElement === document.querySelector("#ipam-form-subnet > summary"),
      );
    }
    check(reached, "Tab alone reaches the subnet form from the top of `/ipam`", `reached=${reached}`);

    // A `<summary>` is focusable by construction; what is worth measuring is that Enter OPENS it,
    // because a disclosure that only a pointer can expand hides the gesture from a keyboard.
    await page.evaluate(() =>
      document.querySelector("#ipam-form-subnet > summary").focus(),
    );
    await page.keyboard.press("Enter");
    await wait(120);
    const opened = await page.$eval("#ipam-form-subnet", (el) => el.open);
    check(opened === true, "Enter on the summary OPENS the form", `open=${opened}`);
    // ⚠️ Opened by script if Enter did not, so that failure is COUNTED (exit 1) rather than turning
    // every check below into a timeout reported as "could not run" (exit 2) — the review's finding.
    if (opened !== true) {
      await page.$eval("#ipam-form-subnet", (el) => {
        el.open = true;
      });
    }

    // ⚠️ A CIDR of this probe's own, so it collides with neither the seed's two `/25`s nor the
    // Rust tests' `/24`s — `198.18.0.0/24`, RFC 2544's benchmarking block. 🔴 It read
    // `203.0.113.0/24` under this same sentence, which `repo.rs`'s contention test inserts; the
    // review caught the comment asserting the opposite of the code. The FIRST submit writes and
    // the SECOND is refused, measured separately because they take structurally different paths.
    await page.type("#ipam-cidr", "198.18.0.0/24");
    await page.type("#ipam-subnet-label", "kbd probe");
    await page.evaluate(() =>
      document.querySelector("#ipam-form-subnet button[type=submit]").focus(),
    );
    const posted = page.waitForResponse(
      (response) => response.request().method() === "POST",
      { timeout: NAV_TIMEOUT_MS },
    );
    await page.keyboard.press("Enter");
    let status = null;
    try {
      status = (await posted).status();
    } catch (error) {
      cannotRun(`the plan form's press never answered — ${error.message}`);
    }
    // 🔑 **A 409 HERE NAMES ITS OWN CAUSE rather than becoming a count.** Left tolerant, a re-run
    // over a store nobody re-seeded skipped the success half and died on the floor with
    // *"34 checks ran where 36 are declared"* — true, and it tells the reader nothing. The
    // documenting probe carries the same instruction one block up: re-seed before this step.
    if (status === 409) {
      cannotRun(
        `the plan already holds this probe's subnet, so the write half could not be exercised. ` +
          `Re-seed (a11y/seed.sql clears the plan) before running the keyboard gate.`,
      );
    }
    if (status !== 201) {
      cannotRun(
        `the plan form answered ${status} where 201 was due — the request never reached the ` +
          `route, so nothing below was exercised.`,
      );
    }
    await wait(SETTLE_WAIT_MS);

    // 🔴 **ON SUCCESS THERE IS NO SWAP, AND THE FIRST VERSION OF THIS PROBE ASSERTED THERE WAS.**
    // The route answers 201 with `HX-Redirect`, so htmx NAVIGATES instead of swapping: the answer
    // region stays empty, `after-swap` never fires, and focus is wherever a fresh page load puts
    // it. Measured — `status=201 answer="" activeElement=""` — and the two checks written against
    // the documenting gesture's REFUSAL path were simply wrong here. 🔑 *A contract copied from a
    // neighbouring screen is a contract nobody has measured on this one.*
    //
    // 🔑 What the operator gets instead is stronger than a sentence: the subnet they just defined
    // is DRAWN. On `/triage` a row disappearing needs a sentence to say why; here the plan visibly
    // becomes the thing they asked for, which is why no confirmation rides in the URL.
    {
      const landed = await page.evaluate(() => ({
        url: location.pathname + location.search,
        tabs: [...document.querySelectorAll("nav.filters a.filter")].map((a) =>
          (a.textContent ?? "").trim(),
        ),
      }));
      check(
        landed.url.startsWith("/ipam?subnet="),
        "a written plan sends the browser back to the plan, selected on what it just defined",
        `url=${landed.url}`,
      );
      check(
        landed.tabs.some((label) => label.includes("198.18.0.0/24")),
        "and the subnet is DRAWN — the screen is the confirmation, which is why none rides in " +
          "the URL as it does on /triage",
        `selector=${JSON.stringify(landed.tabs)}`,
      );
    }

    // The REFUSAL path, which is where the swap and the focus contract actually live. A second
    // press of the same CIDR meets the unique index, and `hx-on::before-swap` forces the swap on a
    // 4xx precisely so a refusal is not silent.
    {
      const again = await open("/ipam");
      await again.evaluate(() =>
        document.querySelector("#ipam-form-subnet > summary").focus(),
      );
      await again.keyboard.press("Enter");
      await wait(120);
      await again.type("#ipam-cidr", "198.18.0.0/24");
      await again.type("#ipam-subnet-label", "kbd probe");
      await again.evaluate(() =>
        document.querySelector("#ipam-form-subnet button[type=submit]").focus(),
      );
      const refusal = again.waitForResponse(
        (response) => response.request().method() === "POST",
        { timeout: NAV_TIMEOUT_MS },
      );
      await again.keyboard.press("Enter");
      let refused = null;
      try {
        refused = (await refusal).status();
      } catch (error) {
        cannotRun(`the plan form's second press never answered — ${error.message}`);
      }
      if (refused !== 409) {
        cannotRun(
          `the second press answered ${refused} where 409 was due — the plan did not hold the ` +
            `subnet the first press wrote, so the refusal path was not exercised at all.`,
        );
      }
      await wait(SETTLE_WAIT_MS);
      const after = await again.evaluate(() => ({
        text: (document.getElementById("ipam-form-result")?.textContent ?? "").trim(),
        focusedId: document.activeElement?.id ?? "",
      }));
      check(
        after.text !== "",
        "a REFUSED plan write says so on the page rather than failing in silence",
        `status=${refused} answer=${JSON.stringify(after.text.slice(0, 60))}`,
      );
      check(
        after.focusedId === "ipam-form-result",
        "and FOCUS FOLLOWS THE SWAP on this screen too — story 6.4's contract, which sat on the " +
          "wrong element for a whole story and no Rust assertion could see it",
        `activeElement id=${JSON.stringify(after.focusedId)}`,
      );
      await again.close();
    }

    // ── Story 14.3b: the address field warns BEFORE the write, and the findings are reachable ──
    //
    // 🔑 The warning is a GET route no perimeter guard walks by default and no axe state reaches:
    // it appears only when someone TYPES. So this block types, as an operator would, an address the
    // seed shows in use (`192.0.2.20`, two hardware addresses in the `static` range) and reads back
    // three things no render assertion can see — that the warning arrived, that focus STAYED on the
    // field (a live region announces, it never grabs), and that it links the address's triage question.
    {
      const probe = await open("/ipam");
      await probe.evaluate(() =>
        document.querySelector("#ipam-form-address > summary").focus(),
      );
      await probe.keyboard.press("Enter");
      await wait(120);
      await probe.evaluate(() => document.getElementById("ipam-addr").focus());
      // 🔴 **THE DEBOUNCE IS MEASURED HERE AND NOWHERE ELSE.** In Rust it is a string in a template;
      // what it BUYS — one request for a burst of keystrokes rather than one per keystroke — only a
      // browser can see. Every answered check reads the whole plan and the whole summary, so a lost
      // debounce is ten of those per address typed.
      let answers = 0;
      probe.on("response", (response) => {
        if (response.url().includes("/ipam/address-check")) answers += 1;
      });
      const asked = probe
        .waitForResponse((response) => response.url().includes("/ipam/address-check"), {
          timeout: NAV_TIMEOUT_MS,
        })
        .catch(() => null);
      await probe.keyboard.type(SEEN_ADDRESS);
      // 🔑 **A check that never answers is the PRODUCT, not the harness**: the form is open, the
      // field has focus and the seed holds that sighting, so `cannotRun` here would have reported a
      // broken warning as *fix your environment*. The assertions below carry it as a failure.
      const answered = await asked;
      await wait(SETTLE_WAIT_MS);
      const warned = await probe.evaluate(() => ({
        text: (document.getElementById("ipam-addr-warning")?.textContent ?? "").trim(),
        focusedId: document.activeElement?.id ?? "",
        link: document.querySelector("#ipam-addr-warning a")?.getAttribute("href") ?? null,
      }));
      check(
        answered !== null && warned.text !== "",
        "typing an address the network shows in use warns BEFORE the write",
        `answered=${answered !== null} warning=${JSON.stringify(warned.text.slice(0, 80))}`,
      );
      check(
        answers === 1,
        `the field asks the store ONCE for a burst of ${SEEN_ADDRESS.length} keystrokes — the ` +
          "debounce is what keeps a warning from reading the whole plan per character",
        `answers=${answers}`,
      );
      check(
        warned.focusedId === "ipam-addr",
        "and focus STAYS on the field — the warning is announced, never grabbed",
        `activeElement id=${JSON.stringify(warned.focusedId)}`,
      );
      check(
        warned.link === `/triage?sel=nouveau:${SEEN_ADDRESS}`,
        "and the warning links the address's own triage question",
        `link=${JSON.stringify(warned.link)}`,
      );

      // 🔴 **THE LINK IS FOLLOWED, because the audit and triage decide the question's existence
      // from DIFFERENT COMPARISONS** — the audit from the declared register, triage from the
      // observed facts against the declared VALUES as strings. The review measured them disagreeing
      // on a value stored with whitespace. Asserting the href's shape cannot see that; landing on
      // the page and finding the address can. ⚠️ `open` refuses anything but 200, so a link into a
      // question triage does not ask fails here rather than passing quietly.
      if (warned.link !== null) {
        const landed = await open(warned.link);
        const names = await landed.evaluate(
          (address) => (document.body.textContent ?? "").includes(address),
          SEEN_ADDRESS,
        );
        check(
          names,
          "and following it lands on a triage question about that very address",
          `link=${warned.link}`,
        );
        await landed.close();
      }
      await probe.close();

      // ── The RANGE form warns too (Guy, 2026-09-15) ──────────────────────────────────────────
      // 🔑 Same contract as the address field and measured the same way: the answer arrives, and
      // focus STAYS where the operator is typing. A live region announces; it never grabs.
      {
        const ranged = await open("/ipam");
        await ranged.evaluate(() =>
          document.querySelector("#ipam-form-range > summary").focus(),
        );
        await ranged.keyboard.press("Enter");
        await wait(120);
        const rangeAsked = ranged
          .waitForResponse((response) => response.url().includes("/ipam/range-check"), {
            timeout: NAV_TIMEOUT_MS,
          })
          .catch(() => null);
        await ranged.evaluate(() => document.getElementById("ipam-first").focus());
        await ranged.keyboard.type("192.0.2.1");
        await ranged.evaluate(() => document.getElementById("ipam-last").focus());
        await ranged.keyboard.type("192.0.2.40");
        const rangeAnswered = await rangeAsked;
        await wait(SETTLE_WAIT_MS);
        const rangeWarned = await ranged.evaluate(() => ({
          text: (document.getElementById("ipam-range-warning")?.textContent ?? "").trim(),
          focusedId: document.activeElement?.id ?? "",
        }));
        check(
          rangeAnswered !== null && rangeWarned.text !== "",
          "a static range over addresses the network already shows warns BEFORE the write",
          `answered=${rangeAnswered !== null} warning=${JSON.stringify(rangeWarned.text.slice(0, 80))}`,
        );
        check(
          rangeWarned.focusedId === "ipam-last",
          "and focus STAYS in the field being typed — the range's warning announces, like the " +
            "address's",
          `activeElement id=${JSON.stringify(rangeWarned.focusedId)}`,
        );
        await ranged.close();
      }

      // 🔴 **THE FINDINGS WALK STARTS FROM A FRESH PAGE, and it did not until the code review.**
      // It ran on the page the block above had just typed into, so the walk began wherever the
      // address field had left focus — *"Tab alone reaches a link in the findings list"* was
      // measuring a few presses from mid-page, not the journey it names. A blur is not a reset when
      // the page has been scrolled and a disclosure opened.
      const walk = await open("/ipam");
      await walk.evaluate(() => {
        document.activeElement?.blur();
        window.scrollTo(0, 0);
      });
      let reachedFinding = false;
      for (let press = 0; press < 600 && !reachedFinding; press += 1) {
        await walk.keyboard.press("Tab");
        reachedFinding = await walk.evaluate(
          () =>
            document.activeElement?.tagName === "A" &&
            document.activeElement?.closest(".ipam-audit") !== null,
        );
      }
      check(
        reachedFinding,
        "Tab alone reaches a link in the findings list, from the top of a FRESH page",
        `reached=${reachedFinding}`,
      );
      await walk.close();
    }
    await page.close();
  }

  // ── The rail's two lists and their controls (story 14.4) ────────────────────────────────────
  // 🔴 **FOUR of the eight write routes are PER-ROW controls**, so they exist only on a page that
  // holds records; the fifth, the subnet's own removal, names no row and renders over an empty rail.
  // ⚠️ This read *"five"* until story 14.4's slice-D review — the same false sentence the review
  // corrected in two comments on the Rust side, surviving here because this file was in the slice
  // the record itself flagged as unreviewed. That is the concrete cost of an unreviewed slice,
  // arriving in the same push that named it.
  // The seed's default subnet carries three ranges and two addresses, which is why this block opens
  // `/ipam` bare rather than selecting a subnet: the page the operator meets first is the page the
  // gate must measure.
  {
    const rail = await open("/ipam");
    const controls = await rail.evaluate(() => {
      const posts = (route) => document.querySelectorAll(`form[hx-post="${route}"]`).length;
      return {
        deleteRange: posts("/ipam/range/delete"),
        deleteAddress: posts("/ipam/address/delete"),
        deleteSubnet: posts("/ipam/subnet/delete"),
        editRange: posts("/ipam/range/edit"),
        editAddress: posts("/ipam/address/edit"),
        // Story 14.5's tenth route: the subnet's own correction, one per page like its removal.
        editSubnet: posts("/ipam/subnet/edit"),
        names: [...document.querySelectorAll(".ipam-rail-lists button")].map((b) =>
          (b.textContent ?? "").trim(),
        ),
      };
    });
    // ⚠️ **EXACT COUNTS, where four of these five terms read `> 0` until the slice-D review.** The
    // seed is this harness's own committed fixture and it TRUNCATES first, so three ranges and two
    // addresses is a number the gate may assert rather than tolerate — and a floor of `> 0` lets
    // two of the three range controls vanish while still reading as a pass. 🔑 The check sixty
    // lines up records the opposite decision for its own neighbour (*"Scoped rather than loosened
    // to `>= 3` — a floor that tolerates losing one of the three gestures is the shape this project
    // has caught twice"*), and this block then shipped four such floors.
    check(
      controls.deleteRange === 3 &&
        controls.deleteAddress === 2 &&
        controls.deleteSubnet === 1 &&
        controls.editRange === 3 &&
        controls.editAddress === 2 &&
        controls.editSubnet === 1,
      "the rail carries a control for each of the six corrections, at the seed's own counts",
      JSON.stringify(controls),
    );
    // 🔑 A page of buttons all reading *Remove* is a page a screen reader cannot navigate: the
    // record's own description is IN the accessible name, not in a tooltip.
    check(
      controls.names.length > 0 && new Set(controls.names).size === controls.names.length,
      "and every control's accessible name is DISTINCT, naming the record it acts on",
      // ⚠️ ALL of them, where this printed `.slice(0, 8)` over a population of eleven: a duplicate
      // pair sitting at positions 9–11 would have been omitted from the message announcing it.
      // *A measurement read through a truncation is not a measurement* — this story's own rule,
      // invoked twice in its record and then broken in its evidence string.
      JSON.stringify(controls.names),
    );
    // ⚠️ **The DELETE control, deliberately, and the first draft of this check got it wrong.** It
    // took the first button in the rail, which is a correction form's submit INSIDE a collapsed
    // `<details>` — content in a closed disclosure is correctly unfocusable, so the check reddened
    // over a page that was behaving properly. *A check aimed at the wrong element measures the
    // wrong thing in both directions.* The removal control sits outside any disclosure, which is
    // exactly why it is the one an operator meets without opening anything.
    const focusable = await rail.evaluate(() => {
      const button = document.querySelector(
        '.ipam-rail-lists form[hx-post="/ipam/range/delete"] button',
      );
      button?.focus();
      return { reached: document.activeElement === button, tabIndex: button?.tabIndex ?? null };
    });
    check(
      focusable.reached && focusable.tabIndex >= 0,
      "a rail control is reachable by the keyboard — story 6b.4b shipped five controls that " +
        "forty dispatched Tab presses reached none of",
      JSON.stringify(focusable),
    );
    // 🔴 **A PRE-STATE, because without one this check could not red for any product change.** A
    // `<summary>` opens on Enter by browser construction, so asserting only the after-state measures
    // the browser: its two outcomes were green, or a TypeError if the element vanished. ⚠️ And the
    // lookup was unguarded — measured at the review, renaming `.ipam-form` to anything else deletes
    // every correction disclosure from the rail and the gate answered **2**, *the gate could not
    // run*, over a page where the operator can no longer correct anything. Its own sibling twenty
    // lines up uses `?.` and degrades to a failed check; this one threw.
    const disclosure = await rail.evaluate(() => {
      const summary = document.querySelector(".ipam-rail-lists details.ipam-form > summary");
      return {
        found: summary !== null,
        closedBefore: summary?.parentElement?.open === false,
      };
    });
    check(
      disclosure.found && disclosure.closedBefore,
      "a correction disclosure is present and starts CLOSED — the pre-state without which the " +
        "next check measures the browser rather than the product",
      JSON.stringify(disclosure),
    );
    await rail.evaluate(() =>
      document.querySelector(".ipam-rail-lists details.ipam-form > summary")?.focus(),
    );
    await rail.keyboard.press("Enter");
    const opened = await rail.evaluate(
      () => document.querySelector(".ipam-rail-lists details.ipam-form")?.open ?? null,
    );
    check(opened === true, "and Enter opens it", `open=${opened}`);
    // 🔑 **PRE-FILLED, and that is the difference between correcting and re-typing.** An edit form
    // the operator must fill from scratch is a delete-and-redefine wearing another word, and it
    // loses the row's identity — which is what `update_range` exists to keep.
    const prefilled = await rail.evaluate(() => {
      const form = document.querySelector('.ipam-rail-lists form[hx-post="/ipam/range/edit"]');
      return {
        id: form?.querySelector("input[name=id]")?.value ?? "",
        first: form?.querySelector("input[name=first]")?.value ?? "",
        last: form?.querySelector("input[name=last]")?.value ?? "",
        policy: form?.querySelector("select[name=policy]")?.value ?? "",
      };
    });
    check(
      prefilled.id !== "" &&
        prefilled.first !== "" &&
        prefilled.last !== "" &&
        ["static", "dhcp-pool", "reserved", "infrastructure"].includes(prefilled.policy),
      "and it arrives carrying the record's current bounds, policy and id",
      JSON.stringify(prefilled),
    );
    // 🔑 **STORY 14.5's TENTH ROUTE, measured in a browser because its pre-fill is where the VLAN
    // sentinel becomes visible.** The store spells *no VLAN* as 0; the form must spell it as an
    // EMPTY field, because an empty field is what the route reads back as *none* and a pre-filled
    // `0` would be a number the operator never typed and cannot mean. Only the rendered value can
    // say which of the two shipped — a Rust assertion over the struct measures the struct.
    // ⚠️ The seed gives Office VLAN 10 and Workshop none, and `/ipam` opens on Office, so what is
    // asserted here is the PRESENT half; the absent half is asserted by the Rust render test, which
    // can build both. Saying which gate carries which half beats implying one carries both.
    // 🔴 **ITS OWN DISCLOSURE IS OPENED FIRST, and the first draft of this check did not** —
    // measured `reached:false` over a product behaving correctly, because the disclosure the block
    // above opens is a RANGE correction and content inside a CLOSED `<details>` is properly
    // unfocusable. Story 14.4's review recorded exactly this (*a check of mine aimed at a control
    // inside a collapsed `<details>`*) and this file's own comment sixty lines up narrates it; it
    // recurred anyway, one story later, in the block that comment sits in.
    await rail.evaluate(() => {
      document
        .querySelector('.ipam-rail-lists form[hx-post="/ipam/subnet/edit"]')
        ?.closest("details")
        ?.setAttribute("open", "");
    });
    const subnetEdit = await rail.evaluate(() => {
      const form = document.querySelector('.ipam-rail-lists form[hx-post="/ipam/subnet/edit"]');
      const button = form?.querySelector("button");
      button?.focus();
      return {
        id: form?.querySelector("input[name=id]")?.value ?? "",
        label: form?.querySelector("input[name=label]")?.value ?? "",
        vlan: form?.querySelector("input[name=vlan]")?.value ?? "",
        cidrFields: form?.querySelectorAll("[name=cidr], [name=base], [name=prefix_len]").length,
        reached: button != null && document.activeElement === button,
      };
    });
    check(
      subnetEdit.id !== "" &&
        subnetEdit.label !== "" &&
        subnetEdit.vlan === "10" &&
        subnetEdit.cidrFields === 0 &&
        subnetEdit.reached,
      "the subnet's own correction is reachable, pre-filled with its label and its VLAN, and " +
        "offers NO field that could move its address space",
      JSON.stringify(subnetEdit),
    );
    // 🔴 **THE REMOVAL WARNS BEFORE IT FIRES (decision 3), and it is reached only by a GESTURE** —
    // so a source guard cannot see it and this gate must press it. The control asks on focus, the
    // answer lands in that row's own polite region, and the write is untouched: it warns, it does
    // not refuse.
    const warned = await rail.evaluate(async () => {
      const button = document.querySelector(
        '.ipam-rail-lists form[hx-post="/ipam/range/delete"] button',
      );
      const region = document.getElementById(
        button?.getAttribute("aria-describedby") ?? "",
      );
      button?.focus();
      for (let i = 0; i < 40 && (region?.textContent ?? "").trim() === ""; i += 1) {
        await new Promise((resolve) => setTimeout(resolve, 100));
      }
      return {
        text: (region?.textContent ?? "").trim(),
        focusedIsControl: document.activeElement === button,
        region: region?.getAttribute("aria-live") ?? null,
      };
    });
    check(
      warned.text !== "" && warned.region === "polite",
      "focusing a removal control warns BEFORE the write, into a polite live region",
      `${JSON.stringify(warned.text.slice(0, 90))} aria-live=${warned.region}`,
    );
    check(
      warned.focusedIsControl,
      "and focus STAYS on the control — the warning is announced, never grabbed",
      `focusedIsControl=${warned.focusedIsControl}`,
    );
    await rail.close();
  }

  // ── Story 14.4b: the release, found by its CONTROL and PRESSED ─────────────────────────────
  // 🔑 Found by the route it posts to and by the address its hidden field carries — never by a
  // translated word (story 6.4's lesson). The seed's Office subnet is the one `/ipam` opens on, and
  // its findings are a committed fixture, so the addresses below are EXACT rather than a floor:
  // seven releasable findings (`gap` or `undeclared`); `.9` listed as a conflict and DEFINED, so it
  // offers none (decision 4); `.61` sighted and ALREADY released, so it is not a finding at all.
  // `.60` is the dedicated case this block presses — dedicated so the press removes nothing another
  // check walks.
  // ⚠️ **What the `.61` half does and does not prove**: that a sighted, already-released address is
  // no finding. That it IS sighted is not visible in a browser once released — the release hides it
  // everywhere — and is carried by `sighting_repo`'s seed test, which counts its row among fifteen.
  {
    // 🔴 **`10.9.9.9` IS STORY 14.5's AC7 AND IT IS LAST BECAUSE IT IS OUTSIDE.** The seven above
    // are the Office subnet's own findings; this one is an observed address no subnet of the plan
    // contains, and until 14.5 it carried NO release — the outside rows hold `kind: None`, so the
    // findings list's `row.gap || row.undeclared` rendered nothing for them. ⚠️ That was a hole
    // rather than a decision: an address outside every declared subnet is exactly where a machine
    // nobody planned for sits, which is the case the release exists for. Measured here before the
    // repair: this list read seven and the gate reported the eighth as a regression.
    const RELEASABLE = [
      "192.0.2.11",
      "192.0.2.12",
      "192.0.2.13",
      "192.0.2.20",
      "192.0.2.42",
      "192.0.2.50",
      "192.0.2.60",
      "10.9.9.9",
    ];
    const read = (p) =>
      p.evaluate(() => {
        const forms = [...document.querySelectorAll('.ipam-audit form[hx-post="/ipam/release"]')];
        return {
          url: location.pathname + location.search,
          released: forms.map((f) => f.querySelector("input[name=addr]")?.value ?? ""),
          names: forms.map((f) => (f.querySelector("button")?.textContent ?? "").trim()),
          findings: [
            ...document.querySelectorAll(".ipam-audit ul.ipam-findings:not(.ipam-outside) > li > .mono"),
          ].map((e) => (e.textContent ?? "").trim()),
        };
      });
    const held = await open("/ipam");
    const before = await read(held);
    check(
      JSON.stringify(before.released) === JSON.stringify(RELEASABLE) &&
        before.findings.includes("192.0.2.9") &&
        !before.findings.includes("192.0.2.61"),
      "every releasable finding carries a release, the OUTSIDE address included — not the " +
        "DEFINED .9, and the already-released .61 is no finding at all",
      JSON.stringify({ released: before.released, findings: before.findings }),
    );
    check(
      before.names.length === RELEASABLE.length &&
        new Set(before.names).size === before.names.length &&
        before.names.every((name, i) => name.includes(before.released[i])),
      "and each release names its address in its accessible name",
      JSON.stringify(before.names),
    );
    // 🔑 **AC7 in its own words: the OUTSIDE list's release, reached by the keyboard.** Asserted
    // separately from the one below because they are different lists rendered by different branches
    // of the same partial — the findings list's condition renders nothing for an outside row, which
    // is exactly how the control came to be missing there. ⚠️ Focused rather than Tabbed to, like
    // its neighbour: what is at stake is whether the element ACCEPTS focus, which is what story
    // 6b.4b's five unreachable `<span role="button">`s failed.
    //
    // 🔴 **IT RUNS BEFORE ITS NEIGHBOUR, AND THE ORDER IS LOAD-BEARING.** The press below sends
    // `Enter` to whatever holds focus, and what puts `.60` there is the check that follows this
    // one — an implicit coupling nothing declares. Placed after, this check left `10.9.9.9`
    // focused and the gate RELEASED THE WRONG ADDRESS, reddening three downstream checks that had
    // nothing wrong with them. Measured, not reasoned about.
    const outsideRelease = await held.evaluate(() => {
      const button = [...document.querySelectorAll('.ipam-outside form[hx-post="/ipam/release"]')]
        .find((f) => f.querySelector("input[name=addr]")?.value === "10.9.9.9")
        ?.querySelector("button");
      button?.focus();
      return {
        found: button != null,
        reached: button != null && document.activeElement === button,
        tabIndex: button?.tabIndex ?? null,
      };
    });
    check(
      outsideRelease.found && outsideRelease.reached && outsideRelease.tabIndex >= 0,
      "and so is the release on an address OUTSIDE every subnet of the plan, which carried none " +
        "at all until story 14.5",
      JSON.stringify(outsideRelease),
    );
    const focusable = await held.evaluate(() => {
      const button = [...document.querySelectorAll('.ipam-audit form[hx-post="/ipam/release"]')]
        .find((f) => f.querySelector("input[name=addr]")?.value === "192.0.2.60")
        ?.querySelector("button");
      button?.focus();
      return { reached: button != null && document.activeElement === button, tabIndex: button?.tabIndex ?? null };
    });
    check(
      focusable.reached && focusable.tabIndex >= 0,
      "the release control is reachable by the keyboard",
      JSON.stringify(focusable),
    );
    // 🔑 PRESSED with Enter, awaited on its response rather than slept on (the documenting
    // gesture's reason). ⚠️ **This gate is not re-runnable without a re-seed**, and this comment
    // claimed the opposite until the code review: on a store already pressed, `.60` is no finding and
    // the exact list above reds as a product fault. In practice the plan form's 409, earlier, stops a
    // stale run first with *the gate could not run* — which is the re-seed instruction CI follows.
    // 🔴 **The navigation is ARMED BEFORE THE PRESS**: armed after the response, it raced the
    // `HX-Redirect` it waits for (the review's blind layer).
    const answered = held.waitForResponse((r) => r.request().method() === "POST", {
      timeout: NAV_TIMEOUT_MS,
    });
    const navigated = held
      .waitForNavigation({ waitUntil: "networkidle0", timeout: NAV_TIMEOUT_MS })
      .catch(() => {});
    await held.keyboard.press("Enter");
    let status = null;
    try {
      status = (await answered).status();
    } catch (error) {
      cannotRun(`the release never answered — ${error.message}`);
    }
    await navigated;
    const after = await read(held);
    check(
      status === 200 && after.url.startsWith("/ipam?subnet="),
      "pressing it writes, and sends the browser back to the plan it changed",
      `status=${status} url=${after.url}`,
    );
    check(
      !after.findings.includes("192.0.2.60") &&
        !after.released.includes("192.0.2.60") &&
        after.released.includes("192.0.2.20"),
      "and the released address has LEFT the findings, while the others keep theirs",
      JSON.stringify({ released: after.released, findings: after.findings }),
    );
    // 🔑 **The confirmation is SHOWN** (the code review's decision 3): `HX-Redirect` shows no body, so
    // the one sentence saying a release lapses when the network answers rides in the URL.
    const confirmed = await held.evaluate(() => ({
      url: location.search,
      text: (document.querySelector("p.gesture-result[role=status]")?.textContent ?? "").trim(),
    }));
    check(
      confirmed.url.includes("released=192.0.2.60") && confirmed.text.includes("192.0.2.60"),
      "and the page CONFIRMS the release, naming the address",
      JSON.stringify(confirmed),
    );
    await held.close();
  }

  // 🔴 **A FAILURE DECIDES BEFORE THE FLOOR DOES, and the review MEASURED why this order matters.**
  // Checks are not all independent: some run only if an earlier one's subject exists. So a check
  // that fails FOR THE REASON IT WAS WRITTEN TO CATCH can skip its dependants, `executed` falls
  // under the floor, and the floor then overwrites a detected regression with *could not run*.
  // Measured by breaking the address warning's triage link: the gate PRINTED
  // `🔴 the warning links the address's own triage question  link=null`, then answered **2** with a
  // message telling the reader to re-seed — over a real product defect, forever. ⚠️ That is exactly
  // the confusion this file's header exists to prevent (*"the store was empty" and "the keyboard
  // layer is correct" must not be the same answer*), met on the other axis: here a product fault
  // and an instrument fault were made the same answer.
  if (failed > 0) {
    console.log(
      `\nkbd gate: ${executed} check(s) run, ${failed} failed — the keyboard layer has regressed`,
    );
    return 1;
  }
  // 🔑 The floor, which now speaks only when nothing failed: a run that measured less than the full
  // set reports "could not run", not a pass. This is the assertion the file's predecessor did not
  // have.
  if (executed < MIN_CHECKS) {
    cannotRun(
      `${executed} check(s) ran where ${MIN_CHECKS} are declared — a keyboard gate that ` +
        `skips half its checks must not report success.`,
    );
  }
  console.log(`\nkbd gate: ${executed} check(s) run, 0 failed`);
  return 0;
}

let code;
try {
  code = await main();
} catch (error) {
  console.error(
    `kbd gate: ${error instanceof CannotRun ? error.message : `${error.name}: ${error.message}`}`,
  );
  code = 2;
} finally {
  if (browser !== undefined) await browser.close().catch(() => {});
}
process.exitCode = code;
