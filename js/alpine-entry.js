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

window.Alpine = Alpine;
Alpine.start();
