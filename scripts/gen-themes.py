#!/usr/bin/env python3
"""Generate bundled rustpress themes from the vendored Helix palettes.

Reads assets/syntax-themes/helix/*.toml [palette] tables, maps each
scheme onto the rustpress design-token contract (hand-written slot
maps below), derives monotonic text/border ramps and the neutral ramp,
auto-fits accents for WCAG AA (the same ratios tests/theme_contract.rs
enforces), and emits static/themes/<name>.css.

The emitted files are the source of truth once committed — rerun after
editing the slot maps, then regenerate the docs index (node
scripts/gen-theme-index.mjs) and pass the contract tests. The four
hand-tuned themes (github, catppuccin, nord, rose-pine) are NOT
regenerated here; edit them directly.
"""
import tomllib

import os
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HELVIX = f"{ROOT}/assets/syntax-themes/helix"

# --- color math (mirrors tests/theme_contract.rs) ------------------------
def rgb(c):
    n = int(c.lstrip("#"), 16)
    return ((n >> 16) & 255, (n >> 8) & 255, n & 255)

def hexs(c):
    r, g, b = c
    return f"#{r:02x}{g:02x}{b:02x}"

def lum(c):
    def ch(v):
        v = v / 255
        return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4
    r, g, b = rgb(c) if isinstance(c, str) else c
    return 0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)

def contrast(a, b):
    la, lb = lum(a), lum(b)
    hi, lo = max(la, lb), min(la, lb)
    return (hi + 0.05) / (lo + 0.05)

def mix(a, b, t):
    """blend a toward b by t (sRGB per-channel)."""
    (ar, ag, ab), (br, bg_, bb) = rgb(a), rgb(b)
    return hexs((round(ar + (br - ar) * t), round(ag + (bg_ - ag) * t), round(ab + (bb - ab) * t)))

EPS = 0.03  # float-safety margin so python's fit clears rust's check

def fit(c, against, target, mode):
    """darken (light mode) / lighten (dark mode) until target contrast;
    if the primary direction cannot reach it, try the other way."""
    target += EPS
    for t in ((black, white) if mode == "light" else (white, black)):
        cur = c
        for _ in range(60):
            if contrast(cur, against) >= target:
                return cur
            cur = mix(cur, t, 0.04)
        if contrast(cur, against) > contrast(c, against):
            c = cur
    return c

def composite(hex_color, alpha, bg):
    (fr, fg_, fb), (br, bg2, bb) = rgb(hex_color), rgb(bg)
    return hexs((round(fr * alpha + br * (1 - alpha)), round(fg_ * alpha + bg2 * (1 - alpha)), round(fb * alpha + bb * (1 - alpha))))

def fit_on_tint(c, bg, alpha, target, mode):
    """fit c so it passes on its own alpha tint over bg."""
    target += EPS
    for _ in range(80):
        if contrast(c, composite(c, alpha, bg)) >= target and contrast(c, bg) >= target:
            return c
        c = mix(c, black if mode == "light" else white, 0.04)
    return c

black, white = "#000000", "#ffffff"

def _norm_hex(v):
    v = v.lower()
    if len(v) == 4:  # #abc → #aabbcc
        v = "#" + "".join(ch * 2 for ch in v[1:])
    return v

def pal(file):
    data = tomllib.load(open(f"{HELVIX}/{file}.toml", "rb"))
    p = data.get("palette")
    if p:
        return {k: _norm_hex(v) for k, v in p.items()
                if isinstance(v, str) and v.startswith("#")}
    out = {}
    def walk(d, prefix=""):
        for k, v in d.items():
            if isinstance(v, str) and v.startswith("#") and len(v) in (4, 7):
                out[f"{prefix}{k}"] = _norm_hex(v)
            elif isinstance(v, dict):
                walk(v, f"{prefix}{k}.")
    walk(data)
    return out

# --- theme spec -----------------------------------------------------------
# slot keys: bg bg_alt bg_elv text1 text2 text3 border brand1 brand2 brand3
#            success warning danger important sponsor
# light_mode="derived" → build a light half from the dark palette (for
# schemes that publish no light mode; stated in the header).
SPECS = [
 dict(name="tokyonight", desc="calm blue-violet, day and storm",
      light=dict(f="tokyonight_day", m=dict(bg="bg", bg_alt="bg-selection", bg_elv="bg-menu",
        text1="fg", text2="fg-dark", text3="fg-linenr", border="border",
        brand1="blue", success="green", warning="yellow", danger="red", important="purple", sponsor="magenta")),
      dark=dict(f="tokyonight_storm", p2="tokyonight", m=dict(bg="bg", bg_alt="bg-menu", bg_elv="bg-selection",
        text1="#c0caf5", text2="#9d9dbb", text3="#565f89", border="border",
        brand1="p2:blue", success="p2:green", warning="p2:yellow", danger="p2:red", important="p2:purple", sponsor="p2:magenta"))),
 dict(name="gruvbox", desc="retro groove, warm cream and charcoal",
      light=dict(f="gruvbox_light", m=dict(bg="bg0", bg_alt="bg1", bg_elv="bg0",
        text1="fg0", text2="fg1", text3="fg4", border="bg3",
        brand1="blue1", success="green1", warning="yellow1", danger="red1", important="purple1", sponsor="purple1")),
      dark=dict(f="gruvbox", m=dict(bg="bg0", bg_alt="bg0_s", bg_elv="bg1",
        text1="fg1", text2="fg0", text3="fg4", border="bg2",
        brand1="blue1", success="green1", warning="yellow1", danger="red1", important="purple1", sponsor="purple1"))),
 dict(name="gruvbox-material", desc="the material take on gruvbox, softer and calmer",
      light=dict(f="gruvbox_material_light_medium", m=dict(bg="bg0", bg_alt="bg1", bg_elv="bg0",
        text1="fg1", text2="fg0", text3="grey0", border="bg3",
        brand1="blue", success="green", warning="yellow", danger="red", important="purple", sponsor="purple")),
      dark=dict(f="gruvbox_material_dark_medium", m=dict(bg="bg0", bg_alt="bg1", bg_elv="bg2",
        text1="fg1", text2="fg0", text3="grey1", border="bg3",
        brand1="blue", success="green", warning="yellow", danger="red", important="purple", sponsor="purple"))),
 dict(name="onedark", desc="Atom's One: grey-blue surfaces, saturated accents",
      light=dict(f="onelight", m=dict(bg="white", bg_alt="grey-100", bg_elv="white",
        text1="black", text2="grey", text3="grey-500", border="grey-300",
        brand1="blue", success="green", warning="gold", danger="red", important="purple", sponsor="pink")),
      dark=dict(f="onedark", m=dict(bg="black", bg_alt="light-black", bg_elv="gray",
        text1="white", text2="light-gray", text3="linenr", border="gray",
        brand1="blue", success="green", warning="yellow", danger="red", important="purple", sponsor="purple"))),
 dict(name="everforest", desc="green, calm and low-noise by design",
      light=dict(f="everforest_light", m=dict(bg="bg0", bg_alt="bg1", bg_elv="bg0",
        text1="fg", text2="statusline2", text3="grey0", border="bg3",
        brand1="blue", success="green", warning="yellow", danger="red", important="purple", sponsor="purple")),
      dark=dict(f="everforest_dark", m=dict(bg="bg0", bg_alt="bg1", bg_elv="bg2",
        text1="fg", text2="statusline2", text3="grey0", border="bg3",
        brand1="blue", success="green", warning="yellow", danger="red", important="purple", sponsor="orange"))),
 dict(name="ayu", desc="subtle, elegant, one bright accent set",
      light=dict(f="ayu_light", m=dict(bg="background", bg_alt="black", bg_elv="white",
        text1="foreground", text2="#54595f", text3="gray", border="dark_gray",
        brand1="blue", success="green", warning="orange", danger="red", important="magenta", sponsor="red")),
      dark=dict(f="ayu_dark", m=dict(bg="background", bg_alt="black", bg_elv="dark_gray",
        text1="foreground", text2="gray", text3="#454f58", border="dark_gray",
        brand1="blue", success="green", warning="yellow", danger="red", important="magenta", sponsor="magenta")),
      dark_alt=dict(bg_alt="#0b0f14")),
 dict(name="solarized", desc="the classic: selective contrast, warm and marine",
      light=dict(f="solarized_light", m=dict(bg="base03", bg_alt="base02", bg_elv="base03",
        text1="base01", text2="base0", text3="base00", border="base015",
        brand1="blue", success="green", warning="yellow", danger="red", important="violet", sponsor="magenta")),
      dark=dict(f="solarized_dark", m=dict(bg="base03", bg_alt="base025", bg_elv="base015",
        text1="base2", text2="base1", text3="base00", border="base01",
        brand1="blue", success="green", warning="yellow", danger="red", important="violet", sponsor="magenta"))),
 dict(name="kanagawa", desc="wave and lotus: ink, sumi and ukiyo-e color",
      light=dict(f="kanagawa-lotus", m=dict(bg="lotusWhite0", bg_alt="lotusWhite1", bg_elv="lotusWhite0",
        text1="lotusInk0", text2="lotusPunctuation", text3="lotusInk1", border="lotusBlue1",
        brand1="lotusBlue2", success="lotusSuperGreen", warning="lotusYellow2", danger="lotusAutumnRed",
        important="lotusSuperViolet", sponsor="lotusSuperPink")),
      dark=dict(f="kanagawa", m=dict(bg="sumiInk3", bg_alt="sumiInk4", bg_elv="sumiInk5",
        text1="fujiWhite", text2="oldWhite", text3="fujiGray", border="sumiInk6",
        brand1="crystalBlue", success="springGreen", warning="carpYellow", danger="waveRed",
        important="oniViolet", sponsor="sakuraPink"))),
 dict(name="papercolor", desc="plain paper, high legibility, 16-color spirit",
      light=dict(f="papercolor-light", m=dict(bg="background", bg_alt="#e0e0e0", bg_elv="background",
        text1="foreground", text2="regular5", text3="linenumber_fg", border="matchparen_bg",
        brand1="regular4", success="regular2", warning="cursorlinenr_fg", danger="regular1",
        important="bright3", sponsor="bright2")),
      dark=dict(f="papercolor-dark", m=dict(bg="background", bg_alt="cursorline_secondary_bg", bg_elv="popupmenu_bg",
        text1="foreground", text2="bright0", text3="linenumber_fg", border="matchparen_bg",
        brand1="regular4", success="regular2", warning="bright4", danger="regular1",
        important="bright3", sponsor="bright5"))),
 dict(name="seoul256", desc="low-contrast seoul nights, paper days derived",
      light="derived",
      dark=dict(f="seoul256-dark", m=dict(bg="black1", bg_alt="gray1", bg_elv="gray3",
        text1="white", text2="gray13", text3="gray6", border="gray4",
        brand1="blue1", success="green3", warning="yellow5", danger="salmon1",
        important="magenta1", sponsor="mauve"))),
 dict(name="modus", desc="Emacs' accessible flagship: maximum legibility",
      light=dict(f="modus_operandi", m=dict(bg="bg-main", bg_alt="bg-dim", bg_elv="bg-main",
        text1="fg-main", text2="fg-dim", text3="#6f6f6f", border="bg-active",
        brand1="blue", success="green", warning="yellow", danger="red", important="magenta", sponsor="magenta-warmer")),
      dark=dict(f="modus_vivendi", m=dict(bg="bg-main", bg_alt="bg-dim", bg_elv="bg-inactive",
        text1="fg-main", text2="fg-dim", text3="#a8a8a8", border="bg-inactive",
        brand1="blue", success="green", warning="yellow", danger="red", important="magenta", sponsor="magenta-warmer"))),
 dict(name="flexoki", desc="ink on paper: Step's warm absolute palette",
      light=dict(f="flexoki_light", m=dict(bg="bg", bg_alt="bg-2", bg_elv="#ffffff",
        text1="tx", text2="tx-2", text3="tx-3", border="ui-3",
        brand1="bl", success="gr", warning="ye", danger="re", important="pu", sponsor="ma")),
      dark=dict(f="flexoki_dark", m=dict(bg="bg", bg_alt="bg-2", bg_elv="ui-2",
        text1="tx", text2="tx-2", text3="tx-3", border="ui-3",
        brand1="bl", success="gr", warning="ye", danger="re", important="pu", sponsor="ma"))),
 dict(name="iceberg", desc="cool navy-blue calm, light and dark",
      light=dict(f="iceberg-light", m=dict(bg="background_bg", bg_alt="sel_bg", bg_elv="background_bg",
        text1="background_fg", text2="statusline_fg", text3="comment_fg", border="matchparen_bg",
        brand1="blue", success="green", warning="yellow", danger="red", important="magenta", sponsor="light-red")),
      dark=dict(f="iceberg-dark", m=dict(bg="background_bg", bg_alt="linenr_bg", bg_elv="cursorlinenr_bg",
        text1="background_fg", text2="pale", text3="comment_fg", border="matchparen_bg",
        brand1="blue", success="green", warning="yellow", danger="red", important="magenta", sponsor="light-red"))),
 dict(name="adwaita", desc="GNOME's own: libadwaita light and dark",
      light=dict(f="adwaita-dark", m=dict(bg="light_1", bg_alt="light_3", bg_elv="light_1",
        text1="dark_5", text2="dark_2", text3="dark_1", border="light_4",
        brand1="blue_5", success="green_6", warning="yellow_6", danger="red_5",
        important="purple_5", sponsor="red_5")),
      dark=dict(f="adwaita-dark", m=dict(bg="libadwaita_dark", bg_alt="libadwaita_dark_alt", bg_elv="libadwaita_popup",
        text1="light_2", text2="light_5", text3="dark_1", border="split_and_borders",
        brand1="blue_2", success="green_3", warning="yellow_4", danger="red_3",
        important="purple_2", sponsor="red_2"))),
 dict(name="dracula", desc="the famous dark purple, light derived from its palette",
      light="derived",
      dark=dict(f="dracula", m=dict(bg="background", bg_alt="black", bg_elv="current_line",
        text1="foreground", text2="comment", text3="grey", border="indent",
        brand1="purple", success="green", warning="yellow", danger="red",
        important="pink", sponsor="pink"))),
 dict(name="monokai", desc="the classic that colored a decade of editors",
      light="derived",
      dark=dict(f="monokai", p2="monokai_pro", m=dict(bg="background", bg_alt="widget", bg_elv="selection",
        text1="text", text2="#a8aa95", text3="#75715d", border="#3b3c35",
        brand1="#66d9ef", success="#a6e22e", warning="#e6db74", danger="#f92672",
        important="#ae81ff", sponsor="#fd971f"))),
 dict(name="material", desc="Google's material ocean, light derived from its palette",
      light="derived",
      dark=dict(f="material_oceanic", m=dict(bg="bg", bg_alt="active", bg_elv="highlight",
        text1="text", text2="gray", text3="disabled", border="disabled",
        brand1="accent", success="#c3e88d", warning="#ffcb6b", danger="#f07178",
        important="#c792ea", sponsor="#ff5370"))),
 dict(name="night-owl", desc="deep navy, bright primaries; light derived from its palette",
      light="derived",
      dark=dict(f="night_owl", m=dict(bg="background", bg_alt="background2", bg_elv="selection",
        text1="foreground", text2="slate", text3="grey7", border="grey4",
        brand1="blue", success="green", warning="gold", danger="red",
        important="pink", sponsor="pink"))),
 dict(name="sonokai", desc="high-contrast, saturated, EdenEast's flavor",
      light="derived",
      dark=dict(f="sonokai", m=dict(bg="bg0", bg_alt="bg1", bg_elv="bg2",
        text1="fg", text2="grey", text3="grey_dim", border="bg3",
        brand1="blue", success="green", warning="yellow", danger="red",
        important="purple", sponsor="red"))),
 dict(name="darcula", desc="JetBrains' default: warm greys, muted accents",
      light="derived",
      dark=dict(f="darcula", m=dict(bg="grey00", bg_alt="grey01", bg_elv="grey02",
        text1="grey05", text2="grey04", text3="grey03", border="grey02",
        brand1="lightblue", success="darkgreen", warning="yellow", danger="darkred",
        important="purple", sponsor="rainbow_purple"))),
]

def resolve(spec_map, palette, palette2):
    out = {}
    for slot, ref in spec_map.items():
        if isinstance(ref, str) and ref.startswith("#"):
            out[slot] = ref
        elif ref.startswith("p2:"):
            out[slot] = palette2[ref[3:]]
        else:
            out[slot] = palette[ref]
    return out

def build_mode(mode, m, adjusted):
    """mode: 'light'|'dark'; m: resolved slot colors (mutated by AA fits)."""
    def A(c, what):
        if c != what[0]:
            adjusted.add(what[1])
        return c
    orig = dict(m)
    def fitk(slot, against, target, tag):
        c = fit(m[slot], against, target, mode)
        if c != orig[slot]:
            adjusted.add(tag)
        m[slot] = c
    bg, bg_alt, bg_elv = m["bg"], m["bg_alt"], m["bg_elv"]
    bg_soft = bg_alt
    fitk("text1", bg, 7.0, "text")
    # derive the rest of the text ramp from the fitted text-1 so the
    # hierarchy is monotonic by construction (source secondaries can
    # converge or invert once contrast-fitted)
    worst2 = min([bg, bg_alt, bg_soft], key=lambda s: contrast(mix(m["text1"], bg, 0.22), s))
    m["text2"] = fit(mix(m["text1"], bg, 0.22), worst2, 4.5, mode)
    m["text3"] = fit(mix(m["text2"], bg, 0.45), bg, 3.0, mode)
    # border: keep the scheme's own if it separates from the bg, else
    # derive one from the text hue (borders must not vanish)
    if contrast(m["border"], bg) < 1.15:
        m["border"] = mix(m["text1"], bg, 0.78 if mode == "light" else 0.68)
        adjusted.add("border")
    # brand + semantics: fit on their own tint (composite), which the
    # contract test measures; a margin over bg keeps the tint passing
    alpha = 0.10 if mode == "light" else 0.14
    s_alpha = 0.12 if mode == "light" else 0.15
    for slot in ["brand1", "success", "warning", "danger", "important"]:
        a = alpha if slot == "brand1" else s_alpha
        before = m[slot]
        m[slot] = fit_on_tint(m[slot], bg, a, 4.5, mode)
        if m[slot] != before:
            adjusted.add("accents")
    # brand-2 hover / brand-3 button bg: derive from brand-1, keep white
    # text on both >= 3.0
    brand2 = m.get("brand2") or mix(m["brand1"], white if mode == "dark" else black, 0.15)
    brand3 = m.get("brand3") or mix(m["brand1"], black, 0.22)
    for which, c in (("brand2", brand2), ("brand3", brand3)):
        m[which] = fit(c, white, 3.0, "light")  # always darken toward black
    neutral = mix(m["text1"], bg, 0.55)
    if mode == "light":
        default = [mix(neutral, bg, 0.25), mix(neutral, bg, 0.45), mix(neutral, bg, 0.65)]
    else:
        default = [mix(neutral, bg, 0.05), mix(neutral, bg, 0.28), mix(neutral, bg, 0.48)]
    return dict(
        neutral_inverse="#ffffff" if mode == "light" else "#000000",
        bg=bg, bg_alt=bg_alt, bg_elv=bg_elv, bg_soft=bg_soft,
        text1=m["text1"], text2=m["text2"], text3=m["text3"],
        border=m["border"], divider=m["border"], gutter=m["border"],
        brand1=m["brand1"], brand2=m["brand2"], brand3=m["brand3"],
        brand_soft=rgba(m["brand1"], alpha),
        default1=default[0], default2=default[1], default3=default[2],
        default_soft=rgba(neutral, 0.22 if mode == "light" else 0.18),
        tip1=m["brand1"], tip_soft=rgba(m["brand1"], alpha),
        note1=m["brand1"], note_soft=rgba(m["brand1"], alpha),
        success1=m["success"], success_soft=rgba(m["success"], s_alpha),
        important1=m["important"], important_soft=rgba(m["important"], s_alpha),
        warning1=m["warning"], warning_soft=rgba(m["warning"], s_alpha),
        danger1=m["danger"], danger_soft=rgba(m["danger"], s_alpha),
        caution1=m["danger"], caution_soft=rgba(m["danger"], s_alpha),
        sponsor=m["sponsor"],
    )

def rgba(c, a):
    r, g, b = rgb(c)
    return f"rgba({r}, {g}, {b}, {a})"

TOKENS = [
    ("neutral_inverse", "--vp-c-neutral-inverse"), ("bg", "--vp-c-bg"), ("bg_alt", "--vp-c-bg-alt"),
    ("bg_elv", "--vp-c-bg-elv"), ("bg_soft", "--vp-c-bg-soft"),
    ("text1", "--vp-c-text-1"), ("text2", "--vp-c-text-2"), ("text3", "--vp-c-text-3"),
    ("border", "--vp-c-border"), ("divider", "--vp-c-divider"), ("gutter", "--vp-c-gutter"),
    ("brand1", "--vp-c-brand-1"), ("brand2", "--vp-c-brand-2"), ("brand3", "--vp-c-brand-3"),
    ("brand_soft", "--vp-c-brand-soft"),
    ("default1", "--vp-c-default-1"), ("default2", "--vp-c-default-2"), ("default3", "--vp-c-default-3"),
    ("default_soft", "--vp-c-default-soft"),
    ("tip1", "--vp-c-tip-1"), ("tip_soft", "--vp-c-tip-soft"),
    ("note1", "--vp-c-note-1"), ("note_soft", "--vp-c-note-soft"),
    ("success1", "--vp-c-success-1"), ("success_soft", "--vp-c-success-soft"),
    ("important1", "--vp-c-important-1"), ("important_soft", "--vp-c-important-soft"),
    ("warning1", "--vp-c-warning-1"), ("warning_soft", "--vp-c-warning-soft"),
    ("danger1", "--vp-c-danger-1"), ("danger_soft", "--vp-c-danger-soft"),
    ("caution1", "--vp-c-caution-1"), ("caution_soft", "--vp-c-caution-soft"),
    ("sponsor", "--vp-c-sponsor"),
]

def emit_css(name, desc, note, light, dark):
    def block(vals):
        return "\n".join(f"  {css}: {vals[k]};" for k, css in TOKENS)
    shadows = "\n".join(
        f"  --vp-shadow-{i}: {s};"
        for i, s in zip(
            range(1, 6), [
                f"0 1px 2px {rgba(light['text1'], .05)}, 0 1px 2px {rgba(light['text1'], .07)}",
                f"0 3px 12px {rgba(light['text1'], .09)}, 0 1px 4px {rgba(light['text1'], .09)}",
                f"0 12px 32px {rgba(light['text1'], .12)}, 0 2px 6px {rgba(light['text1'], .10)}",
                f"0 14px 44px {rgba(light['text1'], .14)}, 0 3px 9px {rgba(light['text1'], .14)}",
                f"0 18px 56px {rgba(light['text1'], .18)}, 0 4px 12px {rgba(light['text1'], .18)}",
            ]))
    return f"""/* Built-in theme: {name} — {desc}. {note} */
:root {{
{block(light)}

{shadows}
}}

.dark {{
{block(dark)}
}}
"""

report = []
for spec in SPECS:
    adjusted = set()
    dark_spec = spec["dark"]
    pd = pal(dark_spec["f"])
    pd2 = pal(dark_spec["p2"]) if "p2" in dark_spec else {}
    dm = resolve(dark_spec["m"], pd, pd2)
    if "dark_alt" in spec:  # per-theme literal fixes
        dm.update(spec["dark_alt"])
    dark = build_mode("dark", dm, adjusted)

    if spec.get("light") == "derived":
        src = dm if True else None
        tinted = lambda t: mix(spec_dark_bg, white, t)
        note = f"Dark half from the {spec['name']} palette; the scheme publishes no light mode, so the light half is derived from the same hues. Accents auto-adjusted for WCAG AA where the published colors fell short."
        spec_dark_bg = dm["bg"]
        light_map = dict(
            bg=mix(spec_dark_bg, white, 0.94), bg_alt=mix(spec_dark_bg, white, 0.90),
            bg_elv=mix(spec_dark_bg, white, 0.955),
            text1=mix(dm["text1"], black, 0.75), text2=mix(dm["text1"], black, 0.55),
            text3=mix(dm["text1"], black, 0.35), border=mix(spec_dark_bg, white, 0.84),
            brand1=dm["brand1"], brand2=mix(dm["brand1"], black, 0.15), brand3=mix(dm["brand1"], black, 0.28),
            success=dm["success"], warning=dm["warning"], danger=dm["danger"],
            important=dm["important"], sponsor=dm["sponsor"])
    else:
        ls = spec["light"]
        pl = pal(ls["f"])
        pl2 = pal(ls["p2"]) if "p2" in ls else {}
        light_map = resolve(ls["m"], pl, pl2)
        note = f"Both halves from the published palettes (vendored in assets/syntax-themes/helix/). Accents auto-adjusted for WCAG AA where the published colors fell short."
    light = build_mode("light", light_map, adjusted)

    css = emit_css(spec["name"], spec["desc"], note, light, dark)
    path = f"{ROOT}/static/themes/{spec['name']}.css"
    open(path, "w").write(css)
    report.append((spec["name"], sorted(adjusted)))

for name, adj in report:
    print(f"{name:18} adjusted: {', '.join(adj) if adj else 'none'}")

# --- auto-mapping pass ----------------------------------------------------
# Every remaining vendored Helix TOML with a [palette] becomes a theme:
# slots are inferred from key names and hue classification, the other
# mode is derived from the palette's own hues. Curated names above win.


import colorsys
import os
import re

HELI = f"{ROOT}/assets/syntax-themes/helix"
CURATED = {s["name"] for s in SPECS} | {"github", "catppuccin", "nord", "rose-pine"}

BG_RE = re.compile(r"^(bg|background|base|canvas)", re.I)
SURFACE_SKIP = re.compile(r"red|green|blue|yellow|magenta|cyan|visual|selection|menu|focus|inlay|popup|highlight|status|diff|dim|alt|current|hl|line|added|changed|removed|gutter|contrast|search|fg", re.I)
FG_RE = re.compile(r"^(fg|foreground|text)", re.I)
NEUTRAL_RE = re.compile(r"gray|grey|black|white|comment|linenr|border|ui|selection|cursor|whitespace|indent|none|gutter|dark|disabled|widget|faint|muted|dim", re.I)

def bucket_of(h):
    if h >= 345 or h < 12: return "red"
    if h < 40: return "orange"
    if h < 70: return "yellow"
    if h < 165: return "green"
    if h < 200: return "cyan"
    if h < 252: return "blue"
    if h < 292: return "purple"
    return "magenta"

def auto_slots(stem):
    """Infer the contract slots from a palette table. Returns
    (mode, slots) where mode is the palette's own half."""
    data = tomllib.load(open(f"{HELI}/{stem}.toml", "rb"))
    p = {k: _norm_hex(v) for k, v in (data.get("palette") or {}).items()
         if isinstance(v, str) and v.startswith("#")}
    if len(p) < 6:
        return None

    def ui_hint(entry, field):
        """Resolve `ui.<entry> = { <field> = "palette-key or #hex" }`."""
        node = data.get("ui", {}).get(entry)
        if isinstance(node, dict):
            v = node.get(field)
            if isinstance(v, str):
                if v.startswith("#"):
                    return _norm_hex(v)
                if v in p:
                    return p[v]
        return None

    colors = list(p.values())
    bg_named = [(k, v) for k, v in p.items() if BG_RE.match(k) and not SURFACE_SKIP.search(k)]
    fg_named = [(k, v) for k, v in p.items()
                if FG_RE.match(k) and not NEUTRAL_RE.search(k) and not BG_RE.match(k)]
    ui_bg = ui_hint("background", "bg")
    ui_fg = ui_hint("text", "fg")

    def fg_of(colors, bg, light_palette):
        if ui_fg is not None and contrast(ui_fg, bg) > 1.5:
            return ui_fg
        if fg_named:
            return (min if light_palette else max)((v for _, v in fg_named), key=lum)
        rest = [c for c in colors if c != bg]
        return min(rest, key=lum) if light_palette else max(rest, key=lum)

    # direction from the theme's declared background when there is one
    # — accents are bright in dark themes too, so a whole-palette vote
    # lies
    if ui_bg is not None:
        light_palette = lum(ui_bg) > 0.5
    elif bg_named:
        mid = sorted(lum(v) for _, v in bg_named)[len(bg_named) // 2]
        light_palette = mid > 0.5
    else:
        light_palette = sum(1 for c in colors if lum(c) > 0.5) > len(colors) / 2

    if ui_bg is not None:
        bg = ui_bg
        alt = elv = None
    elif bg_named:
        bg = (max if light_palette else min)((v for _, v in bg_named), key=lum)
        surfaces = sorted((v for _, v in bg_named if v != bg), key=lum)
        if surfaces:
            toward_fg = surfaces if not light_palette else list(reversed(surfaces))
            alt, elv = (toward_fg[0], toward_fg[min(1, len(toward_fg) - 1)])
        else:
            alt, elv = mix(bg, fg_of(colors, bg, light_palette), 0.10), None
    else:
        bg = max(colors, key=lum) if light_palette else min(colors, key=lum)
        alt = elv = None

    fg = fg_of(colors, bg, light_palette)
    if alt is None:
        alt = mix(bg, fg, 0.10 if not light_palette else 0.06)
    if elv is None:
        elv = mix(bg, fg, 0.18 if not light_palette else 0.03)
    # elevation convention (matches the stock look): elevated surfaces
    # are never darker than the page bg
    if lum(elv) < lum(bg):
        elv = bg if light_palette else mix(bg, fg, 0.16)

    buckets = {}
    for k, v in p.items():
        if v in (bg, alt, elv, fg) or NEUTRAL_RE.search(k) or BG_RE.match(k):
            continue
        r, g, b = rgb(v)
        h, l, sat = colorsys.rgb_to_hls(r / 255, g / 255, b / 255)
        if sat < 0.15 or l < 0.06 or l > 0.96:
            continue
        bk = bucket_of(h * 360)
        score = sat * (1 - abs(l - (0.40 if not light_palette else 0.55)))
        if bk not in buckets or score > buckets[bk][1]:
            buckets[bk] = (v, score)

    def pick(*names):
        for n in names:
            if n in buckets:
                return buckets[n][0]
        return None

    # muted/monochrome palettes carry no distinct hues — fall back to
    # canonical seeds (such a scheme has no hue to preserve anyway);
    # the contrast fitter restyles them per mode
    seeds = {"brand1": "#2050a8", "success": "#2f7d1f", "warning": "#8f640d",
             "danger": "#a3443f", "important": "#7048a8", "sponsor": "#a02f6f"}
    order = {
        "brand1": ["blue", "cyan", "purple", "magenta", "orange", "green", "red"],
        "success": ["green", "cyan", "yellow"],
        "warning": ["yellow", "orange"],
        "danger": ["red", "orange", "magenta"],
        "important": ["purple", "magenta", "blue"],
        "sponsor": ["magenta", "red", "purple"],
    }
    # assign the discriminating roles first so the signature accents
    # (brand) never steal a hue a container needs
    roles, taken = {}, {bg, fg}
    for role in ("danger", "warning", "success", "important", "brand1", "sponsor"):
        c = pick(*order[role])
        if c is None or c in taken:
            c = seeds[role]
        roles[role] = c
        taken.add(c)

    slots = dict(
        bg=bg, bg_alt=alt, bg_elv=elv, text1=fg, text2=fg, text3=fg,
        border=mix(fg, bg, 0.32),
        brand1=roles["brand1"], success=roles["success"], warning=roles["warning"],
        danger=roles["danger"], important=roles["important"], sponsor=roles["sponsor"],
    )
    # surface guard: body text must be ABLE to reach 7:1 — mid-tone
    # "dark" backgrounds get nudged toward black (and vice versa), or
    # the text fit exhausts at white/black and still fails
    def toward(c, t, stop):
        while lum(c) > stop if t == black else lum(c) < stop:
            c = mix(c, t, 0.03)
        return c
    if not light_palette:
        if lum(bg) > 0.082:
            bg = toward(bg, black, 0.082)
    else:
        if lum(bg) < 0.30:
            bg = toward(bg, white, 0.30)
    elv = mix(bg, fg, 0.18 if not light_palette else 0.03)
    slots["bg"] = bg
    slots["bg_alt"] = mix(bg, fg, 0.10 if not light_palette else 0.06)
    slots["bg_elv"] = elv if lum(elv) >= lum(bg) else bg
    slots["border"] = mix(fg, bg, 0.32)

    return ("dark" if not light_palette else "light"), slots

regenerated = set()

# stems are unique, but two stems can normalize to one name
# (foo_bar + foo-bar) — that would silently clobber; refuse both
norm = {}
for f in sorted(os.listdir(HELI)):
    if f.endswith(".toml"):
        norm.setdefault(f[:-5].replace("_", "-"), []).append(f[:-5])
colliding = {n for n, srcs in norm.items() if len(srcs) > 1}

skipped, mapped = [], 0
for f in sorted(os.listdir(HELI)):
    if not f.endswith(".toml"):
        continue
    stem = f[:-5]
    name = stem.replace("_", "-")
    if name in CURATED:
        continue
    if name in colliding:
        skipped.append(f"{stem} (name collision)")
        continue
    got = auto_slots(stem)
    if not got:
        skipped.append(stem)
        continue
    mode, slots = got
    adjusted = set()
    if mode == "dark":
        dark_map = dict(slots)
        dark = build_mode("dark", dark_map, adjusted)
        light_map = dict(
            bg=mix(slots["bg"], white, 0.94), bg_alt=mix(slots["bg"], white, 0.90),
            bg_elv=mix(slots["bg"], white, 0.955),
            text1=mix(slots["text1"], black, 0.75), text2=mix(slots["text1"], black, 0.55),
            text3=mix(slots["text1"], black, 0.35), border=mix(slots["bg"], white, 0.84),
            brand1=slots["brand1"], brand2=mix(slots["brand1"], black, 0.15),
            brand3=mix(slots["brand1"], black, 0.28),
            success=slots["success"], warning=slots["warning"], danger=slots["danger"],
            important=slots["important"], sponsor=slots["sponsor"])
        light = build_mode("light", light_map, adjusted)
        note = f"The scheme publishes no light mode, so the light half is derived from the same hues. Accents auto-fitted for WCAG AA."
    else:
        light_map = dict(slots)
        light = build_mode("light", light_map, adjusted)
        dark_map = dict(
            bg=mix(slots["bg"], black, 0.90), bg_alt=mix(slots["bg"], black, 0.86),
            bg_elv=mix(slots["bg"], black, 0.82),
            text1=mix(slots["text1"], white, 0.78), text2=mix(slots["text1"], white, 0.58),
            text3=mix(slots["text1"], white, 0.38), border=mix(slots["bg"], black, 0.72),
            brand1=slots["brand1"], brand2=mix(slots["brand1"], white, 0.15),
            brand3=mix(slots["brand1"], white, 0.08),
            success=slots["success"], warning=slots["warning"], danger=slots["danger"],
            important=slots["important"], sponsor=slots["sponsor"])
        dark = build_mode("dark", dark_map, adjusted)
        note = f"The scheme publishes no dark mode, so the dark half is derived from the same hues. Accents auto-fitted for WCAG AA."
    desc = f"auto-mapped from the vendored Helix palette <{stem}>"
    open(f"{ROOT}/static/themes/{name}.css", "w").write(
        emit_css(name, desc, note, light, dark))
    regenerated.add(name)
    mapped += 1

# drop auto-generated files whose source no longer maps (stale runs)
marker = "auto-mapped from the vendored Helix palette"
for f in os.listdir(f"{ROOT}/static/themes"):
    if not f.endswith(".css") or f[:-4] in regenerated:
        continue
    path = f"{ROOT}/static/themes/{f}"
    head = open(path).read(200)
    if marker in head:
        os.remove(path)
print(f"auto-mapped: {mapped} themes; skipped (no usable palette): {len(skipped)}")
if skipped:
    print("  skipped:", ", ".join(sorted(skipped)))
