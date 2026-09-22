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
// The confirmation a documenting gesture leaves behind (story 6.4's code review). It is reachable
// only after a POST redirects here, so no href carries it and the crawl above cannot find it —
// the same reason `?sort=age` is written out. ⚠️ A literal like `?sort=age` is, and for the same
// reason: the alternative is pressing the gesture inside this gate, which would make an
// accessibility pass WRITE to the store.
const STATE_DOCUMENTED = "?documented=1";
// ⚠️ **An empty queue is CI's permanent state unless something seeds it, and the gate is
// green over it** — measured: with the store emptied, `/triage` carries 0 queue rows, 0
// gesture controls and 0 panes, and the story's own defect replanted exits 0. Set
// `AXE_REQUIRE_QUEUE=1` (CI does) and an empty queue is *the gate could not run*, not a
// pass. Left unset — a developer's empty store — the missing coverage is PRINTED, because
// a silent skip is exactly the "derives nothing and reports success" shape.
const REQUIRE_QUEUE = process.env.AXE_REQUIRE_QUEUE === "1";
// 🔴 **THE STATE THE FIRST QUEUE ROW CANNOT REACH** — story 6.4. The documenting gesture is
// offered on the `undeclared` row alone, and `build_triage` emits every gap, absence and
// conflict row BEFORE that loop: measured on a seeded store, the `New` row is fifth of five,
// so `STATE_SELECTED` above — the FIRST row's own href — can never be a pane carrying the
// gesture. The product's only live control would then be walked by no browser at all.
//
// 🔑 **The row is FOUND, never named.** This gate could have matched the `nouveau:` selector
// prefix, or the row's translated `New` / `Nouveau` label — the second is story 6b.3's
// `role_key` defect (a real, resolving, wrong value), and the first couples a browser gate to
// a Rust identifier. Instead each row's pane is opened and asked whether it carries the
// documenting control: the gate looks for the gesture BY THE GESTURE, so it follows the
// product wherever that population moves.
//
// ⚠️ It needs `OPENCMDB_DOCUMENT_ENABLED` on the server, because `POST /document-all` is not
// mounted without it and the page then renders the control nowhere — by design (story 6.1).
// With `AXE_REQUIRE_GESTURE=1` (CI sets it) a run that finds no such row is *the gate could
// not run*, never a pass: an unmounted route and a clean page are indistinguishable to axe.
const REQUIRE_GESTURE = process.env.AXE_REQUIRE_GESTURE === "1";
// 🔴 **`/ipam` IS THE SAME SHAPE AS AN EMPTY QUEUE, and story 14.2 is where it starts mattering.**
// The screen's whole content — 256 cells each carrying its own accessible name, a subnet selector,
// a legend — exists only when the store holds a subnet. Against a virgin store the route answers
// 200 with one sentence, axe finds nothing to complain about, and the gate reports success over a
// surface it never reached. `a11y/seed.sql` seeds a plan; `AXE_REQUIRE_PLAN=1` (CI sets it) turns
// an unseeded run into *the gate could not run*, on `AXE_REQUIRE_QUEUE`'s own precedent.
// 🔑 The grid is DERIVED, never named: the gate asks the page whether it carries cells, so it
// cannot drift from what the product serves.
const REQUIRE_PLAN = process.env.AXE_REQUIRE_PLAN === "1";
// 🔴 **STORY 14.3b's AUDIT IS THE SAME SHAPE AGAIN.** The findings list, its three words and the warning
// the address field shows before a write exist only when the store holds sightings the plan has
// something to say about. `a11y/seed.sql` seeds one per case; `AXE_REQUIRE_AUDIT=1` (CI sets it) turns
// a seeded run with no finding — or without both a `gap` and an `undeclared` to compare — into *the
// gate could not run*, never a pass.
const REQUIRE_AUDIT = process.env.AXE_REQUIRE_AUDIT === "1";
// 🔴 **STORY 14.5's VLAN IS THE SAME SHAPE A THIRD TIME.** The selector tab that names a segment and
// the note saying the plan carries a VLAN while the ranges, the findings and the offer do not exist
// only when a subnet DECLARES one — so over a plan with no VLAN anywhere this gate would walk a
// screen from which the whole story is absent and report a pass. `AXE_REQUIRE_VLAN=1` (CI sets it)
// turns a run that finds NO note into *the gate could not run*.
// ⚠️ **That is ONE half, and this comment claimed both until the code review.** The check asks
// whether `p.ipam-vlan-note` exists; it cannot tell a plan where EVERY subnet declares a VLAN from
// the seeded one. That the other subnet deliberately declares none — so a tab that names no segment
// is on the same page — is a property of `a11y/seed.sql` that this gate does not defend, and the
// Rust render test is what carries it.
const REQUIRE_VLAN = process.env.AXE_REQUIRE_VLAN === "1";
// 🔴 **STORY 14.6's IPv6 PAGE IS A DIFFERENT BRANCH ENTIRELY** — no grid, no occupancy line, no
// offer, and a sentence saying nothing was checked — so without a seeded IPv6 subnet this gate walks
// the IPv4 screen twice and reports a pass over a surface it never opened. `a11y/seed.sql` carries a
// `2001:db8:1466::/64`; `AXE_REQUIRE_V6=1` (CI sets it) turns a run that cannot reach it into *the
// gate could not run*, which is `AXE_REQUIRE_PLAN`'s own shape a fourth time.
const REQUIRE_V6 = process.env.AXE_REQUIRE_V6 === "1";
// The class the unobservable sentence carries. A CLASS and not its text, for `VLAN_NOTE`'s reason.
const UNOBSERVABLE = "p.ipam-unobservable";
// The note's own class. A CLASS and not its text, because the text is translated and this gate runs
// in the default locale — the rule `PLAN_CELL` and `GESTURE` already follow.
const VLAN_NOTE = "p.ipam-vlan-note";
// An address the seed shows in use, typed into the address field to reach the warning.
const SEEN_ADDRESS = "192.0.2.20";
// 🔴 **THE ONE `/ipam` STATE THE GATE WAS CONFIGURED NEVER TO REACH.** `a11y/seed.sql` always
// seeds a subnet and `AXE_REQUIRE_PLAN=1` REFUSES a run that draws no cell — so the EMPTY plan,
// which is where story 14.2b's AC8 puts the link into the gesture, was the one branch of this
// screen no browser ever opened. This mode walks that branch and nothing else: it is run BEFORE
// the seed, over a plan emptied by `a11y/empty-plan.sql`, and it refuses (2) if the plan is not
// empty — because a mode that silently measured a populated screen would report a pass over the
// state it exists for.
const EMPTY_PLAN = process.env.AXE_EMPTY_PLAN === "1";
// The grid's cells. A class the stylesheet guard pins, not a label and not a Rust identifier.
const PLAN_CELL = "ul.ipam-grid li.ipam-cell";
const PLAN_ROUTE = "/ipam";
// The control the gate looks for. A CLASS the stylesheet guard already pins, not a label.
const GESTURE = "button.btn-document";
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

  // ── The empty-plan branch, walked alone and only when asked ───────────────
  if (EMPTY_PLAN) {
    const page = await openPage();
    await goOrGiveUp(page, PLAN_ROUTE);
    // 🔴 NO CELL AND NO SUBNET, and cells alone were not enough (story 14.2b's review): a subnet
    // too large to draw, or an unknown `?subnet=`, also draws zero cells over a plan that is full.
    const cells = await page.$$eval(PLAN_CELL, (nodes) => nodes.length);
    const subnets = await page.$$eval("nav.filters a.filter", (nodes) => nodes.length);
    if (cells !== 0 || subnets !== 0) {
      cannotRun(
        `${PLAN_ROUTE} draws ${cells} cell(s) and offers ${subnets} subnet(s), so the plan is NOT ` +
          `empty and this mode measured the wrong state. Run it before the seed, over a store ` +
          `emptied by a11y/empty-plan.sql.`,
      );
    }
    // The deliverable of AC8: the empty plan LINKS to the gesture, and the link's target is
    // OPEN — an anchor onto a collapsed disclosure is a door that opens onto a door.
    const door = await page.evaluate(() => {
      const link = document.querySelector('a[href="#ipam-form-subnet"]');
      const target = document.getElementById("ipam-form-subnet");
      return {
        link: link === null ? null : (link.textContent ?? "").trim(),
        open: target === null ? null : target.open,
      };
    });
    // 🔴 A PRODUCT DEFECT, NOT "COULD NOT RUN": the page answered and the plan is empty, so a
    // missing link is the product failing AC8 — exit 1. It exited 2 until story 14.2b's review.
    const linkMissing = door.link === null || door.link === "";
    if (linkMissing) {
      console.log(`🔴 ${PLAN_ROUTE} (empty plan)  shows no link into the subnet form, so AC8 is not on the page`);
    }
    await page.addScriptTag({ content: axeSource });
    const results = await page.evaluate(
      async (tags) => await window.axe.run(document, { runOnly: { type: "tag", values: tags } }),
      TAGS,
    );
    const violations = results.violations;
    const emptyNodes = violations.reduce((sum, v) => sum + v.nodes.length, 0);
    for (const violation of violations) {
      console.log(`🔴 ${PLAN_ROUTE} (empty plan)  ${violation.id}(${violation.nodes.length}, ${violation.impact})`);
    }
    if (door.open !== true) {
      console.log(
        `🔴 ${PLAN_ROUTE} (empty plan)  the link's target is collapsed: an in-page anchor onto a ` +
          `closed <details> is a door that opens onto a door`,
      );
    }
    await page.close();
    const clean = violations.length === 0 && door.open === true && !linkMissing;
    console.log(
      `\naxe gate (empty plan): 1 route, ${emptyNodes} violation node(s), ` +
        // 🔴 The link's state is said only when there IS a link: under a mutation removing it this
        // line read "link opens onto an open form" beside the 🔴 saying the link was missing.
        `link ${linkMissing ? "MISSING" : door.open === true ? "opens onto an open form" : "opens onto a closed form"}`,
    );
    return clean ? 0 : 1;
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
  const queueRows = await seedPage.$$eval(".queue .queue-row > a", (rows) =>
    rows.map((row) => row.getAttribute("href")),
  );
  const firstRow = queueRows.length > 0 ? queueRows[0] : null;
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

  const states = [SEED + STATE_SORT, SEED + STATE_DOCUMENTED];
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

  // `/ipam`'s grid, measured the same way: opened, then asked whether it carries cells.
  let planIndistinguishable = null;
  // Story 14.3b: the audit's two words told apart without colour, and the warning's own axe run.
  let auditIndistinguishable = null;
  // Decision 8's marker, measured in the browser because no rule and no guard can see it.
  let markerFailure = null;
  // Story 14.5: two selector tabs carrying one accessible NAME. A product defect, not a harness one
  // — the page renders two links a keyboard user cannot tell apart — so it joins `failing` rather
  // than answering 2.
  let tabNameClash = null;
  // Story 14.6: the IPv6 page drew a grid, or asserted concordance. A product failure, not a
  // harness one.
  let v6Failure = null;
  // A product regression the warning state reveals: the form is there, the store holds the
  // sighting, and no warning arrives. 🔴 It was `cannotRun` (exit 2) until the code review — *the
  // gate could not run* and *the product stopped warning* are not the same answer, and the second
  // one is the one this gate exists to report.
  let warningFailure = null;
  let warningViolations = 0;
  // Whether axe actually ran over the page with the warning shown — the state is counted in the
  // summary only when it was measured.
  let warningStateMeasured = false;
  {
    const page = await openPage();
    try {
      await goOrGiveUp(page, PLAN_ROUTE);
      const cells = await page.$$eval(PLAN_CELL, (nodes) => nodes.length);
      if (cells === 0 && REQUIRE_PLAN) {
        cannotRun(
          `${PLAN_ROUTE} draws no cell, so the surface this state exists to measure — the ` +
            `grid, its per-cell names, the selector, the legend — is not on the page. Seed a ` +
            `plan before the gate runs.`,
        );
      } else if (cells === 0) {
        console.log(
          `⚠️  ${PLAN_ROUTE} draws no cell: the grid was NOT measured. ` +
            `Set AXE_REQUIRE_PLAN=1 to make that a refusal rather than a gap.`,
        );
      } else {
        console.log(`   ${PLAN_ROUTE}: ${cells} cell(s)`);
        // 🔴 **THE ONE CHECK ONLY A BROWSER CAN MAKE, and story 14.2 exists partly because of
        // it.** Until this story `.ipam-cell-free` was `background: transparent` over a base
        // that already draws the border, so a free cell and an uncovered one were IDENTICAL in
        // every computed property — measured, not supposed. A stylesheet guard cannot see that:
        // both rules exist, both are correct, and the page is still wrong (story 6.4's *a source
        // guard cannot see a cascade*). And WCAG 1.4.1 is not satisfied by a colour difference
        // either, so the assertion is on a NON-COLOUR property.
        const pair = await page.evaluate(() => {
          const read = (sel) => {
            const el = document.querySelector(sel);
            if (!el) return null;
            const c = getComputedStyle(el);
            return { image: c.backgroundImage, style: c.borderTopStyle };
          };
          return { free: read("li.ipam-cell-free"), blank: read("li.ipam-cell-not-covered") };
        });
        if (pair.free && pair.blank) {
          if (pair.free.image === pair.blank.image && pair.free.style === pair.blank.style) {
            planIndistinguishable =
              `a free cell and one the plan does not cover render IDENTICALLY in every ` +
              `non-colour property measured: background-image ${pair.free.image}, ` +
              `border-style ${pair.free.style}, on both`;
            console.log(`🔴 ${PLAN_ROUTE}  ${planIndistinguishable}`);
          } else {
            console.log(
              `   ${PLAN_ROUTE}: free and not-covered differ without colour ` +
                `(background-image ${pair.free.image === "none" ? "none" : "pattern"} vs ` +
                `${pair.blank.image === "none" ? "none" : "pattern"})`,
            );
          }
        } else if (REQUIRE_PLAN) {
          cannotRun(
            `${PLAN_ROUTE} carries no free cell or no uncovered cell, so the pair this check ` +
              `exists to compare is not on the page. The seed must draw both.`,
          );
        }

        // ── Story 14.5: the VLAN is on the screen, and the tabs stay distinct ──────────────
        // 🔑 **The PROPERTY is the criterion, not the appearance** (AC4): no two selector tabs may
        // carry the same accessible name. `tab_labels` imposes it on the SET and a Rust property test
        // asserts it over adversarial labels; what this adds is the SERVED page, over a store whose
        // rows nobody wrote for a test.
        // ⚠️ **It reads `textContent`, and this comment claimed the COMPUTED accessible name until
        // the code review** — so `aria-label` is invisible to it, and so is any CSS that hides a
        // segment (`display:none` leaves the text in place). What it does measure, and what the
        // defect actually is, is two links reading alike.
        const vlan = await page.evaluate((noteSelector) => {
          // 🔑 `nav.filters` is what `/ipam` renders its selector as, in BOTH branches (the drawn
          // plan and the one too large to draw). Scoped to `main` so a navigation filter elsewhere
          // on the page could never be mistaken for a subnet tab.
          const tabs = [...document.querySelectorAll("main nav.filters a.filter")];
          return {
            note: document.querySelector(noteSelector) !== null,
            names: tabs.map((tab) => (tab.textContent ?? "").replace(/\s+/g, " ").trim()),
          };
        }, VLAN_NOTE);
        const duplicates = vlan.names.filter(
          (name, i) => vlan.names.indexOf(name) !== i,
        );
        if (duplicates.length > 0) {
          tabNameClash =
            `two selector tabs carry the SAME accessible name ` +
            `(${[...new Set(duplicates)].join(", ")}), so a keyboard user cannot tell which ` +
            `segment of the plan a link leads to`;
          console.log(`🔴 ${PLAN_ROUTE}  ${tabNameClash}`);
        }
        if (!vlan.note) {
          if (REQUIRE_VLAN) {
            cannotRun(
              `${PLAN_ROUTE} carries no ${VLAN_NOTE}: no subnet in this store declares a VLAN, so ` +
                `the tab that names a segment and the sentence saying the audit ignores it are ` +
                `both absent. Seed a VLAN (a11y/seed.sql) before the gate runs.`,
            );
          }
          console.log(
            `⚠️  ${PLAN_ROUTE} declares no VLAN: story 14.5's screen was NOT measured. ` +
              `Set AXE_REQUIRE_VLAN=1 to make that a refusal rather than a gap.`,
          );
        } else {
          // 🔴 **THE SUCCESS LINE WAS UNCONDITIONAL, so one run printed the clash and then announced
          // distinctness — the second sentence asserting as fact what the first had just disproved.**
          // Measured by the code review's edge layer on a colliding store, and it is story 14.4's
          // *"will be refused" / "not a refusal"* in one `aria-live` region, one file over. A report
          // that contradicts itself is worse than a silent one: the reader believes the last line.
          console.log(
            tabNameClash === null
              ? `   ${PLAN_ROUTE}: the VLAN note is on the page and ${vlan.names.length} selector ` +
                  `tab(s) carry distinct names`
              : `   ${PLAN_ROUTE}: the VLAN note is on the page; the tab names are NOT distinct — ` +
                  `see the line above`,
          );
        }

        // ── Story 14.6: the IPv6 page, which is a branch no other state reaches ────────────
        // 🔑 Found by FOLLOWING the selector rather than by naming an id: the gate opens each tab in
        // turn and asks whether the page it lands on carries the unobservable sentence. A gate that
        // named the seed's UUID would measure the seed; this measures the product.
        // 🔴 **THE TAB IS FOUND BY ITS CIDR, NOT BY THE SENTENCE THE PRODUCT IS MEANT TO RENDER**,
        // and the first version of this probe got that backwards. It walked the tabs looking for
        // `p.ipam-unobservable`, so the product REMOVING that sentence — the family branch gone, the
        // template arm gone, the key blanked — produced *no tab carries it*, exit **2**, and a
        // message asserting *the plan holds no IPv6 subnet*: a cause the gate never checked, over a
        // committed seed. That is this project's own rule broken (a cause needs a check) and the
        // keyboard gate's recorded defect one file over (*a real failure converted into could not
        // run*). ⚠️ Worse, it made the two product checks below UNREACHABLE: they only ever ran on a
        // page that already carried the sentence, so the gate could never observe the state
        // mutation T6 produced — an IPv6 page that drew a grid.
        //
        // 🔑 A colon in the tab's own name is what identifies an IPv6 subnet, and it comes from the
        // ADDRESS rather than from any copy this story wrote.
        const v6 = await (async () => {
          const tabs = await page.evaluate(() =>
            [...document.querySelectorAll("main nav.filters a.filter")].map((tab) => ({
              href: tab.getAttribute("href") ?? "",
              name: (tab.textContent ?? "").trim(),
            })),
          );
          for (const { href, name } of tabs) {
            if (!name.includes(":")) continue;
            const probe = await openPage();
            await goOrGiveUp(probe, href);
            const found = await probe.evaluate((sel) => {
              const note = document.querySelector(sel);
              return {
                unobservable: note !== null,
                cells: document.querySelectorAll("ul.ipam-grid li.ipam-cell").length,
                allClear: document.querySelector(".ipam-audit p.empty:not(.ipam-unobservable)") !== null,
              };
            }, UNOBSERVABLE);
            {
              // axe on the branch itself: its markup is not the grid's, so the seeded pass above
              // measured none of it.
              await probe.addScriptTag({ content: axeSource });
              const run = await probe.evaluate(
                async (tags) =>
                  await window.axe.run(document, { runOnly: { type: "tag", values: tags } }),
                TAGS,
              );
              const nodes = run.violations.reduce((sum, v) => sum + v.nodes.length, 0);
              for (const violation of run.violations) {
                console.log(
                  `🔴 ${href} (IPv6)  ${violation.id}(${violation.nodes.length}, ${violation.impact})`,
                );
              }
              await probe.close();
              return { ...found, href, nodes };
            }
            await probe.close();
          }
          return null;
        })();
        if (v6 === null) {
          if (REQUIRE_V6) {
            cannotRun(
              `no selector tab of ${PLAN_ROUTE} names an IPv6 subnet, so the branch story 14.6 ` +
                `exists for was not measured. Seed one (a11y/seed.sql). ⚠️ This says the PLAN holds ` +
                `no IPv6 subnet — which the tab names establish — and says nothing about whether ` +
                `the product would render it correctly; that is the check below.`,
            );
          }
          console.log(
            `⚠️  ${PLAN_ROUTE} holds no IPv6 subnet: story 14.6's page was NOT measured. ` +
              `Set AXE_REQUIRE_V6=1 to make that a refusal rather than a gap.`,
          );
        } else {
          // 🔴 The two things the family decides, measured on the SERVED page rather than reasoned
          // about: no grid at any size, and the all-clear REPLACED rather than accompanied.
          if (!v6.unobservable) {
            v6Failure =
              `${v6.href} is an IPv6 subnet and carries no ${UNOBSERVABLE}: the page says nothing ` +
              `about not having been checked, so an empty audit reads as *nothing is wrong*`;
            console.log(`🔴 ${PLAN_ROUTE}  ${v6Failure}`);
          } else if (v6.cells > 0) {
            v6Failure = `${v6.href} drew ${v6.cells} grid cell(s) on an IPv6 subnet`;
            console.log(`🔴 ${PLAN_ROUTE}  ${v6Failure}`);
          } else if (v6.allClear) {
            v6Failure =
              `${v6.href} rendered the all-clear on an IPv6 subnet — the product asserting ` +
              `concordance about a plan its only connector never looks at`;
            console.log(`🔴 ${PLAN_ROUTE}  ${v6Failure}`);
          } else {
            console.log(
              `   ${v6.href}: the IPv6 page draws no grid, says nothing was checked, ` +
                `${v6.nodes} violation node(s)`,
            );
          }
        }

        // ── Story 14.3b: the findings list, its words told apart WITHOUT colour ─────────────
        // 🔑 Constraint 6 and AC2: `gap` and `undeclared` are distinct by word AND by treatment. A
        // stylesheet guard sees that both rules exist, never that they render differently — so the
        // browser compares a NON-colour property, the badge's border style.
        // 🔴 **THREE WORDS, NOT TWO, AND MORE THAN A BORDER STYLE.** Until the code review this
        // compared `border-style` on `gap` and `undeclared` alone — so « Conflit d'adresse », the
        // third word and the one FR24 exists for, was compared with nothing and could have shipped
        // identical to either. The tuple is every NON-COLOUR channel the three rules use, and the
        // three must be pairwise distinct on it: 1.4.1 is not satisfied by a colour difference.
        const audit = await page.evaluate(() => {
          const read = (sel) => {
            const el = document.querySelector(sel);
            if (el === null) return null;
            const c = getComputedStyle(el);
            return `${c.borderTopStyle}/${c.borderTopWidth}/${c.fontWeight}/${c.textDecorationLine}`;
          };
          return {
            findings: document.querySelectorAll(".ipam-audit .ipam-finding").length,
            gap: read(".ipam-word-gap"),
            undeclared: read(".ipam-word-undeclared"),
            conflict: read(".ipam-word-conflict"),
          };
        });
        const words = [
          ["gap", audit.gap],
          ["undeclared", audit.undeclared],
          ["conflict", audit.conflict],
        ];
        const missing = words.filter(([, treatment]) => treatment === null);
        if (audit.findings === 0 || missing.length > 0) {
          if (REQUIRE_AUDIT) {
            cannotRun(
              `${PLAN_ROUTE} shows ${audit.findings} finding(s) and carries no ` +
                `${missing.map(([name]) => name).join(", ")}: the audit this state exists to ` +
                `measure is not on the page. Seed the sightings (a11y/seed.sql) before the gate runs.`,
            );
          }
          console.log(
            `⚠️  ${PLAN_ROUTE} does not carry all three audit words: the audit was NOT measured. ` +
              `Set AXE_REQUIRE_AUDIT=1 to make that a refusal rather than a gap.`,
          );
        } else {
          const same = words.flatMap(([name, treatment], i) =>
            words
              .slice(i + 1)
              .filter(([, other]) => other === treatment)
              .map(([other]) => `${name} and ${other} (${treatment})`),
          );
          if (same.length > 0) {
            auditIndistinguishable =
              `two of the audit's words carry the SAME non-colour treatment — ${same.join(", ")} — ` +
              `so they are told apart by nothing but their text and their colour`;
            console.log(`🔴 ${PLAN_ROUTE}  ${auditIndistinguishable}`);
          } else {
            console.log(
              `   ${PLAN_ROUTE}: ${audit.findings} finding(s); the three words differ without ` +
                `colour (${words.map(([name, t]) => `${name} ${t}`).join(" · ")})`,
            );
          }
        }

        // ── Story 14.3b, decision 8: the SEEN marker's contrast, computed here ──────────────
        // 🔴 **axe CANNOT SEE A PSEUDO-ELEMENT**, and the marker is one — so the condition Guy put
        // on it (3:1 against every fill a cell can carry) is invisible to every rule this file runs
        // and to every guard in Rust, which can read that the rule EXISTS and never what it renders
        // against. The story first dropped the marker on arithmetic done by hand and got it wrong
        // in the safe direction; this computes the ratio in the browser, on the four fills, and
        // fails under 3:1. ⚠️ A fill that is a PATTERN over a transparent background is measured
        // against the page's own ground, which is what the eye sees between the pattern's strokes.
        const marker = await page.evaluate(() => {
          const parse = (value) => {
            const found = (value ?? "").match(/[\d.]+/g);
            if (found === null || found.length < 3) return null;
            if (found.length > 3 && Number(found[3]) === 0) return null;
            return found.slice(0, 3).map(Number);
          };
          const luminance = (rgb) => {
            const channel = (v) => {
              const s = v / 255;
              return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
            };
            return (
              0.2126 * channel(rgb[0]) + 0.7152 * channel(rgb[1]) + 0.0722 * channel(rgb[2])
            );
          };
          const ratio = (a, b) => {
            const [hi, lo] = luminance(a) > luminance(b) ? [a, b] : [b, a];
            return (luminance(hi) + 0.05) / (luminance(lo) + 0.05);
          };
          const cell = document.querySelector("li.ipam-cell-seen");
          if (cell === null) return { cells: 0 };
          const dot = parse(getComputedStyle(cell, "::after").backgroundColor);
          if (dot === null) return { cells: 1, dot: null };
          const ground = parse(getComputedStyle(document.body).backgroundColor) ?? [255, 255, 255];
          const fills = [
            "ipam-cell-defined",
            "ipam-cell-free",
            "ipam-cell-infrastructure",
            "ipam-cell-not-covered",
          ].map((modifier) => {
            const el = document.querySelector(`li.${modifier}`);
            const own = el === null ? null : parse(getComputedStyle(el).backgroundColor);
            return {
              modifier,
              measured: el !== null,
              onGround: own === null,
              ratio: Math.round(ratio(dot, own ?? ground) * 100) / 100,
            };
          });
          return {
            cells: document.querySelectorAll("li.ipam-cell-seen").length,
            dot,
            fills,
          };
        });
        if (marker.cells === 0) {
          if (REQUIRE_AUDIT) {
            cannotRun(
              `${PLAN_ROUTE} draws no cell the network has been seen on, so decision 8's marker ` +
                `is on no cell this gate can measure. Seed the sightings before the gate runs.`,
            );
          }
          console.log(`⚠️  ${PLAN_ROUTE}: no seen cell, so the marker was NOT measured.`);
        } else if (marker.dot === null) {
          markerFailure =
            `the seen marker paints NOTHING: \`.ipam-cell-seen::after\` has no background colour, ` +
            `so the cells the network has been seen on are marked by nothing at all`;
          console.log(`🔴 ${PLAN_ROUTE}  ${markerFailure}`);
        } else {
          const missedFill = marker.fills.filter((fill) => !fill.measured);
          if (missedFill.length > 0 && REQUIRE_AUDIT) {
            cannotRun(
              `${PLAN_ROUTE} draws no ${missedFill.map((f) => f.modifier).join(", ")} cell, so ` +
                `the marker was not measured against every fill. The seed must draw all four.`,
            );
          }
          const tooFaint = marker.fills.filter((fill) => fill.ratio < 3);
          if (tooFaint.length > 0) {
            markerFailure =
              `the seen marker falls under 3:1 on ${tooFaint
                .map((f) => `${f.modifier} (${f.ratio}:1)`)
                .join(", ")} — decision 8 keeps the marker only while it clears 3:1 on every fill`;
            console.log(`🔴 ${PLAN_ROUTE}  ${markerFailure}`);
          } else {
            console.log(
              `   ${PLAN_ROUTE}: ${marker.cells} seen cell(s); marker contrast ` +
                marker.fills
                  .map((f) => `${f.modifier} ${f.ratio}:1${f.onGround ? " (on the ground)" : ""}`)
                  .join(" · "),
            );
          }
        }

        // ── Story 14.3b: the warning the address field shows BEFORE the write, under axe ────
        // It appears only when someone types, so no URL state can reach it: the gate opens the
        // address form, types an address the seed shows in use, waits for the warning, and runs axe
        // over the page as it then stands.
        const hasForm = await page.$("#ipam-form-address");
        if (hasForm !== null) {
          await page.$eval("#ipam-form-address", (el) => {
            el.open = true;
          });
          const asked = page
            .waitForResponse((r) => r.url().includes("/ipam/address-check"), {
              timeout: NAV_TIMEOUT_MS,
            })
            .catch(() => null);
          await page.focus("#ipam-addr");
          await page.keyboard.type(SEEN_ADDRESS);
          const answered = await asked;
          const warning = await page
            .waitForFunction(
              () => (document.getElementById("ipam-addr-warning")?.textContent ?? "").trim() !== "",
              { timeout: NAV_TIMEOUT_MS },
            )
            .then(() => true)
            .catch(() => false);
          if (answered === null || !warning) {
            // 🔴 **THE FORM IS ON THE PAGE AND THE SEEDED SIGHTING IS IN THE STORE, so a warning
            // that does not arrive is the PRODUCT** — reported as a failure (1), not as *the gate
            // could not run* (2). The `cannotRun` this replaces would have turned a broken warning
            // into an amber *fix your harness*, which is the one thing a gate must never say about
            // a real regression.
            warningFailure =
              `typing ${SEEN_ADDRESS} into the address field produced no warning: the form is on ` +
              `the page and the store holds that sighting, so the warning is broken`;
            console.log(`🔴 ${PLAN_ROUTE}  ${warningFailure}`);
          } else {
            await page.addScriptTag({ content: axeSource });
            const results = await page.evaluate(
              async (tags) =>
                await window.axe.run(document, { runOnly: { type: "tag", values: tags } }),
              TAGS,
            );
            warningStateMeasured = true;
            warningViolations = results.violations.reduce((sum, v) => sum + v.nodes.length, 0);
            if (results.violations.length > 0) {
              for (const violation of results.violations) {
                console.log(
                  `🔴 ${PLAN_ROUTE} (address warning)  ${violation.id}(${violation.nodes.length}, ${violation.impact})`,
                );
              }
            } else {
              console.log(`✅ ${PLAN_ROUTE} (address warning shown)`);
            }
          }
        } else if (REQUIRE_AUDIT) {
          // No FORM is a harness or seed problem — the plan has no subnet in force, so the two
          // forms that hang from one are not rendered at all. That really is *could not run*.
          cannotRun(`${PLAN_ROUTE} carries no address form, so the warning cannot be reached.`);
        } else {
          // 🔴 **AND IT USED TO SKIP IN SILENCE.** Without `AXE_REQUIRE_AUDIT` the missing form
          // produced no line at all, so a run that measured nothing looked exactly like a run that
          // measured everything — the shape this whole file exists to prevent, one branch over.
          console.log(
            `⚠️  ${PLAN_ROUTE} carries no address form: the warning was NOT measured. ` +
              `Set AXE_REQUIRE_AUDIT=1 to make that a refusal rather than a gap.`,
          );
        }
      }
    } finally {
      await page.close();
    }
  }

  // The pane that carries the documenting gesture, found by opening each row and asking for
  // the control — never by recognising a selector or a translated word (see `REQUIRE_GESTURE`).
  let gestureRow = null;
  for (const route of queueRows) {
    const page = await openPage();
    await goOrGiveUp(page, route);
    const controls = await page.$$eval(GESTURE, (found) => found.length);
    await page.close();
    if (controls > 0) {
      gestureRow = route;
      break;
    }
  }
  if (gestureRow !== null) {
    // The first-row state is already in `states` and may BE this row; a duplicate would be
    // walked twice and reported as two states, which overstates the coverage.
    if (!states.includes(gestureRow)) {
      states.push(gestureRow);
    }
  } else if (REQUIRE_GESTURE) {
    cannotRun(
      `no queue row at ${SEED} carries \`${GESTURE}\`, so the product's only live gesture is ` +
        `on no page this gate walks. Either the store holds no undeclared sighting, or the ` +
        `server was started without OPENCMDB_DOCUMENT_ENABLED — neither is a clean product.`,
    );
  } else {
    console.log(
      `⚠️  no queue row carries ${GESTURE}: the documenting gesture was NOT measured. ` +
        `Set AXE_REQUIRE_GESTURE=1 to make that a refusal rather than a gap.`,
    );
  }

  // ── Walk them ─────────────────────────────────────────────────────────────
  let nodes = 0;
  const failing = [];
  // The grid's own non-colour check joins the product failures: it is a defect in what the page
  // renders, exactly like an axe violation, and it must not be reported through a different door.
  if (planIndistinguishable !== null) {
    failing.push(`${PLAN_ROUTE} (free vs not-covered)`);
  }
  if (auditIndistinguishable !== null) {
    failing.push(`${PLAN_ROUTE} (the audit's three words)`);
  }
  if (markerFailure !== null) {
    failing.push(`${PLAN_ROUTE} (the seen marker)`);
  }
  if (tabNameClash !== null) {
    failing.push(`${PLAN_ROUTE} (two selector tabs share a name)`);
  }
  if (v6Failure !== null) {
    failing.push(`${PLAN_ROUTE} (the IPv6 page)`);
  }
  if (warningFailure !== null) {
    failing.push(`${PLAN_ROUTE} (the address warning did not appear)`);
  }
  // 🔑 The warning state is a STATE, counted as one — its violation nodes join the total and its
  // name joins the list of states measured. ⚠️ It was added to `nodes` while being absent from the
  // state COUNT the summary prints, so the line reported N nodes over fewer states than had been
  // walked: a numerator and a denominator that do not describe the same set.
  let extraStates = 0;
  if (warningStateMeasured) {
    extraStates += 1;
    nodes += warningViolations;
    if (warningViolations > 0) {
      failing.push(`${PLAN_ROUTE} (address warning)`);
    }
  }
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
      `${states.length + extraStates} state(s) no href carries, ${nodes} violation node(s)`,
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
