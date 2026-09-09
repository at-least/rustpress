/* Nav screen accordions: the translations expander and the menu groups. */

var d = document;

export function init() {
  var langButton = d.querySelector("#VPNavScreenTranslations > .title");
  if (langButton) {
    langButton.addEventListener("click", function () {
      var wrap = langButton.parentElement;
      var open = wrap.classList.toggle("open");
      langButton.setAttribute("aria-expanded", open ? "true" : "false");
      var list = wrap.querySelector(".list");
      if (list) list.hidden = !open;
    });
  }

  d.querySelectorAll(".VPNavScreenMenuGroup > .button").forEach(function (button) {
    button.addEventListener("click", function () {
      var group = button.parentElement;
      var open = group.classList.toggle("open");
      button.setAttribute("aria-expanded", open ? "true" : "false");
      var items = group.querySelector(".items");
      if (items) items.hidden = !open;
    });
  });
}
