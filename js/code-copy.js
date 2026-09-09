/* Code copy buttons + lang labels.
   Adds the language-* class and span.lang so the ported VitePress
   .vp-doc [class*='language-'] styles apply to Zola's bare <pre.giallo>.
   Labels show the VitePress-style short name; giallo normalizes ids
   (shellscript/typescript/…), so map the common ones back. */

var d = document;

var LANG_ALIASES = {
  shellscript: "sh",
  typescript: "ts",
  javascript: "js",
  markdown: "md",
  plain: "text"
};

/* Inline copy/copy-checked icons; the .vp-doc wrapper utilities toggle
   which one shows through the button's .copied class. */
var COPY_ICONS = {
  normal: '<svg class="icon-copy size-[0.875rem] m-auto" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 2 0 0 1 2 2v14a2 2 2 0 0 1-2 2H6a2 2 2 0 0 1-2-2V6a2 2 2 0 0 1 2-2h2"/></svg>',
  copied: '<svg class="icon-copied size-[0.875rem] m-auto" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" aria-hidden="true"><rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 2 0 0 1 2 2v14a2 2 2 0 0 1-2 2H6a2 2 2 0 0 1-2-2V6a2 2 2 0 0 1 2-2h2"/><path d="m9 14l2 2l4-4"/></svg>'
};

export function init() {
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
    button.className = "vp-copy-button copy flex";
    button.innerHTML = COPY_ICONS.normal + COPY_ICONS.copied;
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
}
