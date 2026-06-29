#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= display: full

#diffst(
  "examples/sources/option-cases/collapse-old.txt",
  "examples/sources/option-cases/collapse-new.txt",
  display: "full",
)
