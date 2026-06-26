#import "../../lib.typ": diffst-content, diffst-report, diffst-summary, diffst-table

#let old = "alpha\nbeta\ngamma\n"
#let new = "alpha\nbetter\ngamma\n"

#diffst-content(old, new, old-label: "old.txt", new-label: "new.txt")

#let report = diffst-report(
  old,
  new,
  old-label: "old.txt",
  new-label: "new.txt",
)

#diffst-summary(report)
#diffst-table(report, range: (2, 2))
