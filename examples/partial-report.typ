#import "../lib.typ": diffst-report, diffst-single-table, diffst-summary, diffst-table

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

#let report = diffst-report(
  "examples/paper-old.typ",
  "examples/paper-new.typ",
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
