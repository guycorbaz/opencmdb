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
// ⚠️ **(E)'s stated cost: for 250 ms the highlighted row and the URL disagree.** That is
// written here so nobody rediscovers it as a bug.
(function () {
  "use strict";

  // 🔑 Idempotence. A double-registered listener moves two rows per press — harmless on a
  // full document load, and a real defect the day story 6.4 adds a swap that re-runs this
  // file. Two lines now, against a bug that would be measured in a browser later.
  if (window.__opencmdbKeyboard) return;
  window.__opencmdbKeyboard = true;

  // How long the operator must stop before the URL catches up with the highlight.
  var SETTLE_MS = 250;

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

  /** Move the highlight to `row` now; let the URL follow once the operator stops. */
  function select(all, row) {
    for (var i = 0; i < all.length; i++) {
      all[i].parentNode.classList.toggle("selected", all[i] === row);
    }
    row.focus();
    if (pending !== null) window.clearTimeout(pending);
    // 🔑 The row's OWN href, never a URL rebuilt from the selector: the hrefs already carry
    // `?sort=age`, so the sort survives arrow navigation for free — and stops surviving the
    // moment someone constructs the address instead of reading it.
    pending = window.setTimeout(function () {
      window.location.assign(row.getAttribute("href"));
    }, SETTLE_MS);
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
})();
