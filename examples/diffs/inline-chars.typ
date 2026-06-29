#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= inline: chars

#diffst(
  "examples/sources/option-cases/inline-old.txt",
  "examples/sources/option-cases/inline-new.txt",
  inline: "chars",
)
