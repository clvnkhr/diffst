#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= algorithm: histogram

#diffst(
  "examples/algorithm-cases/histogram-old.typ",
  "examples/algorithm-cases/histogram-new.typ",
  algorithm: "histogram",
)
