/* Local nav outline dropdown (mobile "On this page" menu). */

var d = document;

export function init() {
  var outlineButton = d.getElementById("VPOutlineDropdownButton");
  var outlineItems = d.getElementById("VPOutlineDropdownItems");

  if (outlineButton && outlineItems) {
    outlineButton.addEventListener("click", function () {
      var open = outlineButton.getAttribute("aria-expanded") === "true";
      outlineButton.setAttribute("aria-expanded", open ? "false" : "true");
      outlineButton.classList.toggle("open", !open);
      outlineItems.hidden = open;
    });
    outlineItems.addEventListener("click", function (e) {
      if (e.target.closest("a")) {
        outlineItems.hidden = true;
        outlineButton.setAttribute("aria-expanded", "false");
        outlineButton.classList.remove("open");
      }
    });
  }
}
