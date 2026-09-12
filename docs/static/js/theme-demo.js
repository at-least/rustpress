/* rustpress docs: Theme (/themes/).
 *
 * Every rustpress build ships all bundled themes under /themes/, so
 * trying one is a stylesheet link away: picking a card swaps that link
 * and the whole site re-colors instantly. The choice lasts for the
 * browser session only (sessionStorage) — stock (no link) is the
 * default.
 *
 * Cards are rendered from <base>themes.json, generated at build time
 * from static/themes/: each carries the theme's own tokens, and the
 * card paints a miniature page mock with them (light and dark side by
 * side — surfaces, text, link, semantics), so the card is the design,
 * not just a palette chip. The stock card paints with the page's live
 * var(--vp-*) tokens instead. Storage may be blocked (sandboxed
 * iframes, private mode); a failure there just means the choice is
 * not remembered — the demo still works.
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
    var cards = [];

    function pick(name) {
      chosen = name;
      store.set(name || null);
      apply(name || null);
      cards.forEach(function (c) {
        c.setAttribute("aria-pressed", c.dataset.theme === chosen ? "true" : "false");
      });
    }

    /* The miniature page: a navbar strip on the elevated surface, a
     * heading, a body line, a link, and the three semantic dots —
     * every color straight from the theme's tokens. */
    function mockHalf(t, label) {
      var half = document.createElement("div");
      half.title = label;
      Object.assign(half.style, {
        padding: "8px",
        backgroundColor: t.bg,
        minWidth: "0",
      });
      var bar = function (color, width, height, mt) {
        var el = document.createElement("div");
        Object.assign(el.style, {
          height: height + "px",
          width: width + "%",
          marginTop: mt + "px",
          borderRadius: "2px",
          backgroundColor: color,
        });
        return el;
      };
      var nav = document.createElement("div");
      Object.assign(nav.style, {
        height: "8px",
        borderRadius: "3px",
        backgroundColor: t.bgAlt,
        border: "1px solid " + t.border,
        display: "flex",
        alignItems: "center",
        padding: "0 3px",
        gap: "3px",
      });
      var logo = document.createElement("span");
      Object.assign(logo.style, {
        width: "5px",
        height: "5px",
        borderRadius: "1px",
        backgroundColor: t.brand,
      });
      nav.appendChild(logo);
      half.appendChild(nav);
      half.appendChild(bar(t.text, 68, 5, 8));
      half.appendChild(bar(t.textMuted, 86, 4, 4));
      half.appendChild(bar(t.brand, 48, 4, 4));
      var dots = document.createElement("div");
      Object.assign(dots.style, { display: "flex", gap: "3px", marginTop: "6px" });
      [t.success, t.warning, t.danger].forEach(function (color) {
        var dot = document.createElement("span");
        Object.assign(dot.style, {
          width: "5px",
          height: "5px",
          borderRadius: "9999px",
          backgroundColor: color,
        });
        dots.appendChild(dot);
      });
      half.appendChild(dots);
      return half;
    }

    function mock(entry) {
      if (!entry.light) {
        // stock: paint with the page's live tokens so the card always
        // shows this site as it is right now, both halves per mode
        var vars = {
          bg: "var(--vp-c-bg)", bgAlt: "var(--vp-c-bg-alt)",
          text: "var(--vp-c-text-1)", textMuted: "var(--vp-c-text-2)",
          border: "var(--vp-c-border)", brand: "var(--vp-c-brand-1)",
          success: "var(--vp-c-success-1)", warning: "var(--vp-c-warning-1)",
          danger: "var(--vp-c-danger-1)",
        };
        return mockHalf(vars, "this site now");
      }
      var wrap = document.createElement("div");
      Object.assign(wrap.style, { display: "grid", gridTemplateColumns: "1fr 1fr", minWidth: "0" });
      var light = mockHalf(entry.light, "light");
      var dark = mockHalf(entry.dark, "dark");
      dark.style.borderLeft = "1px solid " + entry.light.border;
      wrap.appendChild(light);
      wrap.appendChild(dark);
      return wrap;
    }

    function card(entry) {
      var el = document.createElement("button");
      el.type = "button";
      el.dataset.theme = entry.name;
      el.setAttribute("aria-pressed", entry.name === chosen ? "true" : "false");
      el.onclick = function () { pick(entry.name); };
      Object.assign(el.style, {
        display: "block",
        width: "100%",
        padding: "0",
        textAlign: "left",
        borderRadius: "10px",
        border: "1px solid var(--vp-c-border)",
        background: "var(--vp-c-bg)",
        color: "var(--vp-c-text-1)",
        font: "inherit",
        cursor: "pointer",
        overflow: "hidden",
      });
      var preview = mock(entry);
      preview.style.borderRadius = "10px 10px 0 0";
      el.appendChild(preview);
      var body = document.createElement("div");
      Object.assign(body.style, { padding: "10px 14px 12px" });
      var name = document.createElement("strong");
      name.textContent = entry.name || "stock";
      Object.assign(name.style, { display: "block", fontSize: "0.95rem" });
      body.appendChild(name);
      if (entry.desc) {
        var desc = document.createElement("span");
        desc.textContent = entry.desc;
        Object.assign(desc.style, {
          display: "block",
          margin: "3px 0 0",
          fontSize: "0.875rem",
          color: "var(--vp-c-text-2)",
        });
        body.appendChild(desc);
      }
      el.appendChild(body);
      cards.push(el);
      return el;
    }

    host.setAttribute("role", "group");
    host.setAttribute("aria-label", "Live bundled-theme demo");

    var wrap = document.createElement("div");
    Object.assign(wrap.style, {
      display: "grid",
      gridTemplateColumns: "repeat(auto-fill, minmax(230px, 1fr))",
      gap: "12px",
      margin: "12px 0",
    });
    host.appendChild(wrap);

    var stockCell = document.createElement("div");
    stockCell.appendChild(card({ name: "", desc: "the stock look" }));
    wrap.appendChild(stockCell);

    fetch(base + "themes.json")
      .then(function (r) { return r.json(); })
      .then(function (entries) {
        entries.forEach(function (entry) {
          var cell = document.createElement("div");
          cell.appendChild(card(entry));
          wrap.appendChild(cell);
        });
      })
      .catch(function () {
        var note = document.createElement("p");
        note.textContent = "(theme index unavailable — run npm run build:docs)";
        host.appendChild(note);
      });
  });
})();
