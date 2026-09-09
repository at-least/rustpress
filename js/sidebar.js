/* Sidebar: mobile drawer (via the local-nav button + backdrop) and
   collapsible groups (carets). */

var d = document;
var setSidebar = function () {};

export function init() {
  var sidebar = d.getElementById("VPSidebar");
  var localNavMenu = d.getElementById("VPLocalNavMenu");
  var backdrop = d.getElementById("VPBackdrop");

  setSidebar = function (open) {
    if (!sidebar) return;
    sidebar.classList.toggle("open", open);
    if (localNavMenu) localNavMenu.setAttribute("aria-expanded", open ? "true" : "false");
    if (backdrop) backdrop.hidden = !open;
    d.body.style.overflow = open ? "hidden" : "";
  };

  if (localNavMenu) {
    localNavMenu.addEventListener("click", function () {
      setSidebar(!sidebar.classList.contains("open"));
    });
  }

  d.querySelectorAll(".VPSidebarItem .caret").forEach(function (caret) {
    caret.addEventListener("click", function (e) {
      e.stopPropagation();
      var item = caret.closest(".VPSidebarItem");
      var collapsed = item.classList.toggle("collapsed");
      caret.setAttribute("aria-expanded", collapsed ? "false" : "true");
    });
  });
}

export function close() {
  setSidebar(false);
}
