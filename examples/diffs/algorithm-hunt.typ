#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= algorithm: hunt

#diffst(
  path("../sources/algorithm-cases/duplicates-old.typ"),
  path("../sources/algorithm-cases/duplicates-new.typ"),
  algorithm: "hunt",
)
