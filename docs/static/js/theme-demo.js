/* rustpress docs: live bundled-theme demo.
 *
 * Every rustpress build ships all bundled themes under /themes/, so
 * trying one is a stylesheet link away: this script swaps that link
 * and the whole site re-skins instantly. The choice lasts for the
 * browser session only (sessionStorage) — stock (no link) is the
 * default. The gallery buttons on /guide/theming are rendered from
 * <base>themes.json, generated at build time from static/themes/.
 * Storage may be blocked (sandboxed iframes, private mode); a failure
 * there just means the choice is not remembered — the demo still works.
 */
(function () {
  var KEY = "rustpress-theme-demo";
  var base = "/";
  try {
    var self = document.currentScript;
    if (self && self.src) base = self.src.replace(/js\/theme-demo\.js.*$/, "");
  } catch (e) { /* keep "/" */ }

  var store = {
    get: function () {
      try { return sessionStorage.getItem(KEY); } catch (e) { return null; }
    },
    set: function (name) {
      try {
        if (name) sessionStorage.setItem(KEY, name);
        else sessionStorage.removeItem(KEY);
      } catch (e) { /* not remembered */ }
    },
  };

  var link = null;

  function apply(name) {
    var old = link;
    if (!name) {
      if (old) old.remove();
      link = null;
      return;
    }
    link = document.createElement("link");
    link.rel = "stylesheet";
    link.setAttribute("data-theme-demo", "");
    link.href = base + "themes/" + name + ".css";
    if (old) {
      // keep the previous sheet until the replacement has loaded, so
      // switching themes does not flash the stock look
      link.onload = function () { old.remove(); };
      link.onerror = function () { link.remove(); link = old; };
    }
    document.head.appendChild(link);
  }

  apply(store.get());

  document.addEventListener("DOMContentLoaded", function () {
    var host = document.getElementById("theme-gallery");
    if (!host) return;

    var chosen = store.get() || "";
    var buttons = [];

    function pick(name) {
      chosen = name;
      store.set(name || null);
      apply(name || null);
      buttons.forEach(function (b) {
        b.setAttribute("aria-pressed", b.dataset.theme === chosen ? "true" : "false");
      });
    }

    function button(name, label) {
      var el = document.createElement("button");
      el.type = "button";
      el.textContent = label;
      el.dataset.theme = name;
      el.setAttribute("aria-pressed", name === chosen ? "true" : "false");
      el.onclick = function () { pick(name); };
      Object.assign(el.style, {
        margin: "0 0 5px 0",
        marginRight: "8px",
        padding: "5px 14px",
        borderRadius: "9999px",
        border: "1px solid var(--vp-c-border)",
        background: "var(--vp-c-bg)",
        color: "var(--vp-c-text-1)",
        font: "inherit",
        fontSize: "0.875rem",
        fontWeight: "500",
        cursor: "pointer",
      });
      buttons.push(el);
      return el;
    }

    host.setAttribute("role", "group");
    host.setAttribute("aria-label", "Live bundled-theme demo");
    host.appendChild(button("", "stock"));

    fetch(base + "themes.json")
      .then(function (r) { return r.json(); })
      .then(function (names) {
        names.forEach(function (name) {
          host.appendChild(button(name, name));
        });
      })
      .catch(function () {
        var note = document.createElement("p");
        note.textContent = "(theme index unavailable — run npm run build:docs)";
        host.appendChild(note);
      });

    var hint = document.createElement("p");
    hint.textContent = "Pick a theme and every page of this site re-skins for the rest of the session — links, containers, code blocks, dark mode, everything. “stock” is the default VitePress look.";
    Object.assign(hint.style, {
      margin: "12px 0 0",
      fontSize: "0.875rem",
      color: "var(--vp-c-text-2)",
    });
    host.appendChild(hint);
  });
})();
