#import "../../lib.typ": diffst-hunks, diffst-report

#set page(width: 297mm, height: auto, margin: 12mm)

= hunks

#let old-file = path("../sources/option-cases/collapse-old.txt")
#let new-file = path("../sources/option-cases/collapse-new.txt")
#let report = diffst-report(
  read(old-file),
  read(new-file),
  old-label: "examples/sources/option-cases/collapse-old.txt",
  new-label: "examples/sources/option-cases/collapse-new.txt",
)

#let hunks = diffst-hunks(report, context-lines: 1)

#assert.eq(hunks.len(), 2)
#assert.eq(hunks.first().context_before, 1)
#assert.eq(hunks.first().context_after, 1)
#assert.eq(hunks.last().context_before, 1)

Hunks: #hunks.len()
