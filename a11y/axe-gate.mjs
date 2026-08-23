// The accessibility gate — axe-core over every screen the navigation offers.
//
// 🔴 **This is the ONLY gate in this project that is not Rust, and that is an EXCEPTION
// rather than a precedent.** `ci.yml:1` states the rule it breaks: *"a THIN runner (D56).
// All gate logic lives in `cargo xtask ci`, in Rust, never here."* The reason it cannot
// obey: axe-core measures the RENDERED, COMPUTED page — contrast against the ground that
// actually paints, ARIA against the role a browser actually resolves — and `cargo xtask ci`
// has no browser. A Rust re-implementation would be a second, weaker axe.
//
// 🔑 **The routes are DERIVED, never listed here.** They are scraped from the rendered
// navigation, which `page.rs:79` builds from `Screen::ALL` over `NavGroup::ALL` — so the
// list this gate walks is the list the product serves, and a screen added in Rust is
// covered the day it appears. ⚠️ **Its one limit, stated rather than implied: a screen
// hidden from the navigation is invisible to this gate.** `/gap` is such an address today
// — a bare fragment with no doctype, deliberately out of scope.
//
// 🔴 **The exit codes distinguish two failures a CI cannot otherwise tell apart.**
//
// ⚠️ **The mechanism this paragraph used to describe is PRE-6b.10 and was refuted by
// measurement at story 6b.11's second review round.** It said: *"with the database paused, the
// derivation succeeds, `/triage` blocks on sqlx's acquire timeout, the navigation times out"* —
// and called that measured. Re-measured with the store paused: `/triage` answers **500 in
// 5.00 s**, because story 6b.10's review installed `PAGE_STORE_BUDGET` (`page.rs:1475`) for
// exactly that reason, and `/triage` is also this gate's own `SEED` route, so the FIRST fetch
// fails and the derivation is never reached at all. *A sentence describing behaviour two
// stories old, in a comment whose rhetorical point is "measured rather than assumed".*
//
// 🔑 **The distinction still holds, by a different branch**: the run ends `2` on *"answered
// 500, so nothing there can be measured"* rather than on a navigation timeout — and a naive
// harness would still have died with the code it uses for *"the product has violations"*.
//
//   0 — every route clean
//   1 — the product has accessibility violations   (a real regression: fix the product)
//   2 — the gate could not run                     (fix the harness or the environment)
//
// ⚠️ **THE 1/2 SPLIT HOLDS FOR EVERYTHING THIS FILE DOES, AND FOR NOTHING BEFORE IT** —
// story 6b.11's arbitration 1 (2026-08-23), taken on a measurement. The whole body runs
// inside one `try`, so a missing `puppeteer-core`, an axe bundle that moved, a page that
// navigates during `evaluate` and every other throw answer **2**; before the repair only
// `puppeteer.launch` did, and the code review measured `mv node_modules` → **1** and a
// broken axe payload → **1**, i.e. *the product has violations* for *the gate could not
// run*. **What no repair inside this file can reach is the shell that invokes it**: `npm ci`
// failing exits npm's code, the readiness `curl` exits 22, and a missing Node makes the
// shell exit **127** (measured). Read the contract as *"once `node` is running this file,
// 1 means the product"* — never as *"the step fails with 1 or 2"*.
//
// 🔑 **`process.exitCode` and a return, NEVER `process.exit()`** — under a CI runner stdout
// and stderr are pipes, Node's writes to them are asynchronous, and `process.exit()` does
// not flush pending writes. The two messages this gate exists to print are precisely the
// ones that distinguish a harness failure from a product failure, so losing them is the
// worst of both.
// ⚠️ **`puppeteer-core` is imported DYNAMICALLY, inside the `try`, and that is not a
// style choice.** A static `import` is resolved before any statement of this module runs, so
// a missing package throws where no `catch` of ours exists and Node exits **1** — measured:
// the repair for arbitration 1 was written with a top-level `try` and `mv node_modules` still
// answered 1. `node:fs` stays static: it ships with the runtime, and if it is missing the
// exit code is the least of anyone’s problems.
import { readFileSync } from "node:fs";

const BASE = process.env.AXE_BASE_URL ?? "http://127.0.0.1:8080";
const USER = process.env.OPENCMDB_BASIC_USER ?? "";
const PASS = process.env.OPENCMDB_BASIC_PASSWORD ?? "";
const CHROME = process.env.AXE_CHROME ?? "/usr/bin/google-chrome";
// The seed page is where the navigation is read from. Any screen serves the same nav.
const SEED = process.env.AXE_SEED_ROUTE ?? "/triage";
// 🔑 **AN EXACT COUNT OVER DISTINCT ROUTES, not a floor over an array length.** A floor
// (`routes.length < 10`) is satisfied by ten anchors all pointing at one screen: the code
// review planted exactly that and the gate printed `✅ /triage ×10 … 0 violation node(s)`
// and exited 0 while a planted violation was live on five screens it never visited. Length
// cannot tell ten screens from one screen ten times — which is the failure mode this file's
// own header names. Distinctness is checked, and the comparison is `!==`, so a screen added
// in Rust reds this gate until someone moves the number deliberately; under `<` an eleventh
// screen passed in silence and "moves deliberately" was a hope with no mechanism behind it.
const EXPECTED_ROUTES = 10;
// 🔑 **THE STATES THE NAVIGATION CANNOT REACH** — story 6b.11's arbitration 2. The gate
// walks hrefs, and a query-string state is on no href: `aria-current` on the sort toggle
// renders only under `?sort=age`, and the selected-row pane only under `?sel=…`. So the
// replacement for an attribute a browser rated critical was verified by no browser at all.
// These are derived from the rendered page too — never spelled out here — so they cannot
// drift from what the product serves.
const STATE_SORT = "?sort=age";
// ⚠️ **An empty queue is CI's permanent state unless something seeds it, and the gate is
// green over it** — measured: with the store emptied, `/triage` carries 0 queue rows, 0
// gesture controls and 0 panes, and the story's own defect replanted exits 0. Set
// `AXE_REQUIRE_QUEUE=1` (CI does) and an empty queue is *the gate could not run*, not a
// pass. Left unset — a developer's empty store — the missing coverage is PRINTED, because
// a silent skip is exactly the "derives nothing and reports success" shape.
const REQUIRE_QUEUE = process.env.AXE_REQUIRE_QUEUE === "1";
const TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"];
const NAV_TIMEOUT_MS = 20_000;

/**
 * The gate could not run. Thrown, never exited on, so the one `catch` decides the code —
 * and so a `finally` can still close the browser.
 */
class CannotRun extends Error {}

/** Raise "the gate could not run" from anywhere in the body. */
function cannotRun(message) {
  throw new CannotRun(message);
}

let browser;

/**
 * The whole gate. Returns the process exit code: 0 clean, 1 the product has violations.
 * Every other outcome leaves by `CannotRun` and is answered with 2 by the caller.
 */
async function main() {
  const { default: puppeteer } = await import("puppeteer-core");
  const axeSource = readFileSync(
    new URL("./node_modules/axe-core/axe.min.js", import.meta.url),
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

  /** A page with the operator's credentials already supplied. */
  async function openPage() {
    const page = await browser.newPage();
    page.setDefaultNavigationTimeout(NAV_TIMEOUT_MS);
    if (USER !== "" || PASS !== "") {
      await page.authenticate({ username: USER, password: PASS });
    }
    return page;
  }

  /** Navigate, or end the run as "could not run" — never with the violation code. */
  async function goOrGiveUp(page, route) {
    let response;
    try {
      response = await page.goto(BASE + route, { waitUntil: "networkidle0" });
    } catch (error) {
      cannotRun(`${route} did not answer — ${error.message}`);
    }
    const status = response?.status() ?? 0;
    if (status !== 200) {
      cannotRun(`${route} answered ${status}, so nothing there can be measured`);
    }
    return page;
  }

  // ── Derive the routes from the rendered navigation ────────────────────────
  const seedPage = await openPage();
  await goOrGiveUp(seedPage, SEED);
  const scraped = await seedPage.$$eval("nav.nav a.nav-entry", (entries) =>
    entries.map((entry) => entry.getAttribute("href")),
  );
  // A `null` href, a fragment or an absolute URL would build a nonsense address that
  // `goOrGiveUp` then reports as "did not answer" — naming the wrong cause. They are
  // refused here, where the cause is still known.
  const rejected = scraped.filter(
    (href) => typeof href !== "string" || !href.startsWith("/"),
  );
  const routes = [...new Set(scraped.filter((href) => !rejected.includes(href)))];

  // The states no href carries. The selected-row address is the first queue row's own
  // href, so it is whatever the product would navigate to — not a URL invented here.
  const firstRow = await seedPage.$$eval(".queue .queue-row > a", (rows) =>
    rows.length > 0 ? rows[0].getAttribute("href") : null,
  );
  await seedPage.close();

  if (rejected.length > 0) {
    cannotRun(
      `the navigation at ${SEED} offers ${rejected.length} href(s) this gate cannot ` +
        `resolve (${rejected.map((h) => String(h)).join(", ")})`,
    );
  }
  if (routes.length !== EXPECTED_ROUTES) {
    cannotRun(
      `derived ${routes.length} DISTINCT route(s) from the navigation at ${SEED}, ` +
        `expected exactly ${EXPECTED_ROUTES}. A harness that finds the wrong number of ` +
        `screens must not pass — move the constant deliberately.`,
    );
  }

  const states = [SEED + STATE_SORT];
  if (firstRow !== null) {
    states.push(firstRow);
  } else if (REQUIRE_QUEUE) {
    cannotRun(
      `${SEED} carries no queue row, so the surface this gate exists to measure — the ` +
        `queue, the selected pane, the gesture controls — is not on the page. Seed the ` +
        `store before the gate runs.`,
    );
  } else {
    console.log(
      `⚠️  ${SEED} carries no queue row: the selected-pane state was NOT measured. ` +
        `Set AXE_REQUIRE_QUEUE=1 to make that a refusal rather than a gap.`,
    );
  }

  // ── Walk them ─────────────────────────────────────────────────────────────
  let nodes = 0;
  const failing = [];
  for (const route of [...routes, ...states]) {
    const page = await openPage();
    await goOrGiveUp(page, route);
    // A script tag, not `page.evaluate(source)`: evaluating the UMD bundle as an
    // expression asks CDP to serialise its completion value for nothing.
    await page.addScriptTag({ content: axeSource });
    const results = await page.evaluate(
      async (tags) =>
        await window.axe.run(document, { runOnly: { type: "tag", values: tags } }),
      TAGS,
    );
    const violations = results.violations;
    nodes += violations.reduce((sum, v) => sum + v.nodes.length, 0);
    if (violations.length > 0) {
      failing.push(route);
      console.log(
        `🔴 ${route}  ` +
          violations
            .map((v) => `${v.id}(${v.nodes.length}, ${v.impact})`)
            .join("  "),
      );
      for (const violation of violations) {
        for (const node of violation.nodes.slice(0, 3)) {
          console.log(`     ${node.html.slice(0, 120).replace(/\s+/g, " ")}`);
          const why = (node.any[0]?.message ?? "").split("\n")[0];
          if (why !== "") console.log(`       ${why.slice(0, 160)}`);
        }
      }
    } else {
      console.log(`✅ ${route}`);
    }
    await page.close();
  }

  console.log(
    `\naxe gate: ${routes.length} route(s) derived from the navigation plus ` +
      `${states.length} state(s) no href carries, ${nodes} violation node(s)`,
  );
  // 🔑 Keyed on the FAILING ROUTES, not on the node count: a violation carrying zero nodes
  // printed a red line and exited 0 — a failure indistinguishable from a success, which is
  // one of the two shapes this file exists to prevent.
  if (failing.length > 0) {
    console.error(
      `axe gate RED: ${failing.length} route(s) carry accessibility violations — ` +
        `${failing.join(", ")}`,
    );
    return 1;
  }
  return 0;
}

let code;
try {
  code = await main();
} catch (error) {
  console.error(
    `axe gate: ${error instanceof CannotRun ? error.message : `${error.name}: ${error.message}`}`,
  );
  code = 2;
} finally {
  if (browser !== undefined) {
    await browser.close().catch(() => {});
  }
}
process.exitCode = code;
