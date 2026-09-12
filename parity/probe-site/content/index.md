# Diff notation x line numbers

Regression fixture for the `f94888d` bugs (see VIEWPORT-AUDIT.md): the
number gutter must own `::before` on numbered diff lines, removed lines
must keep their background + 0.7 opacity, and unnumbered blocks keep
the `+`/`-` symbol.

```ts:line-numbers
const kept = 'This line stays'
const gone = 'This line goes' // [!code --]
const added = 'This line arrives' // [!code ++]
```

```ts
const gone = 'This line goes' // [!code --]
const added = 'This line arrives' // [!code ++]
```
