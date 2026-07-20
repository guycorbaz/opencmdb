// opencmdb — minimal client behavior.
//
// Keyboard-first accessibility (UX-DR): when HTMX swaps the gap card, move focus
// to the freshly-swapped card so keyboard users are not stranded at the top of
// the document. The card carries tabindex="-1" so it is programmatically focusable
// without joining the tab order.
document.body.addEventListener("htmx:afterSwap", function (event) {
  var card = document.getElementById("gap-card");
  if (card) {
    card.focus();
  }
});
