#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= semantic-cleanup

#diffst(
  "examples/sources/option-cases/semantic-old.txt",
  "examples/sources/option-cases/semantic-new.txt",
  semantic-cleanup: true,
)
