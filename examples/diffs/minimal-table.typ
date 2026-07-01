#import "../../lib.typ": diffst, minimal-table

#set page(width: 297mm, height: auto, margin: 14mm)
#set text(font: "New Computer Modern", size: 9pt)

#show: minimal-table

= minimal table

#diffst(
  path("../sources/paper-old.typ"),
  path("../sources/paper-new.typ"),
  display: "collapsed",
  context-lines: 2,
  inline: "words",
  semantic-cleanup: true,
)
