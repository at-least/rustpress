/* Local search: whitespace-tokenized scoring over Zola's fuse_json
   index, rendered into the ported VitePress search modal. */

var d = document;

var searchBox;
var searchButton;
var searchInput;
var searchResults;
var searchClear;
var indexPromise = null;
var selectedIndex = -1;
var currentResults = [];

function searchIndexUrl() {
  if (window.rustPress && window.rustPress.searchIndexUrl) {
    return window.rustPress.searchIndexUrl;
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
  if (!(window.rustPress && window.rustPress.searchIndexUrl)) return;
  searchBox.hidden = false;
  d.body.style.overflow = "hidden";
  searchInput.focus();
  searchInput.select();
}

function closeSearch() {
  searchBox.hidden = true;
  d.body.style.overflow = "";
}

export function init() {
  searchBox = d.getElementById("VPLocalSearchBox");
  searchButton = d.getElementById("VPSearchButton");
  searchInput = d.getElementById("localsearch-input");
  searchResults = d.getElementById("VPSearchResults");
  searchClear = d.getElementById("VPSearchClear");

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
}
