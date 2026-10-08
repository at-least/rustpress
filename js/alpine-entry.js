/* rustpress front-end: Alpine.js replaces the old vanilla bundle.
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

/* Overlays must auto-close when the viewport grows past their
   breakpoint, or they linger invisibly with the scroll lock held
   (upstream: composables/nav.ts whenever(isTablet, closeScreen) and
   composables/layout.ts watch(isDesktop, closeSidebar)). Upstream also
   closes the sidebar drawer on Escape (useCloseSidebarOnEscape). */
const tabletMQ = window.matchMedia("(min-width: 48rem)");
const desktopMQ = window.matchMedia("(min-width: 60rem)");
const syncOverlaysWithViewport = () => {
  if (tabletMQ.matches) Alpine.store("ui").screen = false;
  if (desktopMQ.matches) Alpine.store("ui").sidebar = false;
};
// addEventListener on MediaQueryList needs Safari >= 14; the bundle
// targets es2018, so fall back to the deprecated addListener
const watchMQ = (mq, fn) =>
  mq.addEventListener ? mq.addEventListener("change", fn) : mq.addListener(fn);
watchMQ(tabletMQ, syncOverlaysWithViewport);
watchMQ(desktopMQ, syncOverlaysWithViewport);
syncOverlaysWithViewport();
window.addEventListener("keydown", (e) => {
  // only the drawer's own Escape: with the search modal also open, its
  // handler owns the key and the drawer stays underneath (upstream's
  // refcounted lock behaves the same way)
  if (
    e.key === "Escape" &&
    Alpine.store("ui").sidebar &&
    !Alpine.store("ui").search
  ) {
    Alpine.store("ui").sidebar = false;
    document.getElementById("VPLocalNavMenu")?.focus();
  }
});

/* Same-page hash links (the skip link, outline links, heading anchors)
   move focus to their target once the browser has scrolled there, like
   upstream's router (scrollTo): a target that can't take focus gets a
   temporary tabindex="-1", dropped again on blur. */
document.addEventListener("click", (e) => {
  if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
  const a = e.target.closest && e.target.closest("a[href]");
  if (!a || !a.hash || a.origin !== location.origin || a.pathname !== location.pathname) return;
  let target = null;
  try {
    target = document.getElementById(decodeURIComponent(a.hash.slice(1)));
  } catch (err) {}
  if (!target) return;
  requestAnimationFrame(() => {
    target.focus({ preventScroll: true });
    if (document.activeElement === target || target.hasAttribute("tabindex")) return;
    const restore = () => {
      target.removeAttribute("tabindex");
      target.removeEventListener("blur", restore);
    };
    target.setAttribute("tabindex", "-1");
    target.addEventListener("blur", restore);
    target.focus({ preventScroll: true });
    if (document.activeElement !== target) restore();
  });
});

/* Code-group tab switching. The tab strip (radio inputs + labels) is
   emitted server-side by the markdown preprocessor; this component shows
   the <pre> of whichever radio is checked. `change` fires for a label
   click and for arrow keys alike (a label-only listener left the pane
   behind when the keyboard moved the selection). */
Alpine.data("codeGroup", () => ({
  init() {
    const group = this.$el;
    const pres = Array.from(group.querySelectorAll(":scope > .blocks > pre"));
    const inputs = Array.from(group.querySelectorAll(":scope > .tabs input"));
    const activate = (i) => {
      pres.forEach((p, j) => p.classList.toggle("active", i === j));
    };
    inputs.forEach((input, i) => input.addEventListener("change", () => activate(i)));
    activate(0);
  },
}));

/* Doc-page scrollspy: navbar `.top` state, right-hand outline active
   link + marker position (ported from the old scrollspy.js — rAF
   throttled, active heading = last one whose top is above the nav). */
/* VPLocalNav on a page with neither outline headers nor a sidebar:
   shown once the page has scrolled past the navbar (upstream's
   isScrolled). Like upstream, the nav height comes from a probe element:
   a custom property reads back as its raw token ("4rem"). */
Alpine.data("localNavScroll", () => ({
  scrolled: false,
  init() {
    const probe = document.createElement("div");
    probe.style.cssText = "position: absolute; visibility: hidden; height: var(--vp-nav-height)";
    document.body.appendChild(probe);
    const navHeight = probe.offsetHeight;
    probe.remove();
    const update = () => {
      this.scrolled = window.scrollY >= navHeight;
    };
    window.addEventListener("scroll", update, { passive: true });
    update();
  },
}));

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
const syncAppearanceSwitches = () => {
  const dark = document.documentElement.classList.contains("dark");
  document.querySelectorAll(".VPSwitchAppearance").forEach((el) => {
    el.setAttribute("aria-checked", dark ? "true" : "false");
    // the tooltip names what a click will do
    const title = el.dataset[dark ? "titleLight" : "titleDark"];
    if (title) el.setAttribute("title", title);
  });
};
// the markup is the light-mode state; the anti-flash script may already
// have switched <html> to dark before this bundle runs
syncAppearanceSwitches();

window.gdToggleAppearance = () => {
  const apply = () => {
    const dark = !document.documentElement.classList.contains("dark");
    document.documentElement.classList.toggle("dark", dark);
    try {
      localStorage.setItem("vitepress-theme-appearance", dark ? "dark" : "light");
    } catch (e) {}
    syncAppearanceSwitches();
  };
  // View Transitions API: smooth cross-fade on the color flip
  // (upstream does the same; browsers without it flip immediately)
  if (document.startViewTransition) {
    document.startViewTransition(apply);
  } else {
    apply();
  }
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
        this.$nextTick(() => {
          this.$refs.input.focus();
          this.$refs.input.select();
        });
      }
    });
  },

  load() {
    if (!this.docsPromise) {
      // `$el` is the element that triggered the call (the <input>);
      // the data attributes sit on the component root
      this.docsPromise = fetch(this.$root.dataset.indexUrl)
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
        // upstream pre-selects the top result: typing then Enter opens it
        this.selected = this.results.length ? 0 : -1;
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
    // & < > for text content, quotes too so the result is also safe in a
    // double-quoted attribute (data-url)
    return s
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
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
    // like upstream, the list stays empty until a query is typed
    if (!this.q.trim()) return "";
    if (!this.results.length) {
      return (
        '<li class="no-results">' +
        (this.$root.dataset.noResults || 'No results for "{q}"').replace('{q}', '<strong>' + this.esc(this.q.trim()) + '</strong>') +
        "</li>"
      );
    }
    const tokens = this.q.trim().toLowerCase().split(/\s+/).filter(Boolean);
    return this.results
      .map((r, i) => {
        const path = (r.entry.url || "").replace(/^https?:\/\/[^/]+/, "").split("/").filter(Boolean);
        const chevron =
          '<svg class="inline-block size-[0.875rem] opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><path d="m9 18l6-6l-6-6"/></svg>';
        let titles = '<div class="titles"><span class="title-icon">#</span>';
        path.slice(0, -1).forEach((seg) => {
          titles += '<span class="title"><span class="text">' + this.esc(seg) + "</span>" + chevron + "</span>";
        });
        titles +=
          '<span class="title main"><span class="text">' + this.mark(r.entry.title || r.entry.url, tokens) + "</span></span></div>";
        const excerpt =
          '<p class="excerpt">' + this.mark(this.excerpt(r.entry.body || "", tokens), tokens) + "</p>";
        return (
          '<li class="result' + (i === this.selected ? " selected" : "") + '" role="option" aria-selected="' + (i === this.selected) + '" data-url="' + this.esc(r.entry.url) + '"><div>' + titles + excerpt + "</div></li>"
        );
      })
      .join("");
  },

  move(delta) {
    const count = this.results.length;
    if (!count) return;
    // wraps at both ends, like upstream
    this.selected = (this.selected + delta + count) % count;
    // x-html re-renders the list for the new selection first
    this.$nextTick(() => {
      const el = this.$refs.results.querySelector(".result.selected");
      if (el) el.scrollIntoView({ block: "nearest" });
    });
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

/* Single writer for the body scroll lock: one effect over the whole
   overlay state, so no two overlays can fight over body.overflow
   (upstream: the refcounted useBodyScrollLock composable). Like upstream,
   the root keeps its scrollbar gutter while locked, so hiding a classic
   scrollbar does not shift the layout sideways. */
let gutterToRestore; // the root's inline scrollbar-gutter, while locked
Alpine.effect(() => {
  const locked =
    Alpine.store("ui").screen ||
    Alpine.store("ui").search ||
    Alpine.store("ui").sidebar;
  const root = document.documentElement;
  if (locked && gutterToRestore === undefined) {
    gutterToRestore = root.style.scrollbarGutter;
    // `|| ""`: no scrollbar-gutter support (Safari < 18.2) reads undefined
    if (!(getComputedStyle(root).scrollbarGutter || "").includes("stable")) {
      root.style.scrollbarGutter = "stable";
    }
  } else if (!locked && gutterToRestore !== undefined) {
    root.style.scrollbarGutter = gutterToRestore;
    gutterToRestore = undefined;
  }
  document.body.style.overflow = locked ? "hidden" : "";
});
