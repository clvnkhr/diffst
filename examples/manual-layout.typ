#import "../lib.typ": (
  default-colors,
  diffst-report,
  diffst-hunks,
  diffst-rows,
  diffst-summary,
  diffst-table,
)

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

#let colors = default-colors + (
  replace: rgb("#e0f2fe"),
  replace-text: rgb("#075985"),
  inline-delete: rgb("#f0abfc"),
  inline-insert: rgb("#67e8f9"),
)

#let report = diffst-report(
  "examples/old.typ",
  "examples/new.typ",
  show-whitespace: true,
)

#let hunks = diffst-hunks(report, context-lines: 2)

= manual diffst layout

#grid(
  columns: (2fr, 1fr),
  gutter: 12pt,
  [
    #diffst-summary(report, colors: colors)
  ],
  [
    #align(right)[
      #text(size: 7pt, fill: colors.line-no)[
        #hunks.len() hunks\
        first hunk rows #hunks.first().row_start - #hunks.first().row_end
      ]
    ]
  ],
)

#v(8pt)

#diffst-table(
  report,
  rows: diffst-rows(report, display: "collapsed", collapse-threshold: 4),
  colors: colors,
)
