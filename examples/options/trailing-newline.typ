#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= trailing newline

#diffst(
  "examples/option-cases/trailing-newline-old.txt",
  "examples/option-cases/trailing-newline-new.txt",
)
