/* vitezola — vanilla JS port of the VitePress default theme interactions. */
(function () {
  "use strict";

  var d = document;
  var root = d.documentElement;

  /* ── Appearance ─────────────────────────────────────────────────────── */

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

  /* ── Navbar top state (transparent home navbar) ─────────────────────── */

  var navbar = d.getElementById("VPNavBar");

  /* ── Nav screen (mobile menu) ───────────────────────────────────────── */

  var hamburger = d.getElementById("VPNavBarHamburger");
  var navScreen = d.getElementById("VPNavScreen");
  var backdrop = d.getElementById("VPBackdrop");

  function setScreen(open) {
    if (!hamburger || !navScreen) return;
    hamburger.classList.toggle("active", open);
    hamburger.setAttribute("aria-expanded", open ? "true" : "false");
    navScreen.hidden = !open;
    if (navbar) navbar.classList.toggle("screen-open", open);
    if (backdrop) backdrop.hidden = !open;
    d.body.style.overflow = open ? "hidden" : "";
  }

  if (hamburger) {
    hamburger.addEventListener("click", function () {
      setScreen(navScreen.hidden);
    });
  }

  /* ── Sidebar (mobile drawer + collapsible groups) ───────────────────── */

  var sidebar = d.getElementById("VPSidebar");
  var localNavMenu = d.getElementById("VPLocalNavMenu");

  function setSidebar(open) {
    if (!sidebar) return;
    sidebar.classList.toggle("open", open);
    if (localNavMenu) localNavMenu.setAttribute("aria-expanded", open ? "true" : "false");
    if (backdrop) backdrop.hidden = !open;
    d.body.style.overflow = open ? "hidden" : "";
  }

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

  if (backdrop) {
    backdrop.addEventListener("click", function () {
      setScreen(false);
      setSidebar(false);
    });
  }

  /* ── Navbar flyout menus ────────────────────────────────────────────── */

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

  /* ── Nav screen translations accordion ──────────────────────────────── */

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

  /* ── Nav screen menu groups ─────────────────────────────────────────── */

  d.querySelectorAll(".VPNavScreenMenuGroup > .button").forEach(function (button) {
    button.addEventListener("click", function () {
      var group = button.parentElement;
      var open = group.classList.toggle("open");
      button.setAttribute("aria-expanded", open ? "true" : "false");
      var items = group.querySelector(".items");
      if (items) items.hidden = !open;
    });
  });

  /* ── Code groups ────────────────────────────────────────────────────── */
  /* Builds VitePress-style radio tabs from the `name=` fence annotation. */

  d.querySelectorAll(".vp-code-group").forEach(function (group, gi) {
    var blocks = group.querySelectorAll(".blocks > pre");
    if (blocks.length < 2) return;
    var allNamed = Array.prototype.every.call(blocks, function (pre) {
      return pre.querySelector("code[data-name]");
    });
    if (!allNamed) return;
    var tabs = d.createElement("div");
    tabs.className = "tabs";
    var groupName = "vp-tab-" + gi;
    blocks.forEach(function (pre, i) {
      var code = pre.querySelector("code[data-name]");
      var name = code ? code.getAttribute("data-name") : "";
      if (!name) return;
      var input = d.createElement("input");
      input.type = "radio";
      input.name = groupName;
      input.id = groupName + "-" + i;
      if (i === 0) input.checked = true;
      var label = d.createElement("label");
      label.htmlFor = input.id;
      label.textContent = name;
      input.addEventListener("change", function () {
        blocks.forEach(function (b) {
          b.classList.remove("active");
        });
        pre.classList.add("active");
      });
      tabs.appendChild(input);
      tabs.appendChild(label);
    });
    group.prepend(tabs);
    blocks[0].classList.add("active");
  });

  /* ── Code copy buttons + lang labels ────────────────────────────────── */
  /* Adds the language-* class and span.lang so the ported VitePress
     .vp-doc [class*='language-'] styles apply to Zola's bare <pre.giallo>.
     Labels show the VitePress-style short name; giallo normalizes ids
     (shellscript/typescript/…), so map the common ones back. */

  var LANG_ALIASES = {
    shellscript: "sh",
    typescript: "ts",
    javascript: "js",
    markdown: "md",
    plain: "text"
  };

  d.querySelectorAll("pre.giallo").forEach(function (pre) {
    var code = pre.querySelector("code");
    var rawLang = code ? code.getAttribute("data-lang") : null;
    if (code && rawLang && rawLang !== "plain") {
      pre.classList.add("language-" + rawLang);
      var lang = d.createElement("span");
      lang.className = "lang";
      lang.textContent = LANG_ALIASES[rawLang] || rawLang;
      pre.appendChild(lang);
    }
    var button = d.createElement("button");
    button.type = "button";
    button.className = "vp-copy-button copy";
    button.setAttribute("aria-label", "Copy code");
    button.addEventListener("click", function () {
      if (!code) return;
      if (!navigator.clipboard) return;
      navigator.clipboard.writeText(code.textContent).then(function () {
        button.classList.add("copied");
        setTimeout(function () {
          button.classList.remove("copied");
        }, 1200);
      });
    });
    pre.appendChild(button);
  });

  /* ── Local nav outline dropdown ─────────────────────────────────────── */

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

  /* ── Scrollspy: outline active anchor + marker + navbar top state ───── */

  var outlineContainer = d.querySelector(".VPDocAsideOutline");
  var outlineMarker = d.getElementById("VPOutlineMarker");
  var outlineLinks = outlineContainer
    ? Array.prototype.slice.call(outlineContainer.querySelectorAll("a.outline-link"))
    : [];

  function scrollTick() {
    if (navbar) {
      navbar.classList.toggle("top", window.scrollY <= 0);
    }
    if (!outlineLinks.length) return;
    var activeLink = null;
    outlineLinks.forEach(function (link) {
      var id = decodeURIComponent(link.hash.slice(1));
      var heading = d.getElementById(id);
      if (!heading) return;
      link.classList.remove("active");
      if (heading.getBoundingClientRect().top < 96) activeLink = link;
    });
    if (activeLink) {
      activeLink.classList.add("active");
      if (outlineMarker) {
        outlineMarker.style.opacity = "1";
        outlineMarker.style.top =
          activeLink.offsetTop + 19 - 9 + "px";
      }
    } else if (outlineMarker) {
      outlineMarker.style.opacity = "0";
    }
  }

  var ticking = false;
  window.addEventListener(
    "scroll",
    function () {
      if (!ticking) {
        ticking = true;
        requestAnimationFrame(function () {
          scrollTick();
          ticking = false;
        });
      }
    },
    { passive: true }
  );
  scrollTick();

  /* ── Local search ───────────────────────────────────────────────────── */

  var searchBox = d.getElementById("VPLocalSearchBox");
  var searchButton = d.getElementById("VPSearchButton");
  var searchInput = d.getElementById("localsearch-input");
  var searchResults = d.getElementById("VPSearchResults");
  var searchClear = d.getElementById("VPSearchClear");
  var indexPromise = null;
  var selectedIndex = -1;
  var currentResults = [];

  function searchIndexUrl() {
    if (window.vitezola && window.vitezola.searchIndexUrl) {
      return window.vitezola.searchIndexUrl;
    }
    var lang = d.documentElement.lang || "en";
    return "search_index." + lang + ".json";
  }

  function loadIndex() {
    if (!indexPromise) {
      indexPromise = fetch(searchIndexUrl())
        .then(function (r) {
          if (!r.ok) throw new Error("search index " + r.status);
          return r.json();
        })
        .catch(function (err) {
          indexPromise = null;
          throw err;
        });
    }
    return indexPromise;
  }

  function esc(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  function mark(text, tokens) {
    if (!tokens.length) return esc(text);
    var pattern = new RegExp(
      "(" + tokens.map(function (t) { return t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"); }).join("|") + ")",
      "gi"
    );
    // split on the raw text first, then escape each piece, so tokens can
    // never match inside HTML entities produced by esc()
    var parts = text.split(pattern);
    return parts
      .map(function (part) {
        var isToken = tokens.some(function (t) {
          return part.toLowerCase() === t.toLowerCase();
        });
        return isToken ? "<mark>" + esc(part) + "</mark>" : esc(part);
      })
      .join("");
  }

  function makeExcerpt(content, tokens) {
    var lower = content.toLowerCase();
    var pos = -1;
    for (var i = 0; i < tokens.length && pos < 0; i++) {
      pos = lower.indexOf(tokens[i]);
    }
    var start = Math.max(0, pos - 60);
    var end = Math.min(content.length, (pos < 0 ? 0 : pos) + 240);
    return (start > 0 ? "…" : "") + content.slice(start, end) + (end < content.length ? "…" : "");
  }

  function runSearch(q) {
    if (!searchIndexUrl()) return Promise.resolve([]);
    return loadIndex().then(function (index) {
      var tokens = q.toLowerCase().split(/\s+/).filter(Boolean);
      if (!tokens.length) return [];
      return index
        .map(function (entry) {
          var title = (entry.title || "").toLowerCase();
          var content = (entry.body || entry.content || entry.contents || "").toLowerCase();
          var score = 0;
          tokens.forEach(function (t) {
            if (title === t) score += 20;
            while (title.indexOf(t) !== -1) {
              score += 5;
              title = title.replace(t, " ");
            }
            var count = 0;
            var idx = content.indexOf(t);
            while (idx !== -1 && count < 50) {
              count++;
              idx = content.indexOf(t, idx + t.length);
            }
            score += Math.min(count, 20);
          });
          return { entry: entry, score: score };
        })
        .filter(function (r) {
          return r.score > 0;
        })
        .sort(function (a, b) {
          return b.score - a.score;
        })
        .slice(0, 20);
    });
  }

  function renderResults(results, q) {
    currentResults = results;
    selectedIndex = -1;
    searchResults.innerHTML = "";
    if (!results.length) {
      searchResults.innerHTML =
        '<li class="no-results">' +
        (q ? 'No results for "<b>' + esc(q) + '</b>"' : "") +
        "</li>";
      return;
    }
    var tokens = q.toLowerCase().split(/\s+/).filter(Boolean);
    results.forEach(function (r) {
      var li = d.createElement("li");
      li.className = "result";
      li.setAttribute("role", "option");
      var div = d.createElement("div");
      var path = (r.entry.url || "").replace(/^https?:\/\/[^/]+/, "").split("/").filter(Boolean);
      var titles = '<div class="titles">';
      path.slice(0, -1).forEach(function (seg) {
        titles += '<p class="title">' + esc(seg) + "</p>";
      });
      titles += '<p class="title main">' + mark(r.entry.title || r.entry.url, tokens) + "</p></div>";
      var excerpt = '<p class="excerpt">' + mark(makeExcerpt(r.entry.body || r.entry.content || "", tokens), tokens) + "</p>";
      div.innerHTML = titles + excerpt;
      li.appendChild(div);
      li.addEventListener("click", function () {
        window.location.href = r.entry.url;
      });
      searchResults.appendChild(li);
    });
  }

  function openSearch() {
    if (!(window.vitezola && window.vitezola.searchIndexUrl)) return;
    searchBox.hidden = false;
    d.body.style.overflow = "hidden";
    searchInput.focus();
    searchInput.select();
  }

  function closeSearch() {
    searchBox.hidden = true;
    d.body.style.overflow = "";
  }

  if (searchButton) searchButton.addEventListener("click", openSearch);
  if (searchBox) {
    d.getElementById("VPSearchBackdrop").addEventListener("click", closeSearch);
    searchInput.addEventListener("input", function () {
      var q = searchInput.value.trim();
      searchClear.disabled = !q;
      if (!q) {
        renderResults([], "");
        return;
      }
      runSearch(q)
        .then(function (results) {
          renderResults(results, q);
        })
        .catch(function () {
          searchResults.innerHTML =
            '<li class="no-results">Search index unavailable.</li>';
        });
    });
    searchClear.addEventListener("click", function () {
      searchInput.value = "";
      searchInput.dispatchEvent(new Event("input"));
      searchInput.focus();
    });
    searchInput.addEventListener("keydown", function (e) {
      var items = searchResults.querySelectorAll(".result");
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        if (!items.length) return;
        selectedIndex =
          e.key === "ArrowDown"
            ? Math.min(selectedIndex + 1, items.length - 1)
            : Math.max(selectedIndex - 1, 0);
        items.forEach(function (item, i) {
          item.classList.toggle("selected", i === selectedIndex);
        });
        items[selectedIndex].scrollIntoView({ block: "nearest" });
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (selectedIndex >= 0 && currentResults[selectedIndex]) {
          window.location.href = currentResults[selectedIndex].entry.url;
        }
      }
    });
  }

  d.addEventListener("keydown", function (e) {
    if (searchBox && !searchBox.hidden && e.key === "Escape") {
      closeSearch();
      return;
    }
    if (!searchBox || searchBox.hidden) {
      if (e.key === "k" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        openSearch();
      } else if (e.key === "/" && d.activeElement && !/^(INPUT|TEXTAREA)$/.test(d.activeElement.tagName)) {
        e.preventDefault();
        openSearch();
      }
    }
  });
})();
