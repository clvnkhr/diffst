#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= trailing whitespace

#diffst(
  path("../sources/option-cases/trailing-whitespace-old.txt"),
  path("../sources/option-cases/trailing-whitespace-new.txt"),
  show-whitespace: true,
)
