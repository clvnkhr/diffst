#import "../../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= deadline-ms

#diffst(
  "examples/paper-old.typ",
  "examples/paper-new.typ",
  deadline-ms: 10000,
)
