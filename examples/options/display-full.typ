#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= display: full

#diffst(
  "examples/option-cases/collapse-old.txt",
  "examples/option-cases/collapse-new.txt",
  display: "full",
)
