#import "../../lib.typ": (
  diffst-debug,
  diffst-debug-raw,
  diffst-report,
  diffst-rows,
  diffst-table,
)

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

#let report = diffst-report(
  "examples/option-cases/unicode-old.txt",
  "examples/option-cases/unicode-new.txt",
  algorithm: "patience",
  inline: "words",
  unicode: true,
  semantic-cleanup: true,
  show-whitespace: true,
)

#let rows = diffst-rows(
  report,
  display: "collapsed",
  collapse-threshold: 6,
  context-lines: 2,
)

#let debug = diffst-debug-raw(report, rows: rows, context-lines: 2)

= Debug messages

#diffst-debug(report, rows: rows, context-lines: 2)

#v(8pt)

#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 8pt,
  [*Raw algorithm*\
  #debug.meta.algorithm],
  [*Raw messages*\
  #debug.messages.len()],
  [*Raw hunks*\
  #debug.hunks],
)

#v(8pt)

#diffst-table(report, rows: rows)
