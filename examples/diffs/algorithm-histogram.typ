#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= algorithm: histogram

#diffst(
  path("../sources/algorithm-cases/histogram-old.typ"),
  path("../sources/algorithm-cases/histogram-new.typ"),
  algorithm: "histogram",
)
