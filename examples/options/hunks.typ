#import "../../lib.typ": diffst-hunks-raw, diffst-report

= hunks

#let report = diffst-report(
  "examples/option-cases/collapse-old.txt",
  "examples/option-cases/collapse-new.txt",
)

#let hunks = diffst-hunks-raw(report, context-lines: 1)

#assert.eq(hunks.len(), 2)
#assert.eq(hunks.first().at("context-before"), 1)
#assert.eq(hunks.first().at("context-after"), 1)
#assert.eq(hunks.last().at("context-before"), 1)

Hunks: #hunks.len()


