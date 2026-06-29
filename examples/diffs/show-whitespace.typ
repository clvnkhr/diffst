#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= show-whitespace

#diffst(
  "examples/sources/option-cases/whitespace-old.txt",
  "examples/sources/option-cases/whitespace-new.txt",
  show-whitespace: true,
)
