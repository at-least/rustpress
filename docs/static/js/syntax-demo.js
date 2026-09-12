/* rustpress docs: Syntax Highlight Gallery (/syntax-highlight/).
 *
 * Card mocks are painted inline from syntax-highlight.json (generated at
 * build time by `rustpress syntax-index` from the same Rust resolution
 * the highlighting itself uses): each theme carries its own block
 * background (`ui.background`) and every capture it styles, so the
 * mock is the theme — dark schemes sit on their own dark ground and
 * need no light/dark halves.
 *
 * Picking a card also re-colors every code block on this page live:
 * the entry is turned into one <style> with the same `.tk-*` rules
 * syntax.css uses, appended after it so the later rules win inside
 * `@layer syntax`, plus a `--vp-code-block-bg`/`-color` override for
 * the block's own ground. The choice lasts for the browser session
 * (sessionStorage); a storage failure just means it is not remembered.
 */
(function () {
  var KEY = "rustpress-syntax-demo";
  var base = "/";
  try {
    var self = document.currentScript;
    if (self && self.src) base = self.src.replace(/js\/syntax-demo\.js.*$/, "");
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

  /* A five-line Rust snippet whose spans carry the same `.tk-*` classes
   * syntax.css does; the card paints each span inline with the theme's
   * own resolution of that capture. */
  var MOCK_LINES = [
    [["tk-comment", "// one scheme per card, live from the theme file"]],
    [["tk-keyword-function", "fn "], ["tk-function", "render"], ["tk-punctuation-bracket", "("], ["tk-variable-parameter", "site"], ["tk-punctuation-delimiter", ": "], ["tk-operator", "&"], ["tk-type", "Site"], ["tk-punctuation-bracket", ")"], ["tk-operator", " -> "], ["tk-type", "Page"], ["tk-punctuation-bracket", " {"]],
    [["tk-keyword", "    let "], ["tk-variable", "pages"], ["tk-operator", " = "], ["tk-type", "IndexMap"], ["tk-punctuation-delimiter", "::"], ["tk-function-builtin", "new"], ["tk-punctuation-bracket", "()"], ["tk-punctuation-delimiter", ";"]],
    [["tk-keyword-control-repeat", "    for "], ["tk-variable", "path"], ["tk-keyword", " in "], ["tk-variable", "site"], ["tk-punctuation-delimiter", "."], ["tk-function-method", "paths"], ["tk-punctuation-bracket", "()"], ["tk-punctuation-bracket", " {"]],
    [["tk-keyword-control-return", "        return "], ["tk-constant-builtin", "Ok"], ["tk-punctuation-bracket", "("], ["tk-variable", "pages"], ["tk-punctuation-bracket", ")"], ["tk-punctuation-delimiter", ";"]],
    [["tk-punctuation-bracket", "    }"]],
    [["tk-punctuation-bracket", "}"]],
  ];

  function declarations(style) {
    var parts = [];
    if (style.fg) parts.push("color:" + style.fg);
    if (style.bg) parts.push("background-color:" + style.bg);
    if (style.bold) parts.push("font-weight:600");
    if (style.italic) parts.push("font-style:italic");
    if (style.underline) parts.push("text-decoration:underline");
    return parts.join(";");
  }

  function apply(entry) {
    var el = document.getElementById("syntax-demo-style");
    if (el) el.remove();
    if (!entry) return;
    /* The block's token rules go into `@layer syntax` at a specificity
     * that outranks both of syntax.css's halves — its light `.tk-x`
     * rules and its `html.dark .tk-x` dark rules (tied by specificity,
     * beaten by order) — in either page mode.
     * The block-background/color variables must stay OUTSIDE the layer:
     * vitepress.css defines them unlayered, and unlayered beats layered
     * regardless of order. */
    var rules = [];
    Object.keys(entry.styles).forEach(function (cls) {
      var decl = declarations(entry.styles[cls]);
      // keys already carry the tk- prefix (they are the CSS class names)
      if (decl) rules.push("html .vp-doc ." + cls + ",html.dark .vp-doc ." + cls + "{" + decl + "}");
    });
    var layer = "@layer syntax{" + rules.join("") + "}";
    var vars = "";
    if (entry.bg) vars += "--vp-code-block-bg:" + entry.bg + ";";
    if (entry.fg) vars += "--vp-code-block-color:" + entry.fg + ";";
    if (vars) layer += ".vp-doc{" + vars + "}";
    el = document.createElement("style");
    el.id = "syntax-demo-style";
    el.textContent = layer;
    document.head.appendChild(el);
  }

  function mock(entry) {
    var pre = document.createElement("pre");
    Object.assign(pre.style, {
      margin: "0",
      padding: "10px 12px",
      fontFamily: "var(--vp-font-family-mono, ui-monospace, monospace)",
      fontSize: "10.5px",
      lineHeight: "1.6",
      whiteSpace: "pre",
      overflow: "hidden",
      backgroundColor: entry.bg || "var(--vp-c-bg-alt)",
      color: entry.fg || "var(--vp-c-text-1)",
      minHeight: "128px",
      borderRadius: "10px 10px 0 0",
    });
    MOCK_LINES.forEach(function (line, i) {
      if (i) pre.appendChild(document.createTextNode("\n"));
      line.forEach(function (seg) {
        var span = document.createElement("span");
        span.textContent = seg[1];
        /* every slot is set explicitly (falling back to the theme's own
         * base): the page's syntax.css and the injected live rules would
         * otherwise reach into the mock and recolor spans the theme
         * itself leaves unstyled */
        var style = entry.styles[seg[0]];
        span.style.color = (style && style.fg) || entry.fg || "inherit";
        span.style.backgroundColor = (style && style.bg) || "transparent";
        span.style.fontStyle = style && style.italic ? "italic" : "normal";
        span.style.fontWeight = style && style.bold ? "600" : "normal";
        span.style.textDecoration = style && style.underline ? "underline" : "none";
        pre.appendChild(span);
      });
    });
    return pre;
  }

  function card(entry, chosen, onPick) {
    var el = document.createElement("button");
    el.type = "button";
    el.dataset.theme = entry.name;
    el.setAttribute("aria-pressed", entry.name === chosen ? "true" : "false");
    el.onclick = function () { onPick(entry); };
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
    el.appendChild(mock(entry));
    var body = document.createElement("div");
    Object.assign(body.style, { padding: "8px 14px 10px" });
    var name = document.createElement("strong");
    name.textContent = entry.name;
    Object.assign(name.style, { display: "block", fontSize: "0.9rem" });
    body.appendChild(name);
    el.appendChild(body);
    return el;
  }

  document.addEventListener("DOMContentLoaded", function () {
    var host = document.getElementById("syntax-gallery");
    if (!host) return;

    var cards = [];
    var chosen = store.get() || "";
    var entries = [];

    function pick(entry) {
      chosen = entry ? entry.name : "";
      store.set(entry ? entry.name : null);
      apply(entry || null);
      cards.forEach(function (c) {
        c.setAttribute("aria-pressed", c.dataset.theme === chosen ? "true" : "false");
      });
    }

    host.setAttribute("role", "group");
    host.setAttribute("aria-label", "Live bundled syntax-theme demo");

    var filter = document.createElement("input");
    filter.type = "search";
    filter.placeholder = "Filter themes…";
    filter.setAttribute("aria-label", "Filter syntax highlight themes by name");
    Object.assign(filter.style, {
      display: "block",
      width: "16rem",
      maxWidth: "100%",
      margin: "12px 0",
      padding: "6px 10px",
      borderRadius: "8px",
      border: "1px solid var(--vp-c-border)",
      background: "var(--vp-c-bg)",
      color: "var(--vp-c-text-1)",
      font: "inherit",
    });
    filter.oninput = function () {
      var q = filter.value.toLowerCase();
      cards.forEach(function (c) {
        c.parentNode.style.display = c.dataset.theme.indexOf(q) !== -1 ? "" : "none";
      });
    };
    host.appendChild(filter);

    var grid = document.createElement("div");
    Object.assign(grid.style, {
      display: "grid",
      gridTemplateColumns: "repeat(auto-fill, minmax(250px, 1fr))",
      gap: "12px",
      margin: "12px 0",
    });
    host.appendChild(grid);

    var stockCell = document.createElement("div");
    var stockCard = card(
      { name: "stock", bg: "", fg: "", styles: {} },
      chosen,
      function () { pick(null); }
    );
    stockCard.querySelector("strong").textContent = "stock";
    stockCell.appendChild(stockCard);
    grid.appendChild(stockCell);
    cards.push(stockCard);

    fetch(base + "syntax-highlight.json")
      .then(function (r) { return r.json(); })
      .then(function (list) {
        entries = list;
        list.forEach(function (entry) {
          var cell = document.createElement("div");
          cell.appendChild(card(entry, chosen, pick));
          grid.appendChild(cell);
          cards.push(cell.firstChild);
        });
        if (chosen) {
          var picked = entries.find(function (e) { return e.name === chosen; });
          if (picked) apply(picked);
          else { chosen = ""; store.set(null); }
        }
      })
      .catch(function () {
        var note = document.createElement("p");
        note.textContent = "(syntax highlight index unavailable — run npm run build:docs)";
        host.appendChild(note);
      });
  });
})();
