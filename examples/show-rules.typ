#import "../lib.typ": default-colors, diffst, diffst-style

#set raw(lang: "typst")

#set page(
  width: 297mm,
  height: 210mm,
  margin: (x: 10mm, y: 12mm),
  fill: rgb("#fffaf0"),
)
#set text(font: ("Avenir Next", "Atkinson Hyperlegible"), size: 9pt, fill: rgb("#10131f"))

#let review-colors = (
  default-colors
    + (
      text: rgb("#10131f"),
      line-no: rgb("#545b72"),
      border: rgb("#252a44"),
      header: rgb("#151a2e"),
      equal: rgb("#fcfbf7"),
      delete: rgb("#ffd6df"),
      insert: rgb("#c9f7da"),
      replace: rgb("#ffe08a"),
      inline-delete: rgb("#ff5f7e"),
      inline-insert: rgb("#32d177"),
      delete-text: rgb("#7a1025"),
      insert-text: rgb("#083f20"),
      replace-text: rgb("#5f3900"),
      collapsed: rgb("#eceff6"),
    )
)

#show: diffst-style.with(colors: review-colors)
#show heading: it => block(
  below: 0.9em,
  stroke: (bottom: 1.4pt + rgb("#151a2e")),
  inset: (bottom: 4pt),
)[#it]
#show heading: set text(
  font: ("Avenir Next", "Atkinson Hyperlegible"),
  fill: rgb("#151a2e"),
  weight: "black",
)
#show raw: set text(font: ("JetBrains Mono", "FiraCode Nerd Font"), fill: rgb("#5b21b6"))
#show raw.where(block: true): it => block(
  fill: rgb("#f3e8ff"),
  stroke: 0.8pt + rgb("#7c3aed"),
  inset: 7pt,
  radius: 4pt,
)[#it]
#show block: set block(spacing: 0.9em)
#show table: set text(size: 6.4pt)
#show table: set table(stroke: 1.2pt + rgb("#252a44"))
#show table.header: set table.header(repeat: true)
#show table.cell.where(x: 0): set text(fill: rgb("#334155"), weight: "bold")
#show table.cell.where(x: 2): set text(fill: rgb("#334155"), weight: "bold")
#show table.cell.where(y: 0): set text(weight: "bold", fill: rgb("#f8fafc"))
#show regex("\\b[0-9]+% similar lines\\b"): set text(weight: "black", fill: rgb("#7c2d12"))
#show "unchanged lines hidden": set text(style: "italic", fill: rgb("#475569"))

= show-rule customization

```typ
#set text(font: ("Avenir Next", "Atkinson Hyperlegible"))
#show raw: set text(font: ("JetBrains Mono", "FiraCode Nerd Font"))
#show table.header: set table.header(repeat: true)
```

#diffst(
  "examples/paper-old.typ",
  "examples/paper-new.typ",
  display: "collapsed",
  collapse-threshold: 2,
  inline: "words",
  show-whitespace: true,
)
