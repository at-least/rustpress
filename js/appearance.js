/* Dark/light toggle: persists to localStorage and flips the .dark class
   on <html>; the switch reflects the current state via aria-checked. */

var d = document;

export function init() {
  var root = d.documentElement;
  var switches = d.querySelectorAll(".VPSwitchAppearance");

  function isDark() {
    return root.classList.contains("dark");
  }

  function applyAppearance() {
    var dark = isDark();
    switches.forEach(function (s) {
      s.setAttribute("aria-checked", dark ? "true" : "false");
    });
  }

  function toggleAppearance() {
    try {
      localStorage.setItem("vitepress-theme-appearance", isDark() ? "light" : "dark");
    } catch (e) {}
    root.classList.toggle("dark", !isDark());
    applyAppearance();
  }

  switches.forEach(function (s) {
    s.addEventListener("click", toggleAppearance);
  });
  applyAppearance();
}
