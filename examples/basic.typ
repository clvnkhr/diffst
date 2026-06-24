#import "../lib.typ": diffst

#set page(width: 297mm, height: 210mm, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= diffst example

#diffst("examples/old.typ", "examples/new.typ")
