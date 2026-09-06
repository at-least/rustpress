# vitezola

**VitePress 的 default theme，移植到 Zola。** 盡可能忠實重刻 [VitePress](https://vitepress.dev/) default theme 的版面、配色與互動 —— navbar、自動 sidebar、右側 outline、dark mode、code groups、local search —— 全部由純靜態 HTML + CSS + 少量 vanilla JS 產生，沒有 Node toolchain、沒有 hydration。

![Zola](https://img.shields.io/badge/zola-%E2%89%A50.23-blue) （需要 Zola 0.23+：使用 Tera 2 components 與新的 highlighting 設定）

## 功能

- **版面**：navbar（含 dropdown flyout、social links、Ask AI 按鈕）、自動生成的巢狀 sidebar（每個頂層 section 一份，同 VitePress 的 path-keyed sidebar）、右側 "On this page" outline（含 scrollspy marker，預設深度 h2-h3、front matter `outline = "deep"` 展開全部）、prev/next pager、edit link、last updated、頁尾
- **Dark mode**：CSS variables 直接取自 VitePress `vars.css`；`<head>` 內嵌 blocking script 避免 FOUC；偏好設定存 localStorage
- **Markdown 擴充**（Tera 2 components）：`tip` / `warning` / `danger` / `note` / `info` / `important` / `caution` 容器（支援自訂標題與 `{no-title}`）、`details`、`codegroup`
- **Code blocks**：Zola 0.22+ 的 Giallo 高亮，light/dark 雙主題（預設 github-light/github-dark，同 VitePress）、`name=` 標籤、`hl_lines=` 行高亮、複製按鈕、code group tabs
- **搜尋**：Zola 內建索引 + 手寫 vanilla JS 搜尋 modal（Ctrl+K 或 `/` 開啟）
- **手機版**：hamburger 全螢幕選單、sidebar 抽屜、local nav 的 outline dropdown
- **Badge**：`<span class="VPBadge tip">…</span>`（對應 VitePress 的 `<Badge>` 元件，樣式已移植）
- **示範站即 vitepress.dev 的內容移植**：`content/` 是 vitepress 官方文件（33 頁）的自動轉換結果（`scripts/port-vitepress-docs.py`），用以做逐像素比對
- **像素校正**：navbar 對齊部署版 vitepress.dev 的細節 —— logo 與標題零間距、translations 按鈕（16px option-icon 結構 + 左側 17px 保留）、GitHub icon 改用 simple-icons mask（`vpi-simple-icons-github`）、hero 圖帶 `VPImage` class（站方自訂 drop-shadow 生效）

## 安裝

```sh
git clone https://github.com/your-name/vitezola themes/vitezola
```

然後在你的 `config.toml` 加入（**Zola 0.23 起這些是必要設定**）：

```toml
theme = "vitezola"
compile_sass = true
build_search_index = true

[search]
index_format = "fuse_json"

[markdown.highlighting]
style = "class"
light_theme = "github-light"
dark_theme = "github-dark"

[extra]
vitepress_site_title = "My Site"
vitepress_edit_link = "https://github.com/me/my-site/edit/main/content/"
vitepress_last_updated = true

# Sidebar：預設依當前頁所屬的「頂層 content section」自動選擇（guide/、reference/
# 各一份，同 VitePress 的 path-keyed sidebar）。要固定單一 sidebar 才需要：
# vitepress_sidebar_path = "docs/_index.md"

[[extra.nav]]
text = "Guide"
link = "/guide/introduction/what-is-vitepress/"
active_match = "/guide/"        # 選填：用 URL 前綴判斷 active（同 VitePress activeMatch）

[[extra.nav]]
text = "More"
items = [{ text = "Reference", link = "/reference/" }]

[[extra.social]]
kind = "github"                              # github | twitter | other
link = "https://github.com/me/my-site"
```

> 注意：TOML 中 `[[extra.nav]]` / `[[extra.social]]` 等 array-of-table 必須放在 `[extra]` 的 scalar keys **之後**；`[extra.xxx]` 子表之後也不能再放 bare scalar keys。

**Sidebar 的群組（group）行為**對齊 VitePress 的 tri-state `collapsed`，以 group section（如 `content/guide/introduction/_index.md`）的 front matter 控制：

```toml
[extra]
vitepress_collapsed = false   # 可折疊，預設展開（有 caret 按鈕）
# vitepress_collapsed = true  # 可折疊，預設收起
# 不設                        # 不可折疊（無 caret，永遠展開）
```

把 sidebar root 當成一個「有標題的群組」渲染（同 VitePress `{ text, items }` 根項目）可加：

```toml
[extra]
vitepress_sidebar_title = "Reference"
```

section 的 front matter 也接受 `[[extra.vitepress_extra_items]]`（`text` / `link`），用來放跨 section 的純連結項目（如 Guide sidebar 底部的「Config & API Reference」）。

## Home page

在網站根 section（`content/_index.md`）設定 `template = "index.html"` 並加上：

```toml
[extra.vitepress_home]
name = "My Project"          # brand 色標題
text = "The tagline text"
tagline = "Longer description"
image = { src = "hero.png", alt = "" }   # 選用，放 static/

[[extra.vitepress_home.actions]]
text = "Get Started"
theme = "brand"              # brand | alt
link = "/docs/guide/"

[[extra.vitepress_home.features]]
icon = "⚡"
title = "Feature"
details = "Description"
link = "/docs/feature/"      # 選用
link_text = "Learn more"     # 選用
```

## 在 Markdown 裡使用

Zola 0.23 以 Tera 2 components 取代 shortcode。容器（**block call 必須列出全部參數**）：

```md
{% <tip kind="warning" title="" no_title={false}> %}
**注意** 這段文字會被 markdown 渲染。
{% </tip> %}
```

`kind` 可用：`tip`、`warning`、`danger`、`note`、`info`、`important`、`caution`。自訂標題：`title="Server Support Required"`；隱藏標題列：`no_title={true}`（同 VitePress 的 `::: tip {no-title}`）。另有 `{% <details summary=""> %}…{% </details> %}`（`summary` 留空顯示 DETAILS）。

Code group（tab 標籤來自每個 code block 的 `name=` 註解）：

````md
{% <codegroup> %}
```js,name=a.js
const a = 1;
```
```ts,name=b.ts
const b: number = 2;
```
{% </codegroup> %}
````

## 語法高亮主題

light/dark 高亮 CSS 由 Zola 在 build 時產生到 `public/giallo-light.css` / `giallo-dark.css`，但兩者都是扁平 class（`z-l-*` / `z-d-*`），無法直接用 `html.dark` 切換。本 theme 附帶的 `static/syntax.css` 已把 dark 規則改寫成 `html.dark` scope。**改過 `[markdown.highlighting]` 主題後請重新產生：**

```sh
zola build && python3 scripts/gen-syntax-css.py
```

## Team page 與 Sponsors

**Team page**：建立一個 section（如 `content/team/_index.md`）並設定 `template = "team.html"`：

```toml
[extra.vitepress_team]
title = "Our Team"
lead = "The folks behind this project."

[[extra.vitepress_team.members]]
name = "Ella"
avatar = "images/avatar.svg"      # 放 static/
title = "Creator"
org = "vitezola"
org_link = "https://example.com" # 選用，讓 org 變連結
desc = "Description with **markdown**."
links = [{ kind = "github", link = "https://github.com/you" }]
sponsor = "https://github.com/sponsors/you"   # 選用，卡片底部出現 Sponsor 按鈕

[[extra.vitepress_team.sections]]  # 選用：分組區塊
title = "Contributors"
lead = "..."
size = "small"                     # small | medium

[[extra.vitepress_team.sections.members]]
name = "Alex"
avatar = "images/avatar-2.svg"
```

**Sponsors**：

```toml
# Home 頁尾區塊 → content/_index.md 的 [extra.vitepress_home] 內
[extra.vitepress_home.sponsors]
message = "Special thanks to:"
action_link = "https://github.com/sponsors/you"
action_text = "Become a sponsor"

[[extra.vitepress_home.sponsors.tiers]]
tier = "Diamond"
size = "medium"                    # xmini | mini | small | medium | big（選填，未填依數量自動）
[[extra.vitepress_home.sponsors.tiers.items]]
name = "Acme"
img = "images/sponsor-1.svg"
url = "https://example.com"
```

```toml
# 文件頁右側 aside → config.toml
[[extra.vitepress_sponsors.tiers]]
tier = "Sponsors"
size = "xmini"
[[extra.vitepress_sponsors.tiers.items]]
name = "Acme"
img = "images/sponsor-1.svg"
url = "https://example.com"
```

Grid 欄數行為同 VitePress：依數量自動選尺寸（9+ → xmini、7-8 → mini、5-6 → small、3-4 → medium、1-2 → big），桌面版欄數取「尺寸欄數」與「項目數」的較小值並補空位對齊，窄螢幕自動降為 2 欄／1 欄。暗色模式下贊助商 logo 自動反色（同 VitePress 的處理）。

## 語言切換器

站點有多語言（`[languages.fr]` 等）時，navbar 會自動出現語言下拉選單、手機版選單出現語言手風琴。行為同 VitePress：當前頁有對應翻譯就連過去，否則連到該語言的根路徑。語言顯示名稱可選設定：

```toml
[extra.vitepress_language_labels]
en = "English"
fr = "Français"
```

## VitePress ↔ vitezola 語法對照

| VitePress | vitezola (Zola 0.23) |
| --- | --- |
| `::: tip` … `:::` | `{% <tip kind="tip" title="" no_title={false}> %} … {% </tip> %}` |
| `::: warning SERVER REQUIRED` | `kind="warning" title="SERVER REQUIRED"` |
| `::: tip {no-title}` | `no_title={true}` |
| `::: details` | `{% <details summary=""> %} … {% </details> %}` |
| `<Badge type="warning" text="experimental" />` | `<span class="VPBadge warning">experimental</span>` |
| <code>\`\`\`js [a.js]</code>（code group） | <code>\`\`\`js,name=a.js</code> 包在 `{% <codegroup> %} … {% </codegroup> %}` 裡 |
| <code>\`\`\`js{1,3-4}</code>（行高亮） | <code>\`\`\`js,hl_lines=1 3-4</code> |
| `outline: deep`（front matter） | 同名支援：page/section front matter `outline = "deep"`（預設只到 h3） |
| `themeConfig.sidebar` | content section 結構自動生成（每個頂層 section 一份 sidebar） |
| `themeConfig.appearance` | 永遠開啟（`prefers-color-scheme` + localStorage） |
| `langLabel` / locale 名稱 | `extra.vitepress_language_labels` |

## 在本機開發

repo 根目錄本身就是一個 Zola 示範站（root-level templates 直接生效）：

```sh
zola serve
```

## 注意事項

- 在 **Zola 0.23** 測試；Zola 0.22 以前（舊 Tera / shortcode）不相容。
- 需要支援 `:has()`、CSS nesting 的現代瀏覽器（2023+ 的 Chrome/Edge/Firefox/Safari）。
- 搜尋是以空白分詞的輕量評分，**CJK 內容的召回率會比英文差**；需要更好的 CJK 搜尋可把 `[search]` 換成 elasticlunr 並接上自己的前端。
- 關閉 JS 時網站仍可讀：sidebar/outline 靜態渲染、code group 顯示第一個 tab、外觀跟隨系統 `prefers-color-scheme`；dark toggle 與搜尋則停用。

## 與 VitePress 的差異（設計取捨）

- 沒有 Vue runtime：markdown 內不能嵌入 Vue 元件；互動全部是 vanilla JS
- Data loaders 需在 build pipeline 外先產生資料
- sidebar 由 Zola 的 content section 結構推導，而非 `themeConfig.sidebar` 陣列；折疊語義見上方 tri-state 說明
- sidebar / pager 的標題取自頁面 title（= 內文 H1），無法像 VitePress 在 sidebar 裡另取短名（例如 vitepress.dev 的「Deploy」vs H1「Deploy Your VitePress Site」）；prev/next 順序也只在同一 section 內排序
- 搜尋為自寫的輕量評分（來源為 Zola 的 fuse_json 索引），非 minisearch/Algolia
- Shiki 的行內標記 `[!code highlight]` / `[!code focus]` / `[!code ++]` 不會作用（giallo 沒有 transformers），會以原樣文字出現在 code block 裡；`:line-numbers` 修飾也會被剝除（giallo 不支援逐塊行號）
- vitepress.dev 站方自行加的東西不屬於 default theme，僅部分重現：code group tabs 沒有 package 小圖示（那是 vitepress-plugin-group-icons）、aside 沒有 Carbon Ads（需要帳號）、navbar 的 Ask AI 是普通連結（`vitepress_ask_ai_url`），沒有 DocSearch sidepanel

## 授權

CSS 變數、字體（Inter）與圖示移植自 VitePress（MIT）。其餘以 MIT 發佈。
