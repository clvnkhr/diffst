#import "../../lib.typ": diffst

#set page(width: 297mm, height: auto, margin: 11mm)
#set text(font: "New Computer Modern", size: 8.5pt)

= realistic diffst example

#diffst(
  path("../sources/paper-old.typ"),
  path("../sources/paper-new.typ"),
  inline: "words",
  semantic-cleanup: true,
  show-whitespace: true,
  display: "collapsed",
  collapse-threshold: 6,
)
