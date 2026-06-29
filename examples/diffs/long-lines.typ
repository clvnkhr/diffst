#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= long lines

#diffst(
  "examples/sources/option-cases/long-lines-old.txt",
  "examples/sources/option-cases/long-lines-new.txt",
  inline: "words",
  semantic-cleanup: true,
  show-whitespace: true,
  display: "full",
)
