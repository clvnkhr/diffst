#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= semantic-cleanup

#diffst(
  "examples/option-cases/semantic-old.txt",
  "examples/option-cases/semantic-new.txt",
  semantic-cleanup: true,
)
