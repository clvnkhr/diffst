#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= algorithm: patience

#diffst(
  "examples/algorithm-cases/patience-old.typ",
  "examples/algorithm-cases/patience-new.typ",
  algorithm: "patience",
)
