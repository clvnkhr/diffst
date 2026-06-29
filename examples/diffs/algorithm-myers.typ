#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= algorithm: myers

#diffst(
  "examples/sources/algorithm-cases/duplicates-old.typ",
  "examples/sources/algorithm-cases/duplicates-new.typ",
  algorithm: "myers",
)
