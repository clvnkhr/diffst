#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= ignore-whitespace

== normal diff

#diffst(
  "examples/sources/option-cases/whitespace-old.txt",
  "examples/sources/option-cases/whitespace-new.txt",
  show-whitespace: true,
)

== ignored whitespace

#diffst(
  "examples/sources/option-cases/whitespace-old.txt",
  "examples/sources/option-cases/whitespace-new.txt",
  ignore-whitespace: true,
  show-whitespace: true,
)
