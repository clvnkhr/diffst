#import "../../lib.typ": diffst-hunks, diffst-report

= hunks

#let old-file = "../option-cases/collapse-old.txt"
#let new-file = "../option-cases/collapse-new.txt"
#let report = diffst-report(
  read(old-file),
  read(new-file),
  old-label: "examples/option-cases/collapse-old.txt",
  new-label: "examples/option-cases/collapse-new.txt",
)

#let hunks = diffst-hunks(report, context-lines: 1)

#assert.eq(hunks.len(), 2)
#assert.eq(hunks.first().context_before, 1)
#assert.eq(hunks.first().context_after, 1)
#assert.eq(hunks.last().context_before, 1)

Hunks: #hunks.len()
