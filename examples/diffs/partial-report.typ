#import "../../lib.typ": diffst-report, diffst-single-table, diffst-summary, diffst-table

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

#let old-file = "../sources/paper-old.typ"
#let new-file = "../sources/paper-new.typ"
#let report = diffst-report(
  read(old-file),
  read(new-file),
  old-label: "examples/sources/paper-old.typ",
  new-label: "examples/sources/paper-new.typ",
  inline: "words",
  semantic-cleanup: true,
)

= Partial diff report

#diffst-summary(report)

#v(10pt)

== Methods excerpt

#diffst-single-table(
  report,
  range: (10, 18),
  display: "full",
)

#v(10pt)

== Discussion excerpt

#diffst-table(
  report,
  range: (26, 34),
  range-side: "new",
  display: "full",
)
