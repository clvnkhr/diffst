#import "../../lib.typ": (
  diffst-debug,
  diffst-report,
  diffst-rows,
  diffst-single-table,
  diffst-table,
)

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

#let old-file = "../option-cases/unicode-old.txt"
#let new-file = "../option-cases/unicode-new.txt"
#let report = diffst-report(
  read(old-file),
  read(new-file),
  old-label: "examples/option-cases/unicode-old.txt",
  new-label: "examples/option-cases/unicode-new.txt",
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

= Debug messages

#diffst-debug(report)

#v(8pt)

#diffst-table(report, rows: rows)

#v(8pt)

#diffst-single-table(report, rows: rows)
