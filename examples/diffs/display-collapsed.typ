#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= display: collapsed

#diffst(
  path("../sources/option-cases/collapse-old.txt"),
  path("../sources/option-cases/collapse-new.txt"),
  display: "collapsed",
  collapse-threshold: 3,
  context-lines: 1,
)
