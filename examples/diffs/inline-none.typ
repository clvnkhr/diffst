#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= inline: none

#diffst(
  path("../sources/option-cases/inline-old.txt"),
  path("../sources/option-cases/inline-new.txt"),
  inline: "none",
)
