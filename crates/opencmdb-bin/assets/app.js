// opencmdb — the keyboard layer.
//
// 🔴 **What stood here until story 6b.11 was carried by nothing.** Twelve lines listening
// for `htmx:afterSwap` and focusing `#gap-card` — measured on the ten served screens:
// **zero `hx-get`/`hx-post` anywhere, and `id="gap-card"` on none of them.** Neither the
// event nor the element existed. The focus-after-swap contract it claimed is registered
// with story 6.4, which introduces the first swap; writing a guard for it today would
// have been a guard placed where the defect cannot occur.
//
// 🔑 **THE SHAPE IS (E), and it was chosen by MEASUREMENT** (Guy's arbitration 4). The
// selection is a URL (`?sel=`) and the detail pane is server-rendered, so "the arrow moves
// the selection" had three readings; all three were prototyped in Chrome against a 60-row
// queue, and they differ by **20×**:
//
//   (A) the arrow navigates          a HELD arrow moved 10 rows of 20, at 161 requests —
//                                    presses arriving mid-load are LOST, because the old
//                                    document is being torn down and this file has not run
//   (B) the arrow moves focus only   0 requests, but the URL never moves, so the SELECTION
//                                    does not move and the detail pane keeps its old row
//   (E) this one                     20 rows of 20 at every cadence, 8 requests
//
// ⚠️ **(E)'s stated cost, and the code review measured all three terms of it.** The
// sentence here read *"for 250 ms the highlighted row and the URL disagree"*, and 250 ms is
// the TIMER, not the observed window: the eye and the screen reader were measured
// disagreeing for **360 ms warm**, 1551 ms behind a 1200 ms document and 2240 ms behind a
// 1900 ms one — and there were **three** disagreeing parties, not two, the third being the
// accessible `current` state, unnamed in the story whose subject is accessibility. What is
// left of that cost after story 6b.11's repair pass: the URL still trails the highlight by
// one settle, and nothing else does — `aria-current` moves with the class, the operator's
// own gesture always outranks the pending timer, and the focus survives the navigation.
(function () {
  "use strict";

  // 🔑 Idempotence. A double-registered listener moves two rows per press — harmless on a
  // full document load, and a real defect the day story 6.4 adds a swap that re-runs this
  // file. Two lines now, against a bug that would be measured in a browser later.
  if (window.__opencmdbKeyboard) return;
  window.__opencmdbKeyboard = true;

  // How long the operator must stop before the URL catches up with the highlight.
  var SETTLE_MS = 250;
  // The note a keyboard-driven navigation leaves for the document that replaces it.
  var RESTORE_KEY = "opencmdb.kbd.restore";

  var pending = null;

  function rows() {
    return Array.prototype.slice.call(
      document.querySelectorAll(".queue .queue-row > a"),
    );
  }

  /** The row the operator is on: the focused one, else the selected one. */
  function currentIndex(all) {
    var focused = all.indexOf(document.activeElement);
    if (focused !== -1) return focused;
    for (var i = 0; i < all.length; i++) {
      if (all[i].getAttribute("aria-current") === "true") return i;
    }
    return -1;
  }

  /** Forget any queued navigation. The operator's own gesture always outranks it. */
  function cancelPending() {
    if (pending !== null) {
      window.clearTimeout(pending);
      pending = null;
    }
  }

  /** Move the highlight to `row` now; let the URL follow once the operator stops. */
  function select(all, row) {
    for (var i = 0; i < all.length; i++) {
      var isRow = all[i] === row;
      all[i].parentNode.classList.toggle("selected", isRow);
      // 🔴 **`aria-current` moves WITH the class, and until story 6b.11's repair it did
      // not.** The class is what the eye reads and `aria-current` is what a screen reader
      // reads, so for the whole settle window the DOM named two different rows as current —
      // measured at 360 ms even on a warm store. This file also READS the attribute back as
      // its own fallback position, so a stale one made the highlight jump backwards
      // whenever focus had left the queue without a navigation.
      if (isRow) {
        all[i].setAttribute("aria-current", "true");
      } else {
        all[i].removeAttribute("aria-current");
      }
    }
    row.focus();
    cancelPending();
    // 🔑 The row's OWN href, never a URL rebuilt from the selector: the hrefs already carry
    // `?sort=age`, so the sort survives arrow navigation for free — and stops surviving the
    // moment someone constructs the address instead of reading it.
    var href = row.getAttribute("href");
    // ↓ then ↑ back to the row already selected is a net movement of zero, and it used to
    // cost a full document load to the identical URL — the operator losing their place for
    // having overshot by one and corrected.
    if (absolute(href) === window.location.href) return;
    pending = window.setTimeout(function () {
      pending = null;
      remember(href);
      // 🔑 `replace`, not `assign`. Every settled press pushed a history entry, so Back
      // walked the operator's own selections one by one and could no longer leave `/triage`
      // (measured: `history.length` 2 → 5 for three presses) — while `_triage.html`'s own
      // comment justifies choosing `aria-current` over `role="button"` precisely to keep the
      // back button. One entry per visit, which is what a selection is.
      window.location.replace(href);
    }, SETTLE_MS);
  }

  /** An href as the browser would resolve it, so it can be compared with `location`. */
  function absolute(href) {
    var probe = document.createElement("a");
    probe.href = href;
    return probe.href;
  }

  /**
   * Leave a note for the document that is about to replace this one.
   *
   * 🔴 Without it every settle navigation dropped focus to `<body>`: the arrows kept working
   * (they fall back to `aria-current`), but a Tab after arrowing to row 3 threw the operator
   * back to the first navigation entry — twelve Tab presses to re-enter the queue, measured.
   * ⚠️ The marker is what keeps this from being an autofocus: an ordinary click or a pasted
   * URL loads with focus where the browser put it, and only a keyboard-driven settle
   * restores the row. ⚠️ **`sessionStorage` throwing is a STATED limit, not a covered case** —
   * the code review's edge layer blocked storage access and measured the operator back on
   * `<body>` after the settle, i.e. the original defect returning silently. The `try`/`catch`
   * prevents the crash and there is no fallback: without storage there is nowhere to leave a
   * note for the next document, and a restore keyed on the URL alone would be the autofocus
   * this whole mechanism exists to avoid.
   */
  function remember(href) {
    try {
      window.sessionStorage.setItem(RESTORE_KEY, absolute(href));
    } catch (error) {
      // No marker, no restore. The layer still works; only the Tab position is lost.
    }
  }

  /**
   * Take the note back, once, and only if it was written for THIS address.
   *
   * 🔴 **The marker carried a bare `"1"` until story 6b.11's code review, and its edge layer
   * measured what that costs.** `sessionStorage` outlives the navigation that set it, so a
   * marker written for a settle whose document never committed — the operator presses Stop, a
   * response stalls, a session is restored — sat there until the NEXT load of `/triage` by ANY
   * means and silently pulled focus into the queue: exactly the autofocus the comment above
   * refuses. There was **no way to tell "my own marker" from "a stale one"**. Now the marker
   * IS the address it was written for.
   *
   * ⚠️ **Cleared in EVERY case, matched or not**, so a stale marker cannot survive one load to
   * ambush the next.
   */
  function restore() {
    var wanted;
    try {
      wanted = window.sessionStorage.getItem(RESTORE_KEY);
      window.sessionStorage.removeItem(RESTORE_KEY);
    } catch (error) {
      return;
    }
    if (wanted !== window.location.href) return;
    var current = document.querySelector('.queue .queue-row > a[aria-current="true"]');
    if (current !== null) current.focus();
  }

  document.addEventListener("keydown", function (event) {
    if (event.key !== "ArrowUp" && event.key !== "ArrowDown") return;
    // Modified arrows belong to the browser: ⌘↓ is *end of document*, ⇧↓ extends a
    // selection. Taking them would be taking something the operator already had.
    if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;

    // 🔴 **Inert where there is no queue.** This file is loaded by the shell on ALL TEN
    // screens, and an unconditional `preventDefault` on the arrow KILLS PAGE SCROLLING —
    // measured: `/diagnostic` scrolls 0 → 26 px with this return, 0 → 0 without it.
    var all = rows();
    if (all.length === 0) return;

    // 🔴 **Scoped to the queue.** The listener is on `document`, so without this an
    // operator tabbing the navigation and pressing ↓ NAVIGATES THE PAGE — measured, one
    // Tab from the top lands on the first nav entry. `body` counts as inside: it is where
    // focus sits on a fresh load, and that is the operator's first keypress.
    var focused = document.activeElement;
    var inQueue =
      focused === document.body ||
      focused === null ||
      (focused.closest !== undefined && focused.closest(".queue") !== null);
    if (!inQueue) return;

    // ⚠️ The epic's `INPUT`/`TEXTAREA` exclusion is NOT what guards this. Measured across
    // all ten served pages and the whole template tree: **zero `<input>`, zero
    // `<textarea>`, zero `<select>`, zero `<form>`, zero `contenteditable`** — there is
    // nothing in this product to type into. The scope check above is the exclusion with a
    // live case; this comment is the one without, kept as a statement rather than as code.

    var index = currentIndex(all);
    var next = event.key === "ArrowDown" ? index + 1 : index - 1;
    // At the ends, do nothing AND leave the arrow to the browser: an operator at the last
    // row still expects the page to scroll.
    if (next < 0 || next >= all.length) return;

    event.preventDefault();
    select(all, all[next]);
  });

  // 🔴 **NOTHING CANCELLED THE PENDING NAVIGATION UNTIL STORY 6b.11's ARBITRATION 4, and the
  // defect was measured rather than argued.** With the document answering in 1200 ms — well
  // inside the 2 s per-handler budget story 6b.10 installed on this very screen — the
  // operator pressed ↓, clicked row 6, and the browser landed on row **2**: the old document
  // stays alive until the new response commits, the timer fires meanwhile, and `location`
  // replaced the navigation the operator had actually asked for. The control is what sizes
  // it: against a warm store the click wins, so the defect needs only a store slower than
  // the remaining settle. ⚠️ `keydown` here is deliberately every key EXCEPT the two arrows
  // — an arrow is the layer's own gesture and `select()` re-arms the timer itself.
  document.addEventListener("pointerdown", cancelPending, true);
  document.addEventListener("click", cancelPending, true);
  document.addEventListener(
    "keydown",
    function (event) {
      if (event.key === "ArrowUp" || event.key === "ArrowDown") return;
      cancelPending();
    },
    true,
  );
  window.addEventListener("pagehide", cancelPending);

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", restore);
  } else {
    restore();
  }
})();
