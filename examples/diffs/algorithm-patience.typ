#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= algorithm: patience

#diffst(
  path("../sources/algorithm-cases/patience-old.typ"),
  path("../sources/algorithm-cases/patience-new.typ"),
  algorithm: "patience",
)
