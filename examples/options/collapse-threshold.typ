#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= collapse-threshold

#diffst(
  "examples/option-cases/collapse-old.txt",
  "examples/option-cases/collapse-new.txt",
  collapse-threshold: 3,
  context-lines: 1,
)
