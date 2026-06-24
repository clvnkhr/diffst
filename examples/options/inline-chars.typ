#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= inline: chars

#diffst(
  "examples/option-cases/inline-old.txt",
  "examples/option-cases/inline-new.txt",
  inline: "chars",
)
