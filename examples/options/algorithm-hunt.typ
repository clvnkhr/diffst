#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= algorithm: hunt

#diffst(
  "examples/algorithm-cases/duplicates-old.typ",
  "examples/algorithm-cases/duplicates-new.typ",
  algorithm: "hunt",
)
