#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= unicode: true

#diffst(
  path("../sources/option-cases/unicode-old.txt"),
  path("../sources/option-cases/unicode-new.txt"),
  inline: "chars",
  unicode: true,
)

#pagebreak()

= unicode: false

#diffst(
  path("../sources/option-cases/unicode-old.txt"),
  path("../sources/option-cases/unicode-new.txt"),
  inline: "chars",
  unicode: false,
)
