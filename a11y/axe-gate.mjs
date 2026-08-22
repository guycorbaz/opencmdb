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
// 🔴 **The exit codes distinguish two failures a CI cannot otherwise tell apart**, and the
// distinction was measured rather than assumed: with the database paused, the derivation
// succeeds, `/triage` blocks on sqlx's acquire timeout, the navigation times out — and a
// naive harness dies with the same code it uses for *"the product has violations"*.
//
//   0 — every route clean
//   1 — the product has accessibility violations   (a real regression: fix the product)
//   2 — the gate could not run                     (fix the harness or the environment)
import puppeteer from "puppeteer-core";
import { readFileSync } from "node:fs";

const BASE = process.env.AXE_BASE_URL ?? "http://127.0.0.1:8080";
const USER = process.env.OPENCMDB_BASIC_USER ?? "";
const PASS = process.env.OPENCMDB_BASIC_PASSWORD ?? "";
const CHROME = process.env.AXE_CHROME ?? "/usr/bin/google-chrome";
// The seed page is where the navigation is read from. Any screen serves the same nav.
const SEED = process.env.AXE_SEED_ROUTE ?? "/triage";
// 🔑 A FLOOR, not a nicety: a harness that derives nothing and reports success is the
// failure mode this file exists to avoid. Ten is what `Screen::ALL` carries today; if a
// screen is added, this number moves deliberately.
const EXPECTED_ROUTES = 10;
const TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"];
const NAV_TIMEOUT_MS = 20_000;

const axeSource = readFileSync(
  new URL("./node_modules/axe-core/axe.min.js", import.meta.url),
  "utf8",
);

/** The gate could not run. Never conflated with "the product has violations". */
function cannotRun(message) {
  console.error(`axe gate: ${message}`);
  process.exit(2);
}

let browser;
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

/** Navigate, or end the run with exit 2 — never with the violation code. */
async function goOrGiveUp(page, route) {
  let response;
  try {
    response = await page.goto(BASE + route, { waitUntil: "networkidle0" });
  } catch (error) {
    await browser.close();
    cannotRun(`${route} did not answer — ${error.message}`);
  }
  const status = response?.status() ?? 0;
  if (status !== 200) {
    await browser.close();
    cannotRun(`${route} answered ${status}, so nothing there can be measured`);
  }
  return page;
}

// ── Derive the routes from the rendered navigation ──────────────────────────
const seedPage = await openPage();
await goOrGiveUp(seedPage, SEED);
const routes = await seedPage.$$eval("nav.nav a.nav-entry", (entries) =>
  entries.map((entry) => entry.getAttribute("href")),
);
await seedPage.close();

if (routes.length < EXPECTED_ROUTES) {
  await browser.close();
  cannotRun(
    `derived ${routes.length} route(s) from the navigation at ${SEED}, expected at ` +
      `least ${EXPECTED_ROUTES}. A harness that finds nothing must not pass.`,
  );
}

// ── Walk them ───────────────────────────────────────────────────────────────
let nodes = 0;
const failing = [];
for (const route of routes) {
  const page = await openPage();
  await goOrGiveUp(page, route);
  await page.evaluate(axeSource);
  const results = await page.evaluate(
    async (tags) =>
      await window.axe.run(document, { runOnly: { type: "tag", values: tags } }),
    TAGS,
  );
  const violations = results.violations;
  const routeNodes = violations.reduce((sum, v) => sum + v.nodes.length, 0);
  nodes += routeNodes;
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
await browser.close();

console.log(
  `\naxe gate: ${routes.length} route(s) derived from the navigation, ` +
    `${nodes} violation node(s)`,
);
if (nodes > 0) {
  console.error(
    `axe gate RED: ${failing.length} route(s) carry accessibility violations — ` +
      `${failing.join(", ")}`,
  );
  process.exit(1);
}
process.exit(0);
