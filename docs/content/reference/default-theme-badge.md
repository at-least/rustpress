---
description: Use the Badge tag to add status labels to headers in rustpress documentation.
---

# Badge

The badge lets you add status to your headers. For example, it could be useful to specify the section's type, or supported version.

## Usage

You may use the `<Badge>` tag anywhere in markdown. It is rewritten at build time into a styled `<span class="VPBadge …">`.

```html
### Title <Badge type="info" text="default" />
### Title <Badge type="tip" text="^1.9.0" />
### Title <Badge type="warning" text="beta" />
### Title <Badge type="danger" text="deprecated" />
```

Code above renders like:

### Title <Badge type="info" text="default" />
### Title <Badge type="tip" text="^1.9.0" />
### Title <Badge type="warning" text="beta" />
### Title <Badge type="danger" text="deprecated" />

## Custom Children

`<Badge>` accepts children, which will be displayed in the badge. Children are inserted as written (HTML included); the `text` attribute, by contrast, is escaped.

```html
### Title <Badge type="info">custom element</Badge>
```

### Title <Badge type="info">custom element</Badge>

## Customize Type Color

You can customize the style of badges by overriding css variables in your [theme CSS](../guide/coming-from-vitepress#theming). The following are the default values:

```css
:root {
  --vp-badge-info-border: transparent;
  --vp-badge-info-text: var(--vp-c-text-2);
  --vp-badge-info-bg: var(--vp-c-default-soft);

  --vp-badge-note-border: transparent;
  --vp-badge-note-text: var(--vp-c-note-1);
  --vp-badge-note-bg: var(--vp-c-note-soft);

  --vp-badge-tip-border: transparent;
  --vp-badge-tip-text: var(--vp-c-tip-1);
  --vp-badge-tip-bg: var(--vp-c-tip-soft);

  --vp-badge-important-border: transparent;
  --vp-badge-important-text: var(--vp-c-important-1);
  --vp-badge-important-bg: var(--vp-c-important-soft);

  --vp-badge-caution-border: transparent;
  --vp-badge-caution-text: var(--vp-c-caution-1);
  --vp-badge-caution-bg: var(--vp-c-caution-soft);

  --vp-badge-warning-border: transparent;
  --vp-badge-warning-text: var(--vp-c-warning-1);
  --vp-badge-warning-bg: var(--vp-c-warning-soft);

  --vp-badge-danger-border: transparent;
  --vp-badge-danger-text: var(--vp-c-danger-1);
  --vp-badge-danger-bg: var(--vp-c-danger-soft);
}
```

## `<Badge>`

`<Badge>` accepts the following attributes:

| attribute | description |
| --- | --- |
| `text` | The badge label. Ignored when children are given. |
| `type` | Defaults to `tip`. One of `info`, `note`, `tip`, `important`, `caution`, `warning`, `danger` — matching the markdown container/alert colors. Any other value is emitted as a class name without styling. |

Badges inside code fences are left untouched.
