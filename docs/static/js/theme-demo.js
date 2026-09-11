/* rustpress docs: Theme Gallery.
 *
 * Every rustpress build ships all bundled themes under /themes/, so
 * trying one is a stylesheet link away: picking a card swaps that link
 * and the whole site re-skins instantly. The choice lasts for the
 * browser session only (sessionStorage) — stock (no link) is the
 * default. Cards are rendered from <base>themes.json, generated at
 * build time from static/themes/. Storage may be blocked (sandboxed
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

    function swatch(color, bordered) {
      var dot = document.createElement("span");
      dot.title = color;
      Object.assign(dot.style, {
        display: "inline-block",
        width: "14px",
        height: "14px",
        borderRadius: "9999px",
        margin: "-2px 6px 0 0",
        verticalAlign: "middle",
        backgroundColor: color,
        border: bordered ? "1px solid var(--vp-c-border)" : "1px solid transparent",
      });
      return dot;
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
        padding: "14px 16px",
        textAlign: "left",
        borderRadius: "10px",
        border: "1px solid var(--vp-c-border)",
        background: "var(--vp-c-bg)",
        color: "var(--vp-c-text-1)",
        font: "inherit",
        cursor: "pointer",
      });
      var name = document.createElement("strong");
      name.textContent = entry.name || "stock";
      Object.assign(name.style, { display: "block", fontSize: "0.95rem" });
      el.appendChild(name);
      if (entry.desc) {
        var desc = document.createElement("span");
        desc.textContent = entry.desc;
        Object.assign(desc.style, {
          display: "block",
          margin: "4px 0 8px",
          fontSize: "0.875rem",
          color: "var(--vp-c-text-2)",
        });
        el.appendChild(desc);
      }
      var row = document.createElement("span");
      row.style.display = "block";
      ["bg", "text", "brand"].forEach(function (k) {
        if (entry[k]) row.appendChild(swatch(entry[k], k === "bg"));
      });
      el.appendChild(row);
      cards.push(el);
      return el;
    }

    host.setAttribute("role", "group");
    host.setAttribute("aria-label", "Live bundled-theme demo");
    host.appendChild(card({ name: "", desc: "the default VitePress look" }));

    var wrap = document.createElement("div");
    Object.assign(wrap.style, {
      display: "grid",
      gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))",
      gap: "12px",
      margin: "12px 0",
    });

    fetch(base + "themes.json")
      .then(function (r) { return r.json(); })
      .then(function (entries) {
        entries.forEach(function (entry) {
          var cell = document.createElement("div");
          cell.appendChild(card(entry));
          wrap.appendChild(cell);
        });
        host.appendChild(wrap);
      })
      .catch(function () {
        var note = document.createElement("p");
        note.textContent = "(theme index unavailable — run npm run build:docs)";
        host.appendChild(note);
      });
  });
})();
