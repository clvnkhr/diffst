#import "../lib.typ": (
  default-colors,
  diffst-hunk-raw,
  diffst-labels-raw,
  diffst-line-counts-raw,
  diffst-report,
  diffst-hunks,
  diffst-rows,
  diffst-row-counts-raw,
  diffst-stat-raw,
  diffst-stats-raw,
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
#let rows = diffst-rows(report, display: "collapsed", collapse-threshold: 4)
#let labels = diffst-labels-raw(report)
#let lines = diffst-line-counts-raw(report)
#let row-counts = diffst-row-counts-raw(rows)
#let first-hunk = diffst-hunk-raw(hunks.first())
#let first_context = first-hunk.at("context-before") + first-hunk.at("context-after")
#let similarity = diffst-stat-raw(report, "similarity")

#let metric(label, value, fill, fg: colors.text) = block[
  #box(
    fill: fill,
    inset: (x: 7pt, y: 5pt),
    radius: 3pt,
    width: 100%,
  )[
    #text(size: 6.3pt, fill: colors.line-no, weight: "bold")[#label]
    #linebreak()
    #text(size: 13pt, fill: fg, weight: "bold")[#str(value)]
  ]
]

#let range-value(start, len) = {
  if start == none {
    "none"
  } else {
    str(start) + "-" + str(start + len - 1)
  }
}

= manual diffst layout

#grid(
  columns: (1fr, auto, 1fr),
  gutter: 12pt,
  align: horizon,
  [
    #text(size: 7pt, fill: colors.line-no, weight: "bold")[old]
    #linebreak()
    #text(size: 10pt, weight: "bold")[#labels.old]
  ],
  [
    #text(size: 16pt, fill: colors.line-no)[#math.mapsto]
  ],
  [
    #align(right)[
      #text(size: 7pt, fill: colors.line-no, weight: "bold")[new]
      #linebreak()
      #text(size: 10pt, weight: "bold")[#labels.new]
    ]
  ],
)

#v(9pt)

#grid(
  columns: (1fr, 1fr, 1fr, 1fr),
  gutter: 6pt,
  metric("similarity", str(similarity) + "%", colors.collapsed),
  ..diffst-stats-raw(report, stats: ("additions", "deletions", "changed-blocks")).map(stat => {
    if stat.key == "additions" {
      metric("additions", "+" + str(stat.value), colors.insert, fg: colors.insert-text)
    } else if stat.key == "deletions" {
      metric("deletions", "-" + str(stat.value), colors.delete, fg: colors.delete-text)
    } else {
      metric("changed blocks", stat.value, colors.replace, fg: colors.replace-text)
    }
  }),
)

#v(8pt)

#box(width: 100%, height: 7pt, fill: colors.collapsed, radius: 2pt)[
  #box(width: similarity * 1%, height: 7pt, fill: colors.insert, radius: 2pt)
]

#v(8pt)

#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 12pt,
  [
    #text(size: 7pt, fill: colors.line-no, weight: "bold")[source size]
    #linebreak()
    #text(size: 9pt)[#lines.old old lines / #lines.new new lines]
  ],
  [
    #text(size: 7pt, fill: colors.line-no, weight: "bold")[visible rows]
    #linebreak()
    #text(size: 9pt)[
      #row-counts.rows rows, #row-counts.hidden hidden
    ]
  ],
  [
    #text(size: 7pt, fill: colors.line-no, weight: "bold")[first hunk]
    #linebreak()
    #text(size: 9pt)[
      old #range-value(first-hunk.at("old-start"), first-hunk.at("old-len")) /
      new #range-value(first-hunk.at("new-start"), first-hunk.at("new-len"))
    ]
  ],
)

#v(8pt)

#table(
  columns: (auto, auto, auto, auto, auto),
  stroke: colors.border,
  inset: (x: 5pt, y: 3pt),
  [kind], [visible], [insert], [delete], [replace],
  [rows], [#row-counts.rows], [#row-counts.insert], [#row-counts.delete], [#row-counts.replace],
  [hunks], [#hunks.len()], [#first-hunk.ops ops], [#first-hunk.rows rows], [#str(first_context) context],
)

#v(8pt)

#diffst-table(
  report,
  rows: rows,
  colors: colors,
)
