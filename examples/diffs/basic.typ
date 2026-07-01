#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= diffst example

#diffst(path("../sources/old.typ"), path("../sources/new.typ"))
