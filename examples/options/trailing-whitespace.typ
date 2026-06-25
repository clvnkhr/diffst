#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= trailing whitespace

#diffst(
  "examples/option-cases/trailing-whitespace-old.txt",
  "examples/option-cases/trailing-whitespace-new.txt",
  show-whitespace: true,
)
