#import "../../lib.typ": diffst, diffst-style, default-colors

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

#let ocean-colors = default-colors + (
  header: rgb("#dbeafe"),
  border: rgb("#93c5fd"),
  delete: rgb("#fee2e2"),
  insert: rgb("#dcfce7"),
  replace: rgb("#e0f2fe"),
  inline-delete: rgb("#fca5a5"),
  inline-insert: rgb("#86efac"),
  replace-text: rgb("#075985"),
)

#show: diffst-style.with(colors: ocean-colors)

= custom diffst colors

#diffst(
  "examples/sources/old.typ",
  "examples/sources/new.typ",
  show-whitespace: true,
)

#pagebreak()

= one-off override

#diffst(
  "examples/sources/old.typ",
  "examples/sources/new.typ",
  colors: (
    replace: rgb("#fef3c7"),
    inline-delete: rgb("#f0abfc"),
    inline-insert: rgb("#67e8f9"),
  ),
)
