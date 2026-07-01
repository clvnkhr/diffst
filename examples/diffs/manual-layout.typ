#import "../../lib.typ": (
  default-colors,
  diffst-report,
  diffst-hunks,
  diffst-rows,
  diffst-table,
)

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

#let colors = default-colors + (
  replace: rgb("#e0f2fe"),
  replace-text: rgb("#075985"),
  inline-delete: rgb("#f0abfc"),
  inline-insert: rgb("#67e8f9"),
)

#let old-file = path("../sources/old.typ")
#let new-file = path("../sources/new.typ")
#let report = diffst-report(
  read(old-file),
  read(new-file),
  old-label: "examples/sources/old.typ",
  new-label: "examples/sources/new.typ",
  show-whitespace: true,
)

#let hunks = diffst-hunks(report, context-lines: 2)
#let rows = diffst-rows(report, display: "collapsed", collapse-threshold: 4)
#let row-counts(rows) = {
  let counts = (rows: rows.len(), insert: 0, delete: 0, replace: 0, hidden: 0)
  for row in rows {
    if row.kind == "insert" {
      counts.insert += 1
    } else if row.kind == "delete" {
      counts.delete += 1
    } else if row.kind == "replace" {
      counts.replace += 1
    } else if row.kind == "collapsed" {
      counts.hidden += row.hidden
    }
  }
  counts
}
#let row-counts = row-counts(rows)
#let first-hunk = hunks.first()
#let first_context = first-hunk.context_before + first-hunk.context_after
#let similarity = calc.round(report.stats.similarity * 100)

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
    #text(size: 10pt, weight: "bold")[#report.old]
  ],
  [
    #text(size: 16pt, fill: colors.line-no)[#math.mapsto]
  ],
  [
    #align(right)[
      #text(size: 7pt, fill: colors.line-no, weight: "bold")[new]
      #linebreak()
      #text(size: 10pt, weight: "bold")[#report.new]
    ]
  ],
)

#v(9pt)

#grid(
  columns: (1fr, 1fr, 1fr, 1fr),
  gutter: 6pt,
  metric("similarity", str(similarity) + "%", colors.collapsed),
  metric("additions", "+" + str(report.stats.additions), colors.insert, fg: colors.insert-text),
  metric("deletions", "-" + str(report.stats.deletions), colors.delete, fg: colors.delete-text),
  metric("changed blocks", report.stats.changed_blocks, colors.replace, fg: colors.replace-text),
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
    #text(size: 9pt)[#report.stats.old_lines old lines / #report.stats.new_lines new lines]
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
      old #range-value(first-hunk.old_start, first-hunk.old_len) /
      new #range-value(first-hunk.new_start, first-hunk.new_len)
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
  [hunks], [#hunks.len()], [#first-hunk.ops.len() ops], [#first-hunk.rows.len() rows], [#str(first_context) context],
)

#v(8pt)

#diffst-table(
  report,
  rows: rows,
  colors: colors,
)
