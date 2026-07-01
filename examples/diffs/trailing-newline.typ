#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= trailing newline

#diffst(
  path("../sources/option-cases/trailing-newline-old.txt"),
  path("../sources/option-cases/trailing-newline-new.txt"),
)
