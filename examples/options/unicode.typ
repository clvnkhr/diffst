#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= unicode: true

#diffst(
  "examples/option-cases/unicode-old.txt",
  "examples/option-cases/unicode-new.txt",
  inline: "chars",
  unicode: true,
)

#pagebreak()

= unicode: false

#diffst(
  "examples/option-cases/unicode-old.txt",
  "examples/option-cases/unicode-new.txt",
  inline: "chars",
  unicode: false,
)
