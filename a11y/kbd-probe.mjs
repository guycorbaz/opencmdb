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
// 🔑 The floor, and it EQUALS what is there rather than sitting under it: twenty checks
// run on a queue of two, which is the shortest queue this gate accepts. A floor below what
// is there tolerates the loss of a check while still reading as a pass — this project has
// caught that twice, once in a privacy floor and once in a word count. If a check is added
// this number moves deliberately; if one is skipped, the gate says so instead of printing
// a green.
const MIN_CHECKS = 26;
const MIN_ROWS = 2;
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
  // ⚠️ **THIS BLOCK WRITES TO THE STORE**, and it is the only check here that does: it adopts
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
        emptyBefore: document.getElementById("gesture-result")?.textContent.trim() === "",
      };
    }, GESTURE);
    check(
      focused.reached && focused.tabIndex === 0 && focused.tag === "BUTTON",
      "the live gesture is focusable by nature, not by an attribute someone remembered",
      `<${focused.tag.toLowerCase()}> tabIndex=${focused.tabIndex} reached=${focused.reached}`,
    );
    check(
      focused.emptyBefore,
      "the premise: the answer region is EMPTY before the press, or the two checks below pass on what was already there",
      `emptyBefore=${focused.emptyBefore}`,
    );

    // The status comes off the wire, so *the harness re-ran* and *the product broke* stay apart.
    let posted = null;
    gesturePage.on("response", (response) => {
      if (response.request().method() === "POST") posted = response.status();
    });
    await gesturePage.keyboard.press("Enter");
    await wait(SETTLE_WAIT_MS);
    if (posted !== null && posted !== 201 && posted !== 200) {
      cannotRun(
        `the documenting gesture answered ${posted}. A 409 means this store was already ` +
          `documented — re-run a11y/seed.sql before this gate rather than reading the ` +
          `absence of a swap as a product defect.`,
      );
    }
    const after = await gesturePage.evaluate(() => {
      const region = document.getElementById("gesture-result");
      return {
        answered: (region?.textContent ?? "").trim(),
        focusedId: document.activeElement?.id ?? "",
      };
    });
    check(
      after.answered !== "",
      "⏎ on the live gesture reaches the route and the answer is swapped in",
      `status=${posted} answer=${JSON.stringify(after.answered.slice(0, 60))}`,
    );
    check(
      after.focusedId === "gesture-result",
      "and FOCUS FOLLOWS THE SWAP — story 6b.11's contract, which no render assertion can see",
      `activeElement id=${JSON.stringify(after.focusedId)}`,
    );
    await gesturePage.close();
  }

  // 🔑 The floor: a run that measured less than the full set reports "could not run", not a
  // pass. This is the assertion the file's predecessor did not have.
  if (executed < MIN_CHECKS) {
    cannotRun(
      `${executed} check(s) ran where ${MIN_CHECKS} are declared — a keyboard gate that ` +
        `skips half its checks must not report success.`,
    );
  }
  console.log(
    `\nkbd gate: ${executed} check(s) run, ${failed} failed` +
      (failed === 0 ? "" : " — the keyboard layer has regressed"),
  );
  return failed === 0 ? 0 : 1;
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
