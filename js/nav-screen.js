/* Nav screen (mobile menu) toggled by the navbar hamburger. */

var d = document;
var setScreen = function () {};

export function init() {
  var navbar = d.getElementById("VPNavBar");
  var hamburger = d.getElementById("VPNavBarHamburger");
  var navScreen = d.getElementById("VPNavScreen");
  var backdrop = d.getElementById("VPBackdrop");

  setScreen = function (open) {
    if (!hamburger || !navScreen) return;
    hamburger.classList.toggle("active", open);
    hamburger.setAttribute("aria-expanded", open ? "true" : "false");
    navScreen.hidden = !open;
    if (navbar) navbar.classList.toggle("screen-open", open);
    if (backdrop) backdrop.hidden = !open;
    d.body.style.overflow = open ? "hidden" : "";
  };

  if (hamburger) {
    hamburger.addEventListener("click", function () {
      setScreen(navScreen.hidden);
    });
  }
}

export function close() {
  setScreen(false);
}
