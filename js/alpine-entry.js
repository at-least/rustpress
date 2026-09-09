/* gen-docs front-end: Alpine.js replaces the old vanilla bundle.
   The FOUC anti-flash script stays inline in <head> (Alpine cannot run
   pre-paint); everything else — menus, flyouts, sidebar drawer and
   carets, appearance switch, code copy buttons, code-group tabs,
   scrollspy — is Alpine components + inline directives. */

import Alpine from "alpinejs";
import intersect from "@alpinejs/intersect";
import collapse from "@alpinejs/collapse";

Alpine.plugin(intersect);
Alpine.plugin(collapse);

Alpine.store("ui", {
  screen: false, // mobile full-screen nav menu
  search: false, // local search modal
  sidebar: false, // mobile sidebar drawer
});

/* Code-group tab switching. The tab strip (radio inputs + labels) is
   emitted server-side by the markdown preprocessor; this component only
   wires label clicks to showing the matching <pre>. */
Alpine.data("codeGroup", () => ({
  init() {
    const group = this.$el;
    const pres = Array.from(group.querySelectorAll(":scope > .blocks > pre"));
    const labels = Array.from(group.querySelectorAll(":scope > .tabs label"));
    const activate = (i) => {
      pres.forEach((p, j) => p.classList.toggle("active", i === j));
    };
    labels.forEach((label, i) => label.addEventListener("click", () => activate(i)));
    activate(0);
  },
}));

/* Doc-page scrollspy: navbar `.top` state, right-hand outline active
   link + marker position (ported from the old scrollspy.js — rAF
   throttled, active heading = last one whose top is above the nav). */
Alpine.data("docPage", () => ({
  activeId: "",
  init() {
    const content = this.$el.querySelector("#main");
    const headings = content
      ? Array.from(content.querySelectorAll("h2[id], h3[id], h4[id], h5[id], h6[id]"))
      : [];
    if (headings.length === 0) return;

    const nav = document.getElementById("VPNavBar");
    const marker = document.getElementById("VPOutlineMarker");
    const links = Array.from(content.closest("#VPContent").querySelectorAll("a.outline-link[href^='#']"));

    let ticking = false;
    const update = () => {
      ticking = false;
      const offset = 96;
      let current = "";
      for (const h of headings) {
        if (h.getBoundingClientRect().top < offset) current = h.id;
      }
      if (nav) nav.classList.toggle("top", window.scrollY <= 0);
      if (current !== this.activeId) {
        this.activeId = current;
        links.forEach((l) => l.classList.toggle("active", l.getAttribute("href") === `#${current}`));
        if (marker) {
          const link = links.find((l) => l.classList.contains("active"));
          marker.style.opacity = link ? "1" : "0";
          if (link) marker.style.top = `${link.offsetTop + 8}px`;
        }
      }
    };
    const onScroll = () => {
      if (!ticking) {
        ticking = true;
        requestAnimationFrame(update);
      }
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    update();
  },
}));

/* Appearance toggle: the visual state is CSS-driven by html.dark; this
   only flips the class, persists the preference under the same
   localStorage key the anti-flash script reads, and syncs aria-checked
   on both switches (navbar + nav screen). */
window.gdToggleAppearance = () => {
  const dark = !document.documentElement.classList.contains("dark");
  document.documentElement.classList.toggle("dark", dark);
  try {
    localStorage.setItem("vitepress-theme-appearance", dark ? "dark" : "light");
  } catch (e) {}
  document.querySelectorAll(".VPSwitchAppearance").forEach((el) => {
    el.setAttribute("aria-checked", dark ? "true" : "false");
  });
};

/* Copy button handler for code blocks (button markup is server-side). */
window.gdCopyCode = (button) => {
  const text = button.closest("pre").innerText;
  const done = () => {
    button.classList.add("copied");
    setTimeout(() => button.classList.remove("copied"), 1200);
  };
  if (navigator.clipboard && navigator.clipboard.writeText) {
    navigator.clipboard.writeText(text).then(done, done);
  } else {
    const ta = document.createElement("textarea");
    ta.value = text;
    document.body.appendChild(ta);
    ta.select();
    try {
      document.execCommand("copy");
    } catch (e) {}
    document.body.removeChild(ta);
    done();
  }
};

/* Local search modal: whitespace-tokenized scoring over the
   build-generated search-docs.json (url/title/body per page), rendered
   into the VitePress search modal shell. Ported line-for-line from the
   old vanilla search.js: title exact +20, +5 per title hit, body
   occurrences capped at +20/token, top 20, <mark>-highlighted
   excerpts, arrow-key navigation, Ctrl/Cmd+K and / to open. */
Alpine.data("searchModal", () => ({
  q: "",
  selected: -1,
  results: [],
  error: false,
  docsPromise: null,

  init() {
    window.addEventListener("keydown", (e) => {
      if (this.$store.ui.search && e.key === "Escape") {
        this.close();
        return;
      }
      if (!this.$store.ui.search) {
        if (e.key === "k" && (e.ctrlKey || e.metaKey)) {
          e.preventDefault();
          this.open();
        } else if (
          e.key === "/" &&
          document.activeElement &&
          !/^(INPUT|TEXTAREA)$/.test(document.activeElement.tagName)
        ) {
          e.preventDefault();
          this.open();
        }
      }
    });
    this.$watch("$store.ui.search", (open) => {
      if (open) {
        document.body.style.overflow = "hidden";
        this.$nextTick(() => this.$refs.input.focus() || this.$refs.input.select());
      } else {
        document.body.style.overflow = "";
      }
    });
  },

  load() {
    if (!this.docsPromise) {
      this.docsPromise = fetch(this.$el.dataset.indexUrl)
        .then((r) => {
          if (!r.ok) throw new Error("search index " + r.status);
          return r.json();
        })
        .catch((err) => {
          this.docsPromise = null;
          throw err;
        });
    }
    return this.docsPromise;
  },

  search() {
    const q = this.q.trim();
    this.selected = -1;
    if (!q) {
      this.results = [];
      this.error = false;
      return;
    }
    this.load()
      .then((docs) => {
        this.error = false;
        this.results = this.score(docs, q);
      })
      .catch(() => {
        this.error = true;
        this.results = [];
      });
  },

  score(docs, q) {
    const tokens = q.toLowerCase().split(/\s+/).filter(Boolean);
    if (!tokens.length) return [];
    return docs
      .map((entry) => {
        let title = (entry.title || "").toLowerCase();
        const content = (entry.body || "").toLowerCase();
        let score = 0;
        tokens.forEach((t) => {
          if (title === t) score += 20;
          while (title.indexOf(t) !== -1) {
            score += 5;
            title = title.replace(t, " ");
          }
          let count = 0;
          let idx = content.indexOf(t);
          while (idx !== -1 && count < 50) {
            count++;
            idx = content.indexOf(t, idx + t.length);
          }
          score += Math.min(count, 20);
        });
        return { entry, score };
      })
      .filter((r) => r.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, 20);
  },

  esc(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  },

  mark(text, tokens) {
    if (!tokens.length) return this.esc(text);
    const pattern = new RegExp(
      "(" + tokens.map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|") + ")",
      "gi",
    );
    // split on the raw text first, then escape each piece, so tokens can
    // never match inside HTML entities produced by esc()
    const parts = text.split(pattern);
    return parts
      .map((part) => {
        const isToken = tokens.some((t) => part.toLowerCase() === t.toLowerCase());
        return isToken ? "<mark>" + this.esc(part) + "</mark>" : this.esc(part);
      })
      .join("");
  },

  excerpt(content, tokens) {
    const lower = content.toLowerCase();
    let pos = -1;
    for (let i = 0; i < tokens.length && pos < 0; i++) {
      pos = lower.indexOf(tokens[i]);
    }
    const start = Math.max(0, pos - 60);
    const end = Math.min(content.length, (pos < 0 ? 0 : pos) + 240);
    return (start > 0 ? "…" : "") + content.slice(start, end) + (end < content.length ? "…" : "");
  },

  get resultsHtml() {
    if (this.error) {
      return '<li class="no-results">Search index unavailable.</li>';
    }
    if (!this.results.length) {
      return (
        '<li class="no-results">' +
        (this.q.trim() ? 'No results for "<b>' + this.esc(this.q.trim()) + '</b>"' : "") +
        "</li>"
      );
    }
    const tokens = this.q.trim().toLowerCase().split(/\s+/).filter(Boolean);
    return this.results
      .map((r) => {
        const path = (r.entry.url || "").replace(/^https?:\/\/[^/]+/, "").split("/").filter(Boolean);
        let titles = '<div class="titles">';
        path.slice(0, -1).forEach((seg) => {
          titles += '<p class="title">' + this.esc(seg) + "</p>";
        });
        titles += '<p class="title main">' + this.mark(r.entry.title || r.entry.url, tokens) + "</p></div>";
        const excerpt =
          '<p class="excerpt">' + this.mark(this.excerpt(r.entry.body || "", tokens), tokens) + "</p>";
        return (
          '<li class="result" role="option" data-url="' + r.entry.url + '"><div>' + titles + excerpt + "</div></li>"
        );
      })
      .join("");
  },

  move(delta) {
    const count = this.results.length;
    if (!count) return;
    this.selected = Math.max(0, Math.min(this.selected + delta, count - 1));
    const el = this.$refs.results.querySelectorAll(".result")[this.selected];
    if (el) el.scrollIntoView({ block: "nearest" });
  },

  enter() {
    const r = this.results[this.selected];
    if (r) window.location.href = r.entry.url;
  },

  pick(e) {
    const li = e.target.closest("li.result");
    if (li && li.dataset.url) window.location.href = li.dataset.url;
  },

  clear() {
    this.q = "";
    this.search();
    this.$refs.input.focus();
  },

  open() {
    this.$store.ui.search = true;
  },

  close() {
    this.$store.ui.search = false;
  },
}));

window.Alpine = Alpine;
Alpine.start();
