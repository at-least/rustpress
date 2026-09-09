/* Navbar flyout menus: button toggles .open, outside click closes. */

var d = document;

export function init() {
  d.querySelectorAll(".VPFlyout > .button").forEach(function (button) {
    button.addEventListener("click", function () {
      button.parentElement.classList.toggle("open");
    });
  });

  d.addEventListener("click", function (e) {
    d.querySelectorAll(".VPFlyout.open").forEach(function (flyout) {
      if (!flyout.contains(e.target)) flyout.classList.remove("open");
    });
  });
}
