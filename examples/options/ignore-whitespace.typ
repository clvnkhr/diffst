#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= ignore-whitespace

#diffst(
  "examples/option-cases/whitespace-old.txt",
  "examples/option-cases/whitespace-new.txt",
  ignore-whitespace: true,
)
