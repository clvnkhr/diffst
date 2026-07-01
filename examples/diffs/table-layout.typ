#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 12mm)
#set text(font: "New Computer Modern", size: 9pt)

= table-layout: single

#diffst(
  path("../sources/option-cases/unicode-old.txt"),
  path("../sources/option-cases/unicode-new.txt"),
  table-layout: "single",
  display: "collapsed",
  collapse-threshold: 6,
  context-lines: 2,
)
